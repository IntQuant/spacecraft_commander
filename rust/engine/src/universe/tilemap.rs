use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

const CHUNK_POS_SHIFT: u16 = 5;

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct ChunkPos(u16, u16, u16);

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct InPos(u16);

#[derive(Hash, PartialEq, Eq, Debug, Serialize, Deserialize, Clone, Copy)]
pub struct TilePos(u32, u32, u32);

struct Chunk<T> {
    items: Vec<(InPos, T)>,
}

pub struct TileMap<T> {
    chunks: IndexMap<ChunkPos, Chunk<T>>,
}
