use bevy_ecs::{
    event::{Event, EventWriter},
    system::{Res, Resource},
};

use crate::OwnedUniverseEvent;

use super::ids::PlayerID;

#[derive(Resource)]
pub(crate) struct PendingEventsRes(pub Vec<OwnedUniverseEvent>);

#[derive(Event)]
pub struct PlayerConnected(pub PlayerID);

pub(crate) fn input_event_producer(
    pending_events: Res<PendingEventsRes>,
    mut player_connected: EventWriter<PlayerConnected>,
) {
    for event in &pending_events.0 {
        match event.event {
            crate::UniverseEvent::PlayerConnected => {
                player_connected.send(PlayerConnected(event.player_id))
            }
            _ => {}
        }
    }
}
