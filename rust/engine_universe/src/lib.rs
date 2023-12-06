use std::{mem, time::Duration};

use engine_ecs::{EntityID, World, WorldRun};
use engine_num::Vec3;
use mcs::{
    events::system_handle_pending_events, BuildingKind, ComponentStorage, PendingEventsRes, Player,
    PlayerID, PlayerMap,
};
use rotations::BuildingOrientation;
use serde::{Deserialize, Serialize};

use self::{
    tilemap::{TileIndex, TilePos},
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
    pub world: World<ComponentStorage>,
}

impl Universe {
    pub fn new() -> Self {
        Universe::default()
    }

    pub fn update_ctx(&mut self) -> UpdateCtx {
        UpdateCtx { universe: self }
    }

    pub fn player_ent_id(&self, player: PlayerID) -> Option<EntityID> {
        self.world.resource::<PlayerMap>().get(player)
    }

    pub fn player_info(&self, player: PlayerID) -> Option<&Player> {
        self.world
            .resource::<PlayerMap>()
            .get(player)
            .and_then(|ent| self.world.get(ent))
    }
}

pub struct UpdateCtx<'a> {
    universe: &'a mut Universe,
}

impl UpdateCtx<'_> {
    #[must_use]
    pub fn evctx(self) -> UiEventCtx {
        mem::take(self.universe.world.resource_mut::<UiEventCtx>())
    }

    pub fn process_event(&mut self, event: OwnedUniverseEvent) {
        self.universe
            .world
            .resource_mut::<PendingEventsRes>()
            .0
            .push(event);
    }

    pub fn step(&mut self) {
        let world = &mut self.universe.world;
        world.query_world().run(system_handle_pending_events);
        world.resource_mut::<PendingEventsRes>().0.clear();
        //system_handle_pending_events(universe, evctx);

        //self.universe.pending_events.clear();
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
        orientation: BuildingOrientation,
    },
    RemoveTile {
        position: TilePos,
        index: TileIndex,
    },
    PlaceBuilding {
        position: TilePos,
        orientation: BuildingOrientation,
        kind: BuildingKind,
    },
    RemoveBuilding {
        entity: EntityID,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OwnedUniverseEvent {
    pub player_id: PlayerID,
    pub event: UniverseEvent,
}
