//! Standard domain projection envelope for representation merge (post–Stage 5).

use bevy::prelude::*;

use crate::gui::RepresentationResult;
use crate::render::extraction::RenderProjectionGraph;
use crate::render::{tactical_fire_visual, FireVisualFramesByView};
use crate::render::sim_visual_extract::FireVisualFrame;
use crate::systems::atmosphere::AtmospherePartialWriteMetrics;
use crate::systems::sim_control::SimStepStamp;

/// Simulation domain ids for unified projection merge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DomainProjectionId {
    Fire,
    Weather,
    Ecology,
    Logistics,
    Factions,
    Traffic,
    Economy,
    Combat,
    Sound,
    Pollution,
}

/// Per-domain GPU/overlay row counts after committed snapshot + policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DomainProjectionSlice {
    pub active_rows: u32,
    pub reserved_rows: u32,
    pub upload_bytes: u64,
}

/// Merged domain projection snapshot consumed by the representation resolver.
#[derive(Resource, Debug, Clone, Default, PartialEq)]
pub struct DomainProjectionFrame {
    pub stamp: SimStepStamp,
    pub fire: DomainProjectionSlice,
    pub weather: DomainProjectionSlice,
    pub ecology: DomainProjectionSlice,
    pub logistics: DomainProjectionSlice,
}

impl DomainProjectionFrame {
    #[must_use]
    pub fn total_active_rows(&self) -> u32 {
        self.fire
            .active_rows
            .saturating_add(self.weather.active_rows)
            .saturating_add(self.ecology.active_rows)
            .saturating_add(self.logistics.active_rows)
    }
}

#[must_use]
pub fn build_domain_projection_frame(
    fire: &FireVisualFrame,
    policy: &RepresentationResult,
    graph: &RenderProjectionGraph,
    partial_metrics: Option<&AtmospherePartialWriteMetrics>,
) -> DomainProjectionFrame {
    let fire_rows = graph.fire.instance_buffer.len() as u32;
    let fire_reserved = policy.gpu_budget.fire_instance_cap as u32;
    let fire_stride = std::mem::size_of::<crate::render::sim_visual_extract::FireVisualGpuInstance>() as u64;
    let weather_bytes = partial_metrics
        .map(|m| m.partial_upload_bytes.saturating_add(m.gpu_texture_upload_bytes))
        .unwrap_or(0);
    DomainProjectionFrame {
        stamp: fire.stamp,
        fire: DomainProjectionSlice {
            active_rows: fire_rows,
            reserved_rows: fire_reserved,
            upload_bytes: fire_rows as u64 * fire_stride,
        },
        weather: DomainProjectionSlice {
            active_rows: partial_metrics
                .map(|m| m.partial_upload_count)
                .unwrap_or(0),
            reserved_rows: partial_metrics
                .map(|m| m.dirty_region_count)
                .unwrap_or(0),
            upload_bytes: weather_bytes,
        },
        ecology: DomainProjectionSlice {
            active_rows: graph.ecology.active_rows,
            reserved_rows: policy.gpu_budget.reserved_capacity,
            upload_bytes: graph.ecology.active_rows as u64 * 16,
        },
        logistics: DomainProjectionSlice {
            active_rows: graph.logistics.active_rows,
            reserved_rows: policy.gpu_budget.reserved_capacity,
            upload_bytes: graph.logistics.active_rows as u64 * 16,
        },
    }
}

pub fn publish_domain_projection_frame(
    fire_by_view: Res<FireVisualFramesByView>,
    policy: Res<RepresentationResult>,
    graph: Res<RenderProjectionGraph>,
    partial_metrics: Option<Res<AtmospherePartialWriteMetrics>>,
    mut frame: ResMut<DomainProjectionFrame>,
) {
    let fire = tactical_fire_visual(fire_by_view.as_ref());
    *frame = build_domain_projection_frame(
        fire,
        policy.as_ref(),
        graph.as_ref(),
        partial_metrics.as_deref(),
    );
}

