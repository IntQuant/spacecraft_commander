use super::{ArchetypeID, InArchetypeID, StorageID, World};
use crate::{
    component_traits::Component,
    internal::{ComponentStorageProvider, DynDispath, OfResources, ResourceStorageProvider},
    system_parameter::{commands::CommandBuffer, ComponentRequests, SystemParameter},
    LocalTypeIndex,
};
use engine_macro::gen_world_run_impls;
use std::{
    cell::RefCell,
    ops::{Deref, DerefMut, Range},
};

pub(crate) enum WorldRef<'wrld, Storage> {
    Shared(&'wrld World<Storage>),
    Exclusive(&'wrld mut World<Storage>),
}

impl<'wrld, Storage> WorldRef<'wrld, Storage> {
    fn ref_mut(&mut self) -> &mut World<Storage> {
        match self {
            WorldRef::Shared(_) => panic!("Can't get mut reference from shared kind of WorldRef"),
            WorldRef::Exclusive(world) => world,
        }
    }
}

impl<'wrld, Storage> Deref for WorldRef<'wrld, Storage> {
    type Target = World<Storage>;

    fn deref(&self) -> &Self::Target {
        match self {
            WorldRef::Shared(world) => world,
            WorldRef::Exclusive(world) => world,
        }
    }
}

pub struct QueryWorld<'wrld, Storage: DynDispath> {
    pub(crate) inner: WorldRef<'wrld, Storage>,
    currently_requested: RefCell<Vec<ComponentRequests>>,
    parameter_index: RefCell<usize>,
    pub(crate) command_buffer: CommandBuffer<Storage>,
}

impl<'wrld, Storage: DynDispath> Drop for QueryWorld<'wrld, Storage> {
    fn drop(&mut self) {
        let mut cmd_buf = Vec::with_capacity(self.command_buffer.len());
        while let Some(cmd) = self.command_buffer.pop() {
            cmd_buf.push(cmd);
        }
        cmd_buf.sort_by_key(|x| x.0);
        for (_key, cmd) in cmd_buf {
            cmd(self.inner.ref_mut())
        }
    }
}

pub struct ParamGuard<'wlrd, 'a, Storage: DynDispath, Param> {
    pub(crate) inner: Param,
    pub(crate) world: &'a QueryWorld<'wlrd, Storage>,
    pub(crate) holds: Range<usize>,
}

impl<'wlrd, 'a, Storage: DynDispath, Param> Deref for ParamGuard<'wlrd, 'a, Storage, Param> {
    type Target = Param;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'wlrd, 'a, Storage: DynDispath, Param> DerefMut for ParamGuard<'wlrd, 'a, Storage, Param> {
    fn deref_mut(&mut self) -> &mut Param {
        &mut self.inner
    }
}

impl<'wlrd, 'a, Storage: DynDispath, Param> Drop for ParamGuard<'wlrd, 'a, Storage, Param> {
    fn drop(&mut self) {
        self.world.release_parameter(self.holds.clone())
    }
}

impl<'wrld, Storage: DynDispath> QueryWorld<'wrld, Storage> {
    pub(crate) fn new(world_ref: WorldRef<'wrld, Storage>) -> Self {
        QueryWorld {
            inner: world_ref,
            currently_requested: Default::default(),
            parameter_index: 0.into(),
            command_buffer: CommandBuffer::default(),
        }
    }

    pub fn exclusive(&self) -> bool {
        matches!(self.inner, WorldRef::Exclusive(..))
    }

    /// # Safety
    ///
    /// Aliasing rules have to be upheld per StorageID and component type.
    pub unsafe fn get<T: Component<Storage>>(
        &self,
        storage: StorageID,
        index_in_arche: InArchetypeID,
    ) -> Option<&T>
    where
        Storage: ComponentStorageProvider<T>,
    {
        self.inner.storage.storage().get(storage, index_in_arche)
    }
    /// # Safety
    ///
    /// See `get` method.
    /// Not safe to call when exclusive is false.
    pub unsafe fn get_mut<T>(
        &self,
        storage: StorageID,
        index_in_arche: InArchetypeID,
    ) -> Option<&mut T>
    where
        Storage: ComponentStorageProvider<T>,
    {
        unsafe {
            self.inner
                .storage
                .storage()
                .get_mut_unsafe(storage, index_in_arche)
        }
    }
    pub fn storage_for_archetype<T: Component<Storage>>(
        &self,
        archetype: ArchetypeID,
    ) -> Option<StorageID> {
        self.inner.archeman.find_storage::<Storage, T>(archetype)
    }

    /// # Safety
    ///
    /// Aliasing rules have to be upheld per resource type.
    pub unsafe fn resource<R>(&self) -> &R
    where
        Storage: ResourceStorageProvider<R>,
    {
        self.inner.storage.storage().get()
    }

    /// # Safety
    ///
    /// See `resource` method.
    /// Not safe to call when exclusive is false.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn resource_mut<R>(&self) -> &mut R
    where
        R: LocalTypeIndex<OfResources<Storage>>,
        Storage: ResourceStorageProvider<R>,
    {
        unsafe {
            self.inner
                .changes_new
                .mark_resource_as_changed_unsafe(R::TYPE_INDEX);
            self.inner.storage.storage().get_mut_unsafe()
        }
    }

    pub fn parameter<Param: SystemParameter<'wrld, Storage>>(
        &'wrld self,
    ) -> ParamGuard<Storage, Param> {
        let (holds, param) = self.parameter_raw();
        ParamGuard {
            inner: param,
            world: self,
            holds,
        }
    }

    pub(crate) fn parameter_raw<Param: SystemParameter<'wrld, Storage>>(
        &'wrld self,
    ) -> (Range<usize>, Param) {
        *self.parameter_index.borrow_mut() += 1;
        let requests = Param::requests();
        let req_start = self.currently_requested.borrow().len();
        for new_request in requests {
            for current_request in self.currently_requested.borrow().iter() {
                if !self.exclusive() && new_request.any_exclusive() {
                    panic!(
                        "Exlusive (&mut) parameters are not supported in non-exclusive QueryWorld"
                    );
                }
                if !new_request.safe_with(current_request) {
                    panic!(
                        "{:?} and {:?} are incompatible, and thus cannot be used at the same time.",
                        current_request, new_request
                    );
                }
            }
            self.currently_requested.borrow_mut().push(new_request);
        }
        let req_end = self.currently_requested.borrow().len();
        let req_range = req_start..req_end;
        // Safety: checked that requests are satisfied.
        (req_range, unsafe { Param::from_world(self) })
    }

    pub(crate) fn release_parameter(&self, holds: Range<usize>) {
        for i in holds {
            self.currently_requested.borrow_mut()[i].release();
        }
    }

    pub(crate) fn current_parameter_index(&self) -> usize {
        *self.parameter_index.borrow()
    }
}

pub trait WorldRun<'wrld, F, Ret, P> {
    fn run(&'wrld self, f: F) -> Ret;
}

gen_world_run_impls!(0);
gen_world_run_impls!(1);
gen_world_run_impls!(2);
gen_world_run_impls!(3);
gen_world_run_impls!(4);
gen_world_run_impls!(5);
gen_world_run_impls!(6);
gen_world_run_impls!(7);
gen_world_run_impls!(8);
gen_world_run_impls!(9);
gen_world_run_impls!(10);
gen_world_run_impls!(11);
gen_world_run_impls!(12);
