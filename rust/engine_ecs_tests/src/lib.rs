#[cfg(test)]
mod tests {
    use std::mem;

    use engine_ecs::{EntityID, World, WorldRun};
    use engine_macro::gen_storage_for_world;
    use serde::{Deserialize, Serialize};

    #[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Component1(u8);
    #[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Component2(u32);
    #[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Component3(u16);
    #[derive(Default, Clone, Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct Resource1(u32);

    gen_storage_for_world! {
        : components
            Component1 Component2 Component3
        : resources
            Resource1
    }

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

    // #[test]
    // fn query_basic() {
    //     let mut world = World::<ComponentStorage>::new();
    //     world.spawn((Component1(0), Component2(1), Component3(2)));
    //     let ent2 = world.spawn((Component1(3), Component2(4), Component3(5)));
    //     world.spawn((Component1(6), Component2(7)));

    //     let query_world = world.query_world();

    //     let mut query: Query<&Component1> = query_world.parameter();
    //     let res = query.iter().collect::<Vec<_>>();
    //     assert_eq!(res.len(), 3);
    //     assert_eq!(*res[0], Component1(0));
    //     assert_eq!(*res[1], Component1(3));
    //     assert_eq!(*res[2], Component1(6));

    //     let mut query: Query<(&Component1, &Component3, EntityID)> = query_world.parameter();
    //     let res = query.iter().collect::<Vec<_>>();
    //     assert_eq!(res.len(), 2);
    //     assert_eq!(*res[0].0, Component1(0));
    //     assert_eq!(*res[1].0, Component1(3));
    //     assert_eq!(*res[0].1, Component3(2));
    //     assert_eq!(*res[1].1, Component3(5));
    //     assert_eq!(res[1].2, ent2);
    // }

    // #[test]
    // fn query_mut() {
    //     let mut world = World::<ComponentStorage>::new();
    //     let ent1 = world.spawn((Component1(0), Component2(1), Component3(2)));
    //     let ent2 = world.spawn((Component1(3), Component2(4), Component3(5)));
    //     let ent3 = world.spawn((Component1(6), Component2(7)));

    //     let query_world = world.query_world();

    //     let mut query: Query<&mut Component1> = query_world.parameter();
    //     for component in query.iter() {
    //         component.0 += 1;
    //     }

    //     assert_eq!(world.get::<Component1>(ent1).unwrap().0, 1);
    //     assert_eq!(world.get::<Component1>(ent2).unwrap().0, 4);
    //     assert_eq!(world.get::<Component1>(ent3).unwrap().0, 7);
    // }

    // #[test]
    // #[should_panic(expected = "are incompatible")]
    // fn query_incompatible_1() {
    //     let mut world = World::<ComponentStorage>::new();
    //     world.spawn((Component1(0), Component2(1), Component3(2)));
    //     world.spawn((Component1(3), Component2(4), Component3(5)));
    //     world.spawn((Component1(6), Component2(7)));

    //     let query_world = world.query_world();

    //     let _query1: Query<(&mut Component1, &Component2)> = query_world.parameter();
    //     let _query2: Query<(&Component1, &Component2)> = query_world.parameter();
    // }

    // #[test]
    // #[should_panic(expected = "are incompatible")]
    // fn query_incompatible_2() {
    //     let mut world = World::<ComponentStorage>::new();
    //     world.spawn((Component1(0), Component2(1), Component3(2)));
    //     world.spawn((Component1(3), Component2(4), Component3(5)));
    //     world.spawn((Component1(6), Component2(7)));

    //     let query_world = world.query_world();

    //     let _query1: Query<(&mut Component1, &Component2)> = query_world.parameter();
    //     let _query2: Query<(&mut Component1, &Component2)> = query_world.parameter();
    // }

    // #[test]
    // fn query_disjoint() {
    //     let mut world = World::<ComponentStorage>::new();
    //     let ent1 = world.spawn((Component1(0), Component2(1), Component3(2)));
    //     let ent2 = world.spawn((Component1(3), Component2(4), Component3(5)));
    //     let ent3 = world.spawn((Component1(6), Component2(7)));

    //     let query_world = world.query_world();

    //     let mut query1: Query<&mut Component1, With<Component3>> = query_world.parameter();
    //     let mut query2: Query<&mut Component1, Without<Component3>> = query_world.parameter();
    //     for component in query1.iter() {
    //         component.0 += 3;
    //     }
    //     for component in query2.iter() {
    //         component.0 += 6;
    //     }

    //     assert_eq!(world.get::<Component1>(ent1).unwrap().0, 3);
    //     assert_eq!(world.get::<Component1>(ent2).unwrap().0, 6);
    //     assert_eq!(world.get::<Component1>(ent3).unwrap().0, 12);
    // }

    #[test]
    fn resource() {
        let mut world = World::<ComponentStorage>::new();
        assert_eq!(world.resource::<Resource1>().0, 0);
        world.resource_mut::<Resource1>().0 += 10;
        assert_eq!(world.resource::<Resource1>().0, 10);
    }

    #[test]
    fn parameter_resource() {
        let mut world = World::<ComponentStorage>::new();
        let query_world = world.query_world();
        let param: &mut Resource1 = query_world.parameter();
        param.0 += 10;

        let query_world = world.query_world();
        let param: &Resource1 = query_world.parameter();
        assert_eq!(param.0, 10);
    }

    #[test]
    #[should_panic(expected = "are incompatible")]
    fn parameter_resource_conflict() {
        let mut world = World::<ComponentStorage>::new();
        let query_world = world.query_world();
        let param: &mut Resource1 = query_world.parameter();
        param.0 += 10;
        let param: &Resource1 = query_world.parameter();
        assert_eq!(param.0, 10);
    }

    #[test]
    fn system() {
        fn sys(res: &mut Resource1) {
            res.0 += 10;
        }
        let mut world = World::<ComponentStorage>::new();
        let ent1 = world.spawn((Component1(5), Component2(1), Component3(2)));

        let query_world = world.query_world();
        query_world.run(|res: &mut Resource1, mut query: Query<&Component2>| {
            res.0 += query.get(ent1).unwrap().0;
        });

        let query_world = world.query_world();
        query_world.run(sys);

        let param = world.resource();
        assert_eq!(param.0, 15);
    }
}
