use std::sync::Arc;

use bevy_ecs::{
    schedule::{IntoSystemConfigs, Schedule, ScheduleLabel},
    world::World,
};
use godot::prelude::{Gd, Node3D};
use universe::mcs::{PlayerID, VesselID};

use crate::universe::{self, ui_events::UiEventCtx, Universe};

pub(crate) mod uecs;

use self::{
    resources::{
        CurrentFacingRes, CurrentPlayerRes, CurrentPlayerRotationRes, CurrentVesselRes, DtRes,
        EvCtxRes, InputStateRes, PlayerNodeRes, RootNodeRes, SceneTreeRes, UniverseEventStorageRes,
        UniverseRes,
    },
    systems::{
        building_facing, building_placer, building_remover, player_controls, update_current_vessel,
        update_player_positions, update_players_on_vessel, upload_current_vessel_tiles,
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
            update_current_vessel,
            update_players_on_vessel.after(update_current_vessel),
            upload_current_vessel_tiles
                .run_if(vessel_upload_condition)
                .after(update_current_vessel),
            player_controls,
            (building_facing, building_placer, building_remover),
        ));

        schedule_render.add_systems(update_player_positions);

        world.insert_resource(CurrentVesselRes(VesselID::default()));
        world.insert_resource(DtRes(1.0 / 60.0));
        world.insert_non_send_resource(None::<PlayerNodeRes>);
        world.insert_non_send_resource(PlacerLocal::default());
        world.insert_resource(CurrentFacingRes(universe::rotations::BuildingFacing::Px));
        world.insert_resource(CurrentPlayerRotationRes::default());

        Self {
            world,
            schedule_update,
            schedule_render,
        }
    }

    pub fn add_temporal_resources(
        &mut self,
        universe: Arc<Universe>,
        input: InputStateRes,
        root: Gd<Node3D>,
        my_id: PlayerID,
    ) {
        self.world.insert_resource(UniverseRes(Some(universe)));
        self.world.insert_resource(input);
        self.world
            .insert_non_send_resource(SceneTreeRes(Some(root.get_tree().unwrap())));
        self.world.insert_non_send_resource(RootNodeRes(Some(root)));
        self.world.insert_resource(CurrentPlayerRes(Some(my_id)));
        self.world
            .insert_resource(UniverseEventStorageRes(Vec::new()));
    }
    pub fn remove_temporal_resources(&mut self) -> Vec<universe::UniverseEvent> {
        self.world.remove_resource::<UniverseRes>();
        self.world.remove_non_send_resource::<RootNodeRes>();
        self.world
            .remove_resource::<UniverseEventStorageRes>()
            .unwrap()
            .0
    }

    pub fn on_update(&mut self, evctx: UiEventCtx) {
        self.world.insert_resource(EvCtxRes(evctx));
        self.schedule_update.run(&mut self.world);
        self.world.remove_resource::<EvCtxRes>();
        self.world.clear_trackers();
    }
    pub fn on_render(&mut self) {
        self.schedule_render.run(&mut self.world);
    }
}
