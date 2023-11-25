use engine_macro::gen_fn_system_impls;
use smallvec::SmallVec;

use crate::{
    system_parameter::{ComponentRequests, SystemParameter},
    QueryWorld,
};

pub trait System<'wrld: 'a, 'a, Storage> {
    fn requests() -> SmallVec<[ComponentRequests; 8]>;
    fn run(self, world: &'a QueryWorld<'wrld, Storage>);
}

impl<'wrld: 'a, 'a, Storage, P0> System<'wrld, 'a, Storage> for fn(P0) -> ()
where
    P0: SystemParameter<'wrld, 'a, Storage>,
{
    fn requests() -> SmallVec<[ComponentRequests; 8]> {
        P0::requests()
    }

    fn run(self, world: &'a QueryWorld<'wrld, Storage>) {
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
