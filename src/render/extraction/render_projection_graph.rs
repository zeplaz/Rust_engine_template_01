//! **Render projection graph** — CPU-side orchestrator for frame → GPU-shaped **visual** views.
//!
//! Sibling to [`crate::compute::ComputeDispatchGraph`]: both consume read-only frame snapshots and
//! [`crate::gui::RepresentationResult`] + [`crate::gui::WorldLodMap`]; neither queries ECS inside evaluation and neither writes the other's buffers.
//!
//! GPU **wgpu** buffer writes stay in the render schedule; they read **extracted** [`RenderProjectionGraph`].

use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use std::collections::HashMap;

use crate::gui::{RepresentationResult, WorldLodBand, WorldLodMap, WorldRepresentationFrame};

use crate::render::gpu_buffer_registry::{
    BufferId, ECOLOGY_OVERLAY_BUFFER, FIRE_VISUAL_INSTANCES_BUFFER, LOGISTICS_OVERLAY_BUFFER,
};
use crate::render::fx_burst_request::{collect_burst_hints_from_fire_visual, FxParticleBurstRequest};
use crate::render::{EcologyVisualSnapshot, LogisticsVisualSnapshot, Stage5ReadinessProfile};
use crate::gui::ViewManager;
use crate::render::view_fire_projection::fire_frame_for_projection_graph;
use crate::render::view_runtime::PerViewRepresentationPolicy;
use crate::render::FireVisualFramesByView;
use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualFrame, FireVisualGpuInstance};
use crate::systems::sim_control::SimStepStamp;

/// Max fire instance rows in the fire projection when band is [`WorldLodBand::Operational`].
pub const CLUSTERED_FIRE_INSTANCE_CAP: usize = 48;

/// Shared read-only inputs for projection nodes (extend with atmosphere / overlay later).
pub struct RenderProjectionContext<'a> {
    pub policy: &'a RepresentationResult,
    pub lod: &'a WorldRepresentationFrame,
    pub lod_map: &'a WorldLodMap,
    pub fire: &'a FireVisualFrame,
    pub logistics: &'a LogisticsVisualSnapshot,
    pub ecology: &'a EcologyVisualSnapshot,
    pub committed_stamp: SimStepStamp,
}

fn cap_domain_rows(snapshot_rows: u32, policy_cap: usize) -> u32 {
    let cap = policy_cap.min(u32::MAX as usize) as u32;
    snapshot_rows.min(cap)
}

