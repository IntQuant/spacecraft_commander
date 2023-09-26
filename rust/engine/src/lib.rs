use std::time::{Duration, Instant};

use godot::{engine::RenderingServer, prelude::*};

mod universe;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(base=Node3D)]
struct GameClass {
    started: Instant,

    #[base]
    base: Base<Node3D>,
}

#[godot_api]
impl Node3DVirtual for GameClass {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            base,
            started: Instant::now(),
        }
    }
    fn ready(&mut self) {
        RenderingServer::singleton().connect(
            "frame_pre_draw".into(),
            Callable::from_object_method(self.base.get_node_as::<Self>("."), "frame_pre_draw"),
        );
        let wall_scene = load::<PackedScene>("vessel/walls/wall1.tscn");
        for i in 0..10 {
            let node = wall_scene.instantiate().unwrap();
            node.clone().cast::<Node3D>().set_position(Vector3 {
                x: (i as f32) * 3.5,
                y: 0.0,
                z: 0.0,
            });
            node.clone().cast::<Node3D>().set_rotation_degrees(Vector3 {
                x: -90.0,
                y: 0.0,
                z: 0.0,
            });
            //node.set_name("wall".into());
            self.base.add_child(node);
        }
    }
}

#[godot_api]
impl GameClass {
    #[func]
    fn frame_pre_draw(&mut self) {
        //godot_print!("pre_draw");

        // self.base
        //     .get_node_as::<Node3D>("wall")
        //     .set_position(Vector3 {
        //         x: self.started.elapsed().as_secs_f32() % 10.0,
        //         y: 0.0,
        //         z: 0.0,
        //     })
    }
}
