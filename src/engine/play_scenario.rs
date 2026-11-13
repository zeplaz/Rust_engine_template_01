//! **PLAY-TRUTH-001** — default play scenarios (G-PLAY-01).
//!
//! Single authority for normal **Simulation** enter: Portland chain via construction commit,
//! one logistics route, sim HUD defaults — without `test_harness` bootstrap.

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::engine::ActiveTestScene;
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
) {
    *state = DefaultIndustrialPlayState::default();
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

pub struct PlayScenarioPlugin;

impl Plugin for PlayScenarioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActivePlayScenario>()
            .init_resource::<DefaultIndustrialPlayState>()
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
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

}
