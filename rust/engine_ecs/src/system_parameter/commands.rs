use std::{marker::PhantomData, sync::Arc};

use crossbeam_queue::SegQueue;
use smallvec::SmallVec;

use crate::{internal::DynDispath, World};

use super::{ComponentRequests, SystemParameter};

pub type CommandFn<Storage> = dyn FnOnce(&mut World<Storage>);

type ParamIndex = usize;

pub(crate) type CommandBuffer<Storage> = Arc<SegQueue<(ParamIndex, Box<CommandFn<Storage>>)>>;

pub struct CommandsG<Storage> {
    pending: CommandBuffer<Storage>,
    param_index: ParamIndex,
    _phantom: PhantomData<fn() -> Storage>,
}

impl<Storage> CommandsG<Storage> {
    pub fn submit(&self, f: impl FnOnce(&mut World<Storage>) + 'static) {
        self.pending.push((self.param_index, Box::new(f)))
    }
}

unsafe impl<'a, Storage: DynDispath> SystemParameter<'a, Storage> for CommandsG<Storage> {
    fn requests() -> SmallVec<[ComponentRequests; 8]> {
        SmallVec::new()
    }

    unsafe fn from_world(world: &'a crate::query_world::QueryWorld<'a, Storage>) -> Self {
        assert!(
            world.exclusive(),
            "Commands can only be used with exclusive QueryWorld"
        );
        Self {
            pending: world.command_buffer.clone(),
            param_index: world.current_parameter_index(),
            _phantom: PhantomData,
        }
    }
}
