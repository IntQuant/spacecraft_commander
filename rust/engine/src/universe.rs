use std::time::Duration;

use engine_num::{Fixed, Vec3};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use tracing::info;

use self::{
    tilemap::{Tile, TileMap, TilePos},
    ui_events::UiEventCtx,
};

pub const TICK_TIME: Duration = Duration::from_micros(16666);

pub mod tilemap;
pub mod ui_events;

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct VesselID(pub u32);

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct PlayerID(pub u32);

#[derive(Serialize, Deserialize, Clone)]
pub struct Vessel {
    pub tiles: TileMap<Tile>,
}

impl Default for Vessel {
    fn default() -> Self {
        Self {
            tiles: TileMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub position: Vec3,
    pub vessel: VesselID,
}

/// The root of simulation. Should be the same on every client.
///
/// Deterministic - same sequence of events and updates(steps) should result in same state.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct Universe {
    pub vessels: IndexMap<VesselID, Vessel>,
    pub players: IndexMap<PlayerID, Player>,
}

impl Universe {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn update_ctx(&mut self) -> UpdateCtx {
        UpdateCtx {
            universe: self,
            evctx: UiEventCtx::default(),
        }
    }
}

pub struct UpdateCtx<'a> {
    universe: &'a mut Universe,
    evctx: UiEventCtx,
}

impl UpdateCtx<'_> {
    pub fn evctx(self) -> UiEventCtx {
        self.evctx
    }

    pub fn process_event(&mut self, event: OwnedUniverseEvent) {
        let player_id = event.player_id;
        match event.event {
            UniverseEvent::PlayerConnected => {
                info!("Creating player for {player_id:?}");
                self.universe.players.entry(player_id).or_insert(Player {
                    position: Vec3::new(Fixed::new_int(0), Fixed::new_int(10), Fixed::new_int(0)),
                    vessel: VesselID(0),
                });
            }
            UniverseEvent::PlayerMoved { new_position } => {
                if let Some(player) = self.universe.players.get_mut(&player_id) {
                    player.position = new_position;
                }
            }
            UniverseEvent::TilePlaced { position } => {
                let Some(player) = self.universe.players.get(&player_id) else {
                    return;
                };
                let Some(vessel) = self.universe.vessels.get_mut(&player.vessel) else {
                    return;
                };
                vessel.tiles.add_at(&mut self.evctx, position, Tile {})
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
    PlayerMoved { new_position: Vec3 },
    TilePlaced { position: TilePos },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OwnedUniverseEvent {
    pub player_id: PlayerID,
    pub event: UniverseEvent,
}
