use engine_num::Vec3;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    mcs::Player, tilemap::Tile, ui_events::UiEventCtx, OwnedUniverseEvent, Universe, UniverseEvent,
};

use super::DefaultVesselRes;

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct PendingEventsRes(pub(crate) Vec<OwnedUniverseEvent>);

pub(crate) fn system_handle_pending_events(
    pending_events: &mut PendingEventsRes,
    default_vessel: &DefaultVesselRes,
    //evctx: &mut UiEventCtx,
) {
    for event in &mut pending_events.0 {
        let player_id = event.player_id;
        // match event.event {
        //     UniverseEvent::PlayerConnected => {
        //         info!("Creating player for {player_id:?}");
        //         universe.players.entry(player_id).or_insert(Player {
        //             position: Vec3::new(0.0, 10.0, 0.0),
        //             vessel: default_vessel.0,
        //         });
        //     }
        //     UniverseEvent::PlayerMoved { new_position } => {
        //         if let Some(player) = universe.players.get_mut(&player_id) {
        //             player.position = new_position;
        //         }
        //     }
        //     UniverseEvent::PlaceTile {
        //         position,
        //         orientation,
        //     } => {
        //         let Some(player) = universe.players.get(&player_id) else {
        //             continue;
        //         };
        //         let Some(vessel) = universe.vessels.get_mut(player.vessel) else {
        //             continue;
        //         };
        //         vessel.0.add_at(evctx, position, Tile { orientation });
        //     }
        //     UniverseEvent::RemoveTile { position, index } => {
        //         let Some(player) = universe.players.get(&player_id) else {
        //             continue;
        //         };
        //         let Some(vessel) = universe.vessels.get_mut(player.vessel) else {
        //             continue;
        //         };
        //         vessel.0.remove_at(evctx, position, index);
        //     }
        // }
    }
}
