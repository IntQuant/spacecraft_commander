use engine_registry::BuildingKind;
use serde::{Deserialize, Serialize};

use crate::{rotations::BuildingOrientation, tilemap::TilePos};

#[derive(Serialize, Deserialize, Clone)]
pub struct Building {
    pub position: TilePos,
    pub orientation: BuildingOrientation,
    pub kind: BuildingKind,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ControlSet {
    controls: Vec<ControlKind>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum ControlKind {
    SingleAxisAnalog(u8),
}