pub fn merge_domain_projection_into_representation(
    projection: Res<DomainProjectionFrame>,
    mut policy: ResMut<RepresentationResult>,
    mut perf: Option<ResMut<crate::render::FramePerf>>,
) {
    let t0 = std::time::Instant::now();
    if projection.stamp != policy.stamp {
        return;
    }
    // Do not zero GPU caps when the projection graph is empty this frame — that disables
    // instanced fire for the rest of the frame before PostUpdate extract repopulates rows.
    if projection.fire.active_rows > 0 {
        let fire_cap = projection
            .fire
            .active_rows
            .min(policy.gpu_budget.fire_instance_cap as u32) as usize;
        policy.gpu_budget.fire_instance_cap = fire_cap;
        policy.gpu_budget.active_capacity = policy
            .gpu_budget
            .active_capacity
            .min(fire_cap as u32);
        let particle_cap = projection
            .fire
            .active_rows
            .min(policy.gpu_budget.particle_rows_cap as u32) as usize;
        policy.gpu_budget.particle_rows_cap = particle_cap;
    }
    if projection.ecology.active_rows > 0 {
        policy.overlay_policy.fire_heat = true;
    }
    if projection.logistics.active_rows > 0 {
        policy.visibility.pathfinding_field = policy.visibility.pathfinding_field
            || projection.logistics.active_rows > 0;
    }
    if let Some(perf) = perf.as_mut() {
        crate::render::record_frame_perf_ms(
            perf,
            t0.elapsed().as_secs_f32() * 1000.0,
            crate::render::FramePerfSlot::DomainMerge,
        );
    }
}

pub struct DomainProjectionFramePlugin;

impl Plugin for DomainProjectionFramePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DomainProjectionFrame>().add_systems(
            Update,
            (
                publish_domain_projection_frame,
                merge_domain_projection_into_representation,
            )
                .chain()
                .after(crate::render::extraction::FireVisualFrameSet::ProjectGpu)
                .after(crate::gui::WorldRepresentationSystemSet::ComputeFrame),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::{GpuBudgetPolicy, RepresentationResult};

    #[test]
    fn domain_projection_frame_totals_active_rows() {
        let fire = FireVisualFrame::default();
        let policy = RepresentationResult {
            gpu_budget: GpuBudgetPolicy {
                fire_instance_cap: 64,
                particle_rows_cap: 64,
                reserved_capacity: 64,
                active_capacity: 64,
            },
            ..Default::default()
        };
        let mut graph = RenderProjectionGraph::default();
        graph.fire.instance_buffer.push(Default::default());
        graph.logistics.active_rows = 2;
        graph.ecology.active_rows = 3;
        let frame = build_domain_projection_frame(&fire, &policy, &graph, None);
        assert_eq!(frame.total_active_rows(), 6);
    }

    #[test]
    fn merge_domain_projection_preserves_gpu_cap_when_fire_rows_empty() {
        use bevy::ecs::system::RunSystemOnce;
        use crate::systems::sim_control::SimStepStamp;

        let stamp = SimStepStamp::default();
        let mut world = World::new();
        world.insert_resource(RepresentationResult {
            stamp,
            gpu_budget: GpuBudgetPolicy {
                fire_instance_cap: 64,
                particle_rows_cap: 64,
                reserved_capacity: 64,
                active_capacity: 64,
            },
            ..Default::default()
        });
        world.insert_resource(DomainProjectionFrame {
            stamp,
            ..Default::default()
        });
        world
            .run_system_once(merge_domain_projection_into_representation)
            .expect("merge system");
        let policy = world.resource::<RepresentationResult>();
        assert_eq!(policy.gpu_budget.fire_instance_cap, 64);
        assert_eq!(policy.gpu_budget.particle_rows_cap, 64);
    }
}
