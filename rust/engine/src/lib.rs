use crate::util::FromGodot;
use std::sync::{atomic::AtomicBool, OnceLock};

use engine_num::Vec3;
use godot::{
    engine::{CharacterBody3D, Engine, Os, RenderingServer},
    prelude::{GodotClass, Inherits, *},
};
use netman::NetmanVariant;
use tokio::runtime::{EnterGuard, Runtime};
use tracing::{info, warn, Level};
use tracing_subscriber::FmtSubscriber;
use ui::{Ui, UiInCtx};
use universe::{PlayerID, Universe};
use util::{ArrayIter, IntoGodot};

mod netman;
mod ui;
mod universe;
mod util;

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
        FIRST_INIT_COMPLETED.store(true, std::sync::atomic::Ordering::Release);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .unwrap();

        TOKIO_RUNTIME
            .set(runtime)
            .expect("tokio runtime not created yet");

        let mut engine = Engine::singleton();
        engine.set_physics_jitter_fix(0.0);

        info!("First-time init has been performed");
    }
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
    netman: Option<NetmanVariant>,
    ui: Ui,
    #[base]
    base: Base<Node3D>,
}

#[godot_api]
impl Node3DVirtual for GameClass {
    fn init(base: Base<Self::Base>) -> Self {
        maybe_first_init();
        let args = Os::singleton().get_cmdline_user_args();
        info!("Args: {:?}", args);
        let netman = if !Engine::singleton().is_editor_hint() {
            let arg1 = if args.len() > 0 {
                String::from(args.get(0))
            } else {
                "".to_string()
            };
            let netman = match arg1.as_str() {
                "client" => NetmanVariant::connect("127.0.0.1:2300").unwrap(),
                _ => NetmanVariant::start_server().unwrap(),
            };
            Some(netman)
        } else {
            info!("Running in editor: skipping init");
            None
        };
        Self {
            universe: Universe::new(),
            netman,
            ui: Ui::new(),
            base,
        }
    }
    fn ready(&mut self) {
        RenderingServer::singleton().connect(
            "frame_pre_draw".into(),
            Callable::from_object_method(self.base.get_node_as::<Self>("."), "frame_pre_draw"),
        );
        let wall_scene = load::<PackedScene>("vessel/walls/wall1.tscn");
        for i in 0..10 {
            for j in 0..10 {
                let node = wall_scene.instantiate().unwrap();
                node.clone().cast::<Node3D>().set_position(Vector3 {
                    x: (i as f32) * 3.5,
                    y: 0.0,
                    z: (j as f32) * 3.5,
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
    fn process(&mut self, _dt: f64) {
        self.netman
            .as_mut()
            .unwrap()
            .process_events(&mut self.universe)
    }
    fn physics_process(&mut self, _dt: f64) {
        let mut ctx = UiInCtx {
            netman: self.netman.as_mut().unwrap(),
            universe: &self.universe,
            scene: &mut self.base.get_tree().unwrap(),
            base: &mut self.base,
        };
        self.ui.update(&mut ctx);
    }
}

impl GameClass {
    fn netman(&self) -> &NetmanVariant {
        self.netman.as_ref().unwrap()
    }

    fn netman_mut(&mut self) -> &mut NetmanVariant {
        self.netman.as_mut().unwrap()
    }

    fn iter_group<Derived>(
        &mut self,
        group_name: impl Into<StringName>,
    ) -> impl Iterator<Item = Gd<Derived>> + '_
    where
        Derived: GodotClass + Inherits<godot::prelude::Node>,
    {
        let group = self
            .base
            .get_tree()
            .unwrap()
            .get_nodes_in_group(group_name.into());
        ArrayIter::new(group).map(|x| x.cast::<Derived>())
    }
}

#[godot_api]
impl GameClass {
    #[func]
    fn frame_pre_draw(&mut self) {
        let my_id = self.netman.as_ref().unwrap().my_id();
        let players = self
            .base
            .get_tree()
            .unwrap()
            .get_nodes_in_group("players".into());
        for player in players.iter_shared() {
            let mut player = player.cast::<CharacterBody3D>();
            let player_id = PlayerID(player.get("player".into()).to::<u32>());
            if my_id != Some(player_id) {
                if let Some(player_info) = self.universe.players.get(&player_id) {
                    player.set_position(player_info.position.into_godot()); // TODO interpolate
                } else {
                    warn!("Player {:?} not found", player_id)
                }
            }
        }
        //godot_print!("pre_draw");

        // self.base
        //     .get_node_as::<Node3D>("wall")
        //     .set_position(Vector3 {
        //         x: self.started.elapsed().as_secs_f32() % 10.0,
        //         y: 0.0,
        //         z: 0.0,
        //     })
    }

    #[func]
    fn my_id(&self) -> u32 {
        self.netman().my_id().unwrap_or(PlayerID(0)).0
    }

    #[func]
    fn update_player_position(&mut self, pos: Vector3) {
        let pos = Vec3::from_godot(pos);
        self.netman
            .as_mut()
            .unwrap()
            .emit_event(universe::UniverseEvent::PlayerMoved { new_position: pos });
    }
}
