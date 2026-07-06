//! Automated **test worlds** (`--test weather|fire|atmosphere|visual`): drive world-gen flow, seed sim
//! chunk slabs + debug defaults, and frame the map camera for fire / atmosphere / precip checks.

use std::collections::HashMap;

use bevy::diagnostic::FrameCount;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::engine::launch_args::{EngineLaunchArgs, TestScene};
use crate::gui::FireDebugOverride;
use crate::render::TileWorldFallbackRasterDirty;
use crate::engine::debug_maneuver::{
    DebugManeuver, FrameLayoutDebugSession, UnittestWorldFixture,
};
use crate::gui::editor::scenario_script_panel::ScenarioScriptPanelState;
use crate::engine::states::{BaseState, WorldGenFlowState};
use crate::engine::ux_orchestration::{ux_begin_world_gen_from_menu, UxBridgeSet};
use crate::engine::{
    ux_enter_world_from_world_gen, AppState, PauseState, WorldGenChromeLatch, WorldGenState,
};
use crate::render::WeatherFireFieldDebugOverlay;
use crate::systems::atmosphere::GlobalWind;
use crate::systems::chunk_environment_set::ChunkEnvironmentSet;
use crate::systems::fire::{ChunkFuelProfile, ChunkSurfaceFire, FireLightEmission};
use crate::systems::terrain::materialize_chunks;
use crate::systems::weather::{ChunkWeather, WeatherVisualSettings};
use crate::terrain::fire::fuel_depot_profile;
use crate::terrain::world_map_scale::TileExtentPreset;
use crate::terrain::world_map_scale::TerrainFieldStorage;
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, GenerateWorldEvent, WorldGenJobSlot, WorldGenParams,
    WorldGenPhase, WorldGenProgress, WorldMarker,
};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::ChunkDependency;
use crate::economy::logistics::ThroughputSolverState;
use crate::strategic::{
    rebuild_logistics_graph_from_transport, CorridorConstructionBook, LogisticsGraph,
    StrategicRasterConfig,
};
use crate::strategic::StrategicFieldPipeline;
use crate::systems::transport::{
    bake_snapshot_from_ordered_tile_markers, edge_traversal_cost, hydrate_transport_from_snapshot,
    refresh_transport_nav_export, TransportCostCache, TransportCostWeights,
    TransportEdgeDirectory, TransportFieldStore, TransportNavExport, TransportTopology,
};

/// Marks chunk entities spawned only for CLI `--test` sim coverage; despawned before regen.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct TestSceneSimChunk;

/// Set when a CLI test world has finished generating and the app entered [`BaseState::Simulation`].
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ActiveTestScene(pub TestScene);

/// One-shot: seed transport + logistics graph after `--test visual` enter-world (LOG-E01).
#[derive(Resource, Clone, Copy, Debug, Default)]
struct VisualLogisticsProofPending;

#[derive(Resource, Debug)]
pub struct TestWorldHarness {
    pub active: bool,
    pub phase: u8,
    pub finished: bool,
    /// Cleared when a new CLI world gen starts (`phase == 0`); set after [`apply_test_scene_defaults`] runs.
    pub defaults_applied: bool,
    /// LOG-E01: transport + graph proof for `--test visual` (independent of scene defaults latch).
    pub logistics_visual_seeded: bool,
    /// IND-E01/E02: Portland concrete chain proof (commit path + operational).
    pub concrete_chain_e2e_seeded: bool,
    /// 0 = enqueue commits, 1 = wait for commit entities, 2 = complete.
    pub concrete_chain_seed_phase: u8,
    /// **S7P-LOG-001**: aluminum chain + throughput witness (visual / play seed).
    pub s7p_logistics_throughput_seeded: bool,
    pub s7p_logistics_finalize_pending: bool,
    pub s7p_logistics_seed_phase: u8,
    pub s7p_logistics_seed_ticks: u32,
    /// **UI-P3-M2-CODER-A**: construction + ecology minimap overlay witness seed.
    pub minimap_m2_overlay_seeded: bool,
    /// Frames since `BaseState::Simulation` — spreads heavy seeds across ticks (VISUAL-STALL-SURFACE-001).
    pub post_enter_sim_frame: u32,
}

impl Default for TestWorldHarness {
    fn default() -> Self {
        Self {
            active: false,
            phase: 0,
            finished: true,
            defaults_applied: false,
            logistics_visual_seeded: false,
            concrete_chain_e2e_seeded: false,
            concrete_chain_seed_phase: 0,
            s7p_logistics_throughput_seeded: false,
            s7p_logistics_finalize_pending: false,
            s7p_logistics_seed_phase: 0,
            s7p_logistics_seed_ticks: 0,
            minimap_m2_overlay_seeded: false,
            post_enter_sim_frame: 0,
        }
    }
}

fn reset_post_enter_sim_frame_on_enter(mut harness: ResMut<TestWorldHarness>) {
    harness.post_enter_sim_frame = 0;
}

fn advance_post_enter_sim_frame(mut harness: ResMut<TestWorldHarness>) {
    harness.post_enter_sim_frame = harness.post_enter_sim_frame.saturating_add(1);
}

fn post_enter_sim_frame_at_least(n: u32) -> impl Fn(Res<TestWorldHarness>) -> bool + Clone {
    move |harness: Res<TestWorldHarness>| harness.post_enter_sim_frame >= n
}

/// Road chain tiles for LOG-E01 / **S7P-LOG-001** (matches logistics live_proof harness).
pub const S7P_LOGISTICS_CHAIN_TILES: [(u32, u32); 3] = [(0, 0), (1, 0), (2, 0)];

/// Menu debug maneuver bootstrap — auto preview → full → enter world (UI stays open unless mode ② CLI).
#[derive(Resource, Debug, Default)]
pub struct DebugQuickWorldGenPending {
    pub active: bool,
    pub phase: u8,
    pub maneuver: DebugManeuver,
    /// VFX / fire / weather scene seeding after enter (mirrors CLI `--test fire|weather|…`).
    pub test_scene: TestScene,
}

/// Arm menu debug flow: preview → full → sim. Caller should despawn world roots first.
pub fn arm_debug_quick_world_gen(
    pending: &mut DebugQuickWorldGenPending,
    harness: &mut TestWorldHarness,
    maneuver: DebugManeuver,
    test_scene: TestScene,
) {
    pending.active = true;
    pending.phase = 0;
    pending.maneuver = maneuver;
    pending.test_scene = test_scene;
    // CLI `--test` owns `TestWorldHarness.active`; menu path uses `DebugQuickWorldGenPending` only.
    if test_scene.menu_vfx_bootstrap() {
        harness.defaults_applied = false;
    }
}

/// Inactive harness resources for menu debug + optional CLI `--test` (DEHACK-ENG-001).
pub struct TestHarnessStatePlugin;

impl Plugin for TestHarnessStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TestWorldHarness>()
            .init_resource::<DebugQuickWorldGenPending>();
    }
}

/// Main-menu debug maneuver bootstrap only (preview → full → enter world).
pub struct TestHarnessMenuPlugin;

impl Plugin for TestHarnessMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_quick_world_bootstrap.after(UxBridgeSet));
    }
}

/// CLI `--test` seeds, scene slabs, and proof wiring (not registered in default ship launch).
pub struct TestHarnessPlugin;

