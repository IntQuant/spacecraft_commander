use std::collections::HashMap;

use internal::{ComponentStorageProvider, DynDispath};
use serde::{Deserialize, Serialize};
use slotmapd::new_key_type;

pub mod internal {
    use serde::{Deserialize, Serialize};

    use crate::{ArchetypeID, InArchetypeId, StorageID, TypeIndex};

    pub trait ComponentStorageProvider<T> {
        fn storage(&self) -> &ComponentList<T>;
        fn storage_mut(&mut self) -> &mut ComponentList<T>;
    }

    pub trait DynDispath {
        fn dispath_mut<F, Ret>(&mut self, index: TypeIndex, f: F) -> Ret
        where
            F: FnOnce(&mut dyn DynComponentList) -> Ret;
    }

    pub trait DynComponentList {
        fn allocate(&mut self) -> StorageID;
    }

    #[derive(Default, Clone, Serialize, Deserialize)]
    pub struct ComponentList<T> {
        list: Vec<Vec<T>>,
    }

    impl<T> ComponentList<T> {
        pub(crate) fn add_to_storage(&mut self, storage: StorageID, component: T) {
            self.list[storage.0 as usize].push(component)
        }
        pub(crate) fn get(&self, storage: StorageID, index_in_arche: InArchetypeId) -> Option<&T> {
            self.list[storage.0 as usize].get(index_in_arche as usize)
        }
        pub(crate) fn get_mut(
            &mut self,
            storage: StorageID,
            index_in_arche: InArchetypeId,
        ) -> Option<&mut T> {
            self.list[storage.0 as usize].get_mut(index_in_arche as usize)
        }
    }

    impl<T> DynComponentList for ComponentList<T> {
        fn allocate(&mut self) -> StorageID {
            let ret = StorageID(
                self.list
                    .len()
                    .try_into()
                    .expect("Less archetypes use this component than IDs available"),
            );
            self.list.push(Vec::new());
            ret
        }
    }
}

mod component_traits;

pub use crate::component_traits::{Bundle, BundleTrait, Component};

new_key_type! { pub struct EntityID; }

pub type TypeIndex = u32;

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

type InArchetypeId = u32;

#[derive(Clone, Serialize, Deserialize)]
struct EntityInfo {
    pub archetype_id: ArchetypeID,
    pub in_archetype_id: InArchetypeId,
}

#[derive(Clone, Serialize, Deserialize)]
struct ArchetypeInfo {
    entities: Vec<EntityID>,
    component_slots: Box<[(TypeIndex, StorageID)]>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
struct ArchetypeManager {
    archetypes: Vec<ArchetypeInfo>,
    archetype_map: HashMap<Box<[TypeIndex]>, ArchetypeID>,
}

impl ArchetypeManager {
    fn register_entity(&mut self, archetype: ArchetypeID, entity: EntityID) -> InArchetypeId {
        let archetype_info = &mut self.archetypes[archetype.0 as usize];
        let ret = archetype_info
            .entities
            .len()
            .try_into()
            .expect("No more than `InArchetypeId` expected per archetype");
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
            .find(|(current_comp_index, storage_id)| *current_comp_index == index)
            .map(|x| x.1)
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct World<Storage> {
    entities: slotmapd::HopSlotMap<EntityID, EntityInfo>,
    archeman: ArchetypeManager,
    storage: Storage,
}

impl<Storage: DynDispath + Default> World<Storage> {
    pub fn new() -> Self {
        Self {
            entities: Default::default(),
            archeman: Default::default(),
            storage: Default::default(),
        }
    }

    fn allocate_storage(&mut self, component: TypeIndex) -> StorageID {
        self.storage.dispath_mut(component, |list| list.allocate())
    }

    fn create_archetype(&mut self, components: &[TypeIndex]) -> ArchetypeID {
        let archetype_id = ArchetypeID(
            self.archeman
                .archetypes
                .len()
                .try_into()
                .expect("Less archetypes than IDs"),
        );
        self.archeman
            .archetype_map
            .insert(components.into(), archetype_id);

        let component_slots = components
            .into_iter()
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

    fn _spawn<B: BundleTrait<Storage>>(&mut self, bundle: B) -> EntityID {
        let mut components = B::type_ids();
        components.sort();
        let archetype = self.find_or_create_archetype(components.as_slice());
        bundle.add_to_archetype_in_storage(self, archetype);
        self.entities.insert_with_key(|entity| EntityInfo {
            archetype_id: archetype,
            in_archetype_id: self.archeman.register_entity(archetype, entity),
        })
    }

    /// Spawn an entity with this bundle of components.
    pub fn spawn<B0, B1>(&mut self, bundle: impl Into<Bundle<B0, B1, Storage>>) -> EntityID
    where
        Bundle<B0, B1, Storage>: BundleTrait<Storage>,
    {
        let bundle = bundle.into();
        self._spawn(bundle)
    }

    pub fn entity_count(&self) -> u32 {
        self.entities.len() as u32
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

    fn add_bundle_to_archetype<T>(&mut self, archetype: ArchetypeID, component: T)
    where
        Storage: ComponentStorageProvider<T>,
        T: Into<Bundle<T, (), Storage>> + LocalTypeIndex<Storage>,
    {
        let storage = self
            .archeman
            .find_storage::<Storage, T>(archetype)
            .expect("Required archetype exists");
        self.storage
            .storage_mut()
            .add_to_storage(storage, component)
    }
}
