//! Automated **test worlds** (`--test weather|fire|atmosphere|visual`): drive world-gen flow, seed sim
//! chunk slabs + debug defaults, and frame the map camera for fire / atmosphere / precip checks.

use std::collections::HashMap;

use bevy::diagnostic::FrameCount;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::engine::launch_args::{EngineLaunchArgs, TestScene};
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
    bake_snapshot_from_ordered_tile_markers, hydrate_transport_from_snapshot,
    TransportCostWeights, TransportEdgeDirectory, TransportFieldStore,
    TransportTopology,
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
        }
    }
}

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

pub struct TestHarnessPlugin;

impl Plugin for TestHarnessPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TestWorldHarness>()
            .init_resource::<DebugQuickWorldGenPending>()
            .add_systems(Startup, startup_seed_visual_logistics_when_cli_visual)
            .add_systems(
                Update,
                test_world_bootstrap.after(UxBridgeSet),
            )
            .add_systems(
                Update,
                spawn_test_scene_chunk_slabs_once
                    .before(materialize_chunks)
                    .run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(
                Update,
                (
                    apply_visual_logistics_proof_pending,
                    seed_visual_test_logistics_proof,
                    seed_visual_concrete_chain_e2e,
                    crate::economy::activation::fast_forward_portland_chain_sites_to_operational,
                )
                    .chain()
                    .after(StrategicFieldPipeline::GraphSync),
            )
            .add_systems(
                Update,
                debug_quick_world_bootstrap.after(UxBridgeSet),
            )
            .add_systems(
                Update,
                apply_test_scene_defaults
                    .after(spawn_test_scene_chunk_slabs_once)
                    .after(ChunkEnvironmentSet::Fire)
                    .run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(
                Update,
                maintain_test_scene_fire_overlay
                    .before(crate::render::extraction::extract_fire_simulation_snapshot)
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
            p.width = 192;
            p.height = 192;
        }
        DebugManeuver::UnittestWorld => {
            let fixture = UnittestWorldFixture::load_resolved(
                launch.unittest_fixture_path.as_deref(),
            );
            fixture.apply_to_params(&mut p);
        }
        DebugManeuver::FullCapture | DebugManeuver::DemoOpen | DebugManeuver::None => {
            p.width = 320;
            p.height = 320;
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
            let maneuver = if launch.maneuver != DebugManeuver::None {
                launch.maneuver
            } else {
                DebugManeuver::FullCapture
            };
            *params = world_gen_params_for_maneuver(launch, maneuver);
            harness.defaults_applied = false;
            harness.logistics_visual_seeded = false;
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

/// Spawns [`Chunk`] + [`ChunkCellMatrix`] slabs so weather / ecology / fire / atmosphere have ECS targets
/// (tiles alone do not run chunk sim). World-gen only spawns tiles — CLI `--test` always needs these slabs.
fn spawn_test_scene_chunk_slabs_once(
    scene: Option<Res<ActiveTestScene>>,
    params: Res<WorldGenParams>,
    mut commands: Commands,
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
) {
    if !directory.by_edge.is_empty() && !graph.edges.is_empty() {
        return;
    }
    let snap = bake_snapshot_from_ordered_tile_markers(
        &[(8u32, 8u32), (9u32, 8u32), (10u32, 8u32)],
        |_, _| 0.5_f32,
        20.0,
        0.25,
    );
    hydrate_transport_from_snapshot(topology, fields, directory, &snap)
        .expect("visual proof transport hydrate");
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

fn apply_visual_logistics_minimap_defaults(
    graph: &LogisticsGraph,
    policy: Option<&mut crate::gui::RepresentationResult>,
    map_views: Option<&mut crate::gui::MapViewInstances>,
    overlay_tray: Option<&mut crate::gui::hud::HudOverlayTrayState>,
) {
    if !graph.edges.is_empty() {
        if let Some(policy) = policy {
            policy.overlay_matrix.logistics = true;
        }
    }
    if let Some(map_views) = map_views {
        map_views.minimap.overlays = crate::gui::simulation_minimap_overlay_defaults();
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
    );
    info!(
        target: "test_harness::logistics",
        "LOG-E01 visual proof: seeded transport_edges={} logistics_edges={} overlay_rows={overlay_rows}",
        directory.by_edge.len(),
        graph.edges.len()
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
            harness.concrete_chain_seed_phase = 2;
            harness.concrete_chain_e2e_seeded = true;
            info!(
                target: "test_harness::industrial",
                "IND-E01/E02 visual seed: portland chain operational (construction commit path)"
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
            seed_test_fire_near_world_center(&params, &mut fire_q, 6, 0.78, 0.65);
        }
        TestScene::Atmosphere => {
            if let Some(gw) = global_wind.as_mut() {
                gw.direction = Vec2::new(1.0, 0.2).normalize_or_zero();
                gw.speed = 6.0;
            }
            seed_test_fire_near_world_center(&params, &mut fire_q, 8, 0.55, 0.5);
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
            // LOD tile debug (green squares) off in visual proof — enable via dev tooling if needed.
            focus_debug.enabled = false;
            tile_debug.use_batched_mesh_overlay = true;
            if let Some(gw) = global_wind.as_mut() {
                gw.direction = Vec2::new(1.0, 0.22).normalize_or_zero();
                gw.speed = 5.8;
            }
            gpu.show = false;
            seed_test_fire_near_world_center(&params, &mut fire_q, 28, 0.92, 0.75);
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
        TestScene::VfxSandbox => {
            wx.particles = true;
            focus_debug.enabled = false;
            tile_debug.use_batched_mesh_overlay = true;
            if let Some(gw) = global_wind.as_mut() {
                gw.direction = Vec2::new(1.0, 0.2).normalize_or_zero();
                gw.speed = 6.0;
            }
            gpu.show = false;
            seed_test_fire_near_world_center(&params, &mut fire_q, 28, 0.92, 0.75);
            let mut wn = 0u32;
            for mut w in &mut wx_q {
                if wn >= 24 {
                    break;
                }
                w.wind_speed = w.wind_speed.max(0.65);
                w.fog_density = w.fog_density.max(0.18);
                w.rain_intensity = w.rain_intensity.max(0.82);
                w.soil_moisture = w.soil_moisture.max(0.55);
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
        sync_test_fire_overlay_from_ecs(&fire_q, &mut shared_overlay);
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
    harness.defaults_applied = true;
    raster_dirty.bump();
}

/// Keeps CLI test worlds burning after world-gen + fire extract (re-seed if sim cooled, refresh overlay).
fn maintain_test_scene_fire_overlay(
    scene: Option<Res<ActiveTestScene>>,
    harness: Res<TestWorldHarness>,
    params: Res<WorldGenParams>,
    frame: Res<FrameCount>,
    mut commands: Commands,
    mut fire_q: Query<(Entity, &Chunk, &ChunkCellMatrix, &mut ChunkSurfaceFire)>,
    mut shared_overlay: ResMut<crate::render::SharedOverlayFieldBuffers>,
    mut last_reseed_frame: Local<u32>,
) {
    let Some(active) = scene else {
        return;
    };
    if !harness.defaults_applied {
        return;
    }
    if !active.0.seeds_fire_overlay() {
        return;
    }
    let burning = fire_q
        .iter()
        .filter(|(_, _, _, fire)| fire.heat > 0.02)
        .count();
    if burning < 3 && frame.0.saturating_sub(*last_reseed_frame) > 45 {
        *last_reseed_frame = frame.0;
        let count = match active.0 {
            TestScene::Fire => 6,
            TestScene::Atmosphere => 8,
            TestScene::Visual | TestScene::VfxSandbox => 28,
            TestScene::None | TestScene::Weather => 0,
        };
        seed_test_fire_near_world_center(&params, &mut fire_q, count, 0.92, 0.75);
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
    sync_test_fire_overlay_from_ecs(&fire_q, &mut shared_overlay);
}

fn sync_test_fire_overlay_from_ecs(
    fire_q: &Query<(Entity, &Chunk, &ChunkCellMatrix, &mut ChunkSurfaceFire)>,
    shared: &mut crate::render::SharedOverlayFieldBuffers,
) {
    let mut next = HashMap::new();
    for (_, chunk, _, fire) in fire_q.iter() {
        if fire.heat >= crate::render::CHUNK_FIRE_OVERLAY_DISPLAY_MIN {
            let e = next.entry(chunk.coord).or_insert(0.0_f32);
            *e = f32::max(*e, fire.heat);
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