impl Plugin for TestHarnessPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::dev::tactical_map_debug::TacticalMapDebugPlugin);
        app.add_systems(
                Startup,
                (
                    startup_seed_visual_logistics_when_cli_visual,
                    startup_seed_visual_minimap_m2_overlays_when_cli_visual,
                ),
            )
            .add_systems(
                OnEnter(BaseState::Simulation),
                (
                    reset_post_enter_sim_frame_on_enter,
                    bootstrap_test_scene_tactical_raster,
                    clear_construction_visuals_on_test_sim_enter,
                ),
            )
            .add_systems(
                Update,
                advance_post_enter_sim_frame.run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(
                Update,
                test_world_bootstrap.after(UxBridgeSet),
            )
            .add_systems(
                Update,
                spawn_test_scene_chunk_slabs_once
                    .before(materialize_chunks)
                    .run_if(in_state(BaseState::Simulation))
                    .run_if(post_enter_sim_frame_at_least(1)),
            )
            .add_systems(
                Update,
                (
                    apply_visual_logistics_proof_pending,
                    seed_visual_test_logistics_proof,
                )
                    .chain()
                    .after(StrategicFieldPipeline::GraphSync)
                    .run_if(in_state(BaseState::Simulation))
                    .run_if(post_enter_sim_frame_at_least(3)),
            )
            .add_systems(
                Update,
                (
                    seed_visual_minimap_m2_overlay_proof,
                    refresh_visual_transport_nav_after_seed,
                )
                    .chain()
                    .after(StrategicFieldPipeline::GraphSync)
                    .run_if(in_state(BaseState::Simulation))
                    .run_if(post_enter_sim_frame_at_least(2)),
            )
            .add_systems(
                Update,
                seed_visual_concrete_chain_e2e
                    .after(StrategicFieldPipeline::GraphSync)
                    .run_if(in_state(BaseState::Simulation))
                    .run_if(post_enter_sim_frame_at_least(4)),
            )
            .add_systems(
                Update,
                (
                    seed_s7p_logistics_throughput_proof,
                    finalize_s7p_logistics_throughput_witness,
                )
                    .chain()
                    .after(crate::economy::logistics::LogisticsSimulationSet::Witness),
            )
            .add_systems(
                Update,
                apply_test_scene_defaults
                    .after(spawn_test_scene_chunk_slabs_once)
                    .after(ChunkEnvironmentSet::Fire)
                    .run_if(in_state(BaseState::Simulation))
                    .run_if(post_enter_sim_frame_at_least(2)),
            )
            .add_systems(
                Update,
                (
                    maintain_test_scene_fire_overlay
                        .before(preserve_test_scene_fire_after_sim_tick),
                    preserve_test_scene_fire_after_sim_tick
                        .after(ChunkEnvironmentSet::Fire)
                        .before(crate::render::extraction::FireVisualFrameSet::BuildProfiles),
                )
                    .run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(
                Update,
                (
                    drive_visual_aidv2_proof_harness,
                    drive_visual_aidv2_esc_injection
                        .after(drive_visual_aidv2_proof_harness),
                )
                    .chain()
                    .before(crate::gui::hud::panel_state::hud_panel_escape_collapse_system)
                    .run_if(in_state(BaseState::Simulation))
                    .run_if(post_enter_sim_frame_at_least(1)),
            )
            .add_systems(
                PostUpdate,
                (
                    apply_visual_aidv2_macro_zoom_camera,
                    arm_visual_test_exit_on_va2_live_proof,
                )
                    .run_if(in_state(BaseState::Simulation)),
            );
    }
}

#[derive(SystemParam)]
struct TestHarnessEnterWorld<'w> {
    latch: ResMut<'w, WorldGenChromeLatch>,
    world_gen_ui: ResMut<'w, crate::gui::editor::world_gen_ui::WorldGenUiState>,
    preview_ui: ResMut<'w, crate::gui::editor::world_preview::WorldPreviewUiState>,
    lifecycle: ResMut<'w, crate::gui::editor::world_preview::WorldPreviewLifecycle>,
    next_app: ResMut<'w, NextState<AppState>>,
    next_wg: ResMut<'w, NextState<WorldGenState>>,
    next_pause: ResMut<'w, NextState<PauseState>>,
    next_base: ResMut<'w, NextState<BaseState>>,
}

/// UI menu path: drive world-gen after **Debug: quick world gen** (does not use CLI `--test visual`).
fn debug_quick_world_bootstrap(
    launch: Option<Res<EngineLaunchArgs>>,
    mut pending: ResMut<DebugQuickWorldGenPending>,
    mut harness: ResMut<TestWorldHarness>,
    flow: Res<State<WorldGenFlowState>>,
    progress: Res<WorldGenProgress>,
    job: Res<WorldGenJobSlot>,
    params: Res<WorldGenParams>,
    mut gen_ev: MessageWriter<GenerateWorldEvent>,
    mut next_flow: ResMut<NextState<WorldGenFlowState>>,
    mut enter_world: TestHarnessEnterWorld,
    mut script_panel: ResMut<ScenarioScriptPanelState>,
    test_scene_chunks: Query<Entity, With<TestSceneSimChunk>>,
    mut commands: Commands,
) {
    if launch.is_some_and(|l| l.test_mode()) || !pending.active {
        return;
    }
    let busy = progress.running || job.is_busy();
    match pending.phase {
        0 => {
            if busy {
                return;
            }
            let flow_state = *flow.get();
            if !matches!(flow_state, WorldGenFlowState::NewWorldSetup) {
                if matches!(
                    flow_state,
                    WorldGenFlowState::FullReady
                        | WorldGenFlowState::PreviewReady
                        | WorldGenFlowState::Idle
                ) {
                    next_flow.set(WorldGenFlowState::NewWorldSetup);
                    info!(
                        target: "app_shell",
                        ?flow_state,
                        "Debug bootstrap: re-arming world-gen flow to NewWorldSetup"
                    );
                } else {
                    bevy::log::debug!(
                        target: "app_shell",
                        flow = ?flow_state,
                        phase = pending.phase,
                        "Debug bootstrap waiting for NewWorldSetup before preview gen"
                    );
                }
                return;
            }
            for e in &test_scene_chunks {
                commands.entity(e).despawn();
            }
            script_panel.window_open = false;
            script_panel.tools_entry_visible = false;
            if pending.test_scene != TestScene::None {
                harness.defaults_applied = false;
            }
            gen_ev.write(GenerateWorldEvent {
                params: params.clone(),
                phase: WorldGenPhase::Preview,
            });
            pending.phase = 1;
            info!(
                target: "app_shell",
                test_scene = pending.test_scene.menu_label(),
                "Debug bootstrap: preview generation started"
            );
        }
        1 => {
            if *flow.get() == WorldGenFlowState::NewWorldSetup {
                if !busy {
                    gen_ev.write(GenerateWorldEvent {
                        params: params.clone(),
                        phase: WorldGenPhase::Preview,
                    });
                }
                return;
            }
            if *flow.get() != WorldGenFlowState::PreviewReady || busy {
                return;
            }
            gen_ev.write(GenerateWorldEvent {
                params: params.clone(),
                phase: WorldGenPhase::Full,
            });
            pending.phase = 2;
            info!(target: "app_shell", "Debug bootstrap: full generation started");
        }
        2 => {
            if *flow.get() != WorldGenFlowState::FullReady || busy {
                return;
            }
            let scene = pending.test_scene;
            let maneuver = pending.maneuver;
            ux_enter_world_from_world_gen(
                &mut enter_world.latch,
                &mut enter_world.next_app,
                &mut enter_world.next_wg,
                &mut enter_world.next_pause,
                &mut enter_world.next_base,
                &mut enter_world.world_gen_ui,
                &mut enter_world.preview_ui,
                &mut enter_world.lifecycle,
            );
            if scene != TestScene::None {
                commands.insert_resource(ActiveTestScene(scene));
                harness.defaults_applied = false;
                harness.finished = true;
            } else {
                commands.remove_resource::<ActiveTestScene>();
            }
            pending.active = false;
            pending.test_scene = TestScene::None;
            info!(
                target: "app_shell",
                maneuver = maneuver.menu_label(),
                test_scene = scene.menu_label(),
                fire_seeded = scene.seeds_fire_overlay(),
                "Debug maneuver: entered simulation — World Generator dismissed; \
                 test scene defaults apply next sim tick"
            );
        }
        _ => {
            pending.active = false;
        }
    }
}