pub trait ProjectionNodeTrait {
    fn evaluate(&mut self, ctx: &RenderProjectionContext<'_>);
}

fn bin_merge_chunk_heat(rows: &[ChunkFireHeat], bin: i32) -> Vec<ChunkFireHeat> {
    debug_assert!(bin >= 1);
    let mut m: HashMap<(i32, i32), ChunkFireHeat> = HashMap::new();
    for h in rows {
        let k = (h.chunk.x.div_euclid(bin), h.chunk.y.div_euclid(bin));
        m.entry(k)
            .and_modify(|e| {
                e.heat = e.heat.max(h.heat);
                e.smoke = e.smoke.max(h.smoke);
            })
            .or_insert(ChunkFireHeat {
                chunk: IVec2::new(k.0 * bin, k.1 * bin),
                heat: h.heat,
                smoke: h.smoke,
            });
    }
    m.into_values().collect()
}

fn top_heat_instances(src: &[FireVisualGpuInstance], cap: usize) -> Vec<FireVisualGpuInstance> {
    let mut v: Vec<FireVisualGpuInstance> = src.to_vec();
    v.sort_by(|a, b| {
        b.heat()
            .partial_cmp(&a.heat())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    v.into_iter().take(cap).collect()
}

fn project_fire_instances(
    frame: &FireVisualFrame,
    policy: &RepresentationResult,
) -> Vec<FireVisualGpuInstance> {
    if !policy.extract_plan.fire_instances {
        return Vec::new();
    }
    let mut out: Vec<FireVisualGpuInstance> = frame.instances.to_vec();
    let cap = policy
        .extract_plan
        .fire_instance_cap
        .min(CLUSTERED_FIRE_INSTANCE_CAP);
    if cap != usize::MAX && out.len() > cap {
        out = top_heat_instances(&out, cap);
    }
    out
}

fn project_chunk_heat(
    frame: &FireVisualFrame,
    policy: &RepresentationResult,
) -> Vec<ChunkFireHeat> {
    if !policy.visibility.fire_chunk_heat {
        return Vec::new();
    }
    let bin = policy.overlay_policy.chunk_heat_bin.max(1);
    if bin <= 1 {
        return frame.chunk_heat.clone();
    }
    bin_merge_chunk_heat(&frame.chunk_heat, bin)
}

/// Fire domain: LOD-shaped instance list + chunk heat for GPU / uniforms (not sim truth).
#[derive(Debug, Clone)]
pub struct FireProjectionNode {
    pub buffer_id: BufferId,
    /// Sim step copied from [`WorldRepresentationFrame::sim_step_stamp`] when evaluated (registry write frame id).
    pub snapshot_stamp: u64,
    pub instance_buffer: Vec<FireVisualGpuInstance>,
    pub chunk_heat: Vec<ChunkFireHeat>,
    pub burst_hints: Vec<FxParticleBurstRequest>,
    pub lod: WorldLodBand,
    /// LOD-shaped allocation ceiling for the fire instance GPU buffer.
    pub gpu_instance_capacity: usize,
}

impl Default for FireProjectionNode {
    fn default() -> Self {
        Self {
            buffer_id: FIRE_VISUAL_INSTANCES_BUFFER,
            snapshot_stamp: 0,
            instance_buffer: Vec::new(),
            chunk_heat: Vec::new(),
            burst_hints: Vec::new(),
            lod: WorldLodBand::LocalTactical,
            gpu_instance_capacity: usize::MAX,
        }
    }
}

fn fire_projection_stamp_lag(fire: &FireVisualFrame, committed: SimStepStamp) -> u64 {
    if fire.stamp.tick >= committed.tick {
        fire.stamp.tick.saturating_sub(committed.tick)
    } else {
        u64::MAX
    }
}

/// F2-PR-1 — witness hook: extract stamp matches committed fence, or ≤1-tick hold with retained buffer.
#[must_use]
pub fn fire_projection_stamp_aligned(fire: &FireVisualFrame, committed: SimStepStamp) -> bool {
    if fire.stamp == committed {
        return true;
    }
    fire_projection_stamp_lag(fire, committed) <= 1
}

impl ProjectionNodeTrait for FireProjectionNode {
    fn evaluate(&mut self, ctx: &RenderProjectionContext<'_>) {
        if ctx.fire.stamp != ctx.committed_stamp {
            // MAP-BLINK-001 / VR-05: fence can trail fire extract by one tick — do not zero graph.
            let lag = fire_projection_stamp_lag(ctx.fire, ctx.committed_stamp);
            if lag <= 1
                && (!self.instance_buffer.is_empty() || !self.chunk_heat.is_empty())
            {
                self.snapshot_stamp = ctx.committed_stamp.tick;
                return;
            }
            self.instance_buffer.clear();
            self.chunk_heat.clear();
            self.burst_hints.clear();
            self.gpu_instance_capacity = 0;
            self.snapshot_stamp = ctx.committed_stamp.tick;
            return;
        }
        self.gpu_instance_capacity = ctx.policy.gpu_budget.fire_instance_cap;
        self.instance_buffer = project_fire_instances(ctx.fire, ctx.policy);
        self.chunk_heat = project_chunk_heat(ctx.fire, ctx.policy);
        self.burst_hints = if ctx.policy.extract_plan.fire_instances {
            collect_burst_hints_from_fire_visual(&self.instance_buffer, 0.9)
        } else {
            Vec::new()
        };
        self.lod = ctx.policy.world_lod_band;
        self.snapshot_stamp = ctx.committed_stamp.tick;
    }
}

/// Logistics overlay projection (corridor revision → GPU row count).
#[derive(Debug, Clone)]
pub struct LogisticsProjectionNode {
    pub buffer_id: BufferId,
    pub snapshot_stamp: u64,
    pub active_rows: u32,
}

impl Default for LogisticsProjectionNode {
    fn default() -> Self {
        Self {
            buffer_id: LOGISTICS_OVERLAY_BUFFER,
            snapshot_stamp: 0,
            active_rows: 0,
        }
    }
}

impl ProjectionNodeTrait for LogisticsProjectionNode {
    fn evaluate(&mut self, ctx: &RenderProjectionContext<'_>) {
        self.buffer_id = LOGISTICS_OVERLAY_BUFFER;
        self.snapshot_stamp = ctx.committed_stamp.tick;
        let snapshot_rows = if ctx.logistics.stamp == ctx.committed_stamp {
            ctx.logistics.active_overlay_rows
        } else {
            0
        };
        self.active_rows = if ctx.policy.overlay_matrix.logistics {
            cap_domain_rows(
                snapshot_rows,
                ctx.policy.gpu_budget.reserved_capacity.max(1) as usize,
            )
        } else {
            0
        };
    }
}

/// Ecology overlay projection (climate aggregate → GPU row count).
#[derive(Debug, Clone)]
pub struct EcologyProjectionNode {
    pub buffer_id: BufferId,
    pub snapshot_stamp: u64,
    pub active_rows: u32,
}

impl Default for EcologyProjectionNode {
    fn default() -> Self {
        Self {
            buffer_id: ECOLOGY_OVERLAY_BUFFER,
            snapshot_stamp: 0,
            active_rows: 0,
        }
    }
}

impl ProjectionNodeTrait for EcologyProjectionNode {
    fn evaluate(&mut self, ctx: &RenderProjectionContext<'_>) {
        self.buffer_id = ECOLOGY_OVERLAY_BUFFER;
        self.snapshot_stamp = ctx.committed_stamp.tick;
        let snapshot_rows = if ctx.ecology.stamp == ctx.committed_stamp {
            ctx.ecology.ecology_chunk_count
        } else {
            0
        };
        self.active_rows = if ctx.policy.overlay_policy.fire_heat {
            cap_domain_rows(snapshot_rows, ctx.policy.gpu_budget.fire_instance_cap)
        } else {
            0
        };
    }
}

/// Root graph resource: orchestrates all projection nodes.
#[derive(Resource, Debug, Clone, ExtractResource)]
pub struct RenderProjectionGraph {
    pub fire: FireProjectionNode,
    pub logistics: LogisticsProjectionNode,
    pub ecology: EcologyProjectionNode,
}

impl Default for RenderProjectionGraph {
    fn default() -> Self {
        Self {
            fire: FireProjectionNode::default(),
            logistics: LogisticsProjectionNode::default(),
            ecology: EcologyProjectionNode::default(),
        }
    }
}

impl ProjectionNodeTrait for RenderProjectionGraph {
    fn evaluate(&mut self, ctx: &RenderProjectionContext<'_>) {
        self.fire.evaluate(ctx);
        self.logistics.evaluate(ctx);
        self.ecology.evaluate(ctx);
    }
}

/// Set by [`run_render_projection_graph`] when evaluate is skipped (stamp-only advance).
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct ProjectionGraphFrameCoherence {
    pub evaluate_skipped: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ProjectionGraphInputFingerprint {
    policy_tick: u64,
    lod_tick: u64,
    fire_instances: usize,
    fire_heat_cells: usize,
    logistics_rows: u32,
    ecology_rows: u32,
    lod_map_cells: usize,
}

impl ProjectionGraphInputFingerprint {
    #[must_use]
    fn from_ctx(ctx: &RenderProjectionContext<'_>) -> Self {
        Self {
            policy_tick: ctx.policy.stamp.tick,
            lod_tick: ctx.lod.sim_step_stamp.tick,
            fire_instances: ctx.fire.instances.len(),
            fire_heat_cells: ctx.fire.chunk_heat.len(),
            logistics_rows: ctx.logistics.edge_rows.len() as u32,
            ecology_rows: ctx.ecology.chunk_rows.len() as u32,
            lod_map_cells: ctx.lod_map.cells.len(),
        }
    }

    #[must_use]
    fn same_except_ticks(self, other: Self) -> bool {
        Self {
            policy_tick: other.policy_tick,
            lod_tick: other.lod_tick,
            ..self
        } == other
    }
}

/// Stamp-only advance when graph evaluate is skipped — keeps VT-4 particle projection aligned with fence.
pub(crate) fn overlay_projection_idle(
    policy: &crate::gui::RepresentationResult,
    fire: &crate::render::FireVisualFrame,
) -> bool {
    let m = &policy.overlay_matrix;
    !m.fire_heat
        && !m.logistics
        && !m.ecology
        && !m.construction_phase
        && fire.instances.is_empty()
        && fire.chunk_heat.is_empty()
}

/// Stamp-only advance when graph evaluate is skipped — keeps VT-4 particle projection aligned with fence.
fn advance_projection_graph_snapshot_stamps(
    graph: &mut RenderProjectionGraph,
    committed: SimStepStamp,
) {
    let tick = committed.tick;
    graph.fire.snapshot_stamp = tick;
    graph.logistics.snapshot_stamp = tick;
    graph.ecology.snapshot_stamp = tick;
}

/// Single-line snapshot for live readiness: confirms **fire → logistics → ecology** slots
/// on the resource after `run_render_projection_graph` (same evaluate order as the graph).
#[must_use]
pub fn projection_graph_build_signature(graph: &RenderProjectionGraph) -> String {
    format!(
        "order=fire+logistics+ecology fire_inst={} fire_heat={} log_rows={} eco_rows={}",
        graph.fire.instance_buffer.len(),
        graph.fire.chunk_heat.len(),
        graph.logistics.active_rows,
        graph.ecology.active_rows,
    )
}

#[must_use]
pub fn projection_graph_runtime_order_snapshot(graph: &RenderProjectionGraph) -> String {
    format!(
        "{} fire_snap={} log_snap={} eco_snap={}",
        projection_graph_build_signature(graph),
        graph.fire.snapshot_stamp,
        graph.logistics.snapshot_stamp,
        graph.ecology.snapshot_stamp,
    )
}

/// F2-PR-2 — headless tactical fixture: non-empty `fire.instance_buffer` via `project_fire_instances`.
#[must_use]
pub fn f2_tactical_fire_projection_fixture() -> RenderProjectionGraph {
    use bevy::math::{IVec2, Vec4};
    use crate::gui::{
        build_representation_inputs, build_representation_result, resolution_for_band, LodZoneRegistry,
        VisualBudgetSettings, VisualCadence, WorldLodBand, WorldLodMap, WorldRepresentationFrame,
    };
    use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualFrame, FireVisualGpuInstance};
    use crate::systems::sim_control::SimStepStamp;

    let stamp = SimStepStamp::new(9, 0);
    let mut fire_frame = FireVisualFrame::default();
    fire_frame.stamp = stamp;
    let mut row = FireVisualGpuInstance::default();
    row.chunk_xy_heat_lum = Vec4::new(2.0, 3.0, 0.85, 1.0);
    row.world_xyz_radius = Vec4::new(160.0, 224.0, 0.0, 32.0);
    row.smoke_ember_vis_priority = Vec4::new(0.12, 0.45, 0.0, 1.0);
    fire_frame.instances.push(row);
    fire_frame.chunk_heat.push(ChunkFireHeat {
        chunk: IVec2::new(2, 3),
        heat: 0.85,
        smoke: 0.3,
    });

    let mut lod = WorldRepresentationFrame::default();
    lod.bands.global = WorldLodBand::LocalTactical;
    lod.resolution = resolution_for_band(WorldLodBand::LocalTactical);
    let lod_map = WorldLodMap::default();
    let policy_inputs = build_representation_inputs(
        &crate::gui::CameraVisualState::default(),
        &LodZoneRegistry::default(),
        &VisualBudgetSettings::default(),
        &VisualCadence::from(&VisualBudgetSettings::default()),
        stamp,
    );
    let policy = build_representation_result(&lod, &policy_inputs);

    let logistics = LogisticsVisualSnapshot::default();
    let ecology = EcologyVisualSnapshot::default();
    let ctx = RenderProjectionContext {
        policy: &policy,
        lod: &lod,
        lod_map: &lod_map,
        fire: &fire_frame,
        logistics: &logistics,
        ecology: &ecology,
        committed_stamp: stamp,
    };
    let mut graph = RenderProjectionGraph::default();
    graph.evaluate(&ctx);
    graph
}

/// Single **Update** entry point: builds [`RenderProjectionContext`] and runs the graph (no per-domain projection systems).
pub fn run_render_projection_graph(
    policy: Res<RepresentationResult>,
    lod: Res<WorldRepresentationFrame>,
    lod_map: Res<WorldLodMap>,
    fire_by_view: Res<FireVisualFramesByView>,
    manager: Option<Res<ViewManager>>,
    per_view_policy: Res<PerViewRepresentationPolicy>,
    logistics: Res<LogisticsVisualSnapshot>,
    ecology: Res<EcologyVisualSnapshot>,
    fence: Res<crate::render::CommittedVisualSnapshotFence>,
    profile: Res<Stage5ReadinessProfile>,
    mut graph: ResMut<RenderProjectionGraph>,
    mut coherence: ResMut<ProjectionGraphFrameCoherence>,
    mut perf: Option<ResMut<crate::render::FramePerf>>,
    mut last_fingerprint: Local<Option<ProjectionGraphInputFingerprint>>,
    mut last_build_log: Local<Option<String>>,
) {
    coherence.evaluate_skipped = false;
    let per_view = per_view_policy.as_ref();
    let fire = fire_frame_for_projection_graph(
        fire_by_view.as_ref(),
        manager.as_deref(),
        per_view,
        policy.as_ref(),
    );
    if overlay_projection_idle(policy.as_ref(), &fire) {
        advance_projection_graph_snapshot_stamps(&mut graph, fence.fire);
        coherence.evaluate_skipped = true;
        return;
    }
    let ctx = RenderProjectionContext {
        policy: &policy,
        lod: &lod,
        lod_map: &lod_map,
        fire: &fire,
        logistics: &logistics,
        ecology: &ecology,
        committed_stamp: fence.fire,
    };
    let fingerprint = ProjectionGraphInputFingerprint::from_ctx(&ctx);
    if let Some(prev) = *last_fingerprint {
        if prev == fingerprint {
            advance_projection_graph_snapshot_stamps(&mut graph, fence.fire);
            coherence.evaluate_skipped = true;
            return;
        }
        if prev.same_except_ticks(fingerprint) {
            *last_fingerprint = Some(fingerprint);
            advance_projection_graph_snapshot_stamps(&mut graph, fence.fire);
            coherence.evaluate_skipped = true;
            return;
        }
    }
    *last_fingerprint = Some(fingerprint);
    let t0 = std::time::Instant::now();
    graph.evaluate(&ctx);
    if let Some(perf) = perf.as_mut() {
        crate::render::record_frame_perf_ms(
            perf,
            t0.elapsed().as_secs_f32() * 1000.0,
            crate::render::FramePerfSlot::ProjectionGraph,
        );
    }
    if *profile == Stage5ReadinessProfile::FULL_APP
        && (crate::render::frame_perf_verbose()
            || crate::render::frame_perf::stage5_readiness_live_verbose())
    {
        let signature = projection_graph_build_signature(&graph);
        if last_build_log.as_deref() != Some(signature.as_str()) {
            info!(
                target: "stage5_readiness::live",
                "READINESS_PROJECTION_GRAPH_BUILD dom=3 tick={} {}",
                fence.fire.tick,
                projection_graph_runtime_order_snapshot(&graph)
            );
            *last_build_log = Some(signature);
        }
    }
}

#[must_use]
pub fn spatial_distribution_stats(rows: &[FireVisualGpuInstance]) -> (usize, f32, f32) {
    if rows.is_empty() {
        return (0, 0.0, 0.0);
    }
    let mut chunks = std::collections::HashSet::new();
    let mut sum_dist = 0.0f32;
    let mut sum_sq = 0.0f32;
    for row in rows {
        let xy = row.chunk_grid_xy();
        chunks.insert((xy.x.floor() as i32, xy.y.floor() as i32));
        let dist = xy.length();
        sum_dist += dist;
        sum_sq += dist * dist;
    }
    let n = rows.len() as f32;
    let mean = sum_dist / n;
    let variance = (sum_sq / n) - mean * mean;
    (chunks.len(), mean, variance.max(0.0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{
        build_representation_inputs, build_representation_result, LodZoneRegistry,
        VisualBudgetSettings, VisualCadence, WorldLodBands,
    };
    use crate::render::{EcologyVisualSnapshot, LogisticsVisualSnapshot};

    fn sample_instance(chunk: IVec2, heat: f32) -> FireVisualGpuInstance {
        let mut row = FireVisualGpuInstance::default();
        row.chunk_xy_heat_lum = Vec4::new(chunk.x as f32, chunk.y as f32, heat, 1.0);
        row
    }

    #[test]
    fn spatial_stats_detect_multi_chunk_distribution() {
        let mut a = FireVisualGpuInstance::default();
        a.chunk_xy_heat_lum = Vec4::new(0.0, 0.0, 0.5, 1.0);
        let mut b = FireVisualGpuInstance::default();
        b.chunk_xy_heat_lum = Vec4::new(8.0, 4.0, 0.5, 1.0);
        let (occupied, mean, variance) = spatial_distribution_stats(&[a, b]);
        assert!(occupied > 1);
        assert!(mean > 1.0);
        assert!(variance > 0.0);
    }

    #[test]
    fn strategic_policy_thins_fire_instances_in_projection() {
        let mut frame = FireVisualFrame::default();
        frame.instances.push(sample_instance(IVec2::new(0, 0), 0.9));
        frame.instances.push(sample_instance(IVec2::new(5, 0), 0.8));

        let lod = WorldRepresentationFrame {
            bands: WorldLodBands {
                global: WorldLodBand::Strategic,
            },
            visibility: crate::gui::visibility_for_band(WorldLodBand::Strategic),
            resolution: crate::gui::resolution_for_band(WorldLodBand::Strategic),
            ..Default::default()
        };

        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            frame.stamp,
        );
        let policy = build_representation_result(&lod, &policy_inputs);

        let projected = project_fire_instances(&frame, &policy);
        assert!(!projected.is_empty());
        assert!(projected.len() <= policy.extract_plan.fire_instance_cap.max(1));
    }

    #[test]
    fn tactical_policy_keeps_instances_until_extract_cap() {
        let mut frame = FireVisualFrame::default();
        frame.instances.push(sample_instance(IVec2::new(0, 0), 0.9));
        frame.instances.push(sample_instance(IVec2::new(5, 0), 0.8));

        let lod = WorldRepresentationFrame {
            bands: WorldLodBands {
                global: WorldLodBand::LocalTactical,
            },
            ..Default::default()
        };

        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            frame.stamp,
        );
        let policy = build_representation_result(&lod, &policy_inputs);

        let projected = project_fire_instances(&frame, &policy);
        assert_eq!(projected.len(), 2);
    }

    #[test]
    fn committed_stamp_mismatch_clears_projection_and_burst_hints() {
        let mut frame = FireVisualFrame::default();
        frame.stamp = crate::systems::sim_control::SimStepStamp::new(2, 0);
        frame.instances.push(sample_instance(IVec2::new(0, 0), 0.95));

        let lod = WorldRepresentationFrame::default();
        let lod_map = WorldLodMap::default();
        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            frame.stamp,
        );
        let policy = build_representation_result(&lod, &policy_inputs);

        let mut graph = RenderProjectionGraph::default();
        let logistics = LogisticsVisualSnapshot::default();
        let ecology = EcologyVisualSnapshot::default();
        let ctx = RenderProjectionContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            fire: &frame,
            logistics: &logistics,
            ecology: &ecology,
            committed_stamp: crate::systems::sim_control::SimStepStamp::new(1, 0),
        };
        graph.evaluate(&ctx);
        assert!(graph.fire.instance_buffer.is_empty());
        assert!(graph.fire.burst_hints.is_empty());
    }

    #[test]
    fn committed_stamp_one_tick_lag_holds_prior_projection() {
        let mut frame = FireVisualFrame::default();
        frame.stamp = crate::systems::sim_control::SimStepStamp::new(2, 0);
        frame.instances.push(sample_instance(IVec2::new(0, 0), 0.95));

        let lod = WorldRepresentationFrame::default();
        let lod_map = WorldLodMap::default();
        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            frame.stamp,
        );
        let policy = build_representation_result(&lod, &policy_inputs);

        let mut graph = RenderProjectionGraph::default();
        graph.fire.instance_buffer.push(sample_instance(IVec2::new(1, 1), 0.8));
        graph.fire.chunk_heat.push(ChunkFireHeat {
            chunk: IVec2::new(1, 1),
            heat: 0.8,
            smoke: 0.1,
        });

        let logistics = LogisticsVisualSnapshot::default();
        let ecology = EcologyVisualSnapshot::default();
        let ctx = RenderProjectionContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            fire: &frame,
            logistics: &logistics,
            ecology: &ecology,
            committed_stamp: crate::systems::sim_control::SimStepStamp::new(1, 0),
        };
        graph.evaluate(&ctx);
        assert_eq!(graph.fire.instance_buffer.len(), 1);
        assert_eq!(graph.fire.chunk_heat.len(), 1);
    }

    #[test]
    fn burst_hints_follow_projected_instances_not_raw_frame() {
        let mut frame = FireVisualFrame::default();
        frame.instances.push(sample_instance(IVec2::new(0, 0), 0.95));
        frame.instances.push(sample_instance(IVec2::new(8, 0), 0.95));

        let lod = WorldRepresentationFrame {
            bands: WorldLodBands {
                global: WorldLodBand::Strategic,
            },
            visibility: crate::gui::visibility_for_band(WorldLodBand::Strategic),
            resolution: crate::gui::resolution_for_band(WorldLodBand::Strategic),
            ..Default::default()
        };
        let lod_map = WorldLodMap::default();
        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            frame.stamp,
        );
        let policy = build_representation_result(&lod, &policy_inputs);

        let mut graph = RenderProjectionGraph::default();
        let logistics = LogisticsVisualSnapshot::default();
        let ecology = EcologyVisualSnapshot::default();
        let ctx = RenderProjectionContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            fire: &frame,
            logistics: &logistics,
            ecology: &ecology,
            committed_stamp: frame.stamp,
        };
        graph.evaluate(&ctx);
        assert_eq!(graph.fire.instance_buffer.len(), 2);
        assert!(
            !graph.fire.burst_hints.is_empty(),
            "burst hints should follow non-empty projected instances"
        );
    }

    #[test]
    fn tactical_projection_fills_fire_instance_buffer() {
        let graph = super::f2_tactical_fire_projection_fixture();
        assert!(
            !graph.fire.instance_buffer.is_empty(),
            "F2-PR-2 tactical fixture must populate instance_buffer"
        );
        assert_eq!(graph.fire.snapshot_stamp, 9);
    }

    #[test]
    fn fire_projection_stamp_aligned_allows_one_tick_lag() {
        let mut frame = FireVisualFrame::default();
        frame.stamp = SimStepStamp::new(5, 0);
        let committed = SimStepStamp::new(4, 0);
        assert!(fire_projection_stamp_aligned(&frame, committed));
        frame.stamp = SimStepStamp::new(7, 0);
        assert!(!fire_projection_stamp_aligned(&frame, committed));
        assert!(fire_projection_stamp_aligned(
            &FireVisualFrame {
                stamp: committed,
                ..Default::default()
            },
            committed,
        ));
    }

    #[test]
    fn committed_stamp_mismatch_zeros_projection_rows() {
        let mut frame = FireVisualFrame::default();
        frame.stamp = SimStepStamp::new(5, 0);
        frame.instances.push(sample_instance(IVec2::ZERO, 0.9));
        frame.chunk_heat.push(ChunkFireHeat {
            chunk: IVec2::ZERO,
            heat: 0.9,
            smoke: 0.0,
        });
        let lod = WorldRepresentationFrame::default();
        let lod_map = WorldLodMap::default();
        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            frame.stamp,
        );
        let policy = build_representation_result(&lod, &policy_inputs);
        let mut graph = RenderProjectionGraph::default();
        let logistics = LogisticsVisualSnapshot::default();
        let ecology = EcologyVisualSnapshot::default();
        let ctx = RenderProjectionContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            fire: &frame,
            logistics: &logistics,
            ecology: &ecology,
            committed_stamp: SimStepStamp::new(4, 0),
        };
        graph.evaluate(&ctx);
        assert!(graph.fire.instance_buffer.is_empty());
        assert!(graph.fire.chunk_heat.is_empty());
    }

    #[test]
    fn strategic_band_caps_domain_ecology_rows_by_fire_budget() {
        let stamp = SimStepStamp::new(3, 0);
        let logistics = LogisticsVisualSnapshot {
            stamp,
            active_overlay_rows: 8,
            edge_rows: vec![(1, 1.0); 8],
            corridor_revision: 8,
        };
        let ecology = EcologyVisualSnapshot {
            stamp,
            ecology_chunk_count: 6,
            chunk_rows: vec![Vec4::ONE; 6],
            ..Default::default()
        };
        let fire = FireVisualFrame {
            stamp,
            ..Default::default()
        };
        let lod = WorldRepresentationFrame {
            bands: WorldLodBands {
                global: WorldLodBand::Strategic,
            },
            visibility: crate::gui::visibility_for_band(WorldLodBand::Strategic),
            resolution: crate::gui::resolution_for_band(WorldLodBand::Strategic),
            ..Default::default()
        };
        let lod_map = WorldLodMap::default();
        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            stamp,
        );
        let policy = build_representation_result(&lod, &policy_inputs);
        let mut graph = RenderProjectionGraph::default();
        let ctx = RenderProjectionContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            fire: &fire,
            logistics: &logistics,
            ecology: &ecology,
            committed_stamp: stamp,
        };
        graph.evaluate(&ctx);
        assert_eq!(graph.logistics.active_rows, 0);
        assert_eq!(
            graph.ecology.active_rows,
            6,
            "ecology rows are capped by the same GPU budget ceiling as fire projection"
        );
    }

    #[test]
    fn advance_snapshot_stamps_on_evaluate_skip_matches_fence_tick() {
        let mut graph = RenderProjectionGraph::default();
        graph.fire.snapshot_stamp = 37;
        graph.logistics.snapshot_stamp = 37;
        graph.ecology.snapshot_stamp = 37;
        advance_projection_graph_snapshot_stamps(&mut graph, SimStepStamp::new(44, 9_000));
        assert_eq!(graph.fire.snapshot_stamp, 44);
        assert_eq!(graph.logistics.snapshot_stamp, 44);
        assert_eq!(graph.ecology.snapshot_stamp, 44);
    }

    #[test]
    fn overlay_projection_idle_when_no_overlays_and_no_fire() {
        let stamp = SimStepStamp::new(1, 0);
        let lod = WorldRepresentationFrame::default();
        let policy_inputs = build_representation_inputs(
            &crate::gui::CameraVisualState::default(),
            &LodZoneRegistry::default(),
            &VisualBudgetSettings::default(),
            &VisualCadence::from(&VisualBudgetSettings::default()),
            stamp,
        );
        let mut policy = build_representation_result(&lod, &policy_inputs);
        policy.overlay_matrix.fire_heat = false;
        policy.overlay_matrix.logistics = false;
        policy.overlay_matrix.ecology = false;
        policy.overlay_matrix.construction_phase = false;
        let fire = FireVisualFrame::default();
        assert!(super::overlay_projection_idle(&policy, &fire));
    }
}
