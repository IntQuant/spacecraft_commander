use engine_num::Vec3;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    actions::Action, mcs::Player, tilemap::Tile, ui_events::UiEventCtx, OwnedUniverseEvent,
    UniverseEvent,
};

use super::{Building, Commands, DefaultVesselRes, PlayerMap, Query, VesselTiles};

#[derive(Default, Clone, Serialize, Deserialize)]
pub(crate) struct PendingEventsRes(pub(crate) Vec<OwnedUniverseEvent>);

pub(crate) fn system_handle_pending_events<'a>(
    pending_events: &mut PendingEventsRes,
    default_vessel: &DefaultVesselRes,
    player_map: &mut PlayerMap,
    evctx: &mut UiEventCtx,
    mut players: Query<'a, &'a mut Player>,
    mut vessels: Query<'a, &'a mut VesselTiles>,
    commands: Commands,
) {
    let mut actions = Vec::new();
    for event in &mut pending_events.0 {
        let player_id = event.player_id;
        match event.event {
            UniverseEvent::PlayerConnected => {
                let vessel = default_vessel.0;
                if player_map.get(player_id).is_none() {
                    info!("Creating player for {player_id:?}");
                    commands.submit(move |world| {
                        let ent = world.spawn(Player {
                            position: Vec3::new(0.0, 10.0, 0.0),
                            vessel,
                        });
                        world.resource_mut::<PlayerMap>().create(player_id, ent);
                        info!("Created player for {player_id:?}");
                    });
                }
            }
            UniverseEvent::PlayerMoved { new_position } => {
                let Some(player_ent) = player_map.get(player_id) else {
                    continue;
                };
                if let Some(player) = players.get(player_ent) {
                    player.position = new_position;
                }
                actions.push(Action::MovePlayer {
                    player: player_ent,
                    new_position: new_position,
                });
            }
            UniverseEvent::PlaceTile {
                position,
                orientation,
                kind,
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
                info!("Tile placed");
                vessel.0.add_at(evctx, position, Tile { orientation, kind });
                evctx.any_vessel_changed = true;
                actions.push(Action::PlaceTile {
                    vessel: player.vessel,
                    position,
                    orientation,
                    kind,
                })
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
                evctx.any_vessel_changed = true;
                actions.push(Action::RemoveTile {
                    vessel: player.vessel,
                    position,
                    index,
                })
            }
            UniverseEvent::PlaceBuilding {
                position,
                orientation,
                kind,
            } => {
                let Some(player_ent) = player_map.get(player_id) else {
                    continue;
                };
                let Some(player) = players.get(player_ent) else {
                    continue;
                };
                commands.submit(move |world| {
                    world.spawn(Building {
                        position,
                        orientation,
                        kind,
                    });
                });
                evctx.any_vessel_changed = true;
                actions.push(Action::PlaceBuilding {
                    vessel: player.vessel,
                    position,
                    orientation,
                    kind,
                })
            }
            UniverseEvent::RemoveBuilding { entity } => {
                let Some(player_ent) = player_map.get(player_id) else {
                    continue;
                };
                let Some(player) = players.get(player_ent) else {
                    continue;
                };
                commands.submit(move |world| {
                    world.despawn(entity); // TODO check if it's actually a building
                });
                evctx.any_vessel_changed = true;
                actions.push(Action::RemoveBuilding {
                    vessel: player.vessel,
                    entity,
                })
            }
        }
    }
}
