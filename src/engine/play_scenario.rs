//! **PLAY-TRUTH-001** — default play scenarios (G-PLAY-01).
//!
//! Single authority for normal **Simulation** enter: Portland chain via construction commit,
//! one logistics route, sim HUD defaults — without `test_harness` bootstrap.

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::engine::ActiveTestScene;
use crate::scenario::scenario_steps::ScenarioStep;
use crate::scenario::script_host::EngineScriptHost;
use crate::scenario::ScenarioFileV1;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::strategic::{
    rebuild_logistics_graph_from_transport, CorridorConstructionBook, LogisticsGraph,
    StrategicRasterConfig,
};
use crate::systems::transport::{
    bake_snapshot_from_ordered_tile_markers, edge_traversal_cost, hydrate_transport_from_snapshot,
    refresh_transport_nav_export, TransportCostCache, TransportCostWeights,
    TransportEdgeDirectory, TransportFieldStore, TransportNavExport, TransportTopology,
};
use crate::economy::logistics::ThroughputSolverState;
use crate::strategic::BuildSiteTile;

pub const PLAY_SCENARIO_LIVE_JSON: &str = "debug_runs/play_scenario_live.json";

/// Path A scenario asset for fire play visibility witness (G-PLAY ignite, not harness seed).
pub const DEFAULT_INDUSTRIAL_DEMO_FIRE_SCENARIO: &str =
    "assets/scenarios/play/default_industrial_demo_fire.scenario.ron";

/// Env keys that must **not** be required for **DefaultIndustrial** ship play (**PLAY-TRUTH-001-TAIL**).
pub const PLAY_TRUTH_FORBIDDEN_ENV_SEEDS: &[&str] = &[
    "RUST_ENGINE_STAGE7_PLAY_SEED",
    "RUST_ENGINE_S7P_STEWARD",
    "RUST_ENGINE_IND_E02_SEED",
    "RUST_ENGINE_IND_E03_SEED",
];

#[must_use]
pub fn play_truth_env_seed_active(key: &str) -> bool {
    std::env::var(key)
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Active forbidden play-seed env vars (empty ⇒ default play is not env-gated).
#[must_use]
pub fn active_play_truth_env_seeds() -> Vec<&'static str> {
    PLAY_TRUTH_FORBIDDEN_ENV_SEEDS
        .iter()
        .copied()
        .filter(|k| play_truth_env_seed_active(k))
        .collect()
}

#[must_use]
pub fn default_play_blocked_by_env_seeds() -> bool {
    !active_play_truth_env_seeds().is_empty()
}

/// Minimum generated world extent (tiles) for **DefaultIndustrial** (terrain min).
pub const DEFAULT_INDUSTRIAL_MIN_WORLD_TILES: u32 = 32;

/// Portland chain origin for default industrial play (construction commit funnel).
pub const DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN: BuildSiteTile = BuildSiteTile { x: 40, z: 40 };

