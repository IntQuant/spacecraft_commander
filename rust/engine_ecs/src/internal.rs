use serde::{Deserialize, Serialize};

use crate::{ecs_cell::EcsCell, StorageID, TypeIndex};

pub use crate::component_traits::TypeIndexStorage;
pub use crate::system_parameter::{query::QueryParameter, ComponentRequests, SystemParameter};
pub use crate::InArchetypeID;
pub use smallvec::SmallVec;

pub trait ComponentStorageProvider<T> {
    fn storage(&self) -> &ComponentList<T>;
    fn storage_mut(&mut self) -> &mut ComponentList<T>;
}

pub trait DynDispath {
    const RESOURCE_TYPES: TypeIndex;

    fn dispath_mut<F, Ret>(&mut self, type_index: TypeIndex, f: F) -> Ret
    where
        F: FnOnce(&mut dyn DynComponentList) -> Ret;
}

pub trait DynComponentList {
    fn allocate(&mut self) -> StorageID;
    fn swap_remove(&mut self, storage: StorageID, index: InArchetypeID);
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ComponentList<T> {
    list: Vec<EcsCell<Vec<T>>>,
}

impl<T> Default for ComponentList<T> {
    fn default() -> Self {
        Self {
            list: Default::default(),
        }
    }
}

impl<T> ComponentList<T> {
    pub(crate) fn add_to_storage(&mut self, storage: StorageID, component: T) {
        self.list[storage.0 as usize].get_mut().push(component)
    }
    pub(crate) fn get(&self, storage: StorageID, index_in_arche: InArchetypeID) -> Option<&T> {
        self.list[storage.0 as usize]
            .get()
            .get(index_in_arche as usize)
    }
    pub(crate) fn get_mut(
        &mut self,
        storage: StorageID,
        index_in_arche: InArchetypeID,
    ) -> Option<&mut T> {
        self.list[storage.0 as usize]
            .get_mut()
            .get_mut(index_in_arche as usize)
    }
    pub(crate) unsafe fn get_mut_unsafe(
        &self,
        storage: StorageID,
        index_in_arche: InArchetypeID,
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
    fn swap_remove(&mut self, storage: StorageID, index: InArchetypeID) {
        self.list[storage.0 as usize]
            .get_mut()
            .swap_remove(index as usize);
    }
}

pub trait ResourceStorageProvider<T> {
    fn storage(&self) -> &ResourceStorage<T>;
    fn storage_mut(&mut self) -> &mut ResourceStorage<T>;
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct ResourceStorage<T> {
    inner: EcsCell<T>,
}

impl<T> ResourceStorage<T> {
    pub(crate) fn get(&self) -> &T {
        self.inner.get()
    }
    pub(crate) fn get_mut(&mut self) -> &mut T {
        self.inner.get_mut()
    }
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_mut_unsafe(&self) -> &mut T {
        unsafe { self.inner.get_mut_unsafe() }
    }
}

pub struct OfResources<T>(T);
