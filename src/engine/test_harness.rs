//! Automated **test worlds** (`--test weather|fire`): drive world-gen flow and seed sim debug state.

use bevy::prelude::*;

use crate::engine::launch_args::{EngineLaunchArgs, TestScene};
use crate::engine::states::{BaseState, WorldGenFlowState};
use crate::render::WeatherFireFieldDebugOverlay;
use crate::systems::fire::ChunkSurfaceFire;
use crate::systems::weather::{ChunkWeather, WeatherVisualSettings};
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, GenerateWorldEvent, WorldGenJobSlot, WorldGenParams,
    WorldGenPhase, WorldGenProgress, WorldMarker,
};

/// Set when a CLI test world has finished generating and the app entered [`BaseState::Simulation`].
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveTestScene(pub TestScene);

#[derive(Resource, Debug)]
pub struct TestWorldHarness {
    pub active: bool,
    pub phase: u8,
    pub finished: bool,
}

impl Default for TestWorldHarness {
    fn default() -> Self {
        Self {
            active: false,
            phase: 0,
            finished: true,
        }
    }
}

pub struct TestHarnessPlugin;

impl Plugin for TestHarnessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TestWorldHarness>()
            .add_systems(
                Update,
                (
                    test_world_bootstrap,
                    apply_test_scene_defaults.after(test_world_bootstrap),
                ),
            );
    }
}

fn test_world_bootstrap(
    launch: Option<Res<EngineLaunchArgs>>,
    mut harness: ResMut<TestWorldHarness>,
    flow: Res<State<WorldGenFlowState>>,
    progress: Res<WorldGenProgress>,
    job: Res<WorldGenJobSlot>,
    mut params: ResMut<WorldGenParams>,
    mut gen_ev: MessageWriter<GenerateWorldEvent>,
    mut next_flow: ResMut<NextState<WorldGenFlowState>>,
    mut next_base: ResMut<NextState<BaseState>>,
    mut commands: Commands,
    world_roots: Query<Entity, With<WorldMarker>>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.test_mode() || harness.finished || !harness.active {
        return;
    }

    let busy = progress.running || job.is_busy();

    match harness.phase {
        0 => {
            despawn_generated_world_entities(&mut commands, &world_roots);
            let mut p = WorldGenParams::default();
            p.width = 256;
            p.height = 256;
            *params = p;
            NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::NewWorldSetup);
            harness.phase = 1;
        }
        1 => {
            if *flow.get() != WorldGenFlowState::NewWorldSetup || busy {
                return;
            }
            gen_ev.write(GenerateWorldEvent {
                params: params.clone(),
                phase: WorldGenPhase::Preview,
            });
            harness.phase = 2;
        }
        2 => {
            if *flow.get() != WorldGenFlowState::PreviewReady || busy {
                return;
            }
            gen_ev.write(GenerateWorldEvent {
                params: params.clone(),
                phase: WorldGenPhase::Full,
            });
            harness.phase = 3;
        }
        3 => {
            if *flow.get() != WorldGenFlowState::FullReady || busy {
                return;
            }
            NextState::set_if_neq(&mut *next_base, BaseState::Simulation);
            NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
            commands.insert_resource(ActiveTestScene(launch.test_scene));
            harness.finished = true;
        }
        _ => {}
    }
}

fn apply_test_scene_defaults(
    scene: Option<Res<ActiveTestScene>>,
    mut done: Local<bool>,
    mut wx: ResMut<WeatherVisualSettings>,
    mut gpu: ResMut<WeatherFireFieldDebugOverlay>,
    mut wx_q: Query<&mut ChunkWeather>,
    mut fire_q: Query<&mut ChunkSurfaceFire>,
) {
    let Some(kind) = scene.as_ref().map(|r| r.0) else {
        return;
    };
    if *done {
        return;
    }
    *done = true;

    wx.enabled = true;
    wx.overlay = true;
    wx.particles = true;
    gpu.show = true;

    match kind {
        TestScene::Weather => {
            let mut n = 0u32;
            for mut w in &mut wx_q {
                if n >= 12 {
                    break;
                }
                w.rain_intensity = w.rain_intensity.max(0.82);
                w.fog_density = w.fog_density.max(0.18);
                w.wind_speed = w.wind_speed.max(0.35);
                w.soil_moisture = w.soil_moisture.max(0.55);
                n += 1;
            }
        }
        TestScene::Fire => {
            let mut n = 0u32;
            for mut f in &mut fire_q {
                if n >= 6 {
                    break;
                }
                f.heat = f.heat.max(0.78);
                f.fuel = f.fuel.max(0.65);
                n += 1;
            }
        }
        TestScene::None => {}
    }
}
