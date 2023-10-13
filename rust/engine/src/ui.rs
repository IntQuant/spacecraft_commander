use std::collections::HashSet;

use godot::{
    engine::CharacterBody3D,
    prelude::{load, Node3D, PackedScene, SceneTree, ToVariant},
};
use tracing::info;

use crate::{
    netman::NetmanVariant,
    universe::{PlayerID, Universe},
    util::SceneTreeExt,
};

pub struct UiInCtx<'a> {
    pub netman: &'a NetmanVariant,
    pub universe: &'a Universe,
    pub scene: &'a mut SceneTree,
    pub base: &'a mut Node3D,
}

pub struct Ui {}

impl Ui {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, ctx: &mut UiInCtx) {
        self.update_players_on_vessel(ctx);
    }

    fn update_players_on_vessel(&mut self, ctx: &mut UiInCtx) {
        let Some(my_id) = ctx.netman.my_id() else {
            return;
        };
        let Some(my_player) = ctx.universe.players.get(&my_id) else {
            return;
        };
        let current_vessel = my_player.vessel;

        let mut on_current_vessel: HashSet<_> = ctx
            .universe
            .players
            .iter()
            .filter(|(_id, player)| player.vessel == current_vessel)
            .map(|(id, _player)| *id)
            .collect();

        for mut player_character in ctx.scene.iter_group::<CharacterBody3D>("players") {
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
            ctx.base.add_child(player_node);
        }
    }
}
