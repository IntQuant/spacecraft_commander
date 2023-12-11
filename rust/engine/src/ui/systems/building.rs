use crate::{
    ui::{
        resources::{CurrentBuildingIndexRes, CurrentBuildingRotationRes, CurrentTileIndexRes},
        uecs::Changes,
    },
    universe,
    util::RegistryExt,
};
use std::f32::consts::PI;

use engine_registry::Registry;
use engine_universe::{rotations::BuildingOrientation, tilemap::TilePos};
use godot::engine::{load, Input, Node3D, PackedScene, RayCast3D};
use tracing::info;

use crate::{
    ui::{
        resources::{
            BuildingMode, CurrentFacingRes, CurrentPlayerRotationRes, PlacerRes, PlayerNodeRes,
            RootNodeRes, UniverseEventStorageRes,
        },
        uecs::Commands,
    },
    util::{FromGodot, ToGodot},
    BaseStaticBody, BodyKind,
};

pub fn building_facing(
    current_facing: &mut CurrentFacingRes,
    current_rotation: &mut CurrentBuildingRotationRes,
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
                "Current facing: {:?}, current building rotation: {:?}, current: {}, rotation: {}",
                current_facing.0, current_rotation.0, current, current_player_rotation.0
            );
        }
        if input.is_action_just_pressed("g_rot_e".into()) {
            current_rotation.0 = current_rotation.turn(1);
        }
        if input.is_action_just_pressed("g_rot_q".into()) {
            current_rotation.0 = current_rotation.turn(-1);
        }
    }
}

pub fn building_selector(commands: Commands, mode: &BuildingMode) {
    let input = Input::singleton();
    let building_len = Registry::instance().buildings.len();
    let tiles_len = Registry::instance().tiles.len();
    match mode {
        BuildingMode::Disabled => {}
        BuildingMode::Tiles => {
            if input.is_action_just_pressed("g_next_building".into()) {
                commands.submit(move |world| {
                    let current_tile: &mut CurrentTileIndexRes = world.resource_mut();
                    current_tile.0 += 1;
                    if current_tile.0 == tiles_len {
                        current_tile.0 = 0;
                    }
                })
            }
            if input.is_action_just_pressed("g_prev_building".into()) {
                commands.submit(move |world| {
                    let current_tile: &mut CurrentTileIndexRes = world.resource_mut();
                    if current_tile.0 == 0 {
                        current_tile.0 = tiles_len;
                    }
                    current_tile.0 -= 1;
                })
            }
        }
        BuildingMode::Buildings => {
            if input.is_action_just_pressed("g_next_building".into()) {
                commands.submit(move |world| {
                    let current_building: &mut CurrentBuildingIndexRes = world.resource_mut();
                    current_building.0 += 1;
                    if current_building.0 == building_len {
                        current_building.0 = 0;
                    }
                })
            }
            if input.is_action_just_pressed("g_prev_building".into()) {
                commands.submit(move |world| {
                    let current_building: &mut CurrentBuildingIndexRes = world.resource_mut();
                    if current_building.0 == 0 {
                        current_building.0 = building_len;
                    }
                    current_building.0 -= 1;
                })
            }
        }
    }
}

pub fn building_placer(
    player_node: &mut PlayerNodeRes,
    events: &mut UniverseEventStorageRes,
    root_node: &mut RootNodeRes,
    local: &mut PlacerRes,
    current_facing: &CurrentFacingRes,
    current_rotation: &CurrentBuildingRotationRes,
    current_building: &CurrentBuildingIndexRes,
    current_tile: &CurrentTileIndexRes,
    mode: &BuildingMode,
    changes: Changes,
) {
    if player_node.player.is_none() {
        return;
    };
    if changes.resource_changed::<BuildingMode>()
        || changes.resource_changed::<CurrentBuildingIndexRes>()
    {
        if let Some(temp_node) = &mut local.temp_build_node {
            temp_node.queue_free();
            local.temp_build_node = None;
        }
    }
    if *mode == BuildingMode::Disabled {
        return;
    }

    let pos = player_node.get_position();
    let cam = player_node
        .get_node("Camera3D".into())
        .unwrap()
        .cast::<Node3D>();
    let dir = -cam.get_global_transform().basis.col_c();
    let place_pos = pos + dir * 3.0;
    let place_tile = TilePos::from_godot(place_pos);
    let place_pos_q = place_tile.to_godot();
    let registry = Registry::instance();

    if local.temp_build_node.is_none() {
        let scene = match *mode {
            BuildingMode::Disabled => return,
            BuildingMode::Tiles => load::<PackedScene>("vessel/generic/wall_virtual.tscn"),
            BuildingMode::Buildings => registry.scene_by_building_index(current_building.0),
        };

        let node = scene.instantiate().unwrap();
        root_node.add_child(node.clone());
        let node = node.cast::<Node3D>();
        local.temp_build_node = Some(node);
    }

    if let Some(b_node) = &mut local.temp_build_node {
        b_node.set_position(place_pos_q);
        let rotated_basis = current_facing.to_basis().rotate_by(current_rotation.0);
        let final_basis = match *mode {
            BuildingMode::Disabled => return,
            BuildingMode::Tiles => rotated_basis,
            BuildingMode::Buildings => rotated_basis.for_buildings(),
        };
        b_node.set_basis(final_basis.to_godot());
    }

    if Input::singleton().is_action_just_pressed("g_place".into()) {
        match *mode {
            BuildingMode::Disabled => return,
            BuildingMode::Tiles => {
                events.push(universe::UniverseEvent::PlaceTile {
                    position: place_tile,
                    orientation: BuildingOrientation::new(current_facing.0, current_rotation.0),
                    kind: registry.tiles[current_tile.0].kind,
                });
            }
            BuildingMode::Buildings => {
                events.push(universe::UniverseEvent::PlaceBuilding {
                    position: place_tile,
                    orientation: BuildingOrientation::new(current_facing.0, current_rotation.0),
                    kind: registry.buildings[current_building.0].kind,
                });
            }
        }
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
            match kind {
                BodyKind::Tile { index, position } => {
                    events.push(universe::UniverseEvent::RemoveTile { position, index })
                }
                BodyKind::Building { entity } => {
                    events.push(universe::UniverseEvent::RemoveBuilding { entity })
                }
            }
        }
    }
}

pub fn build_mode_switch(commands: Commands) {
    if Input::singleton().is_action_just_pressed("g_build".into()) {
        commands.submit(|world| {
            let current = *world.resource::<BuildingMode>();
            let new = match current {
                BuildingMode::Disabled => BuildingMode::Tiles,
                BuildingMode::Tiles => BuildingMode::Buildings,
                BuildingMode::Buildings => BuildingMode::Disabled,
            };
            *world.resource_mut() = new;
        });
    }
}
