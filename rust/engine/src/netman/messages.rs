use serde::{Deserialize, Serialize};

use crate::universe::{OwnedUniverseEvent, Universe, UniverseEvent};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum QueuedEvent {
    UniverseEvent(OwnedUniverseEvent),
    StepUniverse,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SentByServer {
    SetUniverse(Universe),
    Event(QueuedEvent),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SentByClient {
    UniverseEvent(UniverseEvent),
}
