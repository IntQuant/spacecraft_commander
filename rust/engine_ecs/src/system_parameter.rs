use std::marker::PhantomData;

use engine_macro::gen_query_param_tuple_impls;
use smallvec::SmallVec;

use crate::{
    ArchetypeID, ArchetypeInfo, Component, EntityID, InArchetypeID, QueryWorld, TypeIndex,
};

/// # Safety
///
/// Requests should cover all things that are accessed.
pub unsafe trait SystemParameter<'a, Storage> {
    fn requests() -> SmallVec<[ComponentRequests; 8]>;
    /// # Safety
    ///
    /// Assumes that requests do not "collide" with each other.
    unsafe fn from_world(world: &'a QueryWorld<'a, Storage>) -> Self;
}

/// Query that is Generic over storage.
pub struct QueryG<'wrld, Storage, Param: QueryParameter<'wrld, Storage>, Limits: QueryLimits = ()> {
    world: &'wrld QueryWorld<'wrld, Storage>,
    _phantom: PhantomData<fn() -> (Param, Limits)>,
}

unsafe impl<'wrld, T: QueryParameter<'wrld, Storage>, Limits: QueryLimits, Storage>
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

pub struct QueryIter<'a, 'wrld, Storage, Param: QueryParameter<'wrld, Storage>, Limits: QueryLimits>
{
    query: &'a mut QueryG<'wrld, Storage, Param, Limits>,
    arche_index: ArchetypeID,
    in_arche_index: InArchetypeID,
}