fn world_gen_params_for_maneuver(
    launch: &EngineLaunchArgs,
    maneuver: DebugManeuver,
) -> WorldGenParams {
    let mut p = WorldGenParams::default();
    match maneuver {
        DebugManeuver::FrameScreen => {
            p.apply_tile_extent_preset(TileExtentPreset::TacticalSmall);
        }
        DebugManeuver::UnittestWorld => {
            let fixture = UnittestWorldFixture::load_resolved(
                launch.unittest_fixture_path.as_deref(),
            );
            fixture.apply_to_params(&mut p);
            p.recompute_symbolic_land_features();
        }
        DebugManeuver::FullCapture | DebugManeuver::DemoOpen | DebugManeuver::None => {
            p.apply_tile_extent_preset(TileExtentPreset::MediumSmall);
            p.field_storage = TerrainFieldStorage::ChunkCellMatrixAuthoritative;
        }
    }
    p
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
    mut enter_world: TestHarnessEnterWorld,
    mut commands: Commands,
    mut layout_session: ResMut<FrameLayoutDebugSession>,
    world_roots: Query<Entity, With<WorldMarker>>,
    test_scene_chunks: Query<Entity, With<TestSceneSimChunk>>,
    mut stall: Local<(u8, u32)>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.test_mode() || harness.finished || !harness.active {
        return;
    }

    let busy = progress.running || job.is_busy();

    if stall.0 != harness.phase {
        *stall = (harness.phase, 0);
    } else {
        stall.1 = stall.1.saturating_add(1);
        if stall.1 > 0 && stall.1 % 180 == 0 {
            info!(
                target: "test_harness::bootstrap",
                phase = harness.phase,
                flow = ?flow.get(),
                busy,
                finished = harness.finished,
                maneuver = ?launch.maneuver,
                auto_exit = launch.visual_auto_exit,
                "CLI test world-gen bootstrap still in progress (not frozen — world-gen or proof gate)"
            );
        }
    }

    match harness.phase {
        0 => {
            for e in &test_scene_chunks {
                commands.entity(e).despawn();
            }
            despawn_generated_world_entities(&mut commands, &world_roots);
            let maneuver = if launch.maneuver != DebugManeuver::None {
                launch.maneuver
            } else {
                DebugManeuver::FullCapture
            };
            *params = world_gen_params_for_maneuver(launch, maneuver);
            harness.defaults_applied = false;
            harness.logistics_visual_seeded = false;
            harness.s7p_logistics_throughput_seeded = false;
            harness.s7p_logistics_finalize_pending = false;
            harness.s7p_logistics_seed_phase = 0;
            harness.s7p_logistics_seed_ticks = 0;
            harness.minimap_m2_overlay_seeded = false;
            layout_session.active = maneuver == DebugManeuver::FrameScreen;
            match maneuver {
                DebugManeuver::FullCapture | DebugManeuver::UnittestWorld => {
                    info!(
                        target: "stage5_full_app_harness",
                        label = maneuver.menu_label(),
                        min_frames = launch.min_capture_frames,
                        auto_exit = launch.visual_auto_exit,
                        "--test capture: auto world-gen → sim → proof → graceful exit"
                    );
                }
                DebugManeuver::FrameScreen => {
                    info!(
                        target: "app_shell",
                        label = maneuver.menu_label(),
                        "Frame layout test: UI_LAYOUT_DEBUG active, window stays open"
                    );
                }
                DebugManeuver::DemoOpen => {
                    info!(
                        target: "app_shell",
                        label = maneuver.menu_label(),
                        "Demo world: auto world-gen, no proof auto-exit"
                    );
                }
                DebugManeuver::None => {}
            }
            ux_begin_world_gen_from_menu(
                &mut enter_world.next_app,
                &mut enter_world.next_wg,
                &mut enter_world.next_base,
                &mut next_flow,
                &mut enter_world.latch,
                &mut enter_world.world_gen_ui,
                &mut enter_world.preview_ui,
            );
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
            ux_enter_world_from_world_gen(
                &mut enter_world.latch,
                &mut enter_world.next_app,
                &mut enter_world.next_wg,
                &mut enter_world.next_pause,
                &mut enter_world.next_base,
                &mut enter_world.world_gen_ui,
                &mut enter_world.preview_ui,
                &mut enter_world.lifecycle,
            );
            commands.insert_resource(ActiveTestScene(launch.test_scene));
            if launch.full_capture_active() {
                commands.insert_resource(VisualLogisticsProofPending);
            }
            harness.finished = true;
        }
        _ => {}
    }
}

/// Dry/warm slab cells so surface-fire tick does not zero heat on empty moisture/temperature grids.
fn test_scene_chunk_matrix(size: bevy::math::UVec2) -> ChunkCellMatrix {
    let mut matrix = ChunkCellMatrix::new(size);
    for i in 0..matrix.moisture.len() {
        matrix.moisture[i] = 0.06;
        matrix.temperature[i] = 0.24;
    }
    matrix
}

fn test_scene_fire_seed_count(scene: TestScene) -> u32 {
    match scene {
        TestScene::Fire => 6,
        TestScene::Atmosphere => 8,
        TestScene::Visual | TestScene::VfxSandbox => 28,
        TestScene::None | TestScene::Weather => 0,
    }
}

fn apply_test_scene_fire_seeds(
    scene: TestScene,
    params: &WorldGenParams,
    fire_q: &mut Query<(Entity, &Chunk, &ChunkCellMatrix, &mut ChunkSurfaceFire)>,
    commands: &mut Commands,
) {
    let count = test_scene_fire_seed_count(scene);
    if count == 0 {
        return;
    }
    let (heat, fuel) = match scene {
        TestScene::Fire => (0.78, 0.65),
        TestScene::Atmosphere => (0.55, 0.5),
        TestScene::Visual | TestScene::VfxSandbox => (0.92, 0.75),
        TestScene::None | TestScene::Weather => (0.0, 0.0),
    };
    seed_test_fire_near_world_center(params, fire_q, count, heat, fuel);
    attach_fire_light_emission_for_seeded_chunks(fire_q, commands);
}

fn attach_fire_light_emission_for_seeded_chunks(
    fire_q: &Query<(Entity, &Chunk, &ChunkCellMatrix, &mut ChunkSurfaceFire)>,
    commands: &mut Commands,
) {
    for (entity, chunk, _, fire) in fire_q.iter() {
        if fire.heat <= 0.02 {
            continue;
        }
        let flicker_phase =
            (chunk.coord.x as f32 * 0.37 + chunk.coord.y as f32 * 0.91).fract() * std::f32::consts::TAU;
        let base_intensity = (fire.heat * 1.2).clamp(0.0, 2.5);
        commands.entity(entity).insert(FireLightEmission {
            radius: 120.0 + fire.heat * 180.0,
            base_intensity,
            current_intensity: base_intensity,
            flicker_strength: 0.12,
            flicker_phase,
            extract_priority: 0.5 + fire.heat * 2.0,
        });
    }
}

/// Strip settlement/construction debug overlays on CLI `--test` sim enter.
fn clear_construction_visuals_on_test_sim_enter(
    scene: Option<Res<ActiveTestScene>>,
    launch: Option<Res<EngineLaunchArgs>>,
    mut requests: Option<ResMut<crate::construction::ConstructionVisualRequests>>,
) {
    if scene.is_none() && !launch.as_deref().is_some_and(|l| l.test_mode()) {
        return;
    }
    if let Some(requests) = requests.as_mut() {
        requests.clear();
    }
}

/// Force overworld/minimap CPU raster on sim enter for CLI `--test` worlds.
fn bootstrap_test_scene_tactical_raster(
    scene: Option<Res<ActiveTestScene>>,
    launch: Option<Res<EngineLaunchArgs>>,
    mut raster_dirty: ResMut<TileWorldFallbackRasterDirty>,
    mut raster_ctrl: ResMut<crate::render::TileWorldFallbackRasterCtrl>,
    mut raster_policy: ResMut<crate::render::TileFallbackRasterPolicy>,
) {
    if scene.is_none() && !launch.as_deref().is_some_and(|l| l.test_mode()) {
        return;
    }
    raster_policy.test_harness_boost = true;
    raster_ctrl.chunk_grid.mark_all_dirty();
    raster_ctrl.reset_paint_bookkeeping();
    raster_dirty.bump();
    info!(
        target: "test_harness::raster",
        "test scene sim enter — tactical raster marked all dirty"
    );
}

/// Spawns [`Chunk`] + [`ChunkCellMatrix`] slabs so weather / ecology / fire / atmosphere have ECS targets
/// (tiles alone do not run chunk sim). World-gen only spawns tiles — CLI `--test` always needs these slabs.
fn spawn_test_scene_chunk_slabs_once(
    scene: Option<Res<ActiveTestScene>>,
    params: Res<WorldGenParams>,
    mut commands: Commands,
    tagged: Query<(), With<TestSceneSimChunk>>,
    mut raster_dirty: ResMut<TileWorldFallbackRasterDirty>,
    mut raster_ctrl: ResMut<crate::render::TileWorldFallbackRasterCtrl>,
    mut raster_policy: ResMut<crate::render::TileFallbackRasterPolicy>,
    mut hydration_gate: Option<ResMut<crate::terrain::generation::DenseTerrainHydrationGate>>,
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
                test_scene_chunk_matrix(UVec2::new(slab_x, slab_y)),
                ChunkSurfaceFire {
                    heat: 0.0,
                    fuel: 1.0,
                },
                FireLightEmission {
                    radius: 200.0,
                    base_intensity: 0.0,
                    current_intensity: 0.0,
                    flicker_strength: 0.12,
                    flicker_phase: 0.0,
                    extract_priority: 0.5,
                },
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
    info!(
        target: "test_harness::fire",
        "spawned test scene chunk slabs nx={nx} ny={ny} (world {}x{})",
        params.width,
        params.height
    );
    if let Some(gate) = hydration_gate.as_mut() {
        gate.reset();
    }
    raster_policy.test_harness_boost = true;
    raster_ctrl.chunk_grid.mark_all_dirty();
    raster_ctrl.reset_paint_bookkeeping();
    raster_dirty.bump();
}

