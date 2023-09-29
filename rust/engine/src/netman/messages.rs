use serde::{Deserialize, Serialize};

use crate::universe::{Universe, UniverseEvent};

#[derive(Debug, Serialize, Deserialize)]
pub enum SentByServer {
    SetUniverse(Universe),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SentByClient {}

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
struct PeerId(u64);

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    source: PeerId,
    event: UniverseEvent,
}
