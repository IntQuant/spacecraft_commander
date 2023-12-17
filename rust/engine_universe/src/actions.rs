use engine_ecs::EntityID;
use engine_num::Vec3;
use engine_registry::{BuildingKind, TileKind};

use crate::{
    rotations::BuildingOrientation,
    tilemap::{TileIndex, TilePos},
};

pub enum Action {
    MovePlayer {
        player: EntityID,
        new_position: Vec3,
    },
    PlaceTile {
        vessel: EntityID,
        position: TilePos,
        orientation: BuildingOrientation,
        kind: TileKind,
    },
    RemoveTile {
        vessel: EntityID,
        position: TilePos,
        index: TileIndex,
    },
    PlaceBuilding {
        vessel: EntityID,
        position: TilePos,
        orientation: BuildingOrientation,
        kind: BuildingKind,
    },
    RemoveBuilding {
        vessel: EntityID,
        entity: EntityID,
    },
}