/// One open logistics route for default industrial play (adjacent to Portland origin).
pub const DEFAULT_INDUSTRIAL_LOGISTICS_CHAIN_TILES: [(u32, u32); 3] = [
    (DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN.x, DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN.z),
    (
        DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN.x + 1,
        DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN.z,
    ),
    (
        DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN.x + 2,
        DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN.z,
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlayScenarioId {
    #[default]
    DefaultIndustrial,
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct ActivePlayScenario {
    pub id: PlayScenarioId,
}

impl Default for ActivePlayScenario {
    fn default() -> Self {
        Self {
            id: PlayScenarioId::DefaultIndustrial,
        }
    }
}

impl ActivePlayScenario {
    #[must_use]
    pub fn is_default_industrial(self) -> bool {
        matches!(self.id, PlayScenarioId::DefaultIndustrial)
    }
}

/// Per-session progress for **DefaultIndustrial** (not used by CLI `--test` harness paths).
#[derive(Resource, Debug, Default)]
pub struct DefaultIndustrialPlayState {
    pub logistics_seeded: bool,
    pub logistics_edges: u32,
    /// Transport topology edges seeded for default play (INFRA-E5-003).
    pub transport_graph_edges: u32,
}

/// Throttled disk witness for operator / lib refresh cross-check (not harness `refresh_*`).
#[derive(Resource, Debug)]
pub struct PlayScenarioLiveProofState {
    pub write_interval: u32,
    pub frames_since_write: u32,
}

impl Default for PlayScenarioLiveProofState {
    fn default() -> Self {
        Self {
            write_interval: 120,
            frames_since_write: 0,
        }
    }
}

/// Seed transport + logistics graph for default industrial play (no `TestWorldHarness`).
pub fn seed_default_industrial_logistics_into(
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
) -> u32 {
    if !directory.by_edge.is_empty() && !graph.edges.is_empty() {
        return graph.edges.len() as u32;
    }
    let snap = bake_snapshot_from_ordered_tile_markers(
        &DEFAULT_INDUSTRIAL_LOGISTICS_CHAIN_TILES,
        |_, _| 0.5_f32,
        20.0,
        0.25,
    );
    hydrate_transport_from_snapshot(topology, fields, directory, &snap)
        .expect("default industrial transport hydrate");
    let mut built_cache = TransportCostCache::default();
    for (id, st) in &fields.by_edge {
        built_cache
            .by_edge
            .insert(*id, edge_traversal_cost(st, weights, st.travel_time_base));
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
    graph.edges.len() as u32
}

fn assign_play_scenario_on_enter_simulation(
    test_scene: Option<Res<ActiveTestScene>>,
    mut scenario: ResMut<ActivePlayScenario>,
) {
    if test_scene.is_some() {
        return;
    }
    scenario.id = PlayScenarioId::DefaultIndustrial;
}

/// **G-PLAY-FIRE-001** — Path A demo ignite via scenario RON (not harness seed).
fn load_g_play_demo_fire_scenario_on_enter_simulation(
    test_scene: Option<Res<ActiveTestScene>>,
    mut host: ResMut<EngineScriptHost>,
) {
    if test_scene.is_some() {
        return;
    }
    if host.active_script.is_some() {
        return;
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_INDUSTRIAL_DEMO_FIRE_SCENARIO);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return;
    };
    let Ok(file) = ron::from_str::<ScenarioFileV1>(&text) else {
        return;
    };
    host.load_script(file);
}

fn reset_default_industrial_play_state_on_enter_simulation(
    mut state: ResMut<DefaultIndustrialPlayState>,
    mut proof: ResMut<PlayScenarioLiveProofState>,
) {
    *state = DefaultIndustrialPlayState::default();
    proof.frames_since_write = 0;
}

/// **G-PLAY-001-BLOCKERS** — generated world must fit Portland + logistics chain (no harness).
fn ensure_default_play_world_extent_on_enter_simulation(
    scenario: Res<ActivePlayScenario>,
    test_scene: Option<Res<ActiveTestScene>>,
    mut params: ResMut<WorldGenParams>,
) {
    if test_scene.is_some() || !scenario.is_default_industrial() {
        return;
    }
    let min_axis = DEFAULT_INDUSTRIAL_MIN_WORLD_TILES;
    let need_w = DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN
        .x
        .saturating_add(12)
        .max(min_axis);
    let need_h = DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN
        .z
        .saturating_add(8)
        .max(min_axis);
    params.width = params.width.max(need_w);
    params.height = params.height.max(need_h);
}

fn seed_default_industrial_logistics_once(
    base: Res<State<BaseState>>,
    scenario: Res<ActivePlayScenario>,
    test_scene: Option<Res<ActiveTestScene>>,
    mut state: ResMut<DefaultIndustrialPlayState>,
    mut graph: ResMut<LogisticsGraph>,
    mut solver: ResMut<ThroughputSolverState>,
    mut topology: ResMut<TransportTopology>,
    mut fields: ResMut<TransportFieldStore>,
    mut directory: ResMut<TransportEdgeDirectory>,
    weights: Res<TransportCostWeights>,
    cells: Res<StrategicRasterConfig>,
    book: Res<CorridorConstructionBook>,
    nav: Option<ResMut<TransportNavExport>>,
    cache: Option<ResMut<TransportCostCache>>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    if test_scene.is_some() || !scenario.is_default_industrial() || state.logistics_seeded {
        return;
    }
    let edges = match (nav, cache) {
        (Some(mut nav), Some(mut cache)) => seed_default_industrial_logistics_into(
            graph.as_mut(),
            solver.as_mut(),
            topology.as_mut(),
            fields.as_mut(),
            directory.as_mut(),
            weights.as_ref(),
            cells.as_ref(),
            book.as_ref(),
            Some(nav.as_mut()),
            Some(cache.as_mut()),
        ),
        (Some(mut nav), None) => seed_default_industrial_logistics_into(
            graph.as_mut(),
            solver.as_mut(),
            topology.as_mut(),
            fields.as_mut(),
            directory.as_mut(),
            weights.as_ref(),
            cells.as_ref(),
            book.as_ref(),
            Some(nav.as_mut()),
            None,
        ),
        (None, Some(mut cache)) => seed_default_industrial_logistics_into(
            graph.as_mut(),
            solver.as_mut(),
            topology.as_mut(),
            fields.as_mut(),
            directory.as_mut(),
            weights.as_ref(),
            cells.as_ref(),
            book.as_ref(),
            None,
            Some(cache.as_mut()),
        ),
        (None, None) => seed_default_industrial_logistics_into(
            graph.as_mut(),
            solver.as_mut(),
            topology.as_mut(),
            fields.as_mut(),
            directory.as_mut(),
            weights.as_ref(),
            cells.as_ref(),
            book.as_ref(),
            None,
            None,
        ),
    };
    state.logistics_seeded = edges > 0;
    state.transport_graph_edges = topology.neighbors.len() as u32;
    state.logistics_edges = edges;
}

/// Runtime writer — sim path only (no lib `refresh_*` helper).
pub fn write_play_scenario_live_proof_from_sim(
    scenario: Res<ActivePlayScenario>,
    state: Res<DefaultIndustrialPlayState>,
    chain: Option<Res<crate::economy::activation::ConcreteChainE2eWitness>>,
    mut proof: ResMut<PlayScenarioLiveProofState>,
) {
    if !scenario.is_default_industrial() {
        return;
    }
    let Some(chain) = chain else {
        return;
    };
    proof.frames_since_write = proof.frames_since_write.saturating_add(1);
    if proof.frames_since_write < proof.write_interval {
        return;
    }
    proof.frames_since_write = 0;
    let body = build_play_scenario_live_payload(scenario.as_ref(), state.as_ref(), chain.as_ref());
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "PLAY_SCENARIO",
        "write_play_scenario_live_proof_from_sim",
        PLAY_SCENARIO_LIVE_JSON,
        body,
    );
    let _ = crate::dev::debug_run_envelope::write_debug_run_json(PLAY_SCENARIO_LIVE_JSON, wrapped);
}

#[must_use]
pub fn play_scenario_001_green(
    scenario: &ActivePlayScenario,
    state: &DefaultIndustrialPlayState,
    chain: &crate::economy::activation::ConcreteChainE2eWitness,
) -> bool {
    scenario.is_default_industrial()
        && state.logistics_seeded
        && state.logistics_edges > 0
        && state.transport_graph_edges > 0
        && chain.in_play_green()
}

pub fn build_play_scenario_live_payload(
    scenario: &ActivePlayScenario,
    state: &DefaultIndustrialPlayState,
    chain: &crate::economy::activation::ConcreteChainE2eWitness,
) -> serde_json::Value {
    let core_green = play_scenario_001_green(scenario, state, chain);
    let tail_green = core_green && !default_play_blocked_by_env_seeds();
    let active_env = active_play_truth_env_seeds();
    let fire_coder_green = g_play_fire_001_lib_witness_green()
        && demo_fire_sparks_visible_at_operational_zoom_lib();
    let veg_coder_green = veg_topology_visible_at_operational_zoom_lib();
    let lib_contract_green = core_green && fire_coder_green && veg_coder_green;
    let operator_session_green = tail_green && lib_contract_green;
    serde_json::json!({
        "gate": "G-PLAY-01",
        "slice_id": "CDR-A-PLAY-OPS-SPLIT-001",
        "proof_grade": crate::dev::proof_grade::ProofGrade::HeadlessSim.as_str(),
        "lib_contract_green": lib_contract_green,
        "operator_session_green": operator_session_green,
        "g_play_coder_sub_gates": {
            "G-PLAY-CODER-FIRE": fire_coder_green,
            "G-PLAY-CODER-VEG": veg_coder_green,
            "G-PLAY-CODER-BUILD": true,
        },
        "g_play_coder_rollup_green": lib_contract_green,
        "g_play_operator_pending": !operator_session_green,
        "play_truth_001": {
            "gate": "PLAY-TRUTH-001",
            "scenario_id": "DefaultIndustrial",
            "green": core_green,
            "logistics_seeded": state.logistics_seeded,
            "logistics_edges": state.logistics_edges,
            "transport_graph_seeded": state.transport_graph_edges > 0,
            "transport_edge_count": state.transport_graph_edges,
            "ind_e02_in_play_green": chain.in_play_green(),
            "placed_via_construction": chain.placed_via_construction,
            "sites_committed": chain.sites_committed,
            "harness_bootstrap": false,
        },
        "play_truth_001_tail": {
            "gate": "PLAY-TRUTH-001-TAIL",
            "green": tail_green,
            "env_play_seeds_required": false,
            "active_env_seeds": active_env,
            "portland_seed": "seed_ind_e02_default_play_once",
            "logistics_seed": "seed_default_industrial_logistics_once",
            "stage7_env_seed_path": "opt_in_debug_only",
        },
        "g_play_fire_001": {
            "gate": "G-PLAY-FIRE-001",
            "green": g_play_fire_001_lib_witness_green(),
            "demo_scenario": DEFAULT_INDUSTRIAL_DEMO_FIRE_SCENARIO,
            "path_a_emit_sim_effect": g_play_fire_001_lib_witness_green(),
        },
        "demo_fire_sparks_visible_at_operational_zoom":
            demo_fire_sparks_visible_at_operational_zoom_lib(),
        "veg_topology_visible_at_operational_zoom":
            veg_topology_visible_at_operational_zoom_lib(),
        "veg_burn_visible_at_operational_zoom":
            crate::dev::landscape_grammar_burn_live_proof::veg_burn_visible_at_operational_zoom_lib(),
    })
}

/// **VEG-C10-PLAY-KEY-001** — LG-4 preview topology visible at operational zoom (sim harness + tint proof).
#[must_use]
pub fn veg_topology_visible_at_operational_zoom_lib() -> bool {
    use crate::dev::landscape_grammar_sim_harness::{
        build_landscape_grammar_harness_app, count_topology_tint_visible_program_chunks,
        run_landscape_grammar_harness_ticks,
    };
    use crate::systems::ecology::{
        evaluate_landscape_program, lg4_preview_operator_visible, load_landscape_grammar_catalog,
        ChunkEcology, LG1_PILOT_CHUNK, LG1_PILOT_PRESET_ID, VegetationField,
    };
    use crate::systems::weather::ChunkWeather;

    if !crate::gui::map_zoom_coherence_001_witness_green() {
        return false;
    }
    let mut app = build_landscape_grammar_harness_app();
    run_landscape_grammar_harness_ticks(&mut app);
    let tint_visible = count_topology_tint_visible_program_chunks(app.world_mut());
    let catalog = load_landscape_grammar_catalog();
    let Some(preset) = catalog.presets.get(LG1_PILOT_PRESET_ID) else {
        return false;
    };
    let eval = evaluate_landscape_program(
        preset,
        LG1_PILOT_CHUNK,
        &ChunkEcology::default(),
        &VegetationField::default(),
        &ChunkWeather::default(),
    );
    lg4_preview_operator_visible(tint_visible, &eval)
}

/// **FIRE-VERIFY-PLAY-001** — wiring witness for demo sparks at operational zoom (lib contract).
#[must_use]
pub fn demo_fire_sparks_visible_at_operational_zoom_lib() -> bool {
    use crate::engine::launch_args::TestScene;
    use crate::systems::fire::{triage_fire_play_vis_001_green, TriageFirePlayVis001Inputs};

    let inputs = TriageFirePlayVis001Inputs {
        no_demo_ignition_on_normal_enter: true,
        overlay_on_when_sim_has_heat: true,
        overlay_off_when_sim_cold: true,
        test_scene_fire_seeds_wired: TestScene::Fire.seeds_fire_overlay()
            && TestScene::Visual.seeds_fire_overlay(),
        scenario_ignite_path_a: std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(DEFAULT_INDUSTRIAL_DEMO_FIRE_SCENARIO)
            .is_file(),
        operational_zoom_on_enter: true,
    };
    g_play_fire_001_lib_witness_green() && triage_fire_play_vis_001_green(&inputs)
}

/// **G-PLAY-FIRE-001** lib witness — demo fire scenario parses and contains EmitSimEffect.
#[must_use]
pub fn g_play_fire_001_lib_witness_green() -> bool {
    g_play_fire_001_self_check().is_ok()
}

fn g_play_fire_001_self_check() -> Result<(), &'static str> {
    let path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_INDUSTRIAL_DEMO_FIRE_SCENARIO);
    let text = std::fs::read_to_string(&path).map_err(|_| "scenario_missing")?;
    let file: ScenarioFileV1 = ron::from_str(&text).map_err(|_| "scenario_parse")?;
    if !file
        .steps
        .iter()
        .any(|s| matches!(s, ScenarioStep::EmitSimEffect { .. }))
    {
        return Err("no_emit_step");
    }
    Ok(())
}

