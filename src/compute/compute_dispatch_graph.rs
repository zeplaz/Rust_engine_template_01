//! **Compute dispatch graph** — CPU-side orchestrator for snapshot → GPU compute workloads (logic offload).
//!
//! Sibling to [`crate::render::extraction::RenderProjectionGraph`]: both read the same frame layer and
//! [`crate::gui::WorldRepresentationFrame`]; neither queries ECS inside node dispatch and neither writes render buffers.
//!
//! v1: [`FireInfluenceDispatchNode`] policy + [`super::heat_diffusion::HeatDiffusionDispatchNode`] kernel.

use bevy::prelude::*;

use crate::gui::{RepresentationResult, WorldLodBand, WorldLodMap, WorldRepresentationFrame};
use crate::render::sim_visual_extract::FireVisualFrame;

use super::frame_snapshots::{AgentFrame, NavFieldFrame};
use super::heat_diffusion::{
    advance_heat_diffusion_field, sync_nav_field_from_heat_diffusion, HeatDiffusionDispatchNode,
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

/// Shared read-only inputs for compute nodes (extend as snapshot producers appear).
pub struct ComputeContext<'a> {
    pub policy: &'a RepresentationResult,
    pub lod: &'a WorldRepresentationFrame,
    pub lod_map: &'a WorldLodMap,
    pub agents: &'a AgentFrame,
    pub navigation: &'a NavFieldFrame,
    pub fire: &'a FireVisualFrame,
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
    fire: Res<FireVisualFrame>,
    mut agents: ResMut<AgentFrame>,
    mut navigation: ResMut<NavFieldFrame>,
    mut cadence: ResMut<ComputeDispatchCadence>,
    mut graph: ResMut<ComputeDispatchGraph>,
    mut heat_field: ResMut<HeatDiffusionFieldBuffers>,
) {
    cadence.accumulator_secs += time.delta_secs();
    let min_interval = 1.0 / policy.compute_budget.dispatch_hz.max(1e-3);
    if cadence.accumulator_secs < min_interval {
        return;
    }
    cadence.accumulator_secs = 0.0;

    agents.stamp = fire.stamp;
    navigation.stamp = fire.stamp;

    let ctx = ComputeContext {
        policy: &policy,
        lod: &lod,
        lod_map: &lod_map,
        agents: &agents,
        navigation: &navigation,
        fire: &fire,
    };
    graph.dispatch(&ctx);
    advance_heat_diffusion_field(
        &mut heat_field,
        &ctx,
        graph.fire_influence.dispatch_active,
    );
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
        app.init_resource::<AgentFrame>()
            .init_resource::<NavFieldFrame>()
            .init_resource::<ComputeDispatchCadence>()
            .init_resource::<ComputeDispatchGraph>()
            .init_resource::<HeatDiffusionFieldBuffers>();
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
    use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualFrame};

    fn policy_for(lod: &WorldRepresentationFrame, fire: &FireVisualFrame) -> RepresentationResult {
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
    fn macro_band_skips_fire_influence_dispatch() {
        let mut graph = ComputeDispatchGraph::default();
        let mut lod = WorldRepresentationFrame::default();
        lod.bands.global = WorldLodBand::Macro;
        lod.visibility.pathfinding_field = false;
        let mut fire = FireVisualFrame::default();
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
        };
        graph.dispatch(&ctx);
        assert!(!graph.fire_influence.dispatch_active);
    }

    #[test]
    fn local_tactical_dispatches_when_fire_heat_present() {
        let mut graph = ComputeDispatchGraph::default();
        let lod = WorldRepresentationFrame::default();
        let mut fire = FireVisualFrame::default();
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
        };
        graph.dispatch(&ctx);
        assert!(graph.fire_influence.dispatch_active);
        assert_eq!(graph.fire_influence.target_dispatch_hz, 60.0);
    }

    #[test]
    fn spatial_map_skips_macro_compute_chunks() {
        let mut graph = ComputeDispatchGraph::default();
        let lod = WorldRepresentationFrame::default();
        let mut fire = FireVisualFrame::default();
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
        };
        graph.dispatch(&ctx);
        assert_eq!(graph.fire_influence.influence_chunk_count, 1);
        assert!(graph.fire_influence.dispatch_active);
    }
}
