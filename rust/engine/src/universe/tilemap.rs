use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use super::ui_events::UiEventCtx;

pub type DefVec<T> = SmallVec<[T; 4]>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tile {}

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct TileMap<T> {
    tiles: IndexMap<TilePos, DefVec<T>>,
}

impl<T: Clone> TileMap<T> {
    pub fn new() -> Self {
        Self {
            tiles: IndexMap::new(),
        }
    }

    pub fn get_all_at(&self, pos: TilePos) -> DefVec<T> {
        self.tiles.get(&pos).cloned().unwrap_or_default()
    }
    pub fn add_at(&mut self, evctx: &mut UiEventCtx, pos: TilePos, tile: T) {
        evctx.tiles_changed.push(pos);
        self.tiles.entry(pos).or_default().push(tile)
    }
    pub fn iter(&self) -> impl Iterator<Item = (TilePos, &T)> + '_ {
        self.tiles
            .iter()
            .flat_map(|(k, v)| v.iter().map(|t| (*k, t)))
    }
}