/// Direct graph seed for unit tests (no transport directory).
#[cfg(test)]
fn seed_test_logistics_graph_fallback(graph: &mut LogisticsGraph, solver: &mut ThroughputSolverState) {
    use crate::strategic::{LogisticsEdge, LogisticsNodeId};
    use crate::systems::transport::TransportEdgeId;

    if !graph.edges.is_empty() {
        return;
    }
    graph.revision = graph.revision.saturating_add(1).max(1);
    graph.nodes = vec![
        crate::strategic::LogisticsNode {
            id: LogisticsNodeId(0),
            throughput: 0.0,
            stockpile: 0.0,
            anchor: None,
        },
        crate::strategic::LogisticsNode {
            id: LogisticsNodeId(1),
            throughput: 0.0,
            stockpile: 0.0,
            anchor: None,
        },
        crate::strategic::LogisticsNode {
            id: LogisticsNodeId(2),
            throughput: 0.0,
            stockpile: 0.0,
            anchor: None,
        },
    ];
    graph.edges = vec![
        LogisticsEdge {
            from: LogisticsNodeId(0),
            to: LogisticsNodeId(1),
            transport_edge: Some(TransportEdgeId(1)),
            capacity: 12.0,
            disruption: 0.0,
            traversal_cost: 1.0,
        },
        LogisticsEdge {
            from: LogisticsNodeId(1),
            to: LogisticsNodeId(2),
            transport_edge: Some(TransportEdgeId(2)),
            capacity: 10.0,
            disruption: 0.05,
            traversal_cost: 1.2,
        },
    ];
    solver.topology_revision = graph.revision as u32;
    solver.ensure_len(4);
    solver.load[1] = 5.0;
    solver.capacity[1] = 12.0;
    solver.load[2] = 3.5;
    solver.capacity[2] = 10.0;
}

fn seed_test_logistics_visual_proof_into(
    graph: &mut LogisticsGraph,
    solver: &mut ThroughputSolverState,
    topology: &mut TransportTopology,
    fields: &mut TransportFieldStore,
    directory: &mut TransportEdgeDirectory,
    weights: &TransportCostWeights,
    cells: &StrategicRasterConfig,
    book: &CorridorConstructionBook,
    nav: Option<&mut TransportNavExport>,
    cache: Option<&mut TransportCostCache>,
) {
    if !directory.by_edge.is_empty() && !graph.edges.is_empty() {
        return;
    }
    let snap = bake_snapshot_from_ordered_tile_markers(
        &S7P_LOGISTICS_CHAIN_TILES,
        |_, _| 0.5_f32,
        20.0,
        0.25,
    );
    hydrate_transport_from_snapshot(topology, fields, directory, &snap)
        .expect("visual proof transport hydrate");
    let mut built_cache = TransportCostCache::default();
    for (id, st) in &fields.by_edge {
        built_cache.by_edge.insert(
            *id,
            edge_traversal_cost(st, weights, st.travel_time_base),
        );
    }
    let mut built_nav = TransportNavExport::default();
    refresh_transport_nav_export(topology, &built_cache, directory, &mut built_nav);
    if let Some(nav_res) = nav {
        *nav_res = built_nav;
    }
    if let Some(cache_res) = cache {
        *cache_res = built_cache;
    }
    *graph = rebuild_logistics_graph_from_transport(directory, fields, weights, cells, book, 1);
    solver.topology_revision = graph.revision as u32;
    let max_idx = directory
        .by_edge
        .keys()
        .map(|id| id.0 as usize)
        .max()
        .unwrap_or(3);
    solver.ensure_len(max_idx + 1);
    for edge in &graph.edges {
        let Some(tid) = edge.transport_edge else {
            continue;
        };
        let idx = tid.0 as usize;
        solver.load[idx] = edge.capacity * 0.45;
        solver.capacity[idx] = edge.capacity;
    }
    for (id, st) in fields.by_edge.iter_mut() {
        let idx = id.0 as usize;
        if idx < solver.capacity.len() && solver.capacity[idx] > 0.01 {
            st.congestion = (solver.load[idx] / solver.capacity[idx]).clamp(0.08, 1.0);
        }
    }
}

/// LOG-A-04: publish `TransportNavExport` after visual transport hydrate (≤16 system params).
fn refresh_visual_transport_nav_after_seed(
    launch: Option<Res<EngineLaunchArgs>>,
    topology: Option<Res<TransportTopology>>,
    fields: Option<Res<TransportFieldStore>>,
    directory: Option<Res<TransportEdgeDirectory>>,
    weights: Option<Res<TransportCostWeights>>,
    cache: Option<ResMut<TransportCostCache>>,
    nav: Option<ResMut<TransportNavExport>>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.full_capture_active() {
        return;
    }
    let (Some(top), Some(fields), Some(dir), Some(weights), Some(mut cache), Some(mut nav)) =
        (topology, fields, directory, weights, cache, nav)
    else {
        return;
    };
    if dir.by_edge.is_empty() {
        return;
    }
    for (id, st) in &fields.by_edge {
        cache.by_edge.insert(
            *id,
            edge_traversal_cost(st, &weights, st.travel_time_base),
        );
    }
    refresh_transport_nav_export(&top, &cache, &dir, &mut nav);
}

fn fill_visual_logistics_snapshot_from_seed(
    fire: &crate::render::FireSimulationSnapshot,
    graph: &LogisticsGraph,
    solver: &ThroughputSolverState,
    logistics_snap: &mut crate::render::LogisticsVisualSnapshot,
) -> u32 {
    crate::render::fill_logistics_snapshot(fire, Some(graph), Some(solver), None, logistics_snap);
    logistics_snap.active_overlay_rows
}

fn seed_visual_minimap_m2_overlay_into(
    fire: &crate::render::FireSimulationSnapshot,
    book: &mut CorridorConstructionBook,
    climate: &mut crate::render::ClimateVisualAggregate,
    ecology: &mut crate::render::EcologyVisualSnapshot,
) {
    crate::render::seed_minimap_m2_overlay_witness(fire, book, climate, ecology);
}

fn startup_seed_visual_minimap_m2_overlays_when_cli_visual(
    launch: Option<Res<EngineLaunchArgs>>,
    fire: Option<Res<crate::render::FireSimulationSnapshot>>,
    book: Option<ResMut<CorridorConstructionBook>>,
    climate: Option<ResMut<crate::render::ClimateVisualAggregate>>,
    ecology: Option<ResMut<crate::render::EcologyVisualSnapshot>>,
    operational: Option<ResMut<crate::render::MinimapOperationalSnapshot>>,
    mut harness: ResMut<TestWorldHarness>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.full_capture_active() {
        return;
    }
    let (Some(fire), Some(mut book), Some(mut climate), Some(mut ecology)) =
        (fire, book, climate, ecology)
    else {
        return;
    };
    if harness.minimap_m2_overlay_seeded {
        return;
    }
    seed_visual_minimap_m2_overlay_into(&fire, &mut book, &mut climate, &mut ecology);
    if let Some(mut operational) = operational {
        crate::render::seed_minimap_m3_fow_ew_witness(&mut operational);
    }
    harness.minimap_m2_overlay_seeded = true;
    info!(
        target: "test_harness::minimap_m2",
        construction_rows = book.rows.len(),
        ecology_rows = ecology.chunk_rows.len(),
        "UI-P3-M2-CODER-A: seeded minimap M2 construction + ecology witness"
    );
}

fn seed_visual_minimap_m2_overlay_proof(
    launch: Option<Res<EngineLaunchArgs>>,
    fire: Option<Res<crate::render::FireSimulationSnapshot>>,
    book: Option<ResMut<CorridorConstructionBook>>,
    climate: Option<ResMut<crate::render::ClimateVisualAggregate>>,
    ecology: Option<ResMut<crate::render::EcologyVisualSnapshot>>,
    mut harness: ResMut<TestWorldHarness>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.full_capture_active() {
        return;
    }
    if harness.minimap_m2_overlay_seeded {
        return;
    }
    let (Some(fire), Some(mut book), Some(mut climate), Some(mut ecology)) =
        (fire, book, climate, ecology)
    else {
        return;
    };
    seed_visual_minimap_m2_overlay_into(&fire, &mut book, &mut climate, &mut ecology);
    harness.minimap_m2_overlay_seeded = true;
}

