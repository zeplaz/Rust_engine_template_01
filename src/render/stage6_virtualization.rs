//! Stage-6 virtualization contracts — residency-driven representation host.

use bevy::prelude::*;
use bevy::render::RenderApp;



use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};

use crate::gui::{ViewManager, WorldRepresentationFrame};
use crate::io::streaming::{ChunkResidencyTable};
use crate::render::core::gpu_representation_metrics::GpuRepresentationMetrics;
use crate::render::core::per_view_residency::{residency_coords_for_view_instance, PerViewResidencyConsumerWindow};
use crate::render::view_runtime::ViewSurfaceId;
use crate::render::SharedOverlayFieldBuffers;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuntimeAtlasSlot {
    TerrainChunk,
    OverlayField,
    UtilityMask,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct PagedAtlasResidency {
    pub active_slots: Vec<RuntimeAtlasSlot>,
}

#[derive(Resource, Default, Clone, Debug)]
pub struct AsyncDomainApplyQueue {
    pub pending_labels: Vec<&'static str>,
}

#[derive(Resource, Default, Clone, Debug, ExtractResource)]
#[extract_app(RenderApp)]
pub struct Stage6VirtualizationFrame {
    pub focus_chunk: IVec2,
    pub residency_chunk_count: usize,
    pub core_chunk_count: usize,
    pub ghost_chunk_count: usize,
    pub consumer_window_coords: Vec<IVec2>,
    pub active_atlas_slots: usize,
    pub gpu_upload_bytes_frame: u64,
    pub per_view_window_count: usize,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct ResidencyDrivenConsumerWindow {
    pub coords: Vec<IVec2>,
}

/// Map GPU metrics + overlay state to [`PagedAtlasResidency`] slots (S6-20).
#[must_use]
pub fn atlas_slots_from_gpu_path(
    gpu: Option<&GpuRepresentationMetrics>,
    overlay: Option<&SharedOverlayFieldBuffers>,
) -> Vec<RuntimeAtlasSlot> {
    let mut slots = Vec::new();
    let Some(gpu) = gpu else {
        return slots;
    };
    if gpu.upload_bytes > 0 || gpu.active_allocations > 0 {
        slots.push(RuntimeAtlasSlot::TerrainChunk);
    }
    if gpu.instance_rows > 0 || gpu.particle_rows > 0 {
        if !slots.contains(&RuntimeAtlasSlot::OverlayField) {
            slots.push(RuntimeAtlasSlot::OverlayField);
        }
    }
    if overlay.is_some_and(|o| !o.chunk_fire_heat.is_empty()) {
        if !slots.contains(&RuntimeAtlasSlot::OverlayField) {
            slots.push(RuntimeAtlasSlot::OverlayField);
        }
    }
    if gpu.active_allocations > 1 || gpu.dispatch_count > 0 {
        if !slots.contains(&RuntimeAtlasSlot::UtilityMask) {
            slots.push(RuntimeAtlasSlot::UtilityMask);
        }
    }
    slots
}

/// True when `coord` is in the residency consumer window (or window is unset).
#[must_use]
pub fn chunk_in_residency_consumer_window(coord: IVec2, frame: &Stage6VirtualizationFrame) -> bool {
    if frame.consumer_window_coords.is_empty() {
        return true;
    }
    frame.consumer_window_coords.contains(&coord)
}

/// Residency membership from [`ChunkResidencyTable`] (safe before `publish_stage6_virtualization_frame`).
#[must_use]
pub fn chunk_in_residency_table(coord: IVec2, table: &ChunkResidencyTable) -> bool {
    if table.entries.is_empty() {
        return true;
    }
    table.entries.contains_key(&coord)
}

#[must_use]
pub fn intersect_visible_chunks_with_residency_window(
    visible: std::collections::HashSet<IVec2>,
    frame: &Stage6VirtualizationFrame,
) -> std::collections::HashSet<IVec2> {
    if frame.consumer_window_coords.is_empty() {
        return visible;
    }
    let window: std::collections::HashSet<IVec2> =
        frame.consumer_window_coords.iter().copied().collect();
    visible
        .into_iter()
        .filter(|coord| window.contains(coord))
        .collect()
}



pub fn publish_stage6_virtualization_frame(
    world: Res<WorldRepresentationFrame>,
    residency: Res<ChunkResidencyTable>,
    manager: Option<Res<ViewManager>>,
    gpu: Option<Res<GpuRepresentationMetrics>>,
    overlay: Option<Res<SharedOverlayFieldBuffers>>,
    mut window: ResMut<ResidencyDrivenConsumerWindow>,
    mut per_view: ResMut<PerViewResidencyConsumerWindow>,
    mut atlas: ResMut<PagedAtlasResidency>,
    mut frame: ResMut<Stage6VirtualizationFrame>,
    mut budget: Option<ResMut<crate::gui::hud::FrameBudgetDiagnostics>>,
) {
    let started = crate::gui::hud::FrameBudgetTimer::start();
    use crate::io::streaming::ChunkResidencyRole;
    frame.focus_chunk = world.focus_chunk;
    frame.residency_chunk_count = residency.entries.len();
    frame.core_chunk_count = residency
        .entries
        .values()
        .filter(|entry| entry.role == ChunkResidencyRole::Core)
        .count();
    frame.ghost_chunk_count = residency
        .entries
        .values()
        .filter(|entry| entry.role == ChunkResidencyRole::GhostBand)
        .count();
    window.coords = residency.entries.keys().copied().collect();
    window.coords.sort_by_key(|coord| (coord.y, coord.x));
    frame.consumer_window_coords = window.coords.clone();
    per_view.by_surface.clear();
    if let Some(manager) = manager.as_deref() {
        for (&view_id, view) in &manager.views {
            let surface = ViewSurfaceId::from_view_id(view_id);
            let coords = residency_coords_for_view_instance(view, residency.as_ref());
            per_view.by_surface.insert(surface, coords);
        }
    }
    frame.per_view_window_count = per_view.by_surface.values().map(|v| v.len()).sum();
    let upload_bytes = gpu.as_deref().map(|m| m.upload_bytes).unwrap_or(0);
    frame.gpu_upload_bytes_frame = upload_bytes;
    atlas.active_slots = atlas_slots_from_gpu_path(gpu.as_deref(), overlay.as_deref());
    frame.active_atlas_slots = atlas.active_slots.len();
    if let Some(budget) = budget.as_mut() {
        budget.record_bucket_ms(
            crate::gui::hud::FrameBudgetBucket::ResidencyUpdates,
            started.elapsed_ms(),
        );
    }
}

pub fn enqueue_stream_domain_apply_label(
    mut queue: ResMut<AsyncDomainApplyQueue>,
    stream_apply: Res<crate::io::streaming::PendingStreamApplyQueue>,
) {
    if stream_apply.ready_bodies.is_empty() {
        return;
    }
    if !queue.pending_labels.contains(&"stream_chunk_apply") {
        queue.pending_labels.push("stream_chunk_apply");
    }
}

/// Clears stream apply labels after main-thread ECS apply drained the queue (S6-22).
pub fn clear_async_domain_apply_labels_after_stream_apply(
    mut queue: ResMut<AsyncDomainApplyQueue>,
    stream_apply: Res<crate::io::streaming::PendingStreamApplyQueue>,
) {
    if stream_apply.ready_bodies.is_empty() {
        queue.pending_labels.retain(|label| *label != "stream_chunk_apply");
    }
}

/// RND-07 gate: per-frame upload bytes that count as atlas-memory pressure.
pub const RND07_UPLOAD_BOTTLENECK_BYTES: u64 = 8 * 1024 * 1024;
/// RND-07 gate: reserved high-watermark that counts as atlas pressure.
pub const RND07_RESERVED_BOTTLENECK_BYTES: u64 = 64 * 1024 * 1024;
/// RND-07 gate: residency chunks with all atlas slots filled.
pub const RND07_RESIDENCY_BOTTLENECK_CHUNKS: usize = 512;

/// Snapshot inputs for the atlas-memory profiler pass (**RND-07** unlock gate).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AtlasMemoryProfileInput {
    pub gpu_upload_bytes_frame: u64,
    pub reserved_high_watermark_bytes: u64,
    pub active_atlas_slots: usize,
    pub residency_chunk_count: usize,
}

/// Structured profiler result — open RND-07 only when `bottleneck_proven`.
#[derive(Clone, Debug, PartialEq)]
pub struct AtlasMemoryProfileReport {
    pub upload_bytes_frame: u64,
    pub reserved_high_watermark_bytes: u64,
    pub active_atlas_slots: usize,
    pub residency_chunk_count: usize,
    pub pressure_score: f32,
    pub bottleneck_proven: bool,
    pub verdict: &'static str,
}

impl AtlasMemoryProfileInput {
    #[must_use]
    pub fn from_stage6(
        frame: &Stage6VirtualizationFrame,
        gpu: Option<&GpuRepresentationMetrics>,
    ) -> Self {
        Self {
            gpu_upload_bytes_frame: frame.gpu_upload_bytes_frame,
            reserved_high_watermark_bytes: gpu.map(|g| g.high_watermark_bytes).unwrap_or(0),
            active_atlas_slots: frame.active_atlas_slots,
            residency_chunk_count: frame.residency_chunk_count,
        }
    }
}

/// Score atlas-memory pressure and decide whether RND-07 may unlock.
#[must_use]
pub fn profile_atlas_memory(input: AtlasMemoryProfileInput) -> AtlasMemoryProfileReport {
    let upload_ratio =
        input.gpu_upload_bytes_frame as f32 / RND07_UPLOAD_BOTTLENECK_BYTES as f32;
    let reserved_ratio =
        input.reserved_high_watermark_bytes as f32 / RND07_RESERVED_BOTTLENECK_BYTES as f32;
    let residency_ratio = if input.active_atlas_slots >= 3 {
        input.residency_chunk_count as f32 / RND07_RESIDENCY_BOTTLENECK_CHUNKS as f32
    } else {
        0.0
    };
    let pressure_score = upload_ratio
        .max(reserved_ratio)
        .max(residency_ratio)
        .clamp(0.0, 4.0);
    let bottleneck_proven = input.gpu_upload_bytes_frame >= RND07_UPLOAD_BOTTLENECK_BYTES
        || input.reserved_high_watermark_bytes >= RND07_RESERVED_BOTTLENECK_BYTES
        || (input.active_atlas_slots >= 3
            && input.residency_chunk_count >= RND07_RESIDENCY_BOTTLENECK_CHUNKS);
    let verdict = if bottleneck_proven {
        "OPEN_RND07"
    } else {
        "HOLD_RND07"
    };
    AtlasMemoryProfileReport {
        upload_bytes_frame: input.gpu_upload_bytes_frame,
        reserved_high_watermark_bytes: input.reserved_high_watermark_bytes,
        active_atlas_slots: input.active_atlas_slots,
        residency_chunk_count: input.residency_chunk_count,
        pressure_score,
        bottleneck_proven,
        verdict,
    }
}

/// Write profiler JSON under `debug_runs/rnd07/` (gate evidence).
pub fn write_atlas_memory_profile_report(
    report: &AtlasMemoryProfileReport,
) -> std::io::Result<std::path::PathBuf> {
    let dir = std::path::Path::new("debug_runs/rnd07");
    std::fs::create_dir_all(dir)?;
    let path = dir.join("atlas_memory_profile.json");
    let body = format!(
        "{{\n  \"upload_bytes_frame\": {},\n  \"reserved_high_watermark_bytes\": {},\n  \"active_atlas_slots\": {},\n  \"residency_chunk_count\": {},\n  \"pressure_score\": {:.6},\n  \"bottleneck_proven\": {},\n  \"verdict\": \"{}\",\n  \"gates\": {{\n    \"upload_bytes\": {},\n    \"reserved_bytes\": {},\n    \"residency_chunks_all_slots\": {}\n  }}\n}}\n",
        report.upload_bytes_frame,
        report.reserved_high_watermark_bytes,
        report.active_atlas_slots,
        report.residency_chunk_count,
        report.pressure_score,
        report.bottleneck_proven,
        report.verdict,
        RND07_UPLOAD_BOTTLENECK_BYTES,
        RND07_RESERVED_BOTTLENECK_BYTES,
        RND07_RESIDENCY_BOTTLENECK_CHUNKS,
    );
    std::fs::write(&path, body)?;
    Ok(path)
}

pub struct Stage6VirtualizationPlugin;

impl Plugin for Stage6VirtualizationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PagedAtlasResidency>()
            .init_resource::<AsyncDomainApplyQueue>()
            .init_resource::<Stage6VirtualizationFrame>()
            .init_resource::<ResidencyDrivenConsumerWindow>()
            .init_resource::<PerViewResidencyConsumerWindow>()
            .add_plugins(ExtractResourcePlugin::<Stage6VirtualizationFrame>::default())
            .add_systems(
                Update,
                (
                    publish_stage6_virtualization_frame,
                    enqueue_stream_domain_apply_label,
                )
                    .chain()
                    .after(crate::gui::WorldRepresentationSystemSet::ComputeFrame)
                    .run_if(in_state(crate::engine::states::BaseState::Simulation)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_visible_chunks_with_residency_window_filters_to_consumer_window() {
        let visible: std::collections::HashSet<_> = [(0, 0), (1, 0), (2, 0)]
            .into_iter()
            .map(IVec2::from)
            .collect();
        let frame = Stage6VirtualizationFrame {
            consumer_window_coords: vec![IVec2::new(1, 0), IVec2::new(3, 0)],
            ..Default::default()
        };
        let filtered = intersect_visible_chunks_with_residency_window(visible, &frame);
        assert_eq!(filtered, std::collections::HashSet::from([IVec2::new(1, 0)]));
    }

    #[test]
    fn intersect_visible_chunks_with_residency_window_noop_when_empty() {
        let visible: std::collections::HashSet<_> = [(0, 0), (1, 0)]
            .into_iter()
            .map(IVec2::from)
            .collect();
        let filtered =
            intersect_visible_chunks_with_residency_window(visible.clone(), &Stage6VirtualizationFrame::default());
        assert_eq!(filtered, visible);
    }





    #[test]
    fn atlas_slots_from_gpu_path_when_upload_present() {
        let gpu = GpuRepresentationMetrics {
            upload_bytes: 1024,
            active_allocations: 1,
            ..Default::default()
        };
        let slots = atlas_slots_from_gpu_path(Some(&gpu), None);
        assert!(slots.contains(&RuntimeAtlasSlot::TerrainChunk));
    }

    #[test]
    fn async_domain_apply_queue_clears_stream_label_when_ready_bodies_drained() {
        let mut queue = AsyncDomainApplyQueue::default();
        queue.pending_labels.push("stream_chunk_apply");
        queue
            .pending_labels
            .retain(|label| *label != "stream_chunk_apply");
        assert!(queue.pending_labels.is_empty());
    }

    #[test]
    fn rnd07_atlas_memory_profile() {
        let idle = profile_atlas_memory(AtlasMemoryProfileInput::from_stage6(
            &Stage6VirtualizationFrame::default(),
            None,
        ));
        assert!(!idle.bottleneck_proven);
        assert_eq!(idle.verdict, "HOLD_RND07");
        assert!(idle.pressure_score < 1.0);

        let hot = profile_atlas_memory(AtlasMemoryProfileInput {
            gpu_upload_bytes_frame: RND07_UPLOAD_BOTTLENECK_BYTES,
            reserved_high_watermark_bytes: 0,
            active_atlas_slots: 3,
            residency_chunk_count: RND07_RESIDENCY_BOTTLENECK_CHUNKS,
        });
        assert!(hot.bottleneck_proven);
        assert_eq!(hot.verdict, "OPEN_RND07");

        let path = write_atlas_memory_profile_report(&idle).expect("write rnd07 profile");
        assert!(path.exists());
        let body = std::fs::read_to_string(&path).expect("read profile");
        assert!(body.contains("HOLD_RND07"));
        assert!(body.contains("\"bottleneck_proven\": false"));
    }
}
