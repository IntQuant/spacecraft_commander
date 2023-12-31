use std::{collections::HashMap, mem};

use internal::{ComponentStorageProvider, DynDispath, OfResources, ResourceStorageProvider};
use query_world::WorldRef;
use serde::{Deserialize, Serialize};
use slotmapd::{new_key_type, KeyData};

mod component_traits;
mod ecs_cell;
#[doc(hidden)]
pub mod internal;
mod query_world;
mod system_parameter;

pub use engine_macro::gen_storage_for_world;
use system_parameter::changes::{ChangeManager, ReadOnly, WriteOnly};

pub use crate::{
    component_traits::{Bundle, Component},
    query_world::{ParamGuard, QueryWorld, WorldRun},
    system_parameter::{
        changes::ChangesG,
        commands::CommandsG,
        query::{QueryG, WithG, WithoutG},
    },
};

new_key_type! { pub struct EntityID; }

impl EntityID {
    pub fn to_raw(self) -> u64 {
        self.0.as_ffi()
    }
    pub fn from_raw(raw: u64) -> Self {
        EntityID(KeyData::from_ffi(raw))
    }
}

pub type TypeIndex = u32;

pub trait Resource<Storage>: LocalTypeIndex<OfResources<Storage>> {}

impl<Storage, T> Resource<Storage> for T
where
    Storage: ComponentStorageProvider<T>,
    T: LocalTypeIndex<OfResources<Storage>>,
{
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StorageID(u32);
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ArchetypeID(u32);

/// Provides an index that is unique with all other types that implement this trait for the same `T`.
///
/// It is typically implemented by gen_storage_for_world! macro.
pub trait LocalTypeIndex<T> {
    const TYPE_INDEX: TypeIndex;
}

pub type InArchetypeID = u32;

#[derive(Clone, Copy, Serialize, Deserialize)]
struct EntityInfo {
    pub archetype_id: ArchetypeID,
    pub in_archetype_id: InArchetypeID,
}

#[derive(Clone, Serialize, Deserialize)]
struct ArchetypeInfo {
    entities: Vec<EntityID>,
    component_slots: Box<[(TypeIndex, StorageID)]>,
}
impl ArchetypeInfo {
    fn len(&self) -> InArchetypeID {
        self.entities.len() as InArchetypeID
    }
}

type TypeBox = Box<[TypeIndex]>;

#[derive(Default, Clone, Serialize, Deserialize)]
struct ArchetypeManager {
    archetypes: Vec<ArchetypeInfo>,
    archetype_map: HashMap<TypeBox, ArchetypeID>,
}

impl ArchetypeManager {
    fn register_entity(&mut self, archetype: ArchetypeID, entity: EntityID) -> InArchetypeID {
        let archetype_info = &mut self.archetypes[archetype.0 as usize];
        let ret = archetype_info
            .entities
            .len()
            .try_into()
            .expect("no more than `InArchetypeId` expected per archetype");
        archetype_info.entities.push(entity);
        ret
    }
    fn find_archetype(&self, components: &[TypeIndex]) -> Option<ArchetypeID> {
        self.archetype_map.get(components).copied()
    }
    fn find_storage<Storage, C: Component<Storage>>(
        &self,
        archetype: ArchetypeID,
    ) -> Option<StorageID> {
        let index = C::TYPE_INDEX;
        let arche_info = self.archetypes.get(archetype.0 as usize)?;
        arche_info
            .component_slots
            .iter()
            .find(|(current_comp_index, _storage_id)| *current_comp_index == index)
            .map(|x| x.1)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct World<Storage> {
    entities: slotmapd::HopSlotMap<EntityID, EntityInfo>,
    archeman: ArchetypeManager,
    storage: Storage,

    changes_prev: ChangeManager<Storage, ReadOnly>,
    changes_new: ChangeManager<Storage, WriteOnly>,
}

impl<Storage: DynDispath + Default> Default for World<Storage> {
    fn default() -> Self {
        Self {
            entities: Default::default(),
            archeman: Default::default(),
            storage: Default::default(),
            changes_prev: Default::default(),
            changes_new: Default::default(),
        }
    }
}

impl<Storage: DynDispath + Default> World<Storage> {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<Storage: DynDispath> World<Storage> {
    fn allocate_storage(&mut self, component: TypeIndex) -> StorageID {
        self.storage.dispath_mut(component, |list| list.allocate())
    }

    fn create_archetype(&mut self, components: &[TypeIndex]) -> ArchetypeID {
        let archetype_id = ArchetypeID(
            self.archeman
                .archetypes
                .len()
                .try_into()
                .expect("less archetypes than IDs"),
        );
        self.archeman
            .archetype_map
            .insert(components.into(), archetype_id);

        let component_slots = components
            .iter()
            .map(|&component| (component, self.allocate_storage(component)))
            .collect();

        self.archeman.archetypes.push(ArchetypeInfo {
            entities: Vec::new(),
            component_slots,
        });
        archetype_id
    }
    fn find_or_create_archetype(&mut self, components: &[TypeIndex]) -> ArchetypeID {
        self.archeman
            .find_archetype(components)
            .unwrap_or_else(|| self.create_archetype(components))
    }

    fn remove_from_archetype(&mut self, ent_info: EntityInfo) {
        let arche_info = &mut self.archeman.archetypes[ent_info.archetype_id.0 as usize];

        let last_entity = arche_info.entities.len() - 1;
        let last_entity_id = arche_info.entities[last_entity];
        self.entities
            .get_mut(last_entity_id)
            .expect("should exist")
            .in_archetype_id = ent_info.in_archetype_id;

        arche_info
            .entities
            .swap_remove(ent_info.in_archetype_id as usize);

        for (type_index, storage_id) in arche_info.component_slots.iter() {
            self.storage.dispath_mut(*type_index, |list| {
                list.swap_remove(*storage_id, ent_info.in_archetype_id)
            });
        }
    }

    /// Spawn an entity with this bundle of components.
    pub fn spawn<B: Bundle<Storage>>(&mut self, bundle: B) -> EntityID {
        let mut components = B::type_ids();
        components.sort();
        let archetype = self.find_or_create_archetype(components.as_slice());
        bundle.add_to_archetype_in_storage(self, archetype);
        self.entities.insert_with_key(|entity| EntityInfo {
            archetype_id: archetype,
            in_archetype_id: self.archeman.register_entity(archetype, entity),
        })
    }

    pub fn entity_count(&self) -> u32 {
        self.entities.len() as u32
    }

    fn _despawn(&mut self, entity: EntityID) -> Option<()> {
        let ent_info = self.entities.get(entity)?;
        self.remove_from_archetype(*ent_info);
        self.entities.remove(entity);
        Some(())
    }

    pub fn despawn(&mut self, entity: EntityID) -> bool {
        self._despawn(entity).is_some()
    }

    pub fn get<C>(&self, entity: EntityID) -> Option<&C>
    where
        Storage: ComponentStorageProvider<C>,
        C: Component<Storage>,
    {
        let info = self.entities.get(entity)?;
        let storage_id = self
            .archeman
            .find_storage::<Storage, C>(info.archetype_id)?;
        self.storage.storage().get(storage_id, info.in_archetype_id)
    }

    pub fn get_mut<C>(&mut self, entity: EntityID) -> Option<&mut C>
    where
        Storage: ComponentStorageProvider<C>,
        C: Component<Storage>,
    {
        let info = self.entities.get(entity)?;
        let storage_id = self
            .archeman
            .find_storage::<Storage, C>(info.archetype_id)?;
        self.storage
            .storage_mut()
            .get_mut(storage_id, info.in_archetype_id)
    }

    pub fn resource<R>(&self) -> &R
    where
        Storage: ResourceStorageProvider<R>,
    {
        self.storage.storage().get()
    }

    pub fn resource_mut<R>(&mut self) -> &mut R
    where
        R: LocalTypeIndex<OfResources<Storage>>,
        Storage: ResourceStorageProvider<R>,
    {
        self.changes_new.mark_resource_as_changed(R::TYPE_INDEX);
        self.storage.storage_mut().get_mut()
    }

    /// Used by component bundles to add themselves to an archetype
    #[doc(hidden)]
    pub fn add_bundle_to_archetype<T>(&mut self, archetype: ArchetypeID, component: T)
    where
        Storage: ComponentStorageProvider<T>,
        T: Bundle<Storage> + LocalTypeIndex<Storage>,
    {
        let storage = self
            .archeman
            .find_storage::<Storage, T>(archetype)
            .expect("Required archetype exists");
        self.storage
            .storage_mut()
            .add_to_storage(storage, component)
    }

    pub fn query_world_shared(&self) -> query_world::QueryWorld<Storage> {
        QueryWorld::new(WorldRef::Shared(self))
    }

    pub fn query_world(&mut self) -> query_world::QueryWorld<Storage> {
        QueryWorld::new(WorldRef::Exclusive(self))
    }

    fn cycle_change_managers(&mut self) {
        self.changes_prev = mem::replace(&mut self.changes_new, Default::default()).to_read_only();
    }

    pub fn next_cycle(&mut self) {
        self.cycle_change_managers()
    }
}