fn apply_visual_logistics_minimap_defaults(
    graph: &LogisticsGraph,
    policy: Option<&mut crate::gui::RepresentationResult>,
    map_views: Option<&mut crate::gui::MapViewInstances>,
    overlay_tray: Option<&mut crate::gui::hud::HudOverlayTrayState>,
    full_capture: bool,
) {
    if !graph.edges.is_empty() {
        if let Some(policy) = policy {
            policy.overlay_matrix.logistics = true;
        }
    }
    if let Some(map_views) = map_views {
        map_views.minimap.overlays = if full_capture {
            crate::gui::minimap_overlay_witness_harness()
        } else {
            crate::gui::simulation_minimap_overlay_defaults()
        };
        map_views.minimap.bump_revision();
    }
    if let Some(tray) = overlay_tray {
        tray.logistics_stress_visible = true;
    }
}

/// Immediate seed for `--test visual` so projection graph has `log_rows > 0` even before enter-world.
fn startup_seed_visual_logistics_when_cli_visual(
    launch: Option<Res<EngineLaunchArgs>>,
    topology: Option<ResMut<TransportTopology>>,
    fields: Option<ResMut<TransportFieldStore>>,
    directory: Option<ResMut<TransportEdgeDirectory>>,
    weights: Option<Res<TransportCostWeights>>,
    cells: Option<Res<StrategicRasterConfig>>,
    book: Option<ResMut<CorridorConstructionBook>>,
    graph: Option<ResMut<LogisticsGraph>>,
    solver: Option<ResMut<ThroughputSolverState>>,
    fire: Option<Res<crate::render::FireSimulationSnapshot>>,
    mut logistics_snap: Option<ResMut<crate::render::LogisticsVisualSnapshot>>,
    mut policy: Option<ResMut<crate::gui::RepresentationResult>>,
    mut map_views: Option<ResMut<crate::gui::MapViewInstances>>,
    mut overlay_tray: Option<ResMut<crate::gui::hud::HudOverlayTrayState>>,
    mut harness: ResMut<TestWorldHarness>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.full_capture_active() {
        return;
    }
    let (
        Some(mut topology),
        Some(mut fields),
        Some(mut directory),
        Some(weights),
        Some(cells),
        Some(mut book),
        Some(mut graph),
        Some(mut solver),
    ) = (
        topology, fields, directory, weights, cells, book, graph, solver,
    )
    else {
        return;
    };
    seed_test_logistics_visual_proof_into(
        &mut graph,
        &mut solver,
        &mut topology,
        &mut fields,
        &mut directory,
        &weights,
        &cells,
        &book,
        None,
        None,
    );
    if book.rows.is_empty() {
        book.plan_edge(crate::systems::transport::TransportEdgeId(1));
    }
    let mut overlay_rows = 0u32;
    if let (Some(fire), Some(mut logistics_snap)) = (fire, logistics_snap.as_mut()) {
        overlay_rows = fill_visual_logistics_snapshot_from_seed(
            &fire,
            &graph,
            &solver,
            &mut logistics_snap,
        );
    }
    apply_visual_logistics_minimap_defaults(
        &graph,
        policy.as_deref_mut(),
        map_views.as_deref_mut(),
        overlay_tray.as_deref_mut(),
        launch.full_capture_active(),
    );
    harness.logistics_visual_seeded = !graph.edges.is_empty() && overlay_rows > 0;
    info!(
        target: "test_harness::logistics",
        "LOG-E01 startup seed: transport_edges={} logistics_edges={} overlay_rows={overlay_rows}",
        directory.by_edge.len(),
        graph.edges.len()
    );
}

fn apply_visual_logistics_proof_pending(
    pending: Option<Res<VisualLogisticsProofPending>>,
    topology: Option<ResMut<TransportTopology>>,
    fields: Option<ResMut<TransportFieldStore>>,
    directory: Option<ResMut<TransportEdgeDirectory>>,
    weights: Option<Res<TransportCostWeights>>,
    cells: Option<Res<StrategicRasterConfig>>,
    book: Option<Res<CorridorConstructionBook>>,
    graph: Option<ResMut<LogisticsGraph>>,
    solver: Option<ResMut<ThroughputSolverState>>,
    mut harness: ResMut<TestWorldHarness>,
    mut overlay_tray: Option<ResMut<crate::gui::hud::HudOverlayTrayState>>,
    fire: Option<Res<crate::render::FireSimulationSnapshot>>,
    mut logistics_snap: Option<ResMut<crate::render::LogisticsVisualSnapshot>>,
    mut policy: Option<ResMut<crate::gui::RepresentationResult>>,
    mut map_views: Option<ResMut<crate::gui::MapViewInstances>>,
    mut commands: Commands,
) {
    if pending.is_none() {
        return;
    }
    let (
        Some(mut topology),
        Some(mut fields),
        Some(mut directory),
        Some(weights),
        Some(cells),
        Some(book),
        Some(mut graph),
        Some(mut solver),
    ) = (
        topology, fields, directory, weights, cells, book, graph, solver,
    )
    else {
        return;
    };
    seed_test_logistics_visual_proof_into(
        &mut graph,
        &mut solver,
        &mut topology,
        &mut fields,
        &mut directory,
        &weights,
        &cells,
        &book,
        None,
        None,
    );
    harness.logistics_visual_seeded = true;
    let overlay_rows = if let (Some(fire), Some(mut logistics_snap)) =
        (fire.as_deref(), logistics_snap.as_mut())
    {
        fill_visual_logistics_snapshot_from_seed(fire, &graph, &solver, &mut logistics_snap)
    } else {
        0
    };
    apply_visual_logistics_minimap_defaults(
        &graph,
        policy.as_deref_mut(),
        map_views.as_deref_mut(),
        overlay_tray.as_deref_mut(),
        true,
    );
    info!(
        target: "test_harness::logistics",
        "LOG-E01 visual proof: seeded transport_edges={} logistics_edges={} overlay_rows={overlay_rows}",
        directory.by_edge.len(),
        graph.edges.len()
    );
    commands.remove_resource::<VisualLogisticsProofPending>();
}

/// Fallback seed when pending resource missed (LOG-E01 / VIS-08).
fn seed_visual_test_logistics_proof(
    launch: Option<Res<EngineLaunchArgs>>,
    mut harness: ResMut<TestWorldHarness>,
    topology: Option<ResMut<TransportTopology>>,
    fields: Option<ResMut<TransportFieldStore>>,
    directory: Option<ResMut<TransportEdgeDirectory>>,
    weights: Option<Res<TransportCostWeights>>,
    cells: Option<Res<StrategicRasterConfig>>,
    book: Option<Res<CorridorConstructionBook>>,
    graph: Option<ResMut<LogisticsGraph>>,
    solver: Option<ResMut<ThroughputSolverState>>,
    mut overlay_tray: Option<ResMut<crate::gui::hud::HudOverlayTrayState>>,
    fire: Option<Res<crate::render::FireSimulationSnapshot>>,
    mut logistics_snap: Option<ResMut<crate::render::LogisticsVisualSnapshot>>,
    mut policy: Option<ResMut<crate::gui::RepresentationResult>>,
    mut map_views: Option<ResMut<crate::gui::MapViewInstances>>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.full_capture_active() {
        return;
    }
    let graph_empty = graph.as_ref().is_none_or(|g| g.edges.is_empty());
    if harness.logistics_visual_seeded && !graph_empty {
        return;
    }
    let (
        Some(mut topology),
        Some(mut fields),
        Some(mut directory),
        Some(weights),
        Some(cells),
        Some(book),
        Some(mut graph),
        Some(mut solver),
    ) = (
        topology,
        fields,
        directory,
        weights,
        cells,
        book,
        graph,
        solver,
    )
    else {
        return;
    };
    seed_test_logistics_visual_proof_into(
        &mut graph,
        &mut solver,
        &mut topology,
        &mut fields,
        &mut directory,
        &weights,
        &cells,
        &book,
        None,
        None,
    );
    harness.logistics_visual_seeded = !graph.edges.is_empty();
    let overlay_rows = if let (Some(fire), Some(mut logistics_snap)) =
        (fire.as_deref(), logistics_snap.as_mut())
    {
        fill_visual_logistics_snapshot_from_seed(fire, &graph, &solver, &mut logistics_snap)
    } else {
        0
    };
    apply_visual_logistics_minimap_defaults(
        &graph,
        policy.as_deref_mut(),
        map_views.as_deref_mut(),
        overlay_tray.as_deref_mut(),
        launch.full_capture_active(),
    );
    info!(
        target: "test_harness::logistics",
        "LOG-E01 visual proof: seeded transport_edges={} logistics_edges={} overlay_rows={overlay_rows}",
        directory.by_edge.len(),
        graph.edges.len()
    );
}

