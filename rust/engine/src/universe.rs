use engine_num::Vector3;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct VesselID(u32);

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PlayerID(pub u32);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vessel {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    position: Vector3,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Universe {
    pub vessels: IndexMap<VesselID, Vessel>,
    pub players: IndexMap<PlayerID, Player>,
}

impl Universe {
    pub fn new() -> Self {
        Self {
            vessels: Default::default(),
            players: Default::default(),
        }
    }

    pub fn process_event(&mut self, event: OwnedUniverseEvent) {
        let player_id = event.player_id;
        match event.event {
            UniverseEvent::PlayerConnected => {
                info!("Creating player for {player_id:?}");
                self.players.entry(player_id).or_insert(Player {
                    position: Vector3::default(),
                });
            }
        }
    }

    pub fn step(&mut self) {
        // TODO
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UniverseEvent {
    PlayerConnected,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OwnedUniverseEvent {
    pub player_id: PlayerID,
    pub event: UniverseEvent,
}
