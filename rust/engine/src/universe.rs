use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct VesselID(u128);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Vessel {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Universe {
    pub(crate) vessels: IndexMap<VesselID, Vessel>,
}

impl Universe {
    pub fn new() -> Self {
        Self {
            vessels: Default::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UniverseEvent {}
