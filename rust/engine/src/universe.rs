use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub(crate) struct VesselID(u32);

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PlayerID(pub u32);

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

    pub fn process_event(&mut self, event: OwnedUniverseEvent) {
        todo!()
    }

    pub fn step(&mut self) {
        // TODO
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UniverseEvent {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OwnedUniverseEvent {
    pub player_id: PlayerID,
    pub event: UniverseEvent,
}