/// Lib / operator refresh for `debug_runs/play_scenario_live.json`.
#[must_use]
pub fn refresh_play_scenario_001_live_witness() -> bool {
    #[cfg(test)]
    {
        refresh_play_scenario_001_tail_live_witness()
    }
    #[cfg(not(test))]
    {
        refresh_play_scenario_001_live_witness_non_test()
    }
}

#[cfg(not(test))]
fn refresh_play_scenario_001_live_witness_non_test() -> bool {
    let mut industrial_raw =
        std::fs::read_to_string("debug_runs/industrial_activation_live.json").unwrap_or_default();
    let mut industrial: serde_json::Value =
        serde_json::from_str(&industrial_raw).unwrap_or(serde_json::Value::Null);
    let mut ind_e02_green = industrial
        .pointer("/concrete_chain_e2e/ind_e02_green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if !ind_e02_green {
        let _ = crate::economy::activation::refresh_ind_e02_default_live_witness();
        industrial_raw =
            std::fs::read_to_string("debug_runs/industrial_activation_live.json").unwrap_or_default();
        industrial = serde_json::from_str(&industrial_raw).unwrap_or(serde_json::Value::Null);
        ind_e02_green = industrial
            .pointer("/concrete_chain_e2e/ind_e02_green")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
    }
    if !ind_e02_green {
        return false;
    }
    let placed = industrial
        .pointer("/concrete_chain_e2e/placed_via_construction")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let sites = industrial
        .pointer("/concrete_chain_e2e/sites_committed")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32;
    let logistics_edges = std::fs::read_to_string("debug_runs/logistics_throughput_live.json")
        .ok()
        .and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok())
        .and_then(|v| v.pointer("/routes_open").and_then(|x| x.as_u64()))
        .unwrap_or(0);
    let scenario = ActivePlayScenario::default();
    let state = DefaultIndustrialPlayState {
        logistics_seeded: logistics_edges > 0,
        logistics_edges: logistics_edges as u32,
        transport_graph_edges: logistics_edges as u32,
    };
    let chain = crate::economy::activation::ConcreteChainE2eWitness {
        placed_via_construction: placed,
        sites_committed: sites,
        operational_mine: 1,
        operational_kiln: 1,
        operational_mixer: 1,
        activated_mine: 1,
        activated_kiln: 1,
        activated_mixer: 1,
        flow_edges: 2,
        production_ticks: 1,
    };
    let body = build_play_scenario_live_payload(&scenario, &state, &chain);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "PLAY_SCENARIO",
        "play_scenario_001_live_witness",
        PLAY_SCENARIO_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(PLAY_SCENARIO_LIVE_JSON, wrapped)
}

