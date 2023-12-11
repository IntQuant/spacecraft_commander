use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub struct BuildingKind(u32);

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default)]
pub struct TileKind(u32);

pub struct BuildingEntry {
    pub kind: BuildingKind,
    pub name: &'static str,
}

pub struct TileEntry {
    pub kind: TileKind,
    pub name: &'static str,
}

pub struct Registry {
    pub buildings: Vec<BuildingEntry>,
    pub tiles: Vec<TileEntry>,
}

impl Registry {
    fn new() -> Self {
        Self {
            buildings: vec![
                BuildingEntry {
                    kind: BuildingKind(0),
                    name: "light00",
                },
                BuildingEntry {
                    kind: BuildingKind(1),
                    name: "control00",
                },
            ],
            tiles: vec![
                TileEntry {
                    kind: TileKind(0),
                    name: "wall_normal",
                },
                TileEntry {
                    kind: TileKind(1),
                    name: "wall_glass",
                },
            ],
        }
    }

    pub fn instance() -> &'static Self {
        static REGISTRY: OnceLock<Registry> = OnceLock::new();
        REGISTRY.get_or_init(|| Self::new())
    }

    pub fn building_by_kind(&self, kind: BuildingKind) -> Option<&BuildingEntry> {
        self.buildings.iter().find(|x| x.kind == kind)
    }

    pub fn tile_by_kind(&self, kind: TileKind) -> Option<&TileEntry> {
        self.tiles.iter().find(|x| x.kind == kind)
    }
}
