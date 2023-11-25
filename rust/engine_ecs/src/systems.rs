use engine_macro::gen_fn_system_impls;
use smallvec::SmallVec;

use crate::{
    system_parameter::{ComponentRequests, SystemParameter},
    QueryWorld,
};

pub trait System<Storage> {
    fn requests() -> SmallVec<[ComponentRequests; 8]>;
    fn run(self, world: &QueryWorld<Storage>);
}

impl<Storage, P0> System<Storage> for fn(P0) -> ()
where
    P0: SystemParameter<'a, Storage>,
{
    fn requests() -> SmallVec<[ComponentRequests; 8]> {
        P0::requests()
    }

    fn run(self, world: &QueryWorld<Storage>) {
        let p0: P0 = world.parameter();
        self(p0);
    }
}

// gen_fn_system_impls!(0);
// gen_fn_system_impls!(1);
// gen_fn_system_impls!(2);
// gen_fn_system_impls!(3);
// gen_fn_system_impls!(4);
// gen_fn_system_impls!(5);
// gen_fn_system_impls!(6);
