//! Stage-5 logistics / ecology overlay rows uploaded through [`crate::render::core::gpu_buffer_registry::GPUBufferRegistry`].

use bevy::math::Vec4;
use bevy::prelude::*;
use bevy::render::RenderApp;

use bevy::render::extract_resource::ExtractResource;
use bytemuck::{Pod, Zeroable};

use crate::gui::style::{
    blend_heatmap_linear, default_heatmap_ramp_linear, sample_heatmap_ramp_linear,
};
use crate::render::extraction::ecology_visual_snapshot::EcologyVisualSnapshot;
use crate::render::extraction::RenderProjectionGraph;
use crate::render::extraction::logistics_visual_snapshot::LogisticsVisualSnapshot;
use crate::render::CommittedVisualSnapshotFence;
use crate::systems::sim_control::SimStepStamp;

/// Packed corridor overlay row for [`crate::render::core::gpu_buffer_registry::LOGISTICS_OVERLAY_BUFFER`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct LogisticsOverlayGpuRow {
    pub corridor_xy_revision: Vec4,
}

/// Packed ecology overlay row for [`crate::render::core::gpu_buffer_registry::ECOLOGY_OVERLAY_BUFFER`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct EcologyOverlayGpuRow {
    pub ecology_xy_means: Vec4,
}

/// CPU-side domain overlay upload payload (projection → registry).
#[derive(Resource, Debug, Clone, ExtractResource, Default)]
#[extract_app(RenderApp)]
pub struct DomainOverlayGpuFrame {
    pub stamp: SimStepStamp,
    pub logistics_rows: Vec<LogisticsOverlayGpuRow>,
    pub ecology_rows: Vec<EcologyOverlayGpuRow>,
}

#[must_use]
pub fn project_logistics_overlay_rows(
    snapshot: &LogisticsVisualSnapshot,
    cap: usize,
) -> Vec<LogisticsOverlayGpuRow> {
    snapshot
        .edge_rows
        .iter()
        .take(cap)
        .enumerate()
        .map(|(i, &(edge, traffic))| {
            let haul = snapshot.haul_marker_t.get(i).copied().unwrap_or(0.0);
            LogisticsOverlayGpuRow {
                corridor_xy_revision: Vec4::new(
                    edge as f32,
                    traffic,
                    haul,
                    snapshot.corridor_revision as f32,
                ),
            }
        })
        .collect()
}

#[must_use]
pub fn project_ecology_overlay_rows(
    snapshot: &EcologyVisualSnapshot,
    cap: usize,
) -> Vec<EcologyOverlayGpuRow> {
    snapshot
        .chunk_rows
        .iter()
        .take(cap)
        .map(|row| EcologyOverlayGpuRow {
            ecology_xy_means: *row,
        })
        .collect()
}

/// Map a scalar field pair (base / overlay) through the shared heat ramp; blend in **linear** space (RND-20).
#[must_use]
pub fn blend_domain_overlay_scalar_linear(
    base_scalar: f32,
    overlay_scalar: f32,
    overlay_alpha: f32,
) -> Color {
    let ramp = default_heatmap_ramp_linear();
    blend_heatmap_linear(
        &ramp,
        base_scalar.clamp(0.0, 1.0),
        overlay_scalar.clamp(0.0, 1.0),
        overlay_alpha,
    )
}

/// Pack a scalar into the RGB leg of a domain row (alpha holds overlay strength).
#[must_use]
pub fn scalar_to_domain_overlay_rgb(scalar: f32) -> Vec3 {
    let ramp = default_heatmap_ramp_linear();
    let rgb = sample_heatmap_ramp_linear(&ramp, scalar.clamp(0.0, 1.0));
    Vec3::new(rgb[0], rgb[1], rgb[2])
}

pub fn emit_domain_overlay_frame_from_projection(
    graph: Res<RenderProjectionGraph>,
    logistics: Res<LogisticsVisualSnapshot>,
    ecology: Res<EcologyVisualSnapshot>,
    fence: Res<CommittedVisualSnapshotFence>,
    mut frame: ResMut<DomainOverlayGpuFrame>,
) {
    frame.stamp = fence.fire;
    if graph.logistics.active_rows > 0 && logistics.stamp == fence.fire {
        frame.logistics_rows = project_logistics_overlay_rows(
            &logistics,
            graph.logistics.active_rows as usize,
        );
    } else {
        frame.logistics_rows.clear();
    }
    if graph.ecology.active_rows > 0 && ecology.stamp == fence.fire {
        frame.ecology_rows =
            project_ecology_overlay_rows(&ecology, graph.ecology.active_rows as usize);
    } else {
        frame.ecology_rows.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::style::HEATMAP_RAMP_STOPS;
    use crate::render::extraction::RenderProjectionGraph;
    use crate::render::{EcologyVisualSnapshot, LogisticsVisualSnapshot};
    use crate::systems::sim_control::SimStepStamp;

    #[test]
    fn logistics_rows_pack_committed_edge_ids() {
        let snapshot = LogisticsVisualSnapshot {
            stamp: SimStepStamp::new(2, 0),
            corridor_revision: 2,
            active_overlay_rows: 2,
            edge_rows: vec![(7, 0.5), (9, 1.0)],
            haul_marker_t: vec![0.25, 0.0],
        };
        let rows = project_logistics_overlay_rows(&snapshot, 2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].corridor_xy_revision.x, 7.0);
        assert!((rows[0].corridor_xy_revision.y - 0.5).abs() < 1e-5);
        assert!((rows[0].corridor_xy_revision.z - 0.25).abs() < 1e-5);
    }

    #[test]
    fn domain_overlay_scalar_blend_uses_full_ramp() {
        assert_eq!(HEATMAP_RAMP_STOPS, 16);
        let cold = blend_domain_overlay_scalar_linear(0.0, 1.0, 0.0);
        let hot = blend_domain_overlay_scalar_linear(0.0, 1.0, 1.0);
        assert!(cold.to_srgba().red < hot.to_srgba().red);
    }

    #[test]
    fn stamp_mismatch_skips_domain_overlay_rows() {
        let mut graph = RenderProjectionGraph::default();
        graph.logistics.active_rows = 2;
        graph.ecology.active_rows = 3;
        let logistics = LogisticsVisualSnapshot {
            stamp: SimStepStamp::new(1, 0),
            active_overlay_rows: 2,
            edge_rows: vec![(1, 1.0)],
            ..Default::default()
        };
        let ecology = EcologyVisualSnapshot {
            stamp: SimStepStamp::new(1, 0),
            ecology_chunk_count: 3,
            chunk_rows: vec![Vec4::ONE; 3],
            ..Default::default()
        };
        let fence = CommittedVisualSnapshotFence {
            fire: SimStepStamp::new(2, 0),
        };
        let logistics_rows = if graph.logistics.active_rows > 0 && logistics.stamp == fence.fire {
            project_logistics_overlay_rows(&logistics, graph.logistics.active_rows as usize)
        } else {
            Vec::new()
        };
        let ecology_rows = if graph.ecology.active_rows > 0 && ecology.stamp == fence.fire {
            project_ecology_overlay_rows(&ecology, graph.ecology.active_rows as usize)
        } else {
            Vec::new()
        };
        assert!(logistics_rows.is_empty());
        assert!(ecology_rows.is_empty());
    }
}
