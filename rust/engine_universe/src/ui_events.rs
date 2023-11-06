use super::tilemap::TilePos;

#[derive(Default)]
pub struct UiEventCtx {
    pub tiles_changed: Vec<TilePos>,
}
