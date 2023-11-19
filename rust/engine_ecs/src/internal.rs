use serde::{Deserialize, Serialize};

use crate::{ecs_cell::EcsCell, StorageID, TypeIndex};

pub use crate::component_traits::TypeIndexStorage;
pub use crate::query::{ComponentRequests, QueryParameter};
pub use crate::InArchetypeId;

pub trait ComponentStorageProvider<T> {
    fn storage(&self) -> &ComponentList<T>;
    fn storage_mut(&mut self) -> &mut ComponentList<T>;
}

pub trait DynDispath {
    fn dispath_mut<F, Ret>(&mut self, type_index: TypeIndex, f: F) -> Ret
    where
        F: FnOnce(&mut dyn DynComponentList) -> Ret;
}

pub trait DynComponentList {
    fn allocate(&mut self) -> StorageID;
    fn swap_remove(&mut self, storage: StorageID, index: InArchetypeId);
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ComponentList<T> {
    list: Vec<EcsCell<Vec<T>>>,
}

impl<T> ComponentList<T> {
    pub(crate) fn add_to_storage(&mut self, storage: StorageID, component: T) {
        self.list[storage.0 as usize].get_mut().push(component)
    }
    pub(crate) fn get(&self, storage: StorageID, index_in_arche: InArchetypeId) -> Option<&T> {
        self.list[storage.0 as usize]
            .get()
            .get(index_in_arche as usize)
    }
    pub(crate) fn get_mut(
        &mut self,
        storage: StorageID,
        index_in_arche: InArchetypeId,
    ) -> Option<&mut T> {
        self.list[storage.0 as usize]
            .get_mut()
            .get_mut(index_in_arche as usize)
    }
    pub(crate) unsafe fn get_mut_unsafe(
        &self,
        storage: StorageID,
        index_in_arche: InArchetypeId,
    ) -> Option<&mut T> {
        unsafe {
            self.list[storage.0 as usize]
                .get_mut_unsafe()
                .get_mut(index_in_arche as usize)
        }
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
        self.list.push(EcsCell::new(Vec::new()));
        ret
    }
    fn swap_remove(&mut self, storage: StorageID, index: InArchetypeId) {
        self.list[storage.0 as usize]
            .get_mut()
            .swap_remove(index as usize);
    }
}
