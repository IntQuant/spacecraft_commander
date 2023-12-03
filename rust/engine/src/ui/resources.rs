use derive_more::{Deref, DerefMut};
use engine_universe::mcs::{PlayerID, VesselID};
use std::{ops, sync::Arc};

use godot::{
    engine::CharacterBody3D,
    prelude::{SceneTree, *},
};

use crate::universe::{rotations::BuildingFacing, ui_events::UiEventCtx, Universe, UniverseEvent};

#[derive(Default)]
pub struct UniverseRes(pub Option<Arc<Universe>>);

impl ops::Deref for UniverseRes {
    type Target = Universe;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

#[derive(Default, Clone)]
pub struct InputStateRes {
    pub mouse_rel: Vector2,
}

#[derive(Deref, DerefMut, Default)]
pub struct CurrentPlayerRes(pub Option<PlayerID>);

#[derive(Deref, DerefMut, Default)]
pub struct CurrentVesselRes(pub VesselID);

#[derive(Default)]
pub struct RootNodeRes(pub Option<Gd<Node3D>>);

impl ops::Deref for RootNodeRes {
    type Target = Gd<Node3D>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl ops::DerefMut for RootNodeRes {
    fn deref_mut(&mut self) -> &mut Gd<Node3D> {
        self.0.as_mut().unwrap()
    }
}

#[derive(Default)]
pub struct SceneTreeRes(pub Option<Gd<SceneTree>>);

impl ops::Deref for SceneTreeRes {
    type Target = Gd<SceneTree>;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap()
    }
}

impl ops::DerefMut for SceneTreeRes {
    fn deref_mut(&mut self) -> &mut Gd<SceneTree> {
        self.0.as_mut().unwrap()
    }
}

#[derive(Default)]
pub struct PlayerNodeRes {
    pub player: Option<Gd<CharacterBody3D>>,
}

impl ops::Deref for PlayerNodeRes {
    type Target = Gd<CharacterBody3D>;

    fn deref(&self) -> &Self::Target {
        self.player.as_ref().unwrap()
    }
}

impl ops::DerefMut for PlayerNodeRes {
    fn deref_mut(&mut self) -> &mut Gd<CharacterBody3D> {
        self.player.as_mut().unwrap()
    }
}

#[derive(Deref, DerefMut)]
pub struct DtRes(pub f32);

impl Default for DtRes {
    fn default() -> Self {
        Self(1.0 / 60.0)
    }
}

#[derive(Deref, DerefMut, Default)]
pub struct UniverseEventStorageRes(pub Vec<UniverseEvent>);

#[derive(Deref, DerefMut, Default)]
pub struct EvCtxRes(pub UiEventCtx);

#[derive(Deref, DerefMut, Default)]
pub struct CurrentFacingRes(pub BuildingFacing);

#[derive(Deref, DerefMut, Default)]
pub struct CurrentPlayerRotationRes(pub f32);

#[derive(Default)]
pub struct PlacerRes {
    pub temp_build_node: Option<Gd<Node3D>>,
}
