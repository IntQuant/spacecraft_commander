use std::time::Duration;

use engine_macro::gen_storage_for_world;
use engine_num::Vec3;
use indexmap::IndexMap;
use mcs::{
    events::system_handle_pending_events, DefaultVesselRes, Player, PlayerID, VesselID, VesselTiles,
};
use serde::{Deserialize, Serialize};
use slotmapd::HopSlotMap;

use self::{
    tilemap::{TileIndex, TileOrientation, TilePos},
    ui_events::UiEventCtx,
};

pub const TICK_TIME: Duration = Duration::from_micros(16666);

pub mod rotations;
pub mod tilemap;
pub mod ui_events;

pub mod mcs;

/// The root of simulation. Should be the same on every client.
///
/// Deterministic - same sequence of events and updates(steps) should result in same state.
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Universe {
    pending_events: Vec<OwnedUniverseEvent>,
    pub players: IndexMap<PlayerID, Player>,
    pub vessels: HopSlotMap<VesselID, VesselTiles>,

    pub default_vessel: DefaultVesselRes,
}

impl Universe {
    pub fn new() -> Self {
        Universe::default()
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
        self.universe.pending_events.push(event);
    }

    pub fn step(&mut self) {
        let universe = &mut self.universe;
        let evctx = &mut self.evctx;

        system_handle_pending_events(universe, evctx);

        self.universe.pending_events.clear();
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum UniverseEvent {
    PlayerConnected,
    PlayerMoved {
        new_position: Vec3,
    },
    PlaceTile {
        position: TilePos,
        orientation: TileOrientation,
    },
    RemoveTile {
        position: TilePos,
        index: TileIndex,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OwnedUniverseEvent {
    pub player_id: PlayerID,
    pub event: UniverseEvent,
}

struct Component1;
struct Component2;
struct Component3;

gen_storage_for_world! { Component1 Component2 Component3 }
