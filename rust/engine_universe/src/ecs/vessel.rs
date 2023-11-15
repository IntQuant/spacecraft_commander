use bevy_ecs::{component::Component, entity::Entity, system::Resource};
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};

use crate::tilemap::{Tile, TileMap};

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy, Reflect)]
pub struct VesselEnt(pub Entity);

#[derive(Reflect, Component)]
pub struct VesselTiles(pub TileMap<Tile>);

#[derive(Reflect, Resource)]
pub struct DefaultVessel(pub VesselEnt);
