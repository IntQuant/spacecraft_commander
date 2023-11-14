use std::collections::HashMap;

use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use super::{
    rotations::{BuildingFacing, BuildingRotation, CompactBasis},
    ui_events::UiEventCtx,
};

pub type DefVec<T> = SmallVec<[T; 4]>;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, Reflect)]
pub struct TileOrientation {
    pub facing: BuildingFacing,
    pub rotation: BuildingRotation,
}

impl TileOrientation {
    pub fn new(facing: BuildingFacing, rotation: BuildingRotation) -> Self {
        Self { facing, rotation }
    }
    pub fn to_basis(&self) -> CompactBasis {
        self.facing.to_basis()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Reflect)]
pub struct Tile {
    pub orientation: TileOrientation,
}

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy, Reflect)]
pub struct TilePos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl TilePos {
    pub const GRID_STEP: f32 = 2.0;
}

pub type TileIndex = u8;

#[derive(Default, Serialize, Deserialize, Clone, Reflect)]
pub struct TileMap<T> {
    tiles: HashMap<TilePos, DefVec<T>>,
}

impl<T: Clone> TileMap<T> {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
        }
    }

    pub fn get_all_at(&self, pos: TilePos) -> DefVec<T> {
        self.tiles.get(&pos).cloned().unwrap_or_default()
    }
    pub fn add_at(&mut self, evctx: &mut UiEventCtx, pos: TilePos, tile: T) {
        evctx.tiles_changed.push(pos);
        self.tiles.entry(pos).or_default().push(tile)
    }
    pub fn remove_at(&mut self, evctx: &mut UiEventCtx, pos: TilePos, index: TileIndex) {
        evctx.tiles_changed.push(pos);
        let tile_list = self.tiles.entry(pos).or_default();
        let index = index as usize;
        if index < tile_list.len() {
            tile_list.swap_remove(index);
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (TileIndex, TilePos, &T)> + '_ {
        self.tiles
            .iter()
            .flat_map(|(k, v)| v.iter().enumerate().map(|(i, t)| (i as TileIndex, *k, t)))
    }
}