fn spawn_s7p_aluminum_chain_site(
    commands: &mut Commands,
    catalog_id: &str,
    site_id: u64,
    origin: crate::strategic::BuildSiteTile,
) {
    use crate::economy::activation::{BuildingDefinitionRef, IndustrialFacilityActivated};
    use crate::strategic::{
        ConstructionSite, FootprintTiles, LayerType, PlannedSite, SiteArchetype,
        SiteConstructionPhase, SiteId,
    };

    commands.spawn((
        ConstructionSite {
            site_id,
            owner: Entity::PLACEHOLDER,
            archetype: SiteArchetype::Factory,
            phase: SiteConstructionPhase::Operational,
            operational_readiness: 1.0,
        },
        PlannedSite {
            site_id: SiteId(site_id),
            origin,
            footprint: FootprintTiles {
                width: 3,
                depth: 2,
            },
            archetype: SiteArchetype::Factory,
            layer: LayerType::Surface,
            catalog_id: Some(catalog_id.into()),
            placement: None,
        },
        BuildingDefinitionRef {
            catalog_id: catalog_id.into(),
        },
        IndustrialFacilityActivated,
        Transform::from_translation(crate::economy::site_placement::site_world_position(
            origin,
        )),
        GlobalTransform::default(),
    ));
}

/// **S7P-LOG-001**: aluminum chain on road tiles + throughput witness green in sim / `--test visual`.
fn seed_s7p_logistics_throughput_proof(
    launch: Option<Res<EngineLaunchArgs>>,
    base: Option<Res<State<BaseState>>>,
    mut harness: ResMut<TestWorldHarness>,
    mut commands: Commands,
    directory: Option<Res<TransportEdgeDirectory>>,
    flow: Option<Res<crate::economy::resource_flow::ResourceFlowRegistry>>,
    diagnostics: Option<Res<crate::economy::logistics::LogisticsDiagnostics>>,
    sites: Query<
        &crate::economy::activation::BuildingDefinitionRef,
        With<crate::strategic::ConstructionSite>,
    >,
    mut flow_nodes: Query<
        (
            &crate::economy::activation::BuildingDefinitionRef,
            &mut crate::economy::resource_flow::ResourceFlowNode,
        ),
    >,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.full_capture_active() {
        return;
    }
    if !matches!(base.as_deref().map(|s| s.get()), Some(BaseState::Simulation)) {
        return;
    }
    if harness.s7p_logistics_throughput_seeded {
        return;
    }
    if directory.as_ref().is_none_or(|d| d.by_edge.is_empty()) {
        return;
    }

    harness.s7p_logistics_seed_ticks = harness.s7p_logistics_seed_ticks.saturating_add(1);

    let aluminum_on_chain = |id: &str| -> bool {
        matches!(
            id,
            "aluminum_bauxite_mine" | "aluminum_alumina_refinery" | "aluminum_smelter1"
        )
    };

    match harness.s7p_logistics_seed_phase {
        0 => {
            let chain_count = sites
                .iter()
                .filter(|r| aluminum_on_chain(r.catalog_id.as_str()))
                .count();
            if chain_count < 3 {
                spawn_s7p_aluminum_chain_site(
                    &mut commands,
                    "aluminum_bauxite_mine",
                    901,
                    crate::strategic::BuildSiteTile {
                        x: S7P_LOGISTICS_CHAIN_TILES[0].0,
                        z: S7P_LOGISTICS_CHAIN_TILES[0].1,
                    },
                );
                spawn_s7p_aluminum_chain_site(
                    &mut commands,
                    "aluminum_alumina_refinery",
                    902,
                    crate::strategic::BuildSiteTile {
                        x: S7P_LOGISTICS_CHAIN_TILES[1].0,
                        z: S7P_LOGISTICS_CHAIN_TILES[1].1,
                    },
                );
                spawn_s7p_aluminum_chain_site(
                    &mut commands,
                    "aluminum_smelter1",
                    903,
                    crate::strategic::BuildSiteTile {
                        x: S7P_LOGISTICS_CHAIN_TILES[2].0,
                        z: S7P_LOGISTICS_CHAIN_TILES[2].1,
                    },
                );
                info!(
                    target: "test_harness::logistics",
                    "S7P-LOG-001: spawned aluminum chain on road tiles {:?}",
                    S7P_LOGISTICS_CHAIN_TILES
                );
            }
            harness.s7p_logistics_seed_phase = 1;
        }
        1 => {
            let chain_count = sites
                .iter()
                .filter(|r| aluminum_on_chain(r.catalog_id.as_str()))
                .count();
            if chain_count < 3 {
                return;
            }
            let flow_ready = flow.as_ref().is_some_and(|f| !f.edges.is_empty());
            if !flow_ready && harness.s7p_logistics_seed_ticks < 48 {
                return;
            }
            harness.s7p_logistics_seed_phase = 2;
        }
        2 => {
            for (def, mut node) in flow_nodes.iter_mut() {
                if def.catalog_id == "aluminum_bauxite_mine" {
                    node.buffer_by_tag.insert("Bauxite".into(), 48.0);
                }
            }
            harness.s7p_logistics_seed_phase = 3;
        }
        _ => {
            let routes_open = diagnostics
                .as_ref()
                .map(|d| d.routes_open)
                .unwrap_or(0);
            if routes_open == 0 && harness.s7p_logistics_seed_ticks < 96 {
                return;
            }
            harness.s7p_logistics_finalize_pending = true;
        }
    }
}

/// **S7P-LOG-001** — patch LOG-* witness + refresh `logistics_throughput_live.json` after scenario seed.
fn finalize_s7p_logistics_throughput_witness(
    launch: Option<Res<EngineLaunchArgs>>,
    base: Option<Res<State<BaseState>>>,
    mut harness: ResMut<TestWorldHarness>,
    graph: Option<Res<crate::strategic::LogisticsGraph>>,
    portals: Option<Res<crate::economy::logistics::PortalAttachmentMap>>,
    flow: Option<Res<crate::economy::resource_flow::ResourceFlowRegistry>>,
    diagnostics: Option<Res<crate::economy::logistics::LogisticsDiagnostics>>,
    route_cache: Option<Res<crate::economy::logistics::RouteCache>>,
    solver: Option<Res<crate::economy::logistics::ThroughputSolverState>>,
    mut witness: Option<ResMut<crate::dev::logistics_throughput_todos::LogisticsThroughputWitness>>,
    mut runtime: Option<ResMut<crate::economy::logistics::LogisticsThroughputRuntimeWitness>>,
    board: Option<ResMut<crate::dev::logistics_throughput_todos::LogisticsThroughputTodoBoard>>,
    mut proof_state: Option<ResMut<crate::economy::logistics::LogisticsThroughputLiveProofState>>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.full_capture_active() {
        return;
    }
    if !matches!(base.as_deref().map(|s| s.get()), Some(BaseState::Simulation)) {
        return;
    }
    if !harness.s7p_logistics_finalize_pending || harness.s7p_logistics_throughput_seeded {
        return;
    }

    let (
        Some(graph),
        Some(portals),
        Some(flow),
        Some(diagnostics),
        Some(route_cache),
        Some(solver),
        Some(witness),
        Some(runtime),
        Some(mut board),
    ) = (
        graph.as_deref(),
        portals.as_deref(),
        flow.as_deref(),
        diagnostics.as_deref(),
        route_cache.as_deref(),
        solver.as_deref(),
        witness.as_deref_mut(),
        runtime.as_deref_mut(),
        board,
    )
    else {
        return;
    };

    crate::economy::logistics::align_logistics_throughput_witness_from_live_sim(
        witness,
        runtime,
        graph,
        portals,
        flow,
        diagnostics,
        route_cache,
        solver,
    );
    crate::dev::logistics_throughput_todos::sync_logistics_throughput_board_from_witness(
        witness,
        board.as_mut(),
    );
    let board_open = board.open_count();
    if diagnostics.routes_open == 0 && harness.s7p_logistics_seed_ticks < 160 {
        return;
    }
    if board_open > 0 && harness.s7p_logistics_seed_ticks < 160 {
        return;
    }
    if diagnostics.routes_open == 0 {
        return;
    }
    if let Some(proof_state) = proof_state.as_mut() {
        crate::economy::logistics::witness_collectors::request_logistics_throughput_live_proof_refresh(
            proof_state,
        );
    }
    harness.s7p_logistics_finalize_pending = false;
    harness.s7p_logistics_throughput_seeded = true;
    let routes_open = diagnostics.routes_open;
    info!(
        target: "test_harness::logistics",
        "S7P-LOG-001: logistics throughput witness finalized routes_open={routes_open} board_open={board_open}"
    );
}

