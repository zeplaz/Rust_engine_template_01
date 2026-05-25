//! Stage 6 consumer telemetry — residency, overlay, and GPU resource snapshots.

use bevy::prelude::*;

use crate::io::streaming::{ChunkResidencyRole, ChunkResidencyTable};
use crate::render::{GpuRepresentationMetrics, Stage6VirtualizationFrame};

use super::stage6_consumer::{ResidencyOverlayConsumerDto, StreamedChunkDiagnosticDto};

#[derive(Resource, Clone, Debug, Default)]
pub struct Stage6HudTelemetry {
    pub residency: ResidencyOverlayConsumerDto,
    pub gpu: GpuShellResourceStats,
    pub frame_revision: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GpuShellResourceStats {
    pub upload_bytes: u64,
    pub reserved_bytes: u64,
    pub high_watermark_bytes: u64,
    pub active_rows: u32,
    pub instance_rows: u32,
    pub particle_rows: u32,
    pub dispatch_count: u32,
    pub draw_instances: u32,
    pub active_allocations: u32,
    pub gpu_memory_estimate_bytes: u64,
    pub texture_count_estimate: u32,
    pub render_target_count_estimate: u32,
    pub buffer_residency_bytes: u64,
    pub upload_throughput_bytes_per_sec: f32,
    pub texture_rebuild_count: u32,
    pub egui_texture_registrations_frame: u32,
}

#[must_use]
pub fn gpu_shell_resource_stats_from(metrics: &GpuRepresentationMetrics) -> GpuShellResourceStats {
    GpuShellResourceStats {
        upload_bytes: metrics.upload_bytes,
        reserved_bytes: metrics.reserved_bytes,
        high_watermark_bytes: metrics.high_watermark_bytes,
        active_rows: metrics.active_rows,
        instance_rows: metrics.instance_rows,
        particle_rows: metrics.particle_rows,
        dispatch_count: metrics.dispatch_count,
        draw_instances: metrics.draw_instances,
        active_allocations: metrics.active_allocations,
        gpu_memory_estimate_bytes: metrics
            .reserved_bytes
            .saturating_add(metrics.high_watermark_bytes),
        texture_count_estimate: metrics.active_allocations,
        render_target_count_estimate: 0,
        buffer_residency_bytes: metrics.reserved_bytes,
        upload_throughput_bytes_per_sec: 0.0,
        texture_rebuild_count: 0,
        egui_texture_registrations_frame: 0,
    }
}

#[must_use]
pub fn residency_overlay_consumer_from_frame(
    frame: &Stage6VirtualizationFrame,
    residency: &ChunkResidencyTable,
) -> ResidencyOverlayConsumerDto {
    let mut chunks = Vec::with_capacity(residency.entries.len().min(32));
    for (coord, entry) in residency.entries.iter().take(32) {
        chunks.push(StreamedChunkDiagnosticDto {
            chunk_x: coord.x,
            chunk_y: coord.y,
            residency_ring: match entry.role {
                ChunkResidencyRole::Core => 0,
                ChunkResidencyRole::GhostBand => 1,
            },
            ghost_band: entry.role == ChunkResidencyRole::GhostBand,
        });
    }
    ResidencyOverlayConsumerDto {
        schema_version: ResidencyOverlayConsumerDto::CURRENT_SCHEMA,
        resident_chunks: frame.core_chunk_count as u32,
        ghost_chunks: frame.ghost_chunk_count as u32,
        utility_channel_mask: if frame.active_atlas_slots > 1 {
            0b1011
        } else {
            0b0001
        },
        paged_atlas_pages: frame.active_atlas_slots as u32,
        chunks,
    }
}

pub fn refresh_stage6_hud_telemetry(
    frame: Res<Stage6VirtualizationFrame>,
    residency: Res<ChunkResidencyTable>,
    gpu: Option<Res<GpuRepresentationMetrics>>,
    budget: Option<Res<super::frame_budget_diagnostics::FrameBudgetDiagnostics>>,
    shell_diag: Option<Res<super::shell_diagnostics::ProductShellDiagnostics>>,
    mut async_queue: Option<ResMut<super::hud_async_task_queue::HudAsyncTaskQueue>>,
    mut telemetry: ResMut<Stage6HudTelemetry>,
    mut last_upload: Local<u64>,
) {
    let next_residency = residency_overlay_consumer_from_frame(&frame, &residency);
    let mut next_gpu = gpu
        .as_deref()
        .map(gpu_shell_resource_stats_from)
        .unwrap_or_default();
    let _upload_delta = next_gpu
        .upload_bytes
        .saturating_sub(*last_upload);
    *last_upload = next_gpu.upload_bytes;
    next_gpu.gpu_memory_estimate_bytes = next_gpu.reserved_bytes.saturating_add(next_gpu.high_watermark_bytes);
    next_gpu.buffer_residency_bytes = next_gpu.reserved_bytes;
    next_gpu.texture_count_estimate = next_gpu.active_allocations;
    next_gpu.render_target_count_estimate = frame.active_atlas_slots.min(u32::MAX as usize) as u32;
    next_gpu.upload_throughput_bytes_per_sec = budget
        .as_deref()
        .map(|b| b.upload_bytes_per_sec)
        .unwrap_or(0.0);
    next_gpu.texture_rebuild_count = budget
        .as_deref()
        .map(|b| b.texture_rebuilds_frame)
        .unwrap_or(0);
    next_gpu.egui_texture_registrations_frame = shell_diag
        .as_deref()
        .map(|diag| {
            diag.texture_rebuilds
                .values()
                .map(|count| *count as u32)
                .sum::<u32>()
        })
        .unwrap_or(0);
    if telemetry.residency != next_residency {
        if let Some(queue) = async_queue.as_mut() {
            queue.enqueue(super::hud_async_task_queue::HudAsyncTask::FormatResidencyDto {
                resident_chunks: next_residency.resident_chunks,
                ghost_chunks: next_residency.ghost_chunks,
                atlas_pages: next_residency.paged_atlas_pages,
            });
        }
    }
    if let Some(queue) = async_queue.as_mut() {
        queue.enqueue(super::hud_async_task_queue::HudAsyncTask::TelemetryAggregation {
            frame_revision: telemetry.frame_revision.wrapping_add(1),
        });
        if let Some(gpu_metrics) = gpu.as_deref() {
            queue.enqueue(super::hud_async_task_queue::HudAsyncTask::GpuStatsAggregation {
                upload_bytes: gpu_metrics.upload_bytes,
                texture_rebuilds: budget
                    .as_deref()
                    .map(|b| b.texture_rebuilds_frame)
                    .unwrap_or(0),
            });
        }
    }
    if telemetry.residency != next_residency || telemetry.gpu != next_gpu {
        telemetry.frame_revision = telemetry.frame_revision.wrapping_add(1);
    }
    telemetry.residency = next_residency;
    telemetry.gpu = next_gpu;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::streaming::ChunkResidencyEntry;

    #[test]
    fn residency_overlay_consumer_maps_roles() {
        let mut residency = ChunkResidencyTable::default();
        residency.entries.insert(
            IVec2::ZERO,
            ChunkResidencyEntry {
                coord: IVec2::ZERO,
                role: ChunkResidencyRole::GhostBand,
                orb_priority: 0,
            },
        );
        let frame = Stage6VirtualizationFrame {
            core_chunk_count: 0,
            ghost_chunk_count: 1,
            active_atlas_slots: 2,
            ..Default::default()
        };
        let dto = residency_overlay_consumer_from_frame(&frame, &residency);
        assert_eq!(dto.ghost_chunks, 1);
        assert_eq!(dto.chunks.len(), 1);
        assert!(dto.chunks[0].ghost_band);
    }
}
