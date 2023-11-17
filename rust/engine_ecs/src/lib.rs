use std::collections::HashMap;

use internal::DynDispath;
use slotmapd::new_key_type;

pub mod internal {
    use crate::{ArchetypeID, TypeIndex};

    pub trait ComponentStorageProvider<T> {
        fn storage_mut(&mut self) -> &mut ComponentList<T>;
    }

    pub trait DynDispath {
        fn dispath_mut<F, Ret, T>(&mut self, index: TypeIndex, f: F) -> Ret
        where
            F: FnOnce(&mut dyn DynComponentList) -> Ret;
    }

    pub trait DynComponentList {}

    pub struct ComponentList<T> {
        list: Vec<Vec<T>>,
    }

    impl<T> ComponentList<T> {
        pub(crate) fn add_to_archetype(&mut self, archetype: ArchetypeID, component: T) {
            self.list[archetype.0 as usize].push(component)
        }
    }

    impl<T> DynComponentList for ComponentList<T> {}
}

mod component_traits;

new_key_type! { pub struct EntityID; }

pub type TypeIndex = u32;

#[derive(Debug, Clone, Copy)]
struct StorageID(u32);
#[derive(Debug, Clone, Copy)]
pub struct ArchetypeID(u32);

/// Provides an index that is unique with all other types that implement this trait for the same `T`.
///
/// It is typically implemented by gen_storage_for_world! macro.
pub trait LocalTypeIndex<T> {
    const TYPE_INDEX: TypeIndex;
}

struct EntityInfo {
    pub archetype_id: ArchetypeID,
    pub in_archetype_id: u32,
}

struct ArchetypeInfo {
    entities: Vec<EntityID>,
    typeids: Box<[(TypeIndex, StorageID)]>,
}

pub struct World<Storage> {
    entities: slotmapd::HopSlotMap<EntityID, EntityInfo>,

    archetypes: Vec<ArchetypeInfo>,
    archetype_map: HashMap<Box<[TypeIndex]>, ArchetypeID>,

    storage: Storage,
}

impl<Storage: DynDispath> World<Storage> {
    fn create_archetype(&mut self, components: &[TypeIndex]) -> ArchetypeID {
        todo!()
    }
    fn get_or_create_archetype(&mut self, components: &[TypeIndex]) -> ArchetypeID {
        todo!()
    }
}
