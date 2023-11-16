use serde::{Deserialize, Serialize};
use slotmapd::new_key_type;

use crate::tilemap::{Tile, TileMap};

new_key_type! { pub struct VesselID; }

#[derive(Serialize, Deserialize, Clone)]
pub struct VesselTiles(pub TileMap<Tile>);

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct DefaultVesselRes(pub VesselID);
