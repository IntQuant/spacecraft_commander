pub(crate) mod events;
pub(crate) mod player;
pub(crate) mod vessel;

use engine_macro::gen_storage_for_world;
// pub use bridge::*;
pub use player::*;
pub use vessel::*;

gen_storage_for_world!(
    : components
        VesselTiles
    : resources
        DefaultVesselRes
);
