//! Automated **test worlds** (`--test weather|fire|atmosphere|visual`): drive world-gen flow, seed sim
//! chunk slabs + debug defaults, and frame the map camera for fire / atmosphere / precip checks.

use bevy::prelude::*;

use crate::engine::launch_args::{EngineLaunchArgs, TestScene};
use crate::engine::states::{BaseState, WorldGenFlowState};
use crate::render::WeatherFireFieldDebugOverlay;
use crate::systems::atmosphere::GlobalWind;
use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use crate::systems::fire::{ChunkFuelProfile, ChunkSurfaceFire};
use crate::systems::terrain::materialize_chunks;
use crate::systems::weather::{ChunkWeather, WeatherVisualSettings};
use crate::terrain::fire::fuel_depot_profile;
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, GenerateWorldEvent, WorldGenJobSlot, WorldGenParams,
    WorldGenPhase, WorldGenProgress, WorldMarker,
};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::ChunkDependency;

/// Marks chunk entities spawned only for CLI `--test` sim coverage; despawned before regen.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct TestSceneSimChunk;

/// Set when a CLI test world has finished generating and the app entered [`BaseState::Simulation`].
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveTestScene(pub TestScene);

#[derive(Resource, Debug)]
pub struct TestWorldHarness {
    pub active: bool,
    pub phase: u8,
    pub finished: bool,
    /// Cleared when a new CLI world gen starts (`phase == 0`); set after [`apply_test_scene_defaults`] runs.
    pub defaults_applied: bool,
}

impl Default for TestWorldHarness {
    fn default() -> Self {
        Self {
            active: false,
            phase: 0,
            finished: true,
            defaults_applied: false,
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
                    spawn_test_scene_chunk_slabs_once
                        .after(test_world_bootstrap)
                        .before(materialize_chunks)
                        .run_if(in_state(BaseState::Simulation)),
                    apply_test_scene_defaults
                        .after(spawn_test_scene_chunk_slabs_once)
                        .after(ChunkEnvironmentSet::Fire),
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
    test_scene_chunks: Query<Entity, With<TestSceneSimChunk>>,
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
            for e in &test_scene_chunks {
                commands.entity(e).despawn();
            }
            despawn_generated_world_entities(&mut commands, &world_roots);
            let mut p = WorldGenParams::default();
            // Large enough for wind advection + precip overlay; still quick to generate.
            p.width = 320;
            p.height = 320;
            *params = p;
            harness.defaults_applied = false;
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

/// Spawns [`Chunk`] + [`ChunkCellMatrix`] slabs so weather / ecology / fire / atmosphere have ECS targets
/// (tiles alone do not run chunk sim). Skips if the world already has chunk matrices (e.g. editor).
fn spawn_test_scene_chunk_slabs_once(
    scene: Option<Res<ActiveTestScene>>,
    params: Res<WorldGenParams>,
    mut commands: Commands,
    matrix_chunks: Query<(), With<ChunkCellMatrix>>,
    tagged: Query<(), With<TestSceneSimChunk>>,
) {
    if scene.is_none() {
        return;
    }
    if params.width == 0 || params.height == 0 {
        return;
    }
    if !tagged.is_empty() {
        return;
    }
    if !matrix_chunks.is_empty() {
        return;
    }

    const SLAB: u32 = 32;
    let slab_x = SLAB.min(params.width.max(1));
    let slab_y = SLAB.min(params.height.max(1));
    let nx = (params.width + slab_x - 1) / slab_x;
    let ny = (params.height + slab_y - 1) / slab_y;

    for cy in 0i32..ny as i32 {
        for cx in 0i32..nx as i32 {
            let coord = IVec2::new(cx, cy);
            commands.spawn((
                TestSceneSimChunk,
                Chunk { coord },
                ChunkCellMatrix::new(UVec2::new(slab_x, slab_y)),
                ChunkDependency {
                    source_noise_id: 0,
                    registry_hash: 0,
                    families_hash: 0,
                    rules_hash: 0,
                    tags_hash: 0,
                    tuning_hash: 0,
                    preview_hash: 0,
                },
            ));
        }
    }
}

fn apply_test_scene_defaults(
    scene: Option<Res<ActiveTestScene>>,
    mut harness: ResMut<TestWorldHarness>,
    mut wx: ResMut<WeatherVisualSettings>,
    mut gpu: ResMut<WeatherFireFieldDebugOverlay>,
    mut wx_q: Query<&mut ChunkWeather>,
    mut fire_q: Query<&mut ChunkSurfaceFire>,
    mut fuel_q: Query<&mut ChunkFuelProfile>,
    mut global_wind: Option<ResMut<GlobalWind>>,
) {
    let Some(kind) = scene.as_ref().map(|r| r.0) else {
        return;
    };
    if harness.defaults_applied {
        return;
    }
    if fire_q.is_empty() {
        return;
    }
    harness.defaults_applied = true;

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
        TestScene::Atmosphere => {
            if let Some(gw) = global_wind.as_mut() {
                gw.direction = Vec2::new(1.0, 0.2).normalize_or_zero();
                gw.speed = 6.0;
            }
            let mut n = 0u32;
            for mut f in &mut fire_q {
                if n >= 8 {
                    break;
                }
                f.heat = f.heat.max(0.55);
                n += 1;
            }
            let mut wn = 0u32;
            for mut w in &mut wx_q {
                if wn >= 16 {
                    break;
                }
                w.wind_speed = w.wind_speed.max(0.65);
                w.fog_density = w.fog_density.max(0.08);
                wn += 1;
            }
            let mut fi = 0u32;
            for mut prof in &mut fuel_q {
                if fi >= 3 {
                    break;
                }
                prof.structure_overlay = Some(fuel_depot_profile().to_fuel_layer_overlay());
                fi += 1;
            }
        }
        TestScene::Visual => {
            if let Some(gw) = global_wind.as_mut() {
                gw.direction = Vec2::new(1.0, 0.22).normalize_or_zero();
                gw.speed = 5.8;
            }
            let mut n = 0u32;
            for mut f in &mut fire_q {
                if n >= 12 {
                    break;
                }
                f.heat = f.heat.max(0.72);
                f.fuel = f.fuel.max(0.58);
                n += 1;
            }
            let mut wn = 0u32;
            for mut w in &mut wx_q {
                if wn >= 24 {
                    break;
                }
                w.wind_speed = w.wind_speed.max(0.5);
                w.fog_density = w.fog_density.max(0.1);
                w.rain_intensity = w.rain_intensity.max(0.22);
                wn += 1;
            }
            let mut fi = 0u32;
            for mut prof in &mut fuel_q {
                if fi >= 4 {
                    break;
                }
                prof.structure_overlay = Some(fuel_depot_profile().to_fuel_layer_overlay());
                fi += 1;
            }
        }
        TestScene::None => {}
    }
}