/// IND-E01/E02: Portland chain via construction commit → operational (visual / capture proof).
fn seed_visual_concrete_chain_e2e(
    launch: Option<Res<EngineLaunchArgs>>,
    base: Option<Res<State<BaseState>>>,
    mut harness: ResMut<TestWorldHarness>,
    mut commands: Commands,
    mut commit: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    mut chain_witness: ResMut<crate::economy::activation::ConcreteChainE2eWitness>,
    portland_sites: Query<&crate::economy::activation::BuildingDefinitionRef, With<crate::strategic::ConstructionSite>>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !launch.full_capture_active() {
        return;
    }
    if !matches!(base.as_deref().map(|s| s.get()), Some(BaseState::Simulation)) {
        return;
    }
    if harness.concrete_chain_e2e_seeded {
        return;
    }
    if chain_witness.in_play_green() {
        harness.concrete_chain_e2e_seeded = true;
        harness.concrete_chain_seed_phase = 2;
        return;
    }

    const ORIGIN: crate::strategic::BuildSiteTile = crate::strategic::BuildSiteTile { x: 48, z: 48 };

    match harness.concrete_chain_seed_phase {
        0 => {
            let owner = commands.spawn_empty().id();
            crate::economy::activation::commit_concrete_portland_chain_in_play(
                &mut commit,
                chain_witness.as_mut(),
                owner,
                ORIGIN,
            );
            harness.concrete_chain_seed_phase = 1;
            info!(
                target: "test_harness::industrial",
                "IND-E02 visual seed: committed concrete_portland chain (mine → kiln → mixer)"
            );
        }
        1 => {
            let committed = portland_sites
                .iter()
                .filter(|r| {
                    crate::economy::activation::CONCRETE_PORTLAND_STEPS
                        .iter()
                        .any(|id| *id == r.catalog_id.as_str())
                })
                .count();
            if committed < 3 {
                return;
            }
            if !chain_witness.production_green() {
                return;
            }
            harness.concrete_chain_seed_phase = 2;
            harness.concrete_chain_e2e_seeded = true;
            info!(
                target: "test_harness::industrial",
                "IND-E01/E02 visual seed: portland chain production_green (construction commit path)"
            );
        }
        _ => {}
    }
}

