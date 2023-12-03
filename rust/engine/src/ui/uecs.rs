use engine_ecs::gen_storage_for_world;

use super::resources::{
    CurrentFacingRes, CurrentPlayerRes, CurrentPlayerRotationRes, CurrentVesselRes, DtRes,
    EvCtxRes, InputStateRes, PlayerNodeRes, RootNodeRes, SceneTreeRes, UniverseEventStorageRes,
    UniverseRes,
};

gen_storage_for_world!(
    : no_clone
    : no_serialize
    : components

    : resources
        CurrentFacingRes CurrentPlayerRes CurrentPlayerRotationRes CurrentVesselRes DtRes
        EvCtxRes InputStateRes PlayerNodeRes RootNodeRes SceneTreeRes UniverseEventStorageRes
        UniverseRes
);
