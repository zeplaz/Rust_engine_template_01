//! Stage-6 virtualization contracts — residency-driven representation host.

use bevy::prelude::*;

use crate::render::stage6_live_proof::{
    refresh_stage6_virtualization_witness, write_stage6_virtualization_live_proof_system,
    Stage6LiveProofState, Stage6VirtualizationWitness,
};
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};

use crate::gui::editor::world_preview::PreviewPathAuthority;
use crate::gui::{ViewManager, WorldRepresentationFrame};
use crate::io::streaming::{gather_wave_c_readiness, wave_c_readiness_passes, ChunkResidencyTable};
use crate::render::gpu_representation_metrics::GpuRepresentationMetrics;
use crate::render::per_view_residency::{residency_coords_for_view_instance, PerViewResidencyConsumerWindow};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stage6ReadinessReport {
    pub wave_c_ok: bool,
    pub residency_populated: bool,
    pub projection_window_populated: bool,
    pub atlas_slots_active: bool,
}

#[must_use]
pub fn gather_stage6_readiness(
    authority: &PreviewPathAuthority,
    frame: &Stage6VirtualizationFrame,
) -> Stage6ReadinessReport {
    let wave_c = gather_wave_c_readiness(authority);
    Stage6ReadinessReport {
        wave_c_ok: wave_c_readiness_passes(&wave_c),
        residency_populated: frame.residency_chunk_count > 0,
        projection_window_populated: !frame.consumer_window_coords.is_empty(),
        atlas_slots_active: frame.gpu_upload_bytes_frame > 0,
    }
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

#[must_use]
pub fn stage6_readiness_passes(report: &Stage6ReadinessReport) -> bool {
    report.wave_c_ok
        && report.residency_populated
        && report.projection_window_populated
        && report.atlas_slots_active
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

pub struct Stage6VirtualizationPlugin;

impl Plugin for Stage6VirtualizationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PagedAtlasResidency>()
            .init_resource::<AsyncDomainApplyQueue>()
            .init_resource::<Stage6VirtualizationFrame>()
            .init_resource::<ResidencyDrivenConsumerWindow>()
            .init_resource::<PerViewResidencyConsumerWindow>()
            .init_resource::<Stage6VirtualizationWitness>()
            .init_resource::<Stage6LiveProofState>()
            .add_plugins(ExtractResourcePlugin::<Stage6VirtualizationFrame>::default())
            .add_systems(
                Update,
                (
                    publish_stage6_virtualization_frame,
                    enqueue_stream_domain_apply_label,
                    crate::gui::hud::refresh_stage6_hud_telemetry,
                    refresh_stage6_virtualization_witness,
                    write_stage6_virtualization_live_proof_system,
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
    fn stage6_readiness_passes_with_populated_residency_table() {
        use crate::gui::WorldRepresentationFrame;
        use crate::io::streaming::{
            build_residency_table, chunk_window_coords, primary_interest_orb,
        };
        let world = WorldRepresentationFrame {
            focus_chunk: IVec2::ZERO,
            interest_radius_chunks: 1,
            ..Default::default()
        };
        let orbs = vec![primary_interest_orb(&world)];
        let core = chunk_window_coords(world.focus_chunk, 1);
        let table = build_residency_table(&orbs, &core);
        let coords: Vec<IVec2> = table.entries.keys().copied().collect();
        let frame = Stage6VirtualizationFrame {
            residency_chunk_count: table.entries.len(),
            core_chunk_count: table
                .entries
                .values()
                .filter(|e| e.role == crate::io::streaming::ChunkResidencyRole::Core)
                .count(),
            ghost_chunk_count: table
                .entries
                .values()
                .filter(|e| e.role == crate::io::streaming::ChunkResidencyRole::GhostBand)
                .count(),
            consumer_window_coords: coords,
            active_atlas_slots: 1,
            gpu_upload_bytes_frame: 1,
            ..Default::default()
        };
        let report = gather_stage6_readiness(&PreviewPathAuthority::default(), &frame);
        assert!(stage6_readiness_passes(&report));
    }

    #[test]
    fn stage6_readiness_requires_gpu_upload_bytes() {
        let report = gather_stage6_readiness(
            &PreviewPathAuthority::default(),
            &Stage6VirtualizationFrame {
                residency_chunk_count: 1,
                consumer_window_coords: vec![IVec2::ZERO],
                gpu_upload_bytes_frame: 0,
                ..Default::default()
            },
        );
        assert!(!report.atlas_slots_active);
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
    fn stage6_readiness_requires_wave_c_and_residency() {
        let report = gather_stage6_readiness(
            &PreviewPathAuthority::default(),
            &Stage6VirtualizationFrame {
                residency_chunk_count: 1,
                consumer_window_coords: vec![IVec2::ZERO],
                gpu_upload_bytes_frame: 4096,
                ..Default::default()
            },
        );
        assert!(stage6_readiness_passes(&report));
    }
}