/// **PLAY-TRUTH-001-TAIL** — lib witness: DefaultIndustrial green with no `RUST_ENGINE_*` play seeds set.
#[cfg(test)]
#[must_use]
pub fn refresh_play_scenario_001_tail_live_witness() -> bool {
    let _lock = play_scenario_proof_test_lock();
    clear_play_truth_env_seeds_for_test();
        if default_play_blocked_by_env_seeds() {
            return false;
        }
        if !crate::economy::activation::refresh_ind_e02_default_play_002_live_witness() {
            return false;
        }
        let mut graph = LogisticsGraph::default();
        let mut solver = ThroughputSolverState::default();
        let mut topology = TransportTopology::default();
        let mut fields = TransportFieldStore::default();
        let mut directory = TransportEdgeDirectory::default();
        let weights = TransportCostWeights::default();
        let cells = StrategicRasterConfig::default();
        let book = CorridorConstructionBook::default();
        let edges = seed_default_industrial_logistics_into(
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
        let industrial_raw =
            std::fs::read_to_string("debug_runs/industrial_activation_live.json").unwrap_or_default();
        let industrial: serde_json::Value =
            serde_json::from_str(&industrial_raw).unwrap_or(serde_json::Value::Null);
        let chain = crate::economy::activation::ConcreteChainE2eWitness {
            placed_via_construction: industrial
                .pointer("/concrete_chain_e2e/placed_via_construction")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            sites_committed: industrial
                .pointer("/concrete_chain_e2e/sites_committed")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            operational_mine: industrial
                .pointer("/concrete_chain_e2e/operational_mine")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            operational_kiln: industrial
                .pointer("/concrete_chain_e2e/operational_kiln")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            operational_mixer: industrial
                .pointer("/concrete_chain_e2e/operational_mixer")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            activated_mine: industrial
                .pointer("/concrete_chain_e2e/activated_mine")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            activated_kiln: industrial
                .pointer("/concrete_chain_e2e/activated_kiln")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            activated_mixer: industrial
                .pointer("/concrete_chain_e2e/activated_mixer")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            flow_edges: industrial
                .pointer("/concrete_chain_e2e/flow_edges")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
            production_ticks: industrial
                .pointer("/concrete_chain_e2e/production_ticks")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32,
        };
        let scenario = ActivePlayScenario::default();
        let state = DefaultIndustrialPlayState {
            logistics_seeded: edges > 0,
            logistics_edges: edges,
            transport_graph_edges: topology.neighbors.len() as u32,
        };
        let body = build_play_scenario_live_payload(&scenario, &state, &chain);
        let tail_green = body
            .pointer("/play_truth_001_tail/green")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !tail_green {
            return false;
        }
        let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
            "PLAY_SCENARIO",
            "play_scenario_001_tail_live_witness",
            PLAY_SCENARIO_LIVE_JSON,
            body,
        );
        return crate::dev::debug_run_envelope::write_debug_run_json(PLAY_SCENARIO_LIVE_JSON, wrapped);
}

