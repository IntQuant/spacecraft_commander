use std::collections::HashSet;

use anyhow::anyhow;
use godot::{
    engine::CharacterBody3D,
    prelude::{load, Gd, Node, Node3D, PackedScene, SceneTree, ToVariant, Vector2, Vector3},
};
use tracing::{info, warn};

use crate::{
    universe::{self, tilemap::TilePos, ui_events::UiEventCtx, PlayerID, Universe, VesselID},
    util::{IntoGodot, SceneTreeExt},
};

use self::systems::player_controls;

mod systems;

#[derive(Default)]
pub struct InputState {
    pub mouse_rel: Vector2,
}

/// Ui context that lives for a duration of a single frame or update.
///
/// Has references to everything that should be available from ui.
pub struct UiInCtx<'a> {
    pub my_id: PlayerID,
    pub universe: &'a Universe,
    pub scene: &'a mut SceneTree,
    pub base: &'a mut Node3D,
    pub state: &'a mut UiState,
    pub dt: f32,
    pub events: Vec<universe::UniverseEvent>,
    pub input: &'a InputState,
}

/// Persistent Ui state.
pub struct UiState {
    first_update: bool,
    shown_tiles: Vec<Gd<Node>>,
    my_player_node: Option<Gd<CharacterBody3D>>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            first_update: true,
            shown_tiles: Vec::new(),
            my_player_node: None,
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
        self.upload_current_vessel().unwrap(); // TODO unwrap
    }

    /// Called (ideally) 60 times per second.
    ///
    /// Not synced to universe updates.
    fn on_update(&mut self, evctx: UiEventCtx) {
        self.update_players_on_vessel();
        self.update_tiles(&evctx.tiles_changed).unwrap(); // TODO unwrap
        player_controls(self)
    }

    /// Called before frame is rendered.
    pub fn on_render(&mut self) {
        let players = self.scene.get_nodes_in_group("players".into());
        for player in players.iter_shared() {
            let mut player = player.cast::<CharacterBody3D>();
            let player_id = PlayerID(player.get("player".into()).to::<u32>());
            if self.my_id != player_id {
                if let Some(player_info) = self.universe.players.get(&player_id) {
                    player.set_position(player_info.position.into_godot()); // TODO interpolate
                } else {
                    warn!("Player {:?} not found", player_id)
                }
            }
        }
    }

    fn update_players_on_vessel(&mut self) {
        let Some(my_player) = self.universe.players.get(&self.my_id) else {
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
            let is_me = self.my_id == player_id;
            if is_me {
                self.state.my_player_node = Some(player_node.clone().cast());
            }
            player_node.set("controlled".into(), is_me.to_variant());
            player_node.add_to_group("players".into());
            if let Some(player_info) = self.universe.players.get(&player_id) {
                let position = player_info.position.into_godot();
                player_node
                    .clone()
                    .cast::<CharacterBody3D>()
                    .set_position(position)
            } else {
                warn!("Player {:?} not found", player_id)
            }
            self.base.add_child(player_node);
        }
    }

    fn update_tiles(&mut self, tiles_changed: &[TilePos]) -> anyhow::Result<()> {
        if !tiles_changed.is_empty() {
            self.upload_current_vessel()?;
        }
        Ok(())
    }

    fn upload_current_vessel(&mut self) -> Result<(), anyhow::Error> {
        for shown in &mut self.state.shown_tiles {
            shown.queue_free()
        }
        self.state.shown_tiles.clear();
        let vessel = self
            .universe
            .vessels
            .get(&self.my_vessel_id()?)
            .ok_or_else(|| anyhow!("given vessel does not exist"))?;
        let wall_scene = load::<PackedScene>("vessel/walls/wall1.tscn");
        Ok(for (pos, tile) in vessel.tiles.iter() {
            let node = wall_scene.instantiate().unwrap();
            node.clone().cast::<Node3D>().set_position(Vector3 {
                x: pos.x as f32 * 2.0,
                y: pos.y as f32 * 2.0,
                z: pos.z as f32 * 2.0,
            });
            node.clone().cast::<Node3D>().set_rotation_degrees(Vector3 {
                x: -90.0,
                y: 0.0,
                z: 0.0,
            });
            self.base.add_child(node.clone());
            self.state.shown_tiles.push(node);
        })
    }
}
