pub(crate) mod events;
pub(crate) mod player;
pub(crate) mod vessel;

use engine_macro::gen_storage_for_world;
pub(crate) use events::*;
pub use player::*;
pub use vessel::*;

use crate::UiEventCtx;

gen_storage_for_world!(
    : components
        VesselTiles Player
    : resources
        DefaultVesselRes PendingEventsRes PlayerMap UiEventCtx
);
