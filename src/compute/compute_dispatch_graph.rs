//! **Compute dispatch graph** — CPU-side orchestrator for snapshot → GPU compute workloads (logic offload).
//!
//! Sibling to [`crate::render::extraction::RenderProjectionGraph`]: both read the same frame layer and
//! [`crate::gui::WorldRepresentationFrame`]; neither queries ECS inside node dispatch and neither writes render buffers.
//!
//! v1: [`FireInfluenceDispatchNode`] policy + [`super::heat_diffusion::HeatDiffusionDispatchNode`] kernel.
//!
//! **CONC-10** (gea4v2 17.7.3; LEGACY `concurrency`): [`COMPUTE_STAGE_EDGES`] declares the stage
//! dependency graph; width is sized by critical path ([`compute_parallel_width_for_cores`]), never
//! by core count alone.

use bevy::prelude::*;

use crate::gui::{RepresentationResult, WorldLodBand, WorldLodMap, WorldRepresentationFrame};
use crate::render::FireSimulationSnapshot;

use super::frame_snapshots::{AgentFrame, NavFieldFrame};
use super::heat_diffusion::{
    sync_nav_field_from_heat_diffusion, HeatDiffusionDispatchNode,
    HeatDiffusionFieldBuffers,
};

#[inline]
fn band_fidelity_rank(band: WorldLodBand) -> u8 {
    match band {
        WorldLodBand::LocalTactical => 0,
        WorldLodBand::Operational => 1,
        WorldLodBand::Strategic => 2,
        WorldLodBand::Macro => 3,
    }
}

/// Stages of one compute dispatch pass, in the order [`run_compute_dispatch_graph`] executes them.
///
/// **CONC-10** (gea4v2 17.7.3 *Job dependencies and degree of parallelism*; LEGACY `concurrency` /
/// `concurrent_object_updates`, gea_v2_e4 p620-625): parallel width is bounded by the dependency
/// graph's critical path, **not** by core count. Each stage reads the previous stage's output, so
/// today's graph is a pure chain (width 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComputeStage {
    /// [`FireInfluenceDispatchNode::dispatch`] — LOD policy → `dispatch_active`.
    FireInfluence = 0,
    /// [`HeatDiffusionDispatchNode::plan`] — needs `dispatch_active` from [`Self::FireInfluence`].
    HeatDiffusionPlan = 1,
    /// [`advance_heat_diffusion_field`] — needs the plan's active flag and chunk set.
    HeatDiffusionAdvance = 2,
    /// [`sync_nav_field_from_heat_diffusion`] — reads the advanced field generation.
    NavFieldSync = 3,
}

pub const COMPUTE_STAGE_COUNT: usize = 4;

/// Execution order = one valid topological order of [`COMPUTE_STAGE_EDGES`].
pub const COMPUTE_STAGE_EXECUTION_ORDER: [ComputeStage; COMPUTE_STAGE_COUNT] = [
    ComputeStage::FireInfluence,
    ComputeStage::HeatDiffusionPlan,
    ComputeStage::HeatDiffusionAdvance,
    ComputeStage::NavFieldSync,
];

/// Data dependencies `(producer, consumer)`: the consumer may not start before the producer finishes.
pub const COMPUTE_STAGE_EDGES: [(ComputeStage, ComputeStage); 3] = [
    (ComputeStage::FireInfluence, ComputeStage::HeatDiffusionPlan),
    (ComputeStage::HeatDiffusionPlan, ComputeStage::HeatDiffusionAdvance),
    (ComputeStage::HeatDiffusionAdvance, ComputeStage::NavFieldSync),
];

