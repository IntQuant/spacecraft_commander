use crate::universe::{
    rotations,
    tilemap::{Tile, TilePos},
};
use std::{
    fs::{self, File},
    ops::DerefMut,
    sync::{atomic::AtomicBool, Arc, OnceLock},
};

use engine_ecs::EntityID;
use engine_registry::TileKind;
use godot::{
    engine::{
        Engine, InputEvent, InputEventMouseMotion, Os, RenderingServer, StaticBody3D,
        StaticBody3DVirtual,
    },
    prelude::*,
};
use netman::NetmanVariant;
use ron::ser::PrettyConfig;
use tokio::runtime::{EnterGuard, Runtime};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use ui::{resources::InputStateRes, Ui};
use universe::{
    mcs::{DefaultVesselRes, VesselID, VesselTiles},
    rotations::BuildingOrientation,
    tilemap::TileIndex,
    ui_events::UiEventCtx,
    Universe,
};
use util::OptionNetmanExt;

mod netman;
mod ui;
pub use engine_universe as universe;
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
    universe: Arc<Universe>,
    netman: Option<NetmanVariant>,
    ui: Ui,
    input: InputStateRes,
    #[base]
    base: Base<Node3D>,
}

const TEMP_SAVE: &'static str = "tmp.universe";
const TEMP_SAVE_2: &'static str = "tmp2.universe";

#[godot_api]
impl Node3DVirtual for GameClass {
    fn init(base: Base<Self::Base>) -> Self {
        maybe_first_init();
        let args = Os::singleton().get_cmdline_user_args();
        info!("Args: {:?}", args);
        let netman = if !Engine::singleton().is_editor_hint() {
            let arg1 = if !args.is_empty() {
                String::from(args.get(0))
            } else {
                "client".to_string()
            };
            let netman = match arg1.as_str() {
                "client" => NetmanVariant::connect("10.8.0.2:2300").unwrap(),
                "server" => NetmanVariant::start_server().unwrap(),
                _ => panic!("Unknown `mode` argument"),
            };
            Some(netman)
        } else {
            info!("Running in editor: skipping init");
            None
        };
        // let maybe_universe = File::open(TEMP_SAVE).ok().and_then(|file| {
        //     info!("Trying to deserialize universe");
        //     Some(bincode::deserialize_from(file).expect("can deserialize"))
        // });
        let maybe_universe = fs::read_to_string(TEMP_SAVE_2)
            .map(|x| ron::from_str(&x).expect("can deserialize"))
            .ok();

        let universe = maybe_universe.unwrap_or_else(|| {
            let mut universe = Universe::new();
            let mut evctx = UiEventCtx::default();
            let mut tile_map = universe::tilemap::TileMap::new();
            tile_map.add_at(
                &mut evctx,
                TilePos { x: 0, y: 0, z: 0 },
                Tile {
                    orientation: BuildingOrientation::new(
                        rotations::BuildingFacing::Ny,
                        rotations::BuildingRotation::N,
                    ),
                    kind: TileKind::default(),
                },
            );

            let vessel = universe.world.spawn(VesselTiles(tile_map));
            universe.world.resource_mut::<DefaultVesselRes>().0 = VesselID(vessel);
            universe
        });

        // {
        //     let world = universe.world.query_world();
        //     world.run(
        //         |mut query: mcs::Query<EntityID, With<VesselTiles>>,
        //          commands: mcs::Commands,
        //          default: &DefaultVesselRes| {
        //             for ent in query.iter() {
        //                 if ent != default.0 .0 {
        //                     commands.submit(move |world| {
        //                         world.despawn(ent);
        //                     });
        //                 }
        //             }
        //         },
        //     );
        // }

        let universe = universe.into();
        Self {
            universe,
            netman,
            ui: Ui::new(),
            base,
            input: Default::default(),
        }
    }
    fn ready(&mut self) {
        RenderingServer::singleton().connect(
            "frame_pre_draw".into(),
            Callable::from_object_method(self.base.get_node_as::<Self>("."), "frame_pre_draw"),
        );
    }
    fn process(&mut self, _dt: f64) {}

    fn physics_process(&mut self, _dt: f64) {
        let evctx = self.netman.get_mut().process_events(
            Arc::get_mut(&mut self.universe).expect("expected to be a single owner of a world"),
        );
        if self.netman.get().my_id().is_some() {
            self.with_ui_ctx(|ctx| ctx.on_update(evctx));
        }
        self.input = Default::default();
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if let Some(mouse_event) = event.try_cast::<InputEventMouseMotion>() {
            self.input.mouse_rel += mouse_event.get_relative()
        }
    }
}

impl GameClass {
    fn with_ui_ctx<T>(&mut self, f: impl FnOnce(&mut Ui) -> T) -> Option<T> {
        if let Some(my_id) = self.netman.get().my_id() {
            self.ui.add_temporal_resources(
                self.universe.clone(),
                self.input.clone(),
                self.base.deref_mut().to_owned(),
                my_id,
            );
            let ret = f(&mut self.ui);
            let events = self.ui.remove_temporal_resources();
            for event in events {
                self.netman.get_mut().emit_event(event);
            }
            Some(ret)
        } else {
            None
        }
    }
}

#[godot_api]
impl GameClass {
    #[func]
    fn frame_pre_draw(&mut self) {
        self.with_ui_ctx(|ctx| ctx.on_render());
    }
}

#[derive(Debug, Clone, Copy)]
enum BodyKind {
    Tile { index: TileIndex, position: TilePos },
    Building { entity: EntityID },
}

#[derive(GodotClass, Debug)]
#[class(base=StaticBody3D)]
struct BaseStaticBody {
    kind: Option<BodyKind>,
    #[base]
    _base: Base<StaticBody3D>,
}

#[godot_api]
impl StaticBody3DVirtual for BaseStaticBody {
    fn init(base: Base<Self::Base>) -> Self {
        Self {
            kind: None,
            _base: base,
        }
    }
}

fn save_tmp_universe(universe: &Universe) -> Option<()> {
    let file = File::create(TEMP_SAVE).ok()?;
    bincode::serialize_into(file, universe).ok()?;
    info!("Bincode save ok");
    let universe_str = ron::ser::to_string_pretty(
        universe,
        PrettyConfig::default()
            .indentor(" ".to_string())
            .depth_limit(3),
    )
    .ok()?;
    fs::write(TEMP_SAVE_2, &universe_str).ok()?;
    info!("Ron save ok");
    Some(())
}

impl Drop for GameClass {
    fn drop(&mut self) {
        if !Engine::singleton().is_editor_hint() {
            save_tmp_universe(&self.universe);
        }
    }
}
