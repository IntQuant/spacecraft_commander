use std::sync::Arc;

use bevy_ecs::system::Resource;
use godot::{
    engine::CharacterBody3D,
    prelude::{Gd, Node3D, Vector2},
};

use crate::universe::{PlayerID, Universe, UniverseEvent, VesselID};

#[derive(Resource)]
pub struct UniverseResource(pub Arc<Universe>);

#[derive(Default, Resource, Clone)]
pub struct InputState {
    pub mouse_rel: Vector2,
}

#[derive(Resource)]
pub struct CurrentPlayer(pub PlayerID);

#[derive(Resource)]
pub struct CurrentVessel(pub VesselID);

pub struct RootNode(pub Gd<Node3D>);

pub struct PlayerNode {
    pub player: Gd<CharacterBody3D>,
}

#[derive(Resource)]
pub struct Dt(pub f32);

#[derive(Resource)]
pub struct UniverseEventStorage(pub Vec<UniverseEvent>);
