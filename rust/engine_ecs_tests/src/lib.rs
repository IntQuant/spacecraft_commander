#[cfg(test)]
mod tests {
    use engine_ecs::World;
    use engine_macro::gen_storage_for_world;
    use serde::{Deserialize, Serialize};

    #[derive(Default, Clone, Serialize, Deserialize)]
    struct Component1;
    #[derive(Default, Clone, Serialize, Deserialize)]
    struct Component2;
    #[derive(Default, Clone, Serialize, Deserialize)]
    struct Component3;

    gen_storage_for_world! { Component1 Component2 Component3 }

    #[test]
    fn entity_spawned() {
        let mut world = World::<ComponentStorage>::new();
        //let bundle: Bundle<_, _, ComponentStorage> = (Component1, Component2).into();
        world.spawn((Component1, Component2, Component3));
    }
}