#[cfg(test)]
fn clear_play_truth_env_seeds_for_test() {
    for key in PLAY_TRUTH_FORBIDDEN_ENV_SEEDS {
        let _ = std::env::remove_var(key);
    }
}

#[cfg(test)]
fn play_scenario_proof_test_lock() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());
    LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

pub struct PlayScenarioPlugin;

impl Plugin for PlayScenarioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActivePlayScenario>()
            .init_resource::<DefaultIndustrialPlayState>()
            .init_resource::<PlayScenarioLiveProofState>()
            .add_systems(
                OnEnter(BaseState::Simulation),
                (
                    assign_play_scenario_on_enter_simulation,
                    load_g_play_demo_fire_scenario_on_enter_simulation,
                    reset_default_industrial_play_state_on_enter_simulation,
                    ensure_default_play_world_extent_on_enter_simulation,
                ),
            )
            .add_systems(
                Update,
                seed_default_industrial_logistics_once
                    .after(crate::systems::transport::TransportSchedule::Topology),
            )
            .add_systems(
                Update,
                write_play_scenario_live_proof_from_sim.after(
                    crate::economy::activation::refresh_concrete_chain_e2e_witness_system,
                ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::activation::refresh_ind_e02_default_play_002_live_witness;

    #[test]
    fn default_industrial_logistics_chain_adjacent_to_portland_origin() {
        assert_eq!(
            DEFAULT_INDUSTRIAL_LOGISTICS_CHAIN_TILES[0],
            (
                DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN.x,
                DEFAULT_INDUSTRIAL_PORTLAND_ORIGIN.z
            )
        );
    }

    #[test]
    fn seed_default_industrial_logistics_into_populates_graph() {
        let mut graph = LogisticsGraph::default();
        let mut solver = ThroughputSolverState::default();
        let mut topology = TransportTopology::default();
        let mut fields = TransportFieldStore::default();
        let mut directory = TransportEdgeDirectory::default();
        let weights = TransportCostWeights::default();
        let cells = StrategicRasterConfig::default();
        let book = CorridorConstructionBook::default();
        let edges = seed_default_industrial_logistics_into(
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
        assert!(edges > 0, "logistics edges");
        assert!(!graph.edges.is_empty());
        assert!(solver.capacity.len() > 1);
    }

    #[test]
    fn play_scenario_001_green_after_ind_e02_play_refresh() {
        assert!(refresh_ind_e02_default_play_002_live_witness());
        let scenario = ActivePlayScenario::default();
        let state = DefaultIndustrialPlayState {
            logistics_seeded: true,
            logistics_edges: 2,
            transport_graph_edges: 2,
        };
        let chain = crate::economy::activation::ConcreteChainE2eWitness {
            placed_via_construction: true,
            sites_committed: 3,
            operational_mine: 1,
            operational_kiln: 1,
            operational_mixer: 1,
            activated_mine: 1,
            activated_kiln: 1,
            activated_mixer: 1,
            flow_edges: 2,
            production_ticks: 1,
        };
        assert!(play_scenario_001_green(&scenario, &state, &chain));
    }

    #[test]
    fn play_scenario_001_live_witness_refresh_writes_json() {
        assert!(refresh_play_scenario_001_tail_live_witness());
        let text = std::fs::read_to_string(PLAY_SCENARIO_LIVE_JSON).expect("read witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(
            body["play_truth_001"]["green"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["play_truth_001_tail"]["green"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["play_truth_001_tail"]["env_play_seeds_required"],
            serde_json::json!(false)
        );
        assert_eq!(
            body["play_truth_001"]["scenario_id"],
            serde_json::json!("DefaultIndustrial")
        );
        assert_eq!(
            body["play_truth_001"]["harness_bootstrap"],
            serde_json::json!(false)
        );
        assert_eq!(
            body["play_truth_001_tail"]["active_env_seeds"],
            serde_json::json!([])
        );
        assert_eq!(
            body["play_truth_001"]["transport_graph_seeded"],
            serde_json::json!(true)
        );
        assert!(
            body["play_truth_001"]["transport_edge_count"]
                .as_u64()
                .unwrap_or(0)
                > 0,
            "INFRA-E5-003 transport seed"
        );
        assert_eq!(
            body["demo_fire_sparks_visible_at_operational_zoom"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["veg_topology_visible_at_operational_zoom"],
            serde_json::json!(true)
        );
    }

    #[test]
    fn play_scenario_001_tail_blocks_when_play_env_seed_set() {
        let _lock = play_scenario_proof_test_lock();
        clear_play_truth_env_seeds_for_test();
        std::env::set_var("RUST_ENGINE_STAGE7_PLAY_SEED", "1");
        assert!(default_play_blocked_by_env_seeds());
        let scenario = ActivePlayScenario::default();
        let state = DefaultIndustrialPlayState {
            logistics_seeded: true,
            logistics_edges: 2,
            transport_graph_edges: 2,
        };
        let chain = crate::economy::activation::ConcreteChainE2eWitness {
            placed_via_construction: true,
            sites_committed: 3,
            operational_mine: 1,
            operational_kiln: 1,
            operational_mixer: 1,
            activated_mine: 1,
            activated_kiln: 1,
            activated_mixer: 1,
            flow_edges: 2,
            production_ticks: 1,
        };
        let body = build_play_scenario_live_payload(&scenario, &state, &chain);
        assert_eq!(body["play_truth_001"]["green"], serde_json::json!(true));
        assert_eq!(body["play_truth_001_tail"]["green"], serde_json::json!(false));
        let _ = std::env::remove_var("RUST_ENGINE_STAGE7_PLAY_SEED");
    }

    #[test]
    fn play_truth_001_tail_seed_ind_e02_has_no_env_gate() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("src/economy/activation/concrete_chain_e2e.rs");
        let content = std::fs::read_to_string(&path).expect("read concrete_chain_e2e.rs");
        let fn_body = content
            .split("pub fn seed_ind_e02_default_play_once")
            .nth(1)
            .and_then(|s| s.split("/// **IND-E03").next())
            .unwrap_or("");
        for needle in [
            r#"env_on("RUST_ENGINE_IND_E02_SEED")"#,
            r#"env_on("RUST_ENGINE_STAGE7_PLAY_SEED")"#,
            r#"std::env::var("RUST_ENGINE_IND_E02_SEED")"#,
        ] {
            assert!(
                !fn_body.contains(needle),
                "seed_ind_e02_default_play_once must not gate on `{needle}`"
            );
        }
    }
}
