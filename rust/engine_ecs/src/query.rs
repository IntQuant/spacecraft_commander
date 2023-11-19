use std::marker::PhantomData;

use engine_macro::gen_query_param_tuple_impls;
use smallvec::{Array, SmallVec};

use crate::{ArchetypeID, InArchetypeId, QueryWorld, TypeIndex};

pub trait SystemParameter<Storage> {
    fn from_world<'a>(world: &QueryWorld<'a, Storage>) -> Self;
}

pub struct Query<'a, T: QueryParameter<'a, Storage>, Storage> {
    world: &'a QueryWorld<'a, Storage>,
    _phantom: PhantomData<T>,
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

    fn _disjoint_half(&self, other: &Self) -> bool {
        let mut other_index = 0;
        if other.filter_require.len() == 0 {
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
        if other.requests.len() == 0 {
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

    /// Returns true if both requests can be satisfied at the same time.
    pub fn safe_with(&self, other: &Self) -> bool {
        !self.conflicts_with(other) || self.disjoint_with(other)
    }
}

/// SAFETY: requests should cover all components that are accessed.
pub unsafe trait QueryParameter<'a, Storage> {
    fn add_requests(req: &mut ComponentRequests);
    /// SAFETY: assumes that requests do not "collide" with each other.
    unsafe fn get_from_world(
        world: &'a QueryWorld<'a, Storage>,
        archetype: ArchetypeID,
        index: InArchetypeId,
    ) -> Self;
}

gen_query_param_tuple_impls!(1);
gen_query_param_tuple_impls!(2);
gen_query_param_tuple_impls!(3);
gen_query_param_tuple_impls!(4);
gen_query_param_tuple_impls!(5);
gen_query_param_tuple_impls!(6);

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
