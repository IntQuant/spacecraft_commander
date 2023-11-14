pub(crate) mod player;
pub(crate) mod vessel;

pub mod res {
    pub use crate::ecs::player::PlayerMap;
}
pub mod ids {
    pub use crate::ecs::{player::PlayerID, vessel::VesselID};
}
pub mod cmp {
    pub use crate::ecs::{player::Player, vessel::VesselTiles};
}
