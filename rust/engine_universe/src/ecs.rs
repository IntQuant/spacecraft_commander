pub(crate) mod bridge;
pub(crate) mod player;
pub(crate) mod vessel;

pub mod res {
    pub use crate::ecs::{player::PlayerMap, vessel::DefaultVessel};
}
pub mod ids {
    pub use crate::ecs::{player::PlayerID, vessel::VesselEnt};
}
pub mod cmp {
    pub use crate::ecs::{player::Player, vessel::VesselTiles};
}
pub mod sys {
    pub(crate) use crate::ecs::{bridge::input_event_producer, player::on_player_connected};
}
pub mod evs {
    pub use crate::ecs::bridge::PlayerConnected;
}
