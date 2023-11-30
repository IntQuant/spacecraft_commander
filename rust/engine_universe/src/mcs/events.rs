use engine_num::Vec3;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    mcs::Player, tilemap::Tile, ui_events::UiEventCtx, OwnedUniverseEvent, Universe, UniverseEvent,
};

use super::{DefaultVesselRes, PlayerMap, Query, VesselTiles};

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct PendingEventsRes(pub(crate) Vec<OwnedUniverseEvent>);

pub(crate) fn system_handle_pending_events<'a>(
    pending_events: &mut PendingEventsRes,
    default_vessel: &DefaultVesselRes,
    player_map: &mut PlayerMap,
    evctx: &mut UiEventCtx,
    players: Query<'a, &'a Player>,
    vessels: Query<'a, &'a mut VesselTiles>,
) {
    for event in &mut pending_events.0 {
        let player_id = event.player_id;
        match event.event {
            UniverseEvent::PlayerConnected => {
                info!("Creating player for {player_id:?}");
                universe.players.entry(player_id).or_insert(Player {
                    position: Vec3::new(0.0, 10.0, 0.0),
                    vessel: default_vessel.0,
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
                let Some(player_ent) = player_map.get(player_id) else {
                    continue;
                };
                let Some(player) = players.get(player_ent) else {
                    continue;
                };
                let Some(vessel) = vessels.get(player.vessel.0) else {
                    continue;
                };
                vessel.0.add_at(evctx, position, Tile { orientation });
            }
            UniverseEvent::RemoveTile { position, index } => {
                let Some(player_ent) = player_map.get(player_id) else {
                    continue;
                };
                let Some(player) = players.get(player_ent) else {
                    continue;
                };
                let Some(vessel) = vessels.get(player.vessel.0) else {
                    continue;
                };
                vessel.0.remove_at(evctx, position, index);
            }
        }
    }
}
