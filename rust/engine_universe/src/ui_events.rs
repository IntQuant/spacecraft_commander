use serde::{Deserialize, Serialize};

use super::tilemap::TilePos;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct UiEventCtx {
    pub tiles_changed: Vec<TilePos>,
}
