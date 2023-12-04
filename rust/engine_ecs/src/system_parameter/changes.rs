use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use crate::{
    ecs_cell::EcsCell,
    internal::{DynDispath, OfResources},
    LocalTypeIndex, TypeIndex,
};

use super::SystemParameter;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ReadOnly;
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct WriteOnly;

#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct ChangeManager<Storage, Rw> {
    changed_resources: Box<[EcsCell<bool>]>,
    _phantom: PhantomData<fn() -> (Storage, Rw)>,
}

impl<Storage: DynDispath, Rw> Default for ChangeManager<Storage, Rw> {
    fn default() -> Self {
        Self {
            changed_resources: vec![EcsCell::new(false); Storage::RESOURCE_TYPES as usize]
                .into_boxed_slice(),
            _phantom: PhantomData,
        }
    }
}

impl<Storage: DynDispath> ChangeManager<Storage, WriteOnly> {
    pub(crate) fn mark_resource_as_changed(&mut self, index: TypeIndex) {
        *self.changed_resources[index as usize].get_mut() = true;
    }

    /// # Safety.
    ///
    /// Assumes that we can mutate this resource, meaning that we have exclusive access to it.
    pub(crate) unsafe fn mark_resource_as_changed_unsafe(&self, index: TypeIndex) {
        unsafe { *self.changed_resources[usize::try_from(index).unwrap()].get_mut_unsafe() = true };
    }

    pub(crate) fn to_read_only(self) -> ChangeManager<Storage, ReadOnly> {
        ChangeManager {
            changed_resources: self.changed_resources,
            _phantom: PhantomData,
        }
    }
}

impl<Storage: DynDispath> ChangeManager<Storage, ReadOnly> {
    pub(crate) fn resource_changed(&self, index: TypeIndex) -> bool {
        *self.changed_resources[index as usize].get()
    }
}

pub struct ChangesG<'a, Storage> {
    change_manager: &'a ChangeManager<Storage, ReadOnly>,
}

impl<'a, Storage: DynDispath> ChangesG<'a, Storage> {
    pub fn resource_changed<Res>(&self) -> bool
    where
        Res: LocalTypeIndex<OfResources<Storage>>,
    {
        self.change_manager.resource_changed(Res::TYPE_INDEX)
    }
}

unsafe impl<'wrld, Storage: DynDispath> SystemParameter<'wrld, Storage>
    for ChangesG<'wrld, Storage>
{
    unsafe fn from_world(world: &'wrld crate::QueryWorld<'wrld, Storage>) -> Self {
        ChangesG {
            change_manager: &world.inner.changes_prev,
        }
    }
}
