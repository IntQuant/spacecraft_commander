use super::{QueryLimits, QueryParameter, SystemParameter};
use crate::{
    internal::DynDispath, query_world::QueryWorld, system_parameter::ComponentRequests,
    ArchetypeID, EntityID, InArchetypeID,
};
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
