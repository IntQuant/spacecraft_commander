use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct TilePos(u32, u32, u32);

#[derive(Default)]
pub struct TileMap<T> {
    tiles: IndexMap<(TilePos, u8), T>,
}

impl<T: Clone> TileMap<T> {
    pub fn get_all_at(&self, pos: TilePos) -> SmallVec<[T; 4]> {
        let mut ret = SmallVec::new();
        let mut i = 0;
        loop {
            if let Some(current) = self.tiles.get(&(pos, i)) {
                ret.push(current.clone());
                i += 1;
            } else {
                break;
            }
        }

        ret
    }
}
