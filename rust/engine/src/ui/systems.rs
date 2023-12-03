use std::{collections::HashSet, f32::consts::PI};

use engine_ecs::EntityID;
use engine_num::Vec3;
use godot::{
    engine::{CharacterBody3D, RayCast3D},
    prelude::*,
};
use tracing::{info, warn};
use universe::{
    mcs::{self, Player, VesselTiles},
    rotations::BuildingOrientation,
};

use crate::{
    universe::{
        self,
        rotations::{self},
        tilemap::TilePos,
    },
    util::{FromGodot, SceneTreeExt, ToGodot},
    BaseStaticBody, BodyKind,
};

use super::resources::{
    CurrentFacingRes, CurrentPlayerRes, CurrentPlayerRotationRes, CurrentVesselRes, DtRes,
    EvCtxRes, InputStateRes, PlacerRes, PlayerNodeRes, RootNodeRes, SceneTreeRes,
    UniverseEventStorageRes, UniverseRes,
};

pub fn vessel_upload_condition(_current_vessel: &CurrentVesselRes, evctx: &EvCtxRes) -> bool {
    //current_vessel.is_changed() || !evctx.tiles_changed.is_empty() // TODO
    !evctx.tiles_changed.is_empty()
}

pub fn update_current_vessel(
    current_player: &CurrentPlayerRes,
    current_vessel: &mut CurrentVesselRes,
    universe: &UniverseRes,
) {
    if let Some(player) = current_player.0.and_then(|x| universe.player_info(x)) {
        if player.vessel != current_vessel.0 {
            info!("Current vessel changed");
            current_vessel.0 = player.vessel;
        }
    }
}

pub fn upload_current_vessel_tiles(
    universe: &UniverseRes,
    current_vessel: &CurrentVesselRes,
    scene_tree: &mut SceneTreeRes,
    root_node: &mut RootNodeRes,
) {
    info!("Uploading vessel");

    for mut shown in &mut scene_tree.iter_group::<Node>("tiles") {
        shown.queue_free()
    }

    let Some(tiles) = universe.world.get::<VesselTiles>(current_vessel.0 .0) else {
        warn!("Current vessel does not exist");
        return;
    };

    let wall_scene = load::<PackedScene>("vessel/generic/wall_generic.tscn");
    for (tile_index, pos, tile) in tiles.0.iter() {
        let mut node = wall_scene.instantiate().unwrap().cast::<BaseStaticBody>();

        node.bind_mut().kind = Some(crate::BodyKind::Tile {
            index: tile_index,
            position: pos,
        });

        node.set_position(pos.to_godot());
        let basis = tile.orientation.to_basis().to_godot();
        node.set_basis(basis);
        node.add_to_group("tiles".into());
        root_node.add_child(node.upcast());
    }
}

