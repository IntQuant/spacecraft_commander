use derive_more::{Deref, DerefMut};
use std::sync::Arc;

use bevy_ecs::system::Resource;
use godot::{engine::CharacterBody3D, prelude::*};

use crate::universe::{
    rotations::BuildingFacing, ui_events::UiEventCtx, PlayerID, Universe, UniverseEvent, VesselID,
};

#[derive(Deref, Resource)]
pub struct UniverseResource(pub Arc<Universe>);

#[derive(Default, Resource, Clone)]
pub struct InputState {
    pub mouse_rel: Vector2,
}

#[derive(Deref, DerefMut, Resource)]
pub struct CurrentPlayer(pub PlayerID);

#[derive(Deref, DerefMut, Resource)]
pub struct CurrentVessel(pub VesselID);

#[derive(Deref, DerefMut)]
pub struct RootNode(pub Gd<Node3D>);

#[derive(Deref, DerefMut)]
pub struct SceneTreeRes(pub Gd<godot::prelude::SceneTree>);

#[derive(Deref, DerefMut)]
pub struct PlayerNode {
    pub player: Gd<CharacterBody3D>,
}

#[derive(Deref, DerefMut, Resource)]
pub struct Dt(pub f32);

#[derive(Deref, DerefMut, Resource)]
pub struct UniverseEventStorage(pub Vec<UniverseEvent>);

#[derive(Deref, DerefMut, Resource)]
pub struct EvCtx(pub UiEventCtx);

#[derive(Deref, DerefMut, Resource)]
pub struct CurrentFacing(pub BuildingFacing);

#[derive(Deref, DerefMut, Resource, Default)]
pub struct CurrentPlayerRotation(pub f32);
