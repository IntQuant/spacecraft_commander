use std::f32::consts::PI;

use engine_num::Vec3;
use godot::prelude::{Input, Node3D, Vector3};

use crate::{universe, util::FromGodot};

use super::UiInCtx;

pub fn player_controls(ctx: &mut UiInCtx) {
    let Some(player_node) = &mut ctx.state.my_player_node else {
        return;
    };
    let mut velocity = player_node.get_velocity();
    let mut position = player_node.get_position();
    if !player_node.is_on_floor() {
        velocity.y -= 9.8 * ctx.dt;
    }
    if player_node.is_on_floor() && Input::singleton().is_action_just_pressed("g_jump".into()) {
        velocity.y = 4.5;
    }

    if position.y < -100.0 {
        position.y = 100.0;
    }

    let input_vec = Input::singleton().get_vector(
        "g_left".into(),
        "g_right".into(),
        "g_forward".into(),
        "g_back".into(),
    );
    let direction = player_node.get_transform().basis * Vector3::new(input_vec.x, 0.0, input_vec.y);
    let direction = direction.normalized() * 5.0;
    velocity.x = direction.x;
    velocity.z = direction.z;

    player_node.set_velocity(velocity);
    player_node.set_position(position);
    player_node.move_and_slide();

    let mouse_rotation = ctx.input.mouse_rel * -0.005;
    player_node.rotate(Vector3::UP, mouse_rotation.x);
    let mut cam = player_node
        .get_node("Camera3D".into())
        .unwrap()
        .cast::<Node3D>();
    let mut rotation = cam.get_rotation();
    rotation.x = (rotation.x + mouse_rotation.y).clamp(-PI / 2.0, PI / 2.0);
    cam.set_rotation(rotation);

    let event = universe::UniverseEvent::PlayerMoved {
        new_position: Vec3::from_godot(player_node.get_position()),
    };
    ctx.events.push(event);
}
