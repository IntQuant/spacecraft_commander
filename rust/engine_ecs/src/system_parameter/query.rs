use super::SystemParameter;
use crate::{
    internal::DynDispath, query_world::QueryWorld, system_parameter::ComponentRequests,
    ArchetypeID, Component, EntityID, InArchetypeID,
};
use engine_macro::gen_query_param_tuple_impls;
use smallvec::SmallVec;
use std::marker::PhantomData;

/// Query that is Generic over storage.
pub struct QueryG<
    'wrld,
    Storage: DynDispath,
    Param: QueryParameter<'wrld, Storage>,
    Limits: QueryLimits = (),
> {
    pub(crate) world: &'wrld QueryWorld<'wrld, Storage>,
    pub(crate) _phantom: PhantomData<fn() -> (Param, Limits)>,
}

unsafe impl<'wrld, T: QueryParameter<'wrld, Storage>, Limits: QueryLimits, Storage: DynDispath>
    SystemParameter<'wrld, Storage> for QueryG<'wrld, Storage, T, Limits>
{
    fn requests() -> SmallVec<[ComponentRequests; 8]> {
        let mut req_vec = SmallVec::new();
        let mut req = ComponentRequests::default();
        T::add_requests(&mut req);
        Limits::add_requests(&mut req);
        req_vec.push(req);
        req_vec
    }

    unsafe fn from_world(world: &'wrld QueryWorld<'wrld, Storage>) -> Self {
        QueryG {
            world,
            _phantom: PhantomData,
        }
    }
}

pub struct QueryIter<
    'a,
    'wrld,
    Storage: DynDispath,
    Param: QueryParameter<'wrld, Storage>,
    Limits: QueryLimits,
> {
    pub(crate) query: &'a mut QueryG<'wrld, Storage, Param, Limits>,
    pub(crate) arche_index: ArchetypeID,
    pub(crate) in_arche_index: InArchetypeID,
}

impl<
        'a,
        'wrld,
        Storage: DynDispath,
        Param: QueryParameter<'wrld, Storage>,
        Limits: QueryLimits,
    > QueryIter<'a, 'wrld, Storage, Param, Limits>
{
    pub(crate) fn skip_to_valid_arche(&mut self) {
        let mut req = ComponentRequests::default();
        Param::add_requests(&mut req);
        Limits::add_requests(&mut req);

        while let Some(arche) = self
            .query
            .world
            .inner
            .archeman
            .archetypes
            .get(self.arche_index.0 as usize)
        {
            if !arche.entities.is_empty() && req.satisfied_by(arche) {
                break;
            } else {
                self.arche_index.0 += 1;
            }
        }
    }
}

impl<
        'a,
        'wrld,
        Storage: DynDispath,
        Param: QueryParameter<'wrld, Storage>,
        Limits: QueryLimits,
    > Iterator for QueryIter<'a, 'wrld, Storage, Param, Limits>
{
    type Item = Param;

    fn next(&mut self) -> Option<Self::Item> {
        let archeman = &self.query.world.inner.archeman;
        if self.arche_index.0 as usize >= archeman.archetypes.len() {
            return None;
        }
        let ent_id =
            archeman.archetypes[self.arche_index.0 as usize].entities[self.in_arche_index as usize];

        let param = unsafe {
            Param::get_from_world(
                self.query.world,
                self.arche_index,
                self.in_arche_index,
                ent_id,
            )
        };
        self.in_arche_index += 1;
        if self.in_arche_index >= archeman.archetypes[self.arche_index.0 as usize].len() {
            self.in_arche_index = 0;
            self.arche_index.0 += 1;
            self.skip_to_valid_arche();
        }

        Some(param)
    }
}

impl<'wrld, T, Limits, Storage: DynDispath> QueryG<'wrld, Storage, T, Limits>
where
    T: QueryParameter<'wrld, Storage>,
    Limits: QueryLimits,
{
    pub fn get(&mut self, ent: EntityID) -> Option<T> {
        let ent_info = self.world.inner.entities.get(ent)?;
        // SAFERY: invariant checked when Query was created.
        Some(unsafe {
            T::get_from_world(
                self.world,
                ent_info.archetype_id,
                ent_info.in_archetype_id,
                ent,
            )
        })
    }

    pub fn iter(&mut self) -> QueryIter<'_, 'wrld, Storage, T, Limits> {
        let mut query_iter = QueryIter {
            query: self,
            arche_index: ArchetypeID(0),
            in_arche_index: 0,
        };
        query_iter.skip_to_valid_arche();
        query_iter
    }
}

/// # Safety
///
/// Requests should cover all components that are accessed.
pub unsafe trait QueryParameter<'wrld, Storage: DynDispath> {
    fn add_requests(req: &mut ComponentRequests);
    /// # Safety
    ///
    /// Assumes that requests do not "collide" with each other.
    unsafe fn get_from_world(
        world: &'wrld QueryWorld<'wrld, Storage>,
        archetype: ArchetypeID,
        index: InArchetypeID,
        ent_id: EntityID,
    ) -> Self;
}

unsafe impl<'wrld, Storage: DynDispath> QueryParameter<'wrld, Storage> for EntityID {
    fn add_requests(_req: &mut ComponentRequests) {}

    unsafe fn get_from_world(
        _world: &QueryWorld<Storage>,
        _archetype: ArchetypeID,
        _index: InArchetypeID,
        ent_id: EntityID,
    ) -> Self {
        ent_id
    }
}

gen_query_param_tuple_impls!(1);
gen_query_param_tuple_impls!(2);
gen_query_param_tuple_impls!(3);
gen_query_param_tuple_impls!(4);
gen_query_param_tuple_impls!(5);
gen_query_param_tuple_impls!(6);

pub trait QueryLimits {
    fn add_requests(req: &mut ComponentRequests);
}

impl QueryLimits for () {
    fn add_requests(_req: &mut ComponentRequests) {}
}

pub struct WithG<Storage, T: Component<Storage>>(PhantomData<fn() -> (T, Storage)>);
pub struct WithoutG<Storage, T: Component<Storage>>(PhantomData<fn() -> (T, Storage)>);

impl<Storage, T: Component<Storage>> QueryLimits for WithG<Storage, T> {
    fn add_requests(req: &mut ComponentRequests) {
        req.require(T::TYPE_INDEX);
    }
}
impl<Storage, T: Component<Storage>> QueryLimits for WithoutG<Storage, T> {
    fn add_requests(req: &mut ComponentRequests) {
        req.exclude(T::TYPE_INDEX);
    }
}
