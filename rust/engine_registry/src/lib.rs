use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct BuildingKind(u32);

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
}
