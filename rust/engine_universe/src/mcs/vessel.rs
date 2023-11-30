use engine_ecs::EntityID;
use serde::{Deserialize, Serialize};
use slotmapd::new_key_type;

use crate::tilemap::{Tile, TileMap};

#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct VesselID(pub EntityID);

#[derive(Serialize, Deserialize, Clone)]
pub struct VesselTiles(pub TileMap<Tile>);

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct DefaultVesselRes(pub VesselID);
