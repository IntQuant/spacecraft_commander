#[cfg(test)]
mod tests {
    use engine_ecs::World;
    use engine_macro::gen_storage_for_world;
    use serde::{Deserialize, Serialize};

    #[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Component1(u8);
    #[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Component2(u32);
    #[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Component3(u16);

    gen_storage_for_world! { Component1 Component2 Component3 }

    #[test]
    fn entity_spawned() {
        let mut world = World::<ComponentStorage>::new();
        world.spawn((Component1(0), Component2(1), Component3(2)));
        world.spawn((Component1(3), Component2(4), Component3(5)));
        world.spawn((Component1(6), Component2(7)));
        assert_eq!(world.entity_count(), 3)
    }

    #[test]
    fn entity_spawn_and_get() {
        let mut world = World::<ComponentStorage>::new();
        let ent1 = world.spawn((Component1(0), Component2(1), Component3(2)));
        let ent2 = world.spawn((Component2(7),));
        let ent3 = world.spawn((Component1(3), Component2(4), Component3(5)));

        assert_eq!(
            world.get::<Component1>(ent1).as_deref(),
            Some(Component1(0)).as_ref()
        );
        assert_eq!(
            world.get::<Component2>(ent1).as_deref(),
            Some(Component2(1)).as_ref()
        );
        assert_eq!(
            world.get::<Component3>(ent1).as_deref(),
            Some(Component3(2)).as_ref()
        );
        assert_eq!(
            world.get::<Component2>(ent2).as_deref(),
            Some(Component2(7)).as_ref()
        );
        assert_eq!(
            world.get::<Component1>(ent3).as_deref(),
            Some(Component1(3)).as_ref()
        );
        assert_eq!(
            world.get::<Component2>(ent3).as_deref(),
            Some(Component2(4)).as_ref()
        );
        assert_eq!(
            world.get::<Component3>(ent3).as_deref(),
            Some(Component3(5)).as_ref()
        );
    }

    #[test]
    fn entity_spawn_and_get_mut() {
        let mut world = World::<ComponentStorage>::new();
        let ent1 = world.spawn((Component1(0), Component2(1), Component3(2)));
        assert_eq!(
            world.get::<Component1>(ent1).as_deref(),
            Some(Component1(0)).as_ref()
        );
        world.get_mut::<Component1>(ent1).unwrap().0 = 10;
        assert_eq!(
            world.get::<Component1>(ent1).as_deref(),
            Some(Component1(10)).as_ref()
        );
    }

    #[test]
    fn entity_spawn_and_despawn() {
        let mut world = World::<ComponentStorage>::new();
        let ent1 = world.spawn(Component1(0));
        let ent2 = world.spawn(Component1(2));
        assert_eq!(
            world.get::<Component1>(ent1).as_deref(),
            Some(Component1(0)).as_ref()
        );
        assert_eq!(
            world.get::<Component1>(ent2).as_deref(),
            Some(Component1(2)).as_ref()
        );
        assert_eq!(world.entity_count(), 2);
        world.despawn(ent1);
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.get::<Component1>(ent1).as_deref(), None);
        assert_eq!(
            world.get::<Component1>(ent2).as_deref(),
            Some(Component1(2)).as_ref()
        );
    }

    #[test]
    fn roundtrip_bincode() {
        let mut world_ini = World::<ComponentStorage>::new();
        let ent1 = world_ini.spawn((Component1(0), (Component2(1), Component3(2))));
        let ent2 = world_ini.spawn((Component2(7),));
        let ent3 = world_ini.spawn((Component1(3), Component2(4), Component3(5)));

        let world: World<ComponentStorage> =
            bincode::deserialize(&bincode::serialize(&world_ini).unwrap()).unwrap();

        assert_eq!(
            world.get::<Component1>(ent1).as_deref(),
            Some(Component1(0)).as_ref()
        );
        assert_eq!(
            world.get::<Component2>(ent1).as_deref(),
            Some(Component2(1)).as_ref()
        );
        assert_eq!(
            world.get::<Component3>(ent1).as_deref(),
            Some(Component3(2)).as_ref()
        );
        assert_eq!(
            world.get::<Component2>(ent2).as_deref(),
            Some(Component2(7)).as_ref()
        );
        assert_eq!(
            world.get::<Component1>(ent3).as_deref(),
            Some(Component1(3)).as_ref()
        );
        assert_eq!(
            world.get::<Component2>(ent3).as_deref(),
            Some(Component2(4)).as_ref()
        );
        assert_eq!(
            world.get::<Component3>(ent3).as_deref(),
            Some(Component3(5)).as_ref()
        )
    }
}
