use std::collections::HashMap;

use bevy_ecs::{component::Component, entity::Entity, system::Resource};
use bevy_reflect::Reflect;
use engine_num::Vec3;
use serde::{Deserialize, Serialize};

use super::vessel::VesselID;

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct PlayerID(pub u32);

#[derive(Reflect, Resource, Default)]
pub struct PlayerMap {
    pub map: HashMap<PlayerID, Entity>,
}

#[derive(Reflect, Component)]
pub struct Player {
    pub position: Vec3,
    pub vessel: VesselID,
}
