use std::{collections::HashSet, f32::consts::PI};

use anyhow::anyhow;
use bevy_ecs::{
    change_detection::DetectChanges,
    system::{NonSendMut, Res, ResMut},
};
use engine_num::Vec3;
use godot::{engine::CharacterBody3D, prelude::*};
use tracing::{info, warn};

use crate::{
    universe::{self, tilemap::TilePos, PlayerID},
    util::{FromGodot, SceneTreeExt, ToGodot},
};

use super::resources::{
    CurrentPlayer, CurrentVessel, Dt, EvCtx, InputState, PlayerNode, RootNode, SceneTreeRes,
    UniverseEventStorage, UniverseResource,
};

pub fn vessel_upload_condition(current_vessel: Res<CurrentVessel>, evctx: Res<EvCtx>) -> bool {
    current_vessel.is_changed() || !evctx.tiles_changed.is_empty()
}

pub fn upload_current_vessel(
    universe: Res<UniverseResource>,
    current_vessel: Res<CurrentVessel>,
    mut scene_tree: NonSendMut<SceneTreeRes>,
    mut root_node: NonSendMut<RootNode>,
) {
    info!("Uploading vessel");

    for mut shown in &mut scene_tree.iter_group::<Node>("tiles") {
        shown.queue_free()
    }

    let vessel = universe
        .0
        .vessels
        .get(&current_vessel.0)
        .ok_or_else(|| anyhow!("given vessel does not exist"))
        .unwrap(); // TODO
    let wall_scene = load::<PackedScene>("vessel/walls/wall1.tscn");
    for (pos, _tile) in vessel.tiles.iter() {
        let mut node = wall_scene.instantiate().unwrap();
        node.clone().cast::<Node3D>().set_position(pos.to_godot());
        node.clone().cast::<Node3D>().set_rotation_degrees(Vector3 {
            x: -90.0,
            y: 0.0,
            z: 0.0,
        });
        node.add_to_group("tiles".into());
        root_node.0.add_child(node.clone());
        //self.state.shown_tiles.push(node);
    }
}

pub fn update_players_on_vessel(
    universe: Res<UniverseResource>,
    current_player: Res<CurrentPlayer>,
    current_vessel: Res<CurrentVessel>,
    mut scene_tree: NonSendMut<SceneTreeRes>,
    mut root_node: NonSendMut<RootNode>,
    mut player_node_res: NonSendMut<Option<PlayerNode>>,
) {
    let mut on_current_vessel: HashSet<_> = universe
        .players
        .iter()
        .filter(|(_id, player)| player.vessel == current_vessel.0)
        .map(|(id, _player)| *id)
        .collect();

    for mut player_character in scene_tree.iter_group::<CharacterBody3D>("players") {
        let character_player_id = PlayerID(player_character.get("player".into()).to::<u32>());
        if on_current_vessel.contains(&character_player_id) {
            on_current_vessel.remove(&character_player_id);
        } else {
            info!("Removing {character_player_id:?} from ui");
            player_character.queue_free();
        }
    }
    let not_yet_spawned = on_current_vessel;

    for player_id in not_yet_spawned {
        info!("Adding {player_id:?} to ui");
        let mut player_node = load::<PackedScene>("Character.tscn").instantiate().unwrap();
        player_node.set("player".into(), player_id.0.to_variant());
        let is_me = current_player.0 == player_id;
        if is_me {
            *player_node_res = Some(PlayerNode {
                player: player_node.clone().cast(),
            });
        }
        player_node.set("controlled".into(), is_me.to_variant());
        player_node.add_to_group("players".into());
        if let Some(player_info) = universe.0.players.get(&player_id) {
            let position = player_info.position.to_godot();
            player_node
                .clone()
                .cast::<CharacterBody3D>()
                .set_position(position)
        } else {
            warn!("Player {:?} not found", player_id)
        }
        root_node.add_child(player_node);
    }
}

pub fn player_controls(
    mut player_node: NonSendMut<Option<PlayerNode>>,
    dt: Res<Dt>,
    input: Res<InputState>,
    mut events: ResMut<UniverseEventStorage>,
) {
    let Some(player_node) = player_node.as_mut() else {
        return;
    };
    let player_node = &mut player_node.player;
    let mut velocity = player_node.get_velocity();
    let mut position = player_node.get_position();
    if !player_node.is_on_floor() {
        velocity.y -= 9.8 * dt.0;
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

    let mouse_rotation = input.mouse_rel * -0.005;
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
    events.0.push(event);
}

#[derive(Default)]
pub struct PlacerLocal {
    temp_build_node: Option<Gd<Node3D>>,
}

pub fn player_placer(
    mut player_node: NonSendMut<Option<PlayerNode>>,
    mut events: ResMut<UniverseEventStorage>,
    mut root_node: NonSendMut<RootNode>,
    mut local: NonSendMut<PlacerLocal>,
) {
    let Some(player_node) = player_node.as_mut() else {
        return;
    };
    let pos = player_node.get_position();
    let cam = player_node
        .get_node("Camera3D".into())
        .unwrap()
        .cast::<Node3D>();
    let dir = -cam.get_global_transform().basis.col_c();
    let place_pos = pos + dir * 5.0;
    let place_tile = TilePos::from_godot(place_pos);
    let place_pos_q = place_tile.to_godot();
    if let Some(b_node) = &mut local.temp_build_node {
        b_node.set_position(place_pos_q)
    } else {
        let wall_scene = load::<PackedScene>("vessel/walls/wall1.tscn");
        let node = wall_scene.instantiate().unwrap();
        root_node.add_child(node.clone());
        let mut node = node.cast::<Node3D>();
        node.set_rotation_degrees(Vector3 {
            x: -90.0,
            y: 0.0,
            z: 0.0,
        });
        local.temp_build_node = Some(node);
    }
    if Input::singleton().is_action_just_pressed("g_place".into()) {
        events.push(universe::UniverseEvent::TilePlaced {
            position: place_tile,
        })
    }
}

pub fn update_player_positions(
    mut scene_tree: NonSendMut<SceneTreeRes>,
    universe: Res<UniverseResource>,
    current_player: Res<CurrentPlayer>,
) {
    let players = scene_tree.get_nodes_in_group("players".into());
    for player in players.iter_shared() {
        let mut player = player.cast::<CharacterBody3D>();
        let player_id = PlayerID(player.get("player".into()).to::<u32>());
        if current_player.0 != player_id {
            if let Some(player_info) = universe.players.get(&player_id) {
                player.set_position(player_info.position.to_godot()); // TODO interpolate
            } else {
                warn!("Player {:?} not found", player_id)
            }
        }
    }
}
