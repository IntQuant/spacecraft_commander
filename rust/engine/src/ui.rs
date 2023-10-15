use std::collections::HashSet;

use godot::{
    engine::CharacterBody3D,
    prelude::{load, Node3D, PackedScene, SceneTree, ToVariant},
};
use tracing::{info, warn};

use crate::{
    netman::NetmanVariant,
    universe::{PlayerID, Universe},
    util::{IntoGodot, SceneTreeExt},
};

/// Ui context that lives for a duration of a single frame or update.
///
/// Has references to everything that should be available from ui.
pub struct UiInCtx<'a> {
    pub netman: &'a NetmanVariant,
    pub universe: &'a Universe,
    pub scene: &'a mut SceneTree,
    pub base: &'a mut Node3D,
    pub state: &'a mut UiState,
}

/// Persistent Ui state.
pub struct UiState {}

impl UiState {
    pub fn new() -> Self {
        Self {}
    }
}

impl UiInCtx<'_> {
    /// Called (ideally) 60 times per second.
    ///
    /// Not synced to universe updates.
    pub fn on_update(&mut self) {
        self.update_players_on_vessel();
    }

    /// Called before frame is rendered.
    pub fn on_render(&mut self) {
        let my_id = self.netman.my_id();
        let players = self.scene.get_nodes_in_group("players".into());
        for player in players.iter_shared() {
            let mut player = player.cast::<CharacterBody3D>();
            let player_id = PlayerID(player.get("player".into()).to::<u32>());
            if my_id != Some(player_id) {
                if let Some(player_info) = self.universe.players.get(&player_id) {
                    player.set_position(player_info.position.into_godot()); // TODO interpolate
                } else {
                    warn!("Player {:?} not found", player_id)
                }
            }
        }
    }

    fn update_players_on_vessel(&mut self) {
        let Some(my_id) = self.netman.my_id() else {
            return;
        };
        let Some(my_player) = self.universe.players.get(&my_id) else {
            return;
        };
        let current_vessel = my_player.vessel;

        let mut on_current_vessel: HashSet<_> = self
            .universe
            .players
            .iter()
            .filter(|(_id, player)| player.vessel == current_vessel)
            .map(|(id, _player)| *id)
            .collect();

        for mut player_character in self.scene.iter_group::<CharacterBody3D>("players") {
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
            player_node.set("controlled".into(), (my_id == player_id).to_variant());
            player_node.add_to_group("players".into());
            self.base.add_child(player_node);
        }
    }
}
