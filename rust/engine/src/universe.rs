use indexmap::IndexMap;

#[derive(Debug, Hash, PartialEq, Eq)]
pub(crate) struct VesselID(u128);

pub(crate) struct Vessel {}

pub(crate) struct Universe {
    pub(crate) vessels: IndexMap<VesselID, Vessel>,
}

impl Universe {
    pub fn new() -> Self {
        Self {
            vessels: Default::default(),
        }
    }
}
