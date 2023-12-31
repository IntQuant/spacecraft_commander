use crate::DynDispath;

use smallvec::SmallVec;

pub(crate) mod changes;
pub(crate) mod commands;
pub(crate) mod query;

use crate::{query_world::QueryWorld, ArchetypeInfo, TypeIndex};

/// # Safety
///
/// Requests should cover all things that are accessed.
pub unsafe trait SystemParameter<'a, Storage: DynDispath> {
    fn requests() -> SmallVec<[ComponentRequests; 8]> {
        SmallVec::new()
    }
    /// # Safety
    ///
    /// Assumes that requests do not "collide" with each other.
    unsafe fn from_world(world: &'a QueryWorld<'a, Storage>) -> Self;
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

    pub(crate) fn any_exclusive(&self) -> bool {
        self.requests.iter().any(|x| x.exclusive)
            || self.resource_requests.iter().any(|x| x.exclusive)
    }

    pub(crate) fn release(&mut self) {
        self.requests.clear();
        self.resource_requests.clear();
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
