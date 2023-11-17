use std::collections::HashMap;

use internal::DynDispath;
use serde::{Deserialize, Serialize};
use slotmapd::new_key_type;

pub mod internal {
    use serde::{Deserialize, Serialize};

    use crate::{ArchetypeID, StorageID, TypeIndex};

    pub trait ComponentStorageProvider<T> {
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
        pub(crate) fn add_to_archetype(&mut self, archetype: ArchetypeID, component: T) {
            self.list[archetype.0 as usize].push(component)
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
        bundle.add_to_archetype_in_storage(&mut self.storage, archetype);
        self.entities.insert_with_key(|entity| EntityInfo {
            archetype_id: archetype,
            in_archetype_id: self.archeman.register_entity(archetype, entity),
        })
    }

    pub fn spawn<B0, B1>(&mut self, bundle: impl Into<Bundle<B0, B1, Storage>>) -> EntityID
    where
        Bundle<B0, B1, Storage>: BundleTrait<Storage>,
    {
        let bundle = bundle.into();
        self._spawn(bundle)
    }
}