/// Longest-path level of each stage (sources = 0); `None` when the edge table has a cycle.
#[must_use]
pub fn compute_stage_levels(
    edges: &[(ComputeStage, ComputeStage)],
) -> Option<[u8; COMPUTE_STAGE_COUNT]> {
    let mut level = [0u8; COMPUTE_STAGE_COUNT];
    // A DAG on N nodes settles in at most N-1 relaxation passes; a change on pass N means a cycle.
    for pass in 0..COMPUTE_STAGE_COUNT {
        let mut changed = false;
        for &(from, to) in edges {
            let want = level[from as usize] + 1;
            if want > level[to as usize] {
                level[to as usize] = want;
                changed = true;
            }
        }
        if !changed {
            return Some(level);
        }
        if pass + 1 == COMPUTE_STAGE_COUNT {
            return None;
        }
    }
    None
}

/// Critical path length in stages (serial depth no amount of cores can shorten).
#[must_use]
pub fn compute_critical_path_len(edges: &[(ComputeStage, ComputeStage)]) -> Option<usize> {
    let level = compute_stage_levels(edges)?;
    Some(level.iter().copied().max().map_or(0, |m| m as usize + 1))
}

/// Widest set of stages that share a dependency level — the graph's degree of parallelism.
#[must_use]
pub fn compute_graph_parallel_width(edges: &[(ComputeStage, ComputeStage)]) -> Option<usize> {
    let level = compute_stage_levels(edges)?;
    let mut counts = [0usize; COMPUTE_STAGE_COUNT];
    for l in level {
        counts[l as usize] += 1;
    }
    counts.into_iter().max()
}

/// Workers worth dispatching: `min(cores, graph width)`; extra cores beyond width idle (CONC-10).
#[must_use]
pub fn compute_parallel_width_for_cores(
    edges: &[(ComputeStage, ComputeStage)],
    core_count: usize,
) -> usize {
    let width = compute_graph_parallel_width(edges).unwrap_or(1);
    core_count.max(1).min(width)
}

/// A caller asked for more concurrent compute stages than the dependency graph permits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParallelClaimRejected {
    pub claimed: usize,
    pub graph_width: usize,
}

/// Reject over-parallel claims that the edge table cannot support.
pub fn check_parallel_claim(
    edges: &[(ComputeStage, ComputeStage)],
    claimed: usize,
) -> Result<(), ParallelClaimRejected> {
    let graph_width = compute_graph_parallel_width(edges).unwrap_or(1);
    if claimed > graph_width {
        Err(ParallelClaimRejected { claimed, graph_width })
    } else {
        Ok(())
    }
}

/// Shared read-only inputs for compute nodes (extend as snapshot producers appear).
pub struct ComputeContext<'a> {
    pub policy: &'a RepresentationResult,
    pub lod: &'a WorldRepresentationFrame,
    pub lod_map: &'a WorldLodMap,
    pub agents: &'a AgentFrame,
    pub navigation: &'a NavFieldFrame,
    pub fire: &'a FireSimulationSnapshot,
    /// DH-03a: RO climate means (wind/rain) for dense heat — never writes weather.
    pub climate: Option<&'a crate::render::ClimateVisualAggregate>,
    /// FIDELITY-1 B6: soft cap from [`crate::systems::sim_attention::SimAttentionFrame::budget_path_units`].
    pub path_budget: u32,
}

