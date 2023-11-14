use bevy_reflect::TypeRegistryArc;

use std::sync::OnceLock;

static TYPE_REGISTRY: OnceLock<TypeRegistryArc> = OnceLock::new();

fn init_type_registry() -> TypeRegistryArc {
    let registry = TypeRegistryArc::default();

    registry
}

pub fn get_type_registry() -> &'static TypeRegistryArc {
    TYPE_REGISTRY.get_or_init(init_type_registry)
}