fn apply_test_scene_defaults(
    scene: Option<Res<ActiveTestScene>>,
    mut harness: ResMut<TestWorldHarness>,
    params: Res<WorldGenParams>,
    mut wx: ResMut<WeatherVisualSettings>,
    mut gpu: ResMut<WeatherFireFieldDebugOverlay>,
    mut focus_debug: ResMut<crate::gui::CameraFocusDebug>,
    mut tile_debug: ResMut<crate::gui::TileGpuDebugSettings>,
    mut presentation: ResMut<crate::gui::MapViewPresentationStates>,
    mut shared_overlay: ResMut<crate::render::SharedOverlayFieldBuffers>,
    mut raster_dirty: ResMut<crate::render::TileWorldFallbackRasterDirty>,
    mut wx_q: Query<&mut ChunkWeather>,
    mut commands: Commands,
    mut fire_q: Query<(Entity, &Chunk, &ChunkCellMatrix, &mut ChunkSurfaceFire)>,
    mut fuel_q: Query<&mut ChunkFuelProfile>,
    mut fire_override: Option<ResMut<FireDebugOverride>>,
    mut global_wind: Option<ResMut<GlobalWind>>,
) {
    let Some(kind) = scene.as_ref().map(|r| r.0) else {
        return;
    };
    if harness.defaults_applied {
        return;
    }
    let sim = presentation.get_mut(crate::gui::MapViewInstanceId::SimulationMap);
    sim.overlays.fire_heat = true;
    sim.bump_revision();

    wx.enabled = true;
    wx.overlay = true;
    wx.particles = true;
    gpu.show = false;

    match kind {
        TestScene::Weather => {
            gpu.show = true;
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
            tile_debug.use_batched_mesh_overlay = true;
        }
        TestScene::Atmosphere => {
            if let Some(gw) = global_wind.as_mut() {
                gw.direction = Vec2::new(1.0, 0.2).normalize_or_zero();
                gw.speed = 6.0;
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
            wx.particles = true;
            wx.background_aesthetic = true;
            // LOD tile debug (green squares) off in visual proof — enable via dev tooling if needed.
            focus_debug.enabled = false;
            tile_debug.use_batched_mesh_overlay = true;
            if let Some(gw) = global_wind.as_mut() {
                gw.direction = Vec2::new(1.0, 0.22).normalize_or_zero();
                gw.speed = 5.8;
            }
            gpu.show = false;
            let mut wn = 0u32;
            for mut w in &mut wx_q {
                if wn >= 24 {
                    break;
                }
                w.wind_speed = w.wind_speed.max(0.5);
                w.fog_density = w.fog_density.max(0.12);
                // Keep rain light so harness fire is not extinguished during visual proof.
                w.rain_intensity = w.rain_intensity.min(0.12);
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
        TestScene::VfxSandbox => {
            wx.particles = true;
            wx.background_aesthetic = true;
            focus_debug.enabled = false;
            tile_debug.use_batched_mesh_overlay = false;
            if let Some(gw) = global_wind.as_mut() {
                gw.direction = Vec2::new(1.0, 0.2).normalize_or_zero();
                gw.speed = 6.0;
            }
            gpu.show = true;
            let mut wn = 0u32;
            for mut w in &mut wx_q {
                if wn >= 24 {
                    break;
                }
                w.wind_speed = w.wind_speed.max(0.65);
                w.fog_density = w.fog_density.max(0.18);
                // Heavy rain was zeroing surface fire before extract could publish instances.
                w.rain_intensity = w.rain_intensity.min(0.12);
                w.soil_moisture = w.soil_moisture.max(0.35);
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

    let needs_fire = kind.seeds_fire_overlay();
    if needs_fire && fire_q.is_empty() {
        raster_dirty.bump();
        return;
    }
    if needs_fire {
        if let Some(override_res) = fire_override.as_mut() {
            override_res.force_visible = true;
        }
        apply_test_scene_fire_seeds(kind, &params, &mut fire_q, &mut commands);
        sync_test_fire_overlay_from_ecs(&fire_q, &mut shared_overlay);
    }
    harness.defaults_applied = true;
    raster_dirty.bump();
}

/// Normalized zoom for VA2 macro icon probe (`--test visual` only).
const VA2_PROOF_MACRO_ZOOM_ALPHA: f32 = 0.0;

/// VA2-HARNESS-01/02/03 — seed build ghost + arm ESC/macro probes after sim entry.
fn drive_visual_aidv2_proof_harness(
    launch: Option<Res<EngineLaunchArgs>>,
    harness: Res<TestWorldHarness>,
    mut va2: ResMut<crate::dev::VisualAidV2HarnessState>,
    mut picker: ResMut<crate::gui::hud::SimBuildPickerState>,
    mut strip: ResMut<crate::construction::BuildStripState>,
    mut tool: ResMut<crate::construction::ActiveBuildTool>,
    mut ghost: ResMut<crate::construction::BuildGhostState>,
    mut preview: ResMut<crate::construction::BuildPlacementPreview>,
) {
    if !launch.is_some_and(|l| l.full_capture_active()) {
        return;
    }
    const ESC_ARM_FRAME: u32 = 8;
    const BUILD_FRAME: u32 = 10;
    const MACRO_PROBE_FRAME: u32 = 12;
    let frame = harness.post_enter_sim_frame;

    if frame == ESC_ARM_FRAME && !va2.esc_injected {
        picker.open_for_slot(crate::construction::ToolContext::Civil);
        va2.esc_armed = true;
    }

    if frame >= BUILD_FRAME {
        strip.active = crate::construction::ToolContext::Industry;
        tool.tool = crate::construction::BuildTool::Building(
            crate::construction::BuildingArchetypeId::Factory,
        );
        ghost.origin = Some(crate::strategic::BuildSiteTile { x: 8, z: 8 });
        preview.report.valid = true;
        preview.report.allows_commit = true;
        va2.build_seeded = true;
    }

    if frame >= MACRO_PROBE_FRAME {
        va2.macro_icon_probe = true;
    }
}

/// VA2-HARNESS-01 — inject Escape after build picker opens (same frame as arm+1).
fn drive_visual_aidv2_esc_injection(
    launch: Option<Res<EngineLaunchArgs>>,
    harness: Res<TestWorldHarness>,
    mut va2: ResMut<crate::dev::VisualAidV2HarnessState>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
) {
    if !launch.is_some_and(|l| l.full_capture_active()) {
        return;
    }
    const ESC_INJECT_FRAME: u32 = 9;
    if harness.post_enter_sim_frame != ESC_INJECT_FRAME || va2.esc_injected || !va2.esc_armed {
        return;
    }
    keys.press(KeyCode::Escape);
    va2.esc_injected = true;
}

/// After VA2 live proof commits, request graceful `--test visual` exit (not gated on FINISH-UX-06).
fn arm_visual_test_exit_on_va2_live_proof(
    va2: Res<crate::dev::VisualAidV2HarnessState>,
    mut visual_exit: ResMut<crate::render::VisualTestGracefulExit>,
) {
    if !va2.request_visual_exit || visual_exit.armed {
        return;
    }
    visual_exit.armed = true;
    visual_exit.frames_remaining = crate::render::VisualTestGracefulExit::FRAMES_AFTER_PROOF;
}

/// VA2-HARNESS-03 — one-shot macro zoom for icon scaffold witness (not a per-frame lock).
fn apply_visual_aidv2_macro_zoom_camera(
    launch: Option<Res<EngineLaunchArgs>>,
    va2: Res<crate::dev::VisualAidV2HarnessState>,
    params: Res<WorldGenParams>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    sim_viewport: Res<crate::gui::SimulationMapViewport>,
    mut authority: ResMut<crate::render::view_runtime::ViewProjectionAuthority>,
    mut trace: ResMut<crate::render::view_runtime::ViewRuntimeTrace>,
    mut cam: Query<&mut Transform, With<crate::gui::MainWorldCamera>>,
    mut applied: Local<bool>,
) {
    if *applied {
        return;
    }
    if !launch.is_some_and(|l| l.full_capture_active()) || !va2.macro_icon_probe {
        return;
    }
    if params.width == 0 || params.height == 0 {
        return;
    }
    let world_w = params.width as f32;
    let world_h = params.height as f32;
    let window_px = windows
        .single()
        .ok()
        .map(|w| Vec2::new(w.width().max(1.0), w.height().max(1.0)))
        .unwrap_or(Vec2::new(1280.0, 720.0));
    let viewport = crate::gui::map_camera_viewport_pixels(window_px, Some(sim_viewport.as_ref()));
    let (zoom_lo, zoom_hi) = crate::gui::map_zoom_limits_for_world(world_w, world_h, viewport);
    let zoom = crate::gui::map_scale_for_zoom_alpha(VA2_PROOF_MACRO_ZOOM_ALPHA, zoom_lo, zoom_hi);
    let cx = world_w * 0.5;
    let cy = world_h * 0.5;
    let mut pose = crate::gui::map_camera_desired_from_view_authority(authority.as_ref());
    pose.translation = Vec3::new(cx, cy, 0.0);
    pose.scale = Vec3::splat(zoom);
    crate::gui::commit_map_camera_pose_to_view_authority(
        authority.as_mut(),
        trace.as_mut(),
        &pose,
    );
    for mut t in cam.iter_mut() {
        t.translation.x = cx;
        t.translation.y = cy;
        t.scale = Vec3::splat(zoom);
    }
    *applied = true;
}

/// Pre-extract overlay refresh (cheap); heavy re-seed runs after the fire sim tick.
fn maintain_test_scene_fire_overlay(
    scene: Option<Res<ActiveTestScene>>,
    harness: Res<TestWorldHarness>,
    frame: Res<FrameCount>,
    fire_q: Query<(Entity, &Chunk, &ChunkCellMatrix, &ChunkSurfaceFire)>,
    mut shared_overlay: ResMut<crate::render::SharedOverlayFieldBuffers>,
) {
    let Some(active) = scene else {
        return;
    };
    if !harness.defaults_applied || !active.0.seeds_fire_overlay() {
        return;
    }
    if frame.0 % 15 != 0 {
        return;
    }
    sync_test_fire_overlay_from_heat(
        fire_q.iter().map(|(_, chunk, _, fire)| (chunk.coord, fire.heat)),
        &mut shared_overlay,
    );
}

/// Re-arm harness fire after rain/fuel sim so extract always sees [`FireLightEmission`].
fn preserve_test_scene_fire_after_sim_tick(
    scene: Option<Res<ActiveTestScene>>,
    harness: Res<TestWorldHarness>,
    params: Res<WorldGenParams>,
    mut commands: Commands,
    mut fire_q: Query<(Entity, &Chunk, &ChunkCellMatrix, &mut ChunkSurfaceFire)>,
    mut shared_overlay: ResMut<crate::render::SharedOverlayFieldBuffers>,
) {
    let Some(active) = scene else {
        return;
    };
    if !harness.defaults_applied || !active.0.seeds_fire_overlay() {
        return;
    }
    apply_test_scene_fire_seeds(active.0, &params, &mut fire_q, &mut commands);
    sync_test_fire_overlay_from_ecs(&fire_q, &mut shared_overlay);
}

fn sync_test_fire_overlay_from_ecs(
    fire_q: &Query<(Entity, &Chunk, &ChunkCellMatrix, &mut ChunkSurfaceFire)>,
    shared: &mut crate::render::SharedOverlayFieldBuffers,
) {
    sync_test_fire_overlay_from_heat(
        fire_q.iter().map(|(_, chunk, _, fire)| (chunk.coord, fire.heat)),
        shared,
    );
}

fn sync_test_fire_overlay_from_heat(
    samples: impl Iterator<Item = (bevy::math::IVec2, f32)>,
    shared: &mut crate::render::SharedOverlayFieldBuffers,
) {
    let mut next = HashMap::new();
    for (coord, heat) in samples {
        if heat >= crate::render::CHUNK_FIRE_OVERLAY_DISPLAY_MIN {
            let e = next.entry(coord).or_insert(0.0_f32);
            *e = f32::max(*e, heat);
        }
    }
    if !next.is_empty() {
        let changed = shared.chunk_fire_heat.len() != next.len()
            || shared
                .chunk_fire_heat
                .iter()
                .any(|(k, v)| next.get(k) != Some(v));
        shared.chunk_fire_heat = next;
        if changed {
            shared.bump();
            info!(
                target: "test_harness::fire",
                "test scene seeded shared overlay fire cells={}",
                shared.chunk_fire_heat.len()
            );
        }
    }
}

fn chunk_center_world_tiles(chunk: &Chunk, matrix: &ChunkCellMatrix) -> Vec2 {
    let sx = matrix.size.x.max(1) as f32;
    let sy = matrix.size.y.max(1) as f32;
    Vec2::new(
        chunk.coord.x as f32 * sx + sx * 0.5,
        chunk.coord.y as f32 * sy + sy * 0.5,
    )
}

fn seed_test_fire_near_world_center(
    params: &WorldGenParams,
    fire_q: &mut Query<(Entity, &Chunk, &ChunkCellMatrix, &mut ChunkSurfaceFire)>,
    count: u32,
    heat: f32,
    fuel: f32,
) {
    if params.width == 0 || params.height == 0 {
        return;
    }
    let center = Vec2::new(params.width as f32 * 0.5, params.height as f32 * 0.5);
    let mut ranked: Vec<(f32, Entity)> = fire_q
        .iter()
        .map(|(e, chunk, matrix, _)| {
            let c = chunk_center_world_tiles(chunk, matrix);
            (center.distance_squared(c), e)
        })
        .collect();
    ranked.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (_, entity) in ranked.into_iter().take(count as usize) {
        if let Ok((_, _, _, mut f)) = fire_q.get_mut(entity) {
            f.heat = f.heat.max(heat);
            f.fuel = f.fuel.max(fuel);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::{fill_logistics_snapshot, FireSimulationSnapshot, LogisticsVisualSnapshot};
    use crate::systems::sim_control::SimStepStamp;

    #[test]
    fn arm_debug_quick_sets_test_scene_and_resets_defaults_latch() {
        let mut pending = DebugQuickWorldGenPending::default();
        let mut harness = TestWorldHarness {
            defaults_applied: true,
            ..Default::default()
        };
        arm_debug_quick_world_gen(
            &mut pending,
            &mut harness,
            DebugManeuver::DemoOpen,
            TestScene::VfxSandbox,
        );
        assert!(pending.active);
        assert_eq!(pending.test_scene, TestScene::VfxSandbox);
        assert!(!harness.active);
        assert!(!harness.defaults_applied);
    }

    #[test]
    fn seed_test_logistics_visual_proof_populates_graph_and_solver() {
        let mut graph = LogisticsGraph::default();
        let mut solver = ThroughputSolverState::default();
        seed_test_logistics_graph_fallback(&mut graph, &mut solver);
        assert_eq!(graph.edges.len(), 2);
        assert!(solver.load[1] > 0.0);

        let fire = FireSimulationSnapshot {
            stamp: SimStepStamp::new(1, 0),
            ..Default::default()
        };
        let mut snapshot = LogisticsVisualSnapshot::default();
        fill_logistics_snapshot(&fire, Some(&graph), Some(&solver), None, &mut snapshot);
        assert!(snapshot.active_overlay_rows >= 2);
    }
}
