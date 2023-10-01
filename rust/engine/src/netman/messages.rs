use serde::{Deserialize, Serialize};

use crate::universe::{OwnedUniverseEvent, PlayerID, Universe, UniverseEvent};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum QueuedEvent {
    UniverseEvent(OwnedUniverseEvent),
    StepUniverse,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SentByServer {
    SetUniverse(Universe),
    Event(QueuedEvent),
    IdAssigned(PlayerID),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SentByClient {
    UniverseEvent(UniverseEvent),
}
