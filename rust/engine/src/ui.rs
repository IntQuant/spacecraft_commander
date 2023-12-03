use std::{mem, sync::Arc};

use engine_ecs::{World, WorldRun};
use godot::prelude::{Gd, Node3D};
use universe::mcs::PlayerID;

use crate::universe::{self, ui_events::UiEventCtx, Universe};

pub(crate) mod uecs;

use self::{
    resources::{
        CurrentPlayerRes, EvCtxRes, InputStateRes, RootNodeRes, SceneTreeRes,
        UniverseEventStorageRes, UniverseRes,
    },
    systems::{
        building_facing, building_placer, building_remover, player_controls, update_current_vessel,
        update_player_positions, update_players_on_vessel, upload_current_vessel_tiles,
        vessel_upload_condition,
    },
};

pub mod resources;
mod systems;

pub struct Ui {
    world: World<uecs::ComponentStorage>,
    first_update: bool,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
struct UpdateSchedule;

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
struct RenderSchedule;

impl Ui {
    pub fn new() -> Self {
        let world = World::new();

        Self {
            world,
            first_update: true,
        }
    }

    pub fn add_temporal_resources(
        &mut self,
        universe: Arc<Universe>,
        input: InputStateRes,
        root: Gd<Node3D>,
        my_id: PlayerID,
    ) {
        *self.world.resource_mut() = UniverseRes(Some(universe));
        *self.world.resource_mut() = input;
        *self.world.resource_mut() = SceneTreeRes(Some(root.get_tree().unwrap()));
        *self.world.resource_mut() = RootNodeRes(Some(root));
        *self.world.resource_mut() = CurrentPlayerRes(Some(my_id));
        *self.world.resource_mut() = UniverseEventStorageRes(Vec::new());
    }
    pub fn remove_temporal_resources(&mut self) -> Vec<universe::UniverseEvent> {
        *self.world.resource_mut() = UniverseRes(None);
        *self.world.resource_mut() = SceneTreeRes(None);
        *self.world.resource_mut() = RootNodeRes(None);
        *self.world.resource_mut() = CurrentPlayerRes(None);

        mem::take(self.world.resource_mut::<UniverseEventStorageRes>()).0
    }

    pub fn on_update(&mut self, evctx: UiEventCtx) {
        *self.world.resource_mut() = EvCtxRes(evctx);
        let query_world = self.world.query_world();
        query_world.run(update_current_vessel);
        query_world.run(update_players_on_vessel);
        let upload_cond = query_world.run(vessel_upload_condition);
        if upload_cond || self.first_update {
            query_world.run(upload_current_vessel_tiles);
        }
        query_world.run(player_controls);
        query_world.run(building_facing);
        query_world.run(building_placer);
        query_world.run(building_remover);

        self.first_update = false;
    }
    pub fn on_render(&mut self) {
        let query_world = self.world.query_world();
        query_world.run(update_player_positions);
    }
}
