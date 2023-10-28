use godot::{engine::CharacterBody3D, prelude::Gd};

use super::UiInCtx;

pub struct PlayerControllerState {
    player: Gd<CharacterBody3D>,
}

pub fn player_manager(ui_ctx: &mut UiInCtx) {}
