use std::time::Duration;

use bevy_scene::{
    serde::{SceneDeserializer, SceneSerializer},
    DynamicScene, Scene, SceneSpawnError,
};
use bincode::Options;
use engine_num::{Fixed, Vec3};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::info;
use type_registry::get_type_registry;

use self::{
    tilemap::{Tile, TileIndex, TileMap, TileOrientation, TilePos},
    ui_events::UiEventCtx,
};

pub const TICK_TIME: Duration = Duration::from_micros(16666);

pub mod rotations;
pub mod tilemap;
pub mod ui_events;

mod type_registry;

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

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("Could not deserialize received scene: {0}")]
    CouldNotDeserialize(#[from] bincode::Error),
    #[error("Could not spawn received scene: {0}")]
    CouldNotSpawn(#[from] SceneSpawnError),
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ExportedUniverse {
    world_state: Vec<u8>,
    vessels: IndexMap<VesselID, Vessel>,
    players: IndexMap<PlayerID, Player>,
}

/// The root of simulation. Should be the same on every client.
///
/// Deterministic - same sequence of events and updates(steps) should result in same state.
pub struct Universe {
    pub vessels: IndexMap<VesselID, Vessel>,
    pub players: IndexMap<PlayerID, Player>,
    world: bevy_ecs::world::World,
}

impl Universe {
    pub fn new() -> Self {
        let mut world = bevy_ecs::world::World::new();
        world.insert_resource(bevy_ecs::reflect::AppTypeRegistry(
            get_type_registry().clone(),
        ));
        Universe {
            vessels: Default::default(),
            players: Default::default(),
            world,
        }
    }
    pub fn update_ctx(&mut self) -> UpdateCtx {
        UpdateCtx {
            universe: self,
            evctx: UiEventCtx::default(),
        }
    }
    pub fn to_exported(&self) -> ExportedUniverse {
        info!("Exporting universe");
        let dyn_scene = DynamicScene::from_world(&self.world);
        let serializer = SceneSerializer::new(&dyn_scene, &get_type_registry());
        let world_state = bincode::serialize(&serializer).expect("can export universe");
        ExportedUniverse {
            world_state,
            vessels: self.vessels.clone(),
            players: self.players.clone(),
        }
    }
    pub fn from_exported(exported: ExportedUniverse) -> Result<Self, ImportError> {
        info!("Importing exported universe");
        let scene_deserializer = SceneDeserializer {
            type_registry: &get_type_registry().read(),
        };
        let dyn_scene = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .deserialize_seed(scene_deserializer, &exported.world_state)?;
        let scene = Scene::from_dynamic_scene(
            &dyn_scene,
            &bevy_ecs::reflect::AppTypeRegistry(get_type_registry().clone()),
        )?;
        Ok(Universe {
            vessels: exported.vessels,
            players: exported.players,
            world: scene.world,
        })
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
            UniverseEvent::PlaceTile {
                position,
                orientation,
            } => {
                let Some(player) = self.universe.players.get(&player_id) else {
                    return;
                };
                let Some(vessel) = self.universe.vessels.get_mut(&player.vessel) else {
                    return;
                };
                vessel
                    .tiles
                    .add_at(&mut self.evctx, position, Tile { orientation });
            }
            UniverseEvent::RemoveTile { position, index } => {
                let Some(player) = self.universe.players.get(&player_id) else {
                    return;
                };
                let Some(vessel) = self.universe.vessels.get_mut(&player.vessel) else {
                    return;
                };
                vessel.tiles.remove_at(&mut self.evctx, position, index);
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
