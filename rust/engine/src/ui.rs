use std::sync::Arc;

use bevy_ecs::{
    schedule::{IntoSystemConfigs, Schedule, ScheduleLabel},
    world::World,
};
use godot::prelude::{Gd, Node3D};

use crate::universe::{self, ui_events::UiEventCtx, PlayerID, Universe, VesselID};

use self::{
    resources::{
        CurrentFacing, CurrentPlayer, CurrentVessel, Dt, EvCtx, InputState, PlayerNode, RootNode,
        SceneTreeRes, UniverseEventStorage, UniverseResource,
    },
    systems::{
        building_facing, building_placer, building_remover, player_controls,
        update_player_positions, update_players_on_vessel, upload_current_vessel,
        vessel_upload_condition, PlacerLocal,
    },
};

pub mod resources;
mod systems;

pub struct Ui {
    world: World,
    schedule_update: Schedule,
    schedule_render: Schedule,
}

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
struct UpdateSchedule;

#[derive(ScheduleLabel, Hash, PartialEq, Eq, Debug, Clone)]
struct RenderSchedule;

impl Ui {
    pub fn new() -> Self {
        let mut schedule_update = Schedule::new(UpdateSchedule);
        let mut world = World::new();
        let mut schedule_render = Schedule::new(RenderSchedule);

        schedule_update.add_systems((
            upload_current_vessel.run_if(vessel_upload_condition),
            update_players_on_vessel,
            player_controls,
            (building_facing, building_placer, building_remover),
        ));

        schedule_render.add_systems(update_player_positions);

        world.insert_resource(CurrentVessel(VesselID(0)));
        world.insert_resource(Dt(1.0 / 60.0));
        world.insert_non_send_resource(None::<PlayerNode>);
        world.insert_non_send_resource(PlacerLocal::default());
        world.insert_resource(CurrentFacing(universe::rotations::BuildingFacing::Px));

        Self {
            world,
            schedule_update,
            schedule_render,
        }
    }

    pub fn add_temporal_resources(
        &mut self,
        universe: Arc<Universe>,
        input: InputState,
        root: Gd<Node3D>,
        my_id: PlayerID,
    ) {
        self.world.insert_resource(UniverseResource(universe));
        self.world.insert_resource(input);
        self.world
            .insert_non_send_resource(SceneTreeRes(root.get_tree().unwrap()));
        self.world.insert_non_send_resource(RootNode(root));
        self.world.insert_resource(CurrentPlayer(my_id));
        self.world.insert_resource(UniverseEventStorage(Vec::new()));
    }
    pub fn remove_temporal_resources(&mut self) -> Vec<universe::UniverseEvent> {
        self.world.remove_resource::<UniverseResource>();
        self.world.remove_non_send_resource::<RootNode>();
        self.world
            .remove_resource::<UniverseEventStorage>()
            .unwrap()
            .0
    }

    pub fn on_update(&mut self, evctx: UiEventCtx) {
        self.world.insert_resource(EvCtx(evctx));
        self.schedule_update.run(&mut self.world);
        self.world.remove_resource::<EvCtx>();
        self.world.clear_trackers();
    }
    pub fn on_render(&mut self) {
        self.schedule_render.run(&mut self.world);
    }
}
