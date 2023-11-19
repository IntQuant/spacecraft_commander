use std::marker::PhantomData;

use smallvec::{Array, SmallVec};

use crate::{ArchetypeID, InArchetypeId, QueryWorld, TypeIndex};

pub trait SystemParameter<Storage> {
    fn from_world<'a>(world: &QueryWorld<'a, Storage>) -> Self;
}

pub struct Query<'a, T: QueryParameter<Storage>, Storage> {
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
    pub fn conflics_with(&self, other: &Self) -> bool {
        todo!()
    }

    /// Returns true if both requirements can be satisfied at the same time.
    pub fn safe_with(&self, other: &Self) -> bool {
        todo!()
    }
}

/// SAFETY: requests should cover all components that are accessed.
pub unsafe trait QueryParameter<Storage> {
    fn add_requests(req: &mut ComponentRequests);
    /// SAFETY: assumes that requests do not "collide" with each other.
    unsafe fn get_from_world<'a>(
        world: &QueryWorld<'a, Storage>,
        archetype: ArchetypeID,
        index: InArchetypeId,
    ) -> Self;
}

#[cfg(test)]
mod tests {
    use super::ComponentRequests;

    #[test]
    fn test_disjoint() {
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
        // assert!(req1.disjoint(&req4));
        assert!(req2.disjoint_with(&req3));
        // assert!(!req2.disjoint(&req4));
        assert!(req3.disjoint_with(&req4));

        assert!(!req3.disjoint_with(&req_empty));
        assert!(!req_empty.disjoint_with(&req3));
    }
}
