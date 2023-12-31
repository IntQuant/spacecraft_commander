use engine_ecs::gen_storage_for_world;

use super::resources::{
    BuildingMode, CurrentBuildingIndexRes, CurrentBuildingRotationRes, CurrentFacingRes,
    CurrentPlayerRes, CurrentPlayerRotationRes, CurrentTileIndexRes, CurrentVesselRes, DtRes,
    EvCtxRes, InputStateRes, PlacerRes, PlayerNodeRes, RootNodeRes, SceneTreeRes,
    UniverseEventStorageRes, UniverseRes,
};

gen_storage_for_world!(
    : no_clone
    : no_serialize
    : components

    : resources
        CurrentFacingRes CurrentPlayerRes CurrentPlayerRotationRes CurrentVesselRes DtRes
        EvCtxRes InputStateRes PlayerNodeRes RootNodeRes SceneTreeRes UniverseEventStorageRes
        UniverseRes PlacerRes BuildingMode CurrentBuildingRotationRes CurrentBuildingIndexRes
        CurrentTileIndexRes
);
