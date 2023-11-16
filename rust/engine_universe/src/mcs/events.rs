use engine_num::Vec3;
use tracing::info;

use crate::{mcs::Player, tilemap::Tile, ui_events::UiEventCtx, Universe, UniverseEvent};

pub(crate) fn system_handle_pending_events(universe: &mut Universe, evctx: &mut UiEventCtx) {
    for event in &universe.pending_events {
        let player_id = event.player_id;
        match event.event {
            UniverseEvent::PlayerConnected => {
                info!("Creating player for {player_id:?}");
                universe.players.entry(player_id).or_insert(Player {
                    position: Vec3::new(0.0, 10.0, 0.0),
                    vessel: universe.default_vessel.0,
                });
            }
            UniverseEvent::PlayerMoved { new_position } => {
                if let Some(player) = universe.players.get_mut(&player_id) {
                    player.position = new_position;
                }
            }
            UniverseEvent::PlaceTile {
                position,
                orientation,
            } => {
                let Some(player) = universe.players.get(&player_id) else {
                    continue;
                };
                let Some(vessel) = universe.vessels.get_mut(player.vessel) else {
                    continue;
                };
                vessel.0.add_at(evctx, position, Tile { orientation });
            }
            UniverseEvent::RemoveTile { position, index } => {
                let Some(player) = universe.players.get(&player_id) else {
                    continue;
                };
                let Some(vessel) = universe.vessels.get_mut(player.vessel) else {
                    continue;
                };
                vessel.0.remove_at(evctx, position, index);
            }
        }
    }
}
