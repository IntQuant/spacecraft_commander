use engine_num::Vec3;
use serde::{Deserialize, Serialize};

use super::VesselID;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PlayerID(pub u32);

#[derive(Serialize, Deserialize, Clone)]
pub struct Player {
    pub position: Vec3,
    pub vessel: VesselID,
}
