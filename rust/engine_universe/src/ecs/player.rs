use std::collections::HashMap;

use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::EventReader,
    system::{Commands, Res, Resource},
    world::World,
};
use bevy_reflect::Reflect;
use engine_num::Vec3;
use serde::{Deserialize, Serialize};
use tracing::info;

use super::{
    evs::PlayerConnected,
    vessel::{DefaultVessel, VesselEnt},
};

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy, Reflect, Serialize, Deserialize)]
pub struct PlayerID(pub u32);

#[derive(Reflect, Resource, Default)]
pub struct PlayerMap {
    pub map: HashMap<PlayerID, Entity>,
}

#[derive(Reflect, Component)]
pub struct Player {
    pub position: Vec3,
    pub vessel: VesselEnt,
}

pub fn on_player_connected(
    mut players_connected: EventReader<PlayerConnected>,
    default_vessel: Res<DefaultVessel>,
    mut commands: Commands,
) {
    let vessel = default_vessel.0;
    for event in players_connected.read() {
        let player_id = event.0;
        commands.add(move |world: &mut World| {
            info!("Adding player {player_id:?}");
            let ent = world
                .spawn(Player {
                    position: Vec3 {
                        x: 0.0,
                        y: 10.0,
                        z: 0.0,
                    },
                    vessel,
                })
                .id();
            let mut player_map = world.resource_mut::<PlayerMap>();
            player_map.map.insert(player_id, ent);
        });
    }
}
