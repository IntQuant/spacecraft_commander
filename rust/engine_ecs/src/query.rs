use std::marker::PhantomData;

use crate::QueryWorld;

pub trait SystemParameter<Storage> {
    fn from_world<'a>(world: &QueryWorld<'a, Storage>) -> Self;
}

pub struct Query<'a, T, Storage> {
    world: &'a QueryWorld<'a, Storage>,
    _phantom: PhantomData<T>,
}
