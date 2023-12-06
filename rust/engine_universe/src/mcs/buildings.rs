use serde::{Deserialize, Serialize};

use crate::{rotations::BuildingOrientation, tilemap::TilePos};

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct BuildingKind(u32);

#[derive(Serialize, Deserialize, Clone)]
pub struct Building {
    pub position: TilePos,
    pub orientation: BuildingOrientation,
    pub kind: BuildingKind,
}
