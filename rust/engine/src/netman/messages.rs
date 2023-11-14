use engine_universe::{ecs::ids::PlayerID, ExportedUniverse};
use serde::{Deserialize, Serialize};

use crate::universe::{OwnedUniverseEvent, UniverseEvent};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum QueuedEvent {
    UniverseEvent(OwnedUniverseEvent),
    StepUniverse,
}

#[derive(Serialize, Deserialize)]
pub enum SentByServer {
    SetUniverse(ExportedUniverse),
    Event(QueuedEvent),
    IdAssigned(PlayerID),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SentByClient {
    UniverseEvent(UniverseEvent),
}
