//! Stage-5 logistics / ecology overlay rows uploaded through [`super::gpu_buffer_registry::GPUBufferRegistry`].

use bevy::math::Vec4;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bytemuck::{Pod, Zeroable};

use crate::render::ecology_visual_snapshot::EcologyVisualSnapshot;
use crate::render::extraction::RenderProjectionGraph;
use crate::render::logistics_visual_snapshot::LogisticsVisualSnapshot;
use crate::render::CommittedVisualSnapshotFence;
use crate::systems::sim_control::SimStepStamp;

/// Packed corridor overlay row for [`super::gpu_buffer_registry::LOGISTICS_OVERLAY_BUFFER`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct LogisticsOverlayGpuRow {
    pub corridor_xy_revision: Vec4,
}

/// Packed ecology overlay row for [`super::gpu_buffer_registry::ECOLOGY_OVERLAY_BUFFER`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct EcologyOverlayGpuRow {
    pub ecology_xy_means: Vec4,
}

/// CPU-side domain overlay upload payload (projection → registry).
#[derive(Resource, Debug, Clone, ExtractResource, Default)]
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
        .map(|&(edge, traffic)| LogisticsOverlayGpuRow {
            corridor_xy_revision: Vec4::new(
                edge as f32,
                traffic,
                snapshot.stamp.tick as f32,
                snapshot.corridor_revision as f32,
            ),
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
        };
        let rows = project_logistics_overlay_rows(&snapshot, 2);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].corridor_xy_revision.x, 7.0);
        assert!((rows[0].corridor_xy_revision.y - 0.5).abs() < 1e-5);
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
