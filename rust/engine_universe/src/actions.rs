use engine_ecs::EntityID;
use engine_num::Vec3;
use engine_registry::{BuildingKind, TileKind};
use serde::{Deserialize, Serialize};

use crate::{
    mcs::VesselID,
    rotations::BuildingOrientation,
    tilemap::{TileIndex, TilePos},
};

#[derive(Clone, Serialize, Deserialize)]
pub enum Action {
    MovePlayer {
        player: EntityID,
        new_position: Vec3,
    },
    PlaceTile {
        vessel: VesselID,
        position: TilePos,
        orientation: BuildingOrientation,
        kind: TileKind,
    },
    RemoveTile {
        vessel: VesselID,
        position: TilePos,
        index: TileIndex,
    },
    PlaceBuilding {
        vessel: VesselID,
        position: TilePos,
        orientation: BuildingOrientation,
        kind: BuildingKind,
    },
    RemoveBuilding {
        entity: EntityID,
    },
}