pub fn update_players_on_vessel(
    universe: &UniverseRes,
    current_player: &CurrentPlayerRes,
    current_vessel: &CurrentVesselRes,
    scene_tree: &mut SceneTreeRes,
    root_node: &mut RootNodeRes,
    player_node_res: &mut PlayerNodeRes,
) {
    let binding = universe.world.query_world_shared();
    let mut players = binding.parameter::<mcs::Query<(EntityID, &Player)>>();
    let mut on_current_vessel: HashSet<_> = players
        .iter()
        .filter(|(_id, player)| player.vessel == current_vessel.0)
        .map(|(id, _player)| id)
        .collect();

    for mut player_character in scene_tree.iter_group::<CharacterBody3D>("players") {
        let character_player_id = EntityID::from_godot(player_character.get("player".into()));
        if !on_current_vessel.remove(&character_player_id) {
            info!("Removing {character_player_id:?} from ui");
            player_character.queue_free();
        }
    }
    let not_yet_spawned = on_current_vessel;

    for player_id in not_yet_spawned {
        info!("Adding {player_id:?} to ui");
        let mut player_node = load::<PackedScene>("Character.tscn").instantiate().unwrap();
        player_node.set("player".into(), player_id.to_godot());
        let is_me = current_player.0.and_then(|x| universe.player_ent_id(x)) == Some(player_id);
        if is_me {
            *player_node_res = PlayerNodeRes {
                player: Some(player_node.clone().cast()),
            };
        }
        player_node.set("controlled".into(), is_me.to_variant());
        player_node.add_to_group("players".into());
        if let Some(player_info) = universe.world.get::<Player>(player_id) {
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
    player_node: &mut PlayerNodeRes,
    dt: &DtRes,
    input: &InputStateRes,
    events: &mut UniverseEventStorageRes,
    current_player_rotation: &mut CurrentPlayerRotationRes,
) {
    if player_node.player.is_none() {
        return;
    };

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
    if !Input::singleton().is_action_pressed("g_rot_en".into()) {
        let input_vec = Input::singleton().get_vector(
            "g_left".into(),
            "g_right".into(),
            "g_forward".into(),
            "g_back".into(),
        );
        let direction =
            player_node.get_transform().basis * Vector3::new(input_vec.x, 0.0, input_vec.y);
        let direction = direction.normalized() * 5.0;
        velocity.x = direction.x;
        velocity.z = direction.z;
    }

    player_node.set_velocity(velocity);
    player_node.set_position(position);
    player_node.move_and_slide();

    let mouse_rotation = input.mouse_rel * -0.005;
    player_node.rotate(Vector3::UP, mouse_rotation.x);
    current_player_rotation.0 = player_node.get_rotation().y;
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

pub fn building_facing(
    current_facing: &mut CurrentFacingRes,
    current_player_rotation: &CurrentPlayerRotationRes,
) {
    let input = Input::singleton();
    if input.is_action_pressed("g_rot_en".into()) {
        let actions = [
            input.is_action_just_pressed("g_rot_d".into()),
            input.is_action_just_pressed("g_rot_w".into()),
            input.is_action_just_pressed("g_rot_a".into()),
            input.is_action_just_pressed("g_rot_s".into()),
        ];
        let action_id = actions.iter().position(|x| *x);
        if let Some(action_id) = action_id {
            let current = ((current_player_rotation.0 / (PI / 2.0) + 2.5) % 4.0) as u8;
            current_facing.0 = current_facing.turn(action_id as u8, current);
            info!(
                "Current facing: {:?}, current: {}, rotation: {}",
                current_facing.0, current, current_player_rotation.0
            );
        }
    }
}

pub fn building_placer(
    player_node: &mut PlayerNodeRes,
    events: &mut UniverseEventStorageRes,
    root_node: &mut RootNodeRes,
    local: &mut PlacerRes,
    current_facing: &CurrentFacingRes,
) {
    if player_node.player.is_none() {
        return;
    };

    let pos = player_node.get_position();
    let cam = player_node
        .get_node("Camera3D".into())
        .unwrap()
        .cast::<Node3D>();
    let dir = -cam.get_global_transform().basis.col_c();
    let place_pos = pos + dir * 3.0;
    let place_tile = TilePos::from_godot(place_pos);
    let place_pos_q = place_tile.to_godot();
    if let Some(b_node) = &mut local.temp_build_node {
        b_node.set_position(place_pos_q);
        b_node.set_basis(current_facing.to_basis().to_godot());
    } else {
        let wall_scene = load::<PackedScene>("vessel/generic/wall_virtual.tscn");
        let node = wall_scene.instantiate().unwrap();
        root_node.add_child(node.clone());
        let node = node.cast::<Node3D>();
        local.temp_build_node = Some(node);
    }
    if Input::singleton().is_action_just_pressed("g_place".into()) {
        events.push(universe::UniverseEvent::PlaceTile {
            position: place_tile,
            orientation: BuildingOrientation::new(current_facing.0, rotations::BuildingRotation::N),
        })
    }
}

pub fn building_remover(player_node: &mut PlayerNodeRes, events: &mut UniverseEventStorageRes) {
    if player_node.player.is_none() {
        return;
    };

    let raycast = player_node
        .get_node("Camera3D/RayCast3D".into())
        .unwrap()
        .cast::<RayCast3D>();
    let hit = raycast
        .get_collider()
        .and_then(|c| c.try_cast::<BaseStaticBody>());

    if let Some(hit) = hit {
        if Input::singleton().is_action_just_pressed("g_remove".into()) {
            info!("Remove {:?}", hit.bind());
            let Some(kind) = hit.bind().kind else {
                return;
            };
            let BodyKind::Tile { index, position } = kind;
            events.push(universe::UniverseEvent::RemoveTile { position, index });
        }
    }
}

pub fn update_player_positions(
    scene_tree: &mut SceneTreeRes,
    universe: &UniverseRes,
    current_player: &CurrentPlayerRes,
) {
    let players = scene_tree.get_nodes_in_group("players".into());
    for player in players.iter_shared() {
        let mut player = player.cast::<CharacterBody3D>();
        let player_id = EntityID::from_godot(player.get("player".into()));
        if current_player.0.and_then(|x| universe.player_ent_id(x)) != Some(player_id) {
            let player_info = universe.world.get::<Player>(player_id);
            if let Some(player_info) = player_info.as_ref() {
                player.set_position(player_info.position.to_godot()); // TODO interpolate
            } else {
                warn!("Player {:?} not found", player_id)
            }
        }
    }
}
