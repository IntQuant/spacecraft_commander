use bevy_ecs::{entity::Entity, event::Events};
use bevy_reflect::TypeRegistryArc;
use engine_num::Vec3;

use std::{mem, sync::OnceLock};

use crate::ecs::{
    evs::PlayerConnected,
    ids::{PlayerID, VesselEnt},
    player::{Player, PlayerMap},
    vessel::{DefaultVessel, VesselTiles},
};

static TYPE_REGISTRY: OnceLock<TypeRegistryArc> = OnceLock::new();

fn init_type_registry() -> TypeRegistryArc {
    let registry_arc = TypeRegistryArc::default();
    let mut reg = registry_arc.write();
    reg.register::<Vec3>();
    reg.register::<Entity>();
    reg.register::<PlayerID>();
    reg.register::<VesselEnt>();

    reg.register::<PlayerMap>();
    reg.register::<DefaultVessel>();
    reg.register::<PlayerID>();
    reg.register::<VesselEnt>();
    reg.register::<Player>();
    reg.register::<VesselTiles>();
    mem::drop(reg);
    registry_arc
}

pub fn get_type_registry() -> &'static TypeRegistryArc {
    TYPE_REGISTRY.get_or_init(init_type_registry)
}
