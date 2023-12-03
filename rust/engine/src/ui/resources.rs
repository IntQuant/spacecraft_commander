use derive_more::{Deref, DerefMut};
use engine_universe::mcs::{PlayerID, VesselID};
use std::{ops, sync::Arc};

use bevy_ecs::system::Resource;
use godot::{
    engine::CharacterBody3D,
    prelude::{SceneTree, *},
};

use crate::universe::{rotations::BuildingFacing, ui_events::UiEventCtx, Universe, UniverseEvent};

#[derive(Resource, Default)]
pub struct UniverseRes(pub Option<Arc<Universe>>);

impl ops::Deref for UniverseRes {
    type Target = Universe;

    fn deref(&self) -> &Self::Target {
        &self.0.unwrap()
    }
}

#[derive(Default, Resource, Clone)]
pub struct InputStateRes {
    pub mouse_rel: Vector2,
}

#[derive(Deref, DerefMut, Resource, Default)]
pub struct CurrentPlayerRes(pub Option<PlayerID>);

#[derive(Deref, DerefMut, Resource, Default)]
pub struct CurrentVesselRes(pub VesselID);

#[derive(Deref, DerefMut, Default)]
pub struct RootNodeRes(pub Option<Gd<Node3D>>);

#[derive(Default)]
pub struct SceneTreeRes(pub Option<Gd<SceneTree>>);

impl ops::Deref for SceneTreeRes {
    type Target = Gd<SceneTree>;

    fn deref(&self) -> &Self::Target {
        &self.0.unwrap()
    }
}

impl ops::DerefMut for SceneTreeRes {
    fn deref_mut(&mut self) -> &mut Gd<SceneTree> {
        &mut self.0.unwrap()
    }
}

#[derive(Deref, DerefMut, Default)]
pub struct PlayerNodeRes {
    pub player: Option<Gd<CharacterBody3D>>,
}

#[derive(Deref, DerefMut, Resource)]
pub struct DtRes(pub f32);

impl Default for DtRes {
    fn default() -> Self {
        Self(1.0 / 60.0)
    }
}

#[derive(Deref, DerefMut, Resource, Default)]
pub struct UniverseEventStorageRes(pub Vec<UniverseEvent>);

#[derive(Deref, DerefMut, Resource, Default)]
pub struct EvCtxRes(pub UiEventCtx);

#[derive(Deref, DerefMut, Resource, Default)]
pub struct CurrentFacingRes(pub BuildingFacing);

#[derive(Deref, DerefMut, Resource, Default)]
pub struct CurrentPlayerRotationRes(pub f32);