impl<'a, 'wrld, Storage, Param: QueryParameter<'wrld, Storage>, Limits: QueryLimits>
    QueryIter<'a, 'wrld, Storage, Param, Limits>
{
    fn skip_to_valid_arche(&mut self) {
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

impl<'a, 'wrld, Storage, Param: QueryParameter<'wrld, Storage>, Limits: QueryLimits> Iterator
    for QueryIter<'a, 'wrld, Storage, Param, Limits>
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

impl<'wrld, T, Limits, Storage> QueryG<'wrld, Storage, T, Limits>
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

    pub fn iter_old(&'wrld mut self) -> impl Iterator<Item = T> + 'wrld {
        let mut req = ComponentRequests::default();
        T::add_requests(&mut req);
        Limits::add_requests(&mut req);

        let world = &self.world;

        world
            .inner
            .archeman
            .archetypes
            .iter()
            .enumerate()
            .filter(move |(_, arche)| req.satisfied_by(arche))
            .flat_map(move |(arche_id, arche)| {
                arche.entities.iter().map(move |ent_id| {
                    let ent = world
                        .inner
                        .entities
                        .get(*ent_id)
                        .expect("entity exists, as it exists in archetype");
                    let arche_id = arche_id as u32;
                    assert_eq!(ent.archetype_id.0, arche_id);
                    // SAFERY: invariant checked when Query was created.
                    unsafe {
                        T::get_from_world(
                            world,
                            ArchetypeID(arche_id),
                            ent.in_archetype_id,
                            *ent_id,
                        )
                    }
                })
            })
    }
}

#[derive(Debug)]
pub(crate) struct Request {
    type_index: TypeIndex,
    exclusive: bool,
}

impl Request {
    fn new(type_index: TypeIndex, exclusive: bool) -> Self {
        Self {
            type_index,
            exclusive,
        }
    }
}

#[derive(Debug, Default)]
pub struct ComponentRequests {
    pub(crate) requests: SmallVec<[Request; 8]>,
    pub(crate) filter_require: SmallVec<[TypeIndex; 8]>,
    pub(crate) filter_exclude: SmallVec<[TypeIndex; 8]>,
    pub(crate) resource_requests: Vec<Request>,
}

impl ComponentRequests {
    /// Request to use this component, either exclusively or shared.
    pub fn request(&mut self, type_index: TypeIndex, exclusive: bool) {
        let new_request = Request::new(type_index, exclusive);
        match self
            .requests
            .binary_search_by_key(&type_index, |req| req.type_index)
        {
            Ok(ind) => {
                let req = &self.requests[ind];
                if req.exclusive != exclusive {
                    panic!("Conflicting requests: type with index {type_index} requested as shared and exclusive at the same time");
                }
            }
            Err(ind) => self.requests.insert(ind, new_request),
        }
    }
    /// Require this component to be present.
    pub fn require(&mut self, type_index: TypeIndex) {
        match self.filter_require.binary_search(&type_index) {
            Ok(_ind) => {}
            Err(ind) => {
                self.filter_require.insert(ind, type_index);
            }
        }
    }
    /// Require this component to be absent.
    pub fn exclude(&mut self, type_index: TypeIndex) {
        match self.filter_exclude.binary_search(&type_index) {
            Ok(_ind) => {}
            Err(ind) => {
                self.filter_exclude.insert(ind, type_index);
            }
        }
    }

    pub fn request_resource(&mut self, type_index: TypeIndex, exclusive: bool) {
        let new_request = Request::new(type_index, exclusive);
        match self
            .resource_requests
            .binary_search_by_key(&type_index, |req| req.type_index)
        {
            Ok(ind) => {
                let req = &self.resource_requests[ind];
                if req.exclusive != exclusive {
                    panic!("Conflicting resource requests: type with index {type_index} requested as shared and exclusive at the same time");
                }
            }
            Err(ind) => self.resource_requests.insert(ind, new_request),
        }
    }

    fn _disjoint_half(&self, other: &Self) -> bool {
        let mut other_index = 0;
        if other.filter_require.is_empty() {
            return false;
        }
        for &e in &self.filter_exclude {
            while other.filter_require[other_index] < e {
                other_index += 1;
                if other_index == other.filter_require.len() {
                    return false;
                }
            }
            if e == other.filter_require[other_index] {
                return true;
            }
        }
        false
    }

    /// Returns true if those request can never access the same archetype.
    pub fn disjoint_with(&self, other: &Self) -> bool {
        self._disjoint_half(other) || other._disjoint_half(self)
    }

    /// Returns true if both requests need access to a component and at least one of requests needs exclusive access.
    pub fn conflicts_with(&self, other: &Self) -> bool {
        let mut other_index = 0;
        if other.requests.is_empty() {
            return false;
        }
        for e in &self.requests {
            while other.requests[other_index].type_index < e.type_index {
                other_index += 1;
                if other_index == other.requests.len() {
                    return false;
                }
            }
            if e.type_index == other.requests[other_index].type_index
                && (e.exclusive || other.requests[other_index].exclusive)
            {
                return true;
            }
        }
        false
    }

    /// Returns true if both requests need access to a resource and at least one of requests needs exclusive access.
    pub fn resource_conflicts_with(&self, other: &Self) -> bool {
        let mut other_index = 0;
        if other.resource_requests.is_empty() {
            return false;
        }
        for e in &self.resource_requests {
            while other.resource_requests[other_index].type_index < e.type_index {
                other_index += 1;
                if other_index == other.resource_requests.len() {
                    return false;
                }
            }
            if e.type_index == other.resource_requests[other_index].type_index
                && (e.exclusive || other.resource_requests[other_index].exclusive)
            {
                return true;
            }
        }
        false
    }

    /// Returns true if both requests can be satisfied at the same time.
    pub fn safe_with(&self, other: &Self) -> bool {
        (!self.conflicts_with(other) || self.disjoint_with(other))
            && !self.resource_conflicts_with(other)
    }

    fn satisfied_by(&self, by: &ArchetypeInfo) -> bool {
        self.require_satisfied(by) && self.exclude_satisfied(by)
    }

    fn require_satisfied(&self, by: &ArchetypeInfo) -> bool {
        let mut other_index = 0;
        if by.component_slots.is_empty() {
            return self.filter_require.is_empty();
        }
        for e in &self.filter_require {
            while by.component_slots[other_index].0 < *e {
                other_index += 1;
                if other_index == by.component_slots.len() {
                    return false;
                }
            }
            if *e != by.component_slots[other_index].0 {
                return false;
            }
        }
        true
    }

    fn exclude_satisfied(&self, by: &ArchetypeInfo) -> bool {
        let mut other_index = 0;
        if by.component_slots.is_empty() {
            return true;
        }
        for e in &self.filter_exclude {
            while by.component_slots[other_index].0 < *e {
                other_index += 1;
                if other_index == by.component_slots.len() {
                    return true;
                }
            }
            if *e == by.component_slots[other_index].0 {
                return false;
            }
        }
        true
    }
}

/// # Safety
///
/// Requests should cover all components that are accessed.
pub unsafe trait QueryParameter<'wrld, Storage> {
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

unsafe impl<'wrld, Storage> QueryParameter<'wrld, Storage> for EntityID {
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

#[cfg(test)]
mod tests {
    use super::ComponentRequests;

    #[test]
    fn req_disjoint() {
        let mut req1 = ComponentRequests::default();
        let mut req2 = ComponentRequests::default();
        let mut req3 = ComponentRequests::default();
        let mut req4 = ComponentRequests::default();
        let req_empty = ComponentRequests::default();

        req1.require(0);

        req2.require(0);
        req2.require(1);

        req3.require(0);
        req3.exclude(1);

        req4.require(1);

        assert!(!req1.disjoint_with(&req2));
        assert!(!req1.disjoint_with(&req3));
        assert!(req2.disjoint_with(&req3));
        assert!(req3.disjoint_with(&req4));

        assert!(!req3.disjoint_with(&req_empty));
        assert!(!req_empty.disjoint_with(&req3));
    }

    #[test]
    fn req_conflict() {
        let mut req1 = ComponentRequests::default();
        let mut req2 = ComponentRequests::default();
        let mut req3 = ComponentRequests::default();
        let mut req4 = ComponentRequests::default();

        req1.request(0, false);
        req1.require(0);

        req2.request(0, false);
        req2.require(0);
        req2.require(1);

        req3.request(0, true);
        req3.require(0);
        req3.exclude(1);

        req4.request(1, true);
        req4.require(1);

        assert!(req1.conflicts_with(&req3));
        assert!(req2.conflicts_with(&req3));
        assert!(!req1.conflicts_with(&req2));
        assert!(!req3.conflicts_with(&req4));
    }

    #[test]
    fn req_resource() {
        let mut req1 = ComponentRequests::default();
        let mut req2 = ComponentRequests::default();
        let mut req3 = ComponentRequests::default();
        let mut req4 = ComponentRequests::default();

        req1.request_resource(0, false);
        req2.request_resource(0, false);
        req3.request_resource(0, true);
        req4.request_resource(1, true);

        assert!(req1.resource_conflicts_with(&req3));
        assert!(req2.resource_conflicts_with(&req3));
        assert!(!req1.resource_conflicts_with(&req2));
        assert!(!req3.resource_conflicts_with(&req4));
    }

    #[test]
    fn req_safe() {
        let mut req1 = ComponentRequests::default();
        let mut req2 = ComponentRequests::default();
        let mut req3 = ComponentRequests::default();
        let mut req4 = ComponentRequests::default();

        req1.request(0, false);
        req1.require(0);

        req2.request(0, false);
        req2.require(0);
        req2.require(1);

        req3.request(0, true);
        req3.require(0);
        req3.exclude(1);

        req4.request(1, true);
        req4.require(1);

        assert!(!req1.safe_with(&req3));
        assert!(req2.safe_with(&req3));
        assert!(req1.safe_with(&req2));
        assert!(req3.safe_with(&req4));
    }
}
