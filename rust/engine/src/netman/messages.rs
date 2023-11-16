use serde::{Deserialize, Serialize};

use crate::universe::{mcs::PlayerID, OwnedUniverseEvent, Universe, UniverseEvent};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum QueuedEvent {
    UniverseEvent(OwnedUniverseEvent),
    StepUniverse,
}

#[derive(Serialize, Deserialize)]
pub enum SentByServer {
    SetUniverse(Universe),
    Event(QueuedEvent),
    IdAssigned(PlayerID),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SentByClient {
    UniverseEvent(UniverseEvent),
}
