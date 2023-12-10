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

pub struct Registry {
    pub buildings: Vec<BuildingEntry>,
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
        }
    }

    pub fn instance() -> &'static Self {
        static REGISTRY: OnceLock<Registry> = OnceLock::new();
        REGISTRY.get_or_init(|| Self::new())
    }

    pub fn building_by_kind(&self, kind: BuildingKind) -> Option<&BuildingEntry> {
        self.buildings.iter().find(|x| x.kind == kind)
    }
}
