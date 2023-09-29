use std::sync::{atomic::AtomicBool, OnceLock};

use godot::{engine::RenderingServer, prelude::*};
use netman::NetmanVariant;
use tokio::runtime::{EnterGuard, Runtime};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use universe::Universe;

mod netman;
mod universe;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

static FIRST_INIT_COMPLETED: AtomicBool = AtomicBool::new(false);
static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

fn maybe_first_init() {
    if !FIRST_INIT_COMPLETED.load(std::sync::atomic::Ordering::Acquire) {
        let subscriber = FmtSubscriber::builder()
            .with_max_level(Level::INFO)
            .finish();

        tracing::subscriber::set_global_default(subscriber)
            .expect("setting default subscriber failed");
        info!("First-time init has been performed");
        FIRST_INIT_COMPLETED.store(true, std::sync::atomic::Ordering::Release);
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()
        .unwrap();

    TOKIO_RUNTIME
        .set(runtime)
        .expect("tokio runtime not created yet");
}

fn enter_runtime() -> EnterGuard<'static> {
    let runtime = get_runtime();
    runtime.enter()
}

fn get_runtime() -> &'static Runtime {
    let runtime = TOKIO_RUNTIME
        .get()
        .expect("tokio runtime to be initialized");
    runtime
}

#[derive(GodotClass)]
#[class(base=Node3D)]
struct GameClass {
    universe: Universe,
    netman: NetmanVariant,
    #[base]
    base: Base<Node3D>,
}

#[godot_api]
impl Node3DVirtual for GameClass {
    fn init(base: Base<Self::Base>) -> Self {
        maybe_first_init();
        Self {
            base,
            universe: Universe::new(),
            netman: NetmanVariant::start_server().unwrap(),
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
