use std::time::{Duration, Instant};

use engine_num::Vec3;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tracing::info;

pub const TICK_TIME: Duration = Duration::from_micros(16666);

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct VesselID(u32);

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PlayerID(pub u32);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Vessel {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub position: Vec3,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Universe {
    pub vessels: IndexMap<VesselID, Vessel>,
    pub players: IndexMap<PlayerID, Player>,

    #[serde(skip)]
    unsynced_last_step: Option<Instant>,
}

impl Universe {
    pub fn new() -> Self {
        Self {
            unsynced_last_step: None,
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
                    position: Vec3::default(),
                });
            }
            UniverseEvent::PlayerMoved { new_position } => {
                if let Some(player) = self.players.get_mut(&player_id) {
                    player.position = new_position;
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.unsynced_last_step = Some(Instant::now())
        // TODO
    }

    pub fn since_last_tick(&self) -> f32 {
        self.unsynced_last_step
            .unwrap_or_else(Instant::now)
            .elapsed()
            .as_secs_f32()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UniverseEvent {
    PlayerConnected,
    PlayerMoved { new_position: Vec3 },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OwnedUniverseEvent {
    pub player_id: PlayerID,
    pub event: UniverseEvent,
}
