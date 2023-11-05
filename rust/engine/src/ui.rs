use std::sync::Arc;

use anyhow::anyhow;
use bevy_ecs::{
    schedule::{IntoSystemConfigs, Schedule, ScheduleLabel},
    world::World,
};
use godot::{
    engine::CharacterBody3D,
    prelude::{Gd, Node, Node3D, SceneTree},
};
use tracing::warn;

use crate::{
    universe::{self, tilemap::TilePos, ui_events::UiEventCtx, PlayerID, Universe, VesselID},
    util::IntoGodot,
};

use self::{
    resources::{
        CurrentPlayer, CurrentVessel, Dt, EvCtx, InputState, PlayerNode, RootNode, SceneTreeRes,
        UniverseEventStorage, UniverseResource,
    },
    systems::{
        player_controls, player_placer, update_player_positions, update_players_on_vessel,
        upload_current_vessel, vessel_upload_condition, PlacerLocal,
    },
};

pub mod resources;
mod systems;

/// Ui context that lives for a duration of a single frame or update.
///
/// Has references to everything that should be available from ui.
pub struct UiInCtx<'a> {
    pub my_id: PlayerID,
    pub universe: &'a Universe,
    pub scene: &'a mut SceneTree,
    pub base: &'a mut Node3D,
    pub state: &'a mut UiState2,
    pub dt: f32,
    pub events: Vec<universe::UniverseEvent>,
    pub input: &'a InputState,
}

/// Persistent Ui state.
pub struct UiState2 {
    first_update: bool,
    shown_tiles: Vec<Gd<Node>>,
    my_player_node: Option<Gd<CharacterBody3D>>,
    temp_build_node: Option<Gd<Node3D>>,
}

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
            player_placer,
        ));

        schedule_render.add_systems(update_player_positions);

        world.insert_resource(CurrentVessel(VesselID(0)));
        world.insert_resource(Dt(1.0 / 60.0));
        world.insert_non_send_resource(None::<PlayerNode>);
        world.insert_non_send_resource(PlacerLocal::default());

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

impl UiState2 {
    pub fn new() -> Self {
        Self {
            first_update: true,
            shown_tiles: Vec::new(),
            my_player_node: None,
            temp_build_node: None,
        }
    }
}

impl UiInCtx<'_> {
    fn my_vessel_id(&self) -> anyhow::Result<VesselID> {
        let my_id = self.my_id;
        self.universe
            .players
            .get(&my_id)
            .map(|x| x.vessel)
            .ok_or_else(|| anyhow!("no player with this id"))
    }

    pub fn maybe_update(&mut self, evctx: UiEventCtx) {
        if self.state.first_update {
            self.state.first_update = false;
            self.on_init(evctx)
        } else {
            self.on_update(evctx)
        }
    }

    fn on_init(&mut self, _evctx: UiEventCtx) {
        // self.upload_current_vessel().unwrap(); // TODO unwrap
    }

    /// Called (ideally) 60 times per second.
    ///
    /// Not synced to universe updates.
    fn on_update(&mut self, evctx: UiEventCtx) {
        // self.update_players_on_vessel();
        self.update_tiles(&evctx.tiles_changed).unwrap(); // TODO unwrap
                                                          //player_controls(self);
                                                          // player_placer(self);
    }

    /// Called before frame is rendered.

    fn update_tiles(&mut self, tiles_changed: &[TilePos]) -> anyhow::Result<()> {
        if !tiles_changed.is_empty() {
            // self.upload_current_vessel()?;
        }
        Ok(())
    }
}