pub trait ComputeNodeTrait {
    fn dispatch(&mut self, ctx: &ComputeContext<'_>);
}

/// LOD policy for fire → agent influence / hazard compute (outputs stay in compute-owned state).
#[derive(Debug, Clone)]
pub struct FireInfluenceDispatchNode {
    pub lod: WorldLodBand,
    pub influence_chunk_count: usize,
    /// When false, downstream GPU kernels should skip or use aggregated fields only.
    pub dispatch_active: bool,
    pub target_dispatch_hz: f32,
}

impl Default for FireInfluenceDispatchNode {
    fn default() -> Self {
        Self {
            lod: WorldLodBand::LocalTactical,
            influence_chunk_count: 0,
            dispatch_active: false,
            target_dispatch_hz: 60.0,
        }
    }
}

impl ComputeNodeTrait for FireInfluenceDispatchNode {
    fn dispatch(&mut self, ctx: &ComputeContext<'_>) {
        let fallback = ctx.lod.global_band();
        let mut active_chunks = 0usize;
        let mut coarsest = fallback;
        for row in &ctx.fire.chunk_heat {
            let band = ctx.lod_map.compute_band_at(row.chunk, fallback);
            if ctx.policy.pathfinding_active_at_compute_band(band) {
                active_chunks += 1;
            }
            if band_fidelity_rank(band) > band_fidelity_rank(coarsest) {
                coarsest = band;
            }
        }
        self.lod = coarsest;
        self.target_dispatch_hz = ctx.policy.compute_budget.dispatch_hz;
        self.influence_chunk_count = active_chunks;
        self.dispatch_active =
            ctx.policy.compute_budget.heat_diffusion && active_chunks > 0;
    }
}

/// Root graph: orchestrates compute nodes (fire influence policy + heat diffusion kernel).
#[derive(Resource, Debug, Clone)]
pub struct ComputeDispatchGraph {
    pub fire_influence: FireInfluenceDispatchNode,
    pub heat_diffusion: HeatDiffusionDispatchNode,
}

impl Default for ComputeDispatchGraph {
    fn default() -> Self {
        Self {
            fire_influence: FireInfluenceDispatchNode::default(),
            heat_diffusion: HeatDiffusionDispatchNode::default(),
        }
    }
}

impl ComputeNodeTrait for ComputeDispatchGraph {
    fn dispatch(&mut self, ctx: &ComputeContext<'_>) {
        self.fire_influence.dispatch(ctx);
        self.heat_diffusion
            .plan(ctx, self.fire_influence.dispatch_active);
    }
}

/// Throttles [`run_compute_dispatch_graph`] to [`WorldResolutionPolicy::compute_dispatch_hz`].
#[derive(Resource, Debug, Default)]
pub struct ComputeDispatchCadence {
    pub accumulator_secs: f32,
}

/// Single **Update** entry: build [`ComputeContext`] and run the graph (no per-domain compute systems).
pub fn run_compute_dispatch_graph(
    time: Res<Time>,
    policy: Res<RepresentationResult>,
    lod: Res<WorldRepresentationFrame>,
    lod_map: Res<WorldLodMap>,
    fire: Res<FireSimulationSnapshot>,
    climate: Option<Res<crate::render::ClimateVisualAggregate>>,
    attention: Option<Res<crate::systems::sim_attention::SimAttentionFrame>>,
    mut agents: ResMut<AgentFrame>,
    mut navigation: ResMut<NavFieldFrame>,
    mut cadence: ResMut<ComputeDispatchCadence>,
    mut graph: ResMut<ComputeDispatchGraph>,
    mut heat_field: ResMut<HeatDiffusionFieldBuffers>,
    mut dense_play: ResMut<super::DenseHeatPlayField>,
    heat_cfg: Res<super::HeatDiffusionConfig>,
) {
    cadence.accumulator_secs += time.delta_secs();
    let min_interval = 1.0 / policy.compute_budget.dispatch_hz.max(1e-3);
    if cadence.accumulator_secs < min_interval {
        return;
    }
    cadence.accumulator_secs = 0.0;

    agents.stamp = fire.stamp;
    navigation.stamp = fire.stamp;

    let path_budget = attention
        .as_ref()
        .map(|a| a.budget_path_units)
        .unwrap_or(64);
    let ctx = ComputeContext {
        policy: &policy,
        lod: &lod,
        lod_map: &lod_map,
        agents: &agents,
        navigation: &navigation,
        fire: &fire,
        climate: climate.as_deref(),
        path_budget,
    };
    graph.dispatch(&ctx);
    if heat_cfg.use_sparse_play {
        super::heat_diffusion::advance_heat_diffusion_field_with_config(
            &mut heat_field,
            &ctx,
            graph.fire_influence.dispatch_active,
            &heat_cfg,
        );
    } else {
        // DH-02: dense WGSL-parity owns play (sparse gated off).
        super::heat_diffusion_gpu::advance_dense_heat_play(
            &mut dense_play,
            &mut heat_field,
            &ctx,
            graph.fire_influence.dispatch_active,
        );
    }
    graph.heat_diffusion.last_generation = heat_field.generation;
    sync_nav_field_from_heat_diffusion(&mut navigation, &heat_field);
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComputeDispatchSystemSet {
    Dispatch,
}

pub struct ComputeDispatchPlugin;

impl Plugin for ComputeDispatchPlugin {
    fn build(&self, app: &mut App) {
        debug_assert!(
            compute_stage_levels(&COMPUTE_STAGE_EDGES).is_some(),
            "COMPUTE_STAGE_EDGES must be acyclic"
        );
        app.init_resource::<AgentFrame>()
            .init_resource::<NavFieldFrame>()
            .init_resource::<crate::render::FireSimulationSnapshot>()
            .init_resource::<ComputeDispatchCadence>()
            .init_resource::<ComputeDispatchGraph>()
            .init_resource::<HeatDiffusionFieldBuffers>()
            .init_resource::<super::HeatDiffusionConfig>()
            .init_resource::<super::DenseHeatPlayField>();
        super::heat_diffusion::register_heat_diffusion_gpu_sync(app);
        app.configure_sets(Update, ComputeDispatchSystemSet::Dispatch)
            .add_systems(
                Update,
                run_compute_dispatch_graph.in_set(ComputeDispatchSystemSet::Dispatch),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{
        build_representation_inputs, build_representation_result, CameraVisualState, LodZoneRegistry,
        VisualBudgetSettings, VisualCadence,
    };
    use crate::render::FireSimulationSnapshot;
    use crate::render::extraction::sim_visual_extract::ChunkFireHeat;

    fn policy_for(lod: &WorldRepresentationFrame, fire: &FireSimulationSnapshot) -> RepresentationResult {
        let inputs = build_representation_inputs(
            &CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            fire.stamp,
        );
        build_representation_result(lod, &inputs)
    }

    #[test]
    fn conc10_execution_order_respects_every_dependency_edge() {
        let pos = |s: ComputeStage| {
            COMPUTE_STAGE_EXECUTION_ORDER
                .iter()
                .position(|&x| x == s)
                .expect("stage in order")
        };
        for &(from, to) in &COMPUTE_STAGE_EDGES {
            assert!(pos(from) < pos(to), "{from:?} must precede {to:?}");
        }
        assert!(compute_stage_levels(&COMPUTE_STAGE_EDGES).is_some());
    }

    #[test]
    fn conc10_chain_graph_critical_path_is_full_depth_and_width_one() {
        assert_eq!(compute_critical_path_len(&COMPUTE_STAGE_EDGES), Some(COMPUTE_STAGE_COUNT));
        assert_eq!(compute_graph_parallel_width(&COMPUTE_STAGE_EDGES), Some(1));
        // 64 cores do not widen a dependency chain.
        assert_eq!(compute_parallel_width_for_cores(&COMPUTE_STAGE_EDGES, 64), 1);
        assert_eq!(compute_parallel_width_for_cores(&COMPUTE_STAGE_EDGES, 0), 1);
    }

    #[test]
    fn conc10_over_parallel_claim_is_rejected() {
        assert_eq!(check_parallel_claim(&COMPUTE_STAGE_EDGES, 1), Ok(()));
        assert_eq!(
            check_parallel_claim(&COMPUTE_STAGE_EDGES, 4),
            Err(ParallelClaimRejected { claimed: 4, graph_width: 1 })
        );
    }

    #[test]
    fn conc10_independent_stages_widen_graph_but_cores_still_cap() {
        // Fan-out: Plan and Advance both depend only on FireInfluence; NavFieldSync joins them.
        let fan = [
            (ComputeStage::FireInfluence, ComputeStage::HeatDiffusionPlan),
            (ComputeStage::FireInfluence, ComputeStage::HeatDiffusionAdvance),
            (ComputeStage::HeatDiffusionPlan, ComputeStage::NavFieldSync),
            (ComputeStage::HeatDiffusionAdvance, ComputeStage::NavFieldSync),
        ];
        assert_eq!(compute_critical_path_len(&fan), Some(3));
        assert_eq!(compute_graph_parallel_width(&fan), Some(2));
        assert_eq!(compute_parallel_width_for_cores(&fan, 1), 1);
        assert_eq!(compute_parallel_width_for_cores(&fan, 8), 2);
        assert!(check_parallel_claim(&fan, 3).is_err());
    }

    #[test]
    fn conc10_cyclic_edges_are_rejected() {
        let cyc = [
            (ComputeStage::FireInfluence, ComputeStage::HeatDiffusionPlan),
            (ComputeStage::HeatDiffusionPlan, ComputeStage::FireInfluence),
        ];
        assert_eq!(compute_stage_levels(&cyc), None);
        assert_eq!(compute_critical_path_len(&cyc), None);
    }

    #[test]
    fn macro_band_skips_fire_influence_dispatch() {
        let mut graph = ComputeDispatchGraph::default();
        let mut lod = WorldRepresentationFrame::default();
        lod.bands.global = WorldLodBand::Macro;
        lod.visibility.pathfinding_field = false;
        let mut fire = FireSimulationSnapshot::default();
        fire.chunk_heat.push(ChunkFireHeat {
            chunk: IVec2::ZERO,
            heat: 0.5,
            smoke: 0.1,
        });
        let lod_map = WorldLodMap::default();
        let policy = policy_for(&lod, &fire);
        let ctx = ComputeContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            agents: &AgentFrame::default(),
            navigation: &NavFieldFrame::default(),
            fire: &fire,
            climate: None,
            path_budget: 64,
        };
        graph.dispatch(&ctx);
        assert!(!graph.fire_influence.dispatch_active);
    }

    #[test]
    fn local_tactical_dispatches_when_fire_heat_present() {
        let mut graph = ComputeDispatchGraph::default();
        let lod = WorldRepresentationFrame::default();
        let mut fire = FireSimulationSnapshot::default();
        fire.chunk_heat.push(ChunkFireHeat::default());
        let lod_map = WorldLodMap::default();
        let policy = policy_for(&lod, &fire);
        let ctx = ComputeContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            agents: &AgentFrame::default(),
            navigation: &NavFieldFrame::default(),
            fire: &fire,
            climate: None,
            path_budget: 64,
        };
        graph.dispatch(&ctx);
        assert!(graph.fire_influence.dispatch_active);
        assert_eq!(graph.fire_influence.target_dispatch_hz, 60.0);
    }

    #[test]
    fn spatial_map_skips_macro_compute_chunks() {
        let mut graph = ComputeDispatchGraph::default();
        let lod = WorldRepresentationFrame::default();
        let mut fire = FireSimulationSnapshot::default();
        fire.chunk_heat.push(ChunkFireHeat {
            chunk: IVec2::new(0, 0),
            heat: 0.5,
            smoke: 0.0,
        });
        fire.chunk_heat.push(ChunkFireHeat {
            chunk: IVec2::new(3, 0),
            heat: 0.5,
            smoke: 0.0,
        });
        let lod_map = WorldLodMap {
            cells: vec![
                crate::gui::LodCell {
                    coord: IVec2::new(0, 0),
                    render_band: WorldLodBand::LocalTactical,
                    compute_band: WorldLodBand::LocalTactical,
                    importance: 1.0,
                },
                crate::gui::LodCell {
                    coord: IVec2::new(3, 0),
                    render_band: WorldLodBand::Macro,
                    compute_band: WorldLodBand::Macro,
                    importance: 0.0,
                },
            ],
            ..Default::default()
        };
        let policy = policy_for(&lod, &fire);
        let ctx = ComputeContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            agents: &AgentFrame::default(),
            navigation: &NavFieldFrame::default(),
            fire: &fire,
            climate: None,
            path_budget: 64,
        };
        graph.dispatch(&ctx);
        assert_eq!(graph.fire_influence.influence_chunk_count, 1);
        assert!(graph.fire_influence.dispatch_active);
    }
}
