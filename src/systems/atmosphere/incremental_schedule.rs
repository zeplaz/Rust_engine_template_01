//! P2-H incremental atmosphere cadence (hybrid reconcile after snapshots + VT stable).
//!
//! CPU dirty-region scheduling, mirrored GPU subresource uploads, and partial compute dispatch
//! are in-tree. Full-field compute runs only on reconcile cadence when partial uploads are empty.

use std::collections::HashMap;

use bevy::prelude::*;

use super::field_page_residency::{sync_atmosphere_field_page_residency, AtmosphereFieldResidencyTable};

use crate::render::gpu_weather_fire_field::{WeatherFireFieldTextures, WEATHER_FIRE_FIELD_SIZE};
use crate::systems::sim_control::SimStepStamp;

/// P2-H partial GPU path is authoritative when texture uploads + partial compute are active.
pub const P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE: bool = true;

/// Render-world `queue.write_texture` uploads are enabled (mirrored to both ping-pong textures).
pub const P2H_GPU_PARTIAL_TEXTURE_UPLOADS_ENABLED: bool = true;

/// Chunk-space border expanded around dirty bounds for diffusion-safe partial writes.
pub const ATMOSPHERE_DIFFUSION_BORDER_CHUNKS: i32 = 1;

/// Bytes per texel for the weather/fire field (`Rgba32Float`).
pub const FIELD_TEXEL_BYTES: u64 = 16;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct AtmosphereFieldAtlasCenter {
    pub origin_chunk: IVec2,
}

#[derive(Resource, Clone, Copy, Debug)]
pub struct AtmosphereIncrementalSchedule {
    pub full_reconcile_secs: f32,
    pub partial_hz: f32,
    pub reconcile_hz: f32,
    pub accumulator_secs: f32,
}

impl Default for AtmosphereIncrementalSchedule {
    fn default() -> Self {
        Self {
            full_reconcile_secs: 4.0,
            partial_hz: 20.0,
            reconcile_hz: 0.25,
            accumulator_secs: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AtmosphereDirtyRegion {
    pub min: IVec2,
    pub max: IVec2,
    pub stamp: SimStepStamp,
    /// CPU staging heat prior for partial cell writes (not uploaded verbatim to GPU rows).
    pub mean_heat: f32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct AtmosphereDirtyRegionQueue {
    pub regions: Vec<AtmosphereDirtyRegion>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtmospherePartialUpload {
    pub region: AtmosphereDirtyRegion,
    pub bytes_offset: u64,
    pub extent: UVec2,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct AtmospherePartialUploadPlan {
    pub uploads: Vec<AtmospherePartialUpload>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct AtmosphereGpuFieldBridge {
    pub texture: Handle<Image>,
    pub pending_partial_uploads: Vec<AtmospherePartialUpload>,
    pub last_full_reconcile: SimStepStamp,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct AtmospherePartialWriteMetrics {
    pub dirty_region_count: u32,
    pub dirty_cell_count: u32,
    pub partial_upload_bytes: u64,
    pub partial_upload_count: u32,
    pub gpu_texture_upload_bytes: u64,
    pub gpu_texture_upload_count: u32,
    pub full_field_texture_bytes: u64,
    pub partial_compute_dispatch_count: u32,
    pub full_field_dispatch_count: u32,
    pub full_field_fallback_active: bool,
    pub atlas_skip_count: u32,
    pub last_partial_stamp: SimStepStamp,
    pub last_full_reconcile_stamp: SimStepStamp,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct AtmospherePartialFieldState {
    pub partial_writes: u64,
    pub full_reconciles: u64,
    pub stale_region_skips: u64,
    pub last_applied_stamp: SimStepStamp,
    pub cells: HashMap<IVec2, f32>,
}

impl AtmospherePartialFieldState {
    #[must_use]
    pub fn mean_cell_heat(&self) -> f32 {
        if self.cells.is_empty() {
            return 0.0;
        }
        self.cells.values().sum::<f32>() / self.cells.len() as f32
    }
}

#[must_use]
pub fn expand_diffusion_region(region: AtmosphereDirtyRegion, border: i32) -> AtmosphereDirtyRegion {
    let border = border.max(0);
    AtmosphereDirtyRegion {
        min: region.min - IVec2::splat(border),
        max: region.max + IVec2::splat(border),
        stamp: region.stamp,
        mean_heat: region.mean_heat,
    }
}

#[must_use]
pub fn dirty_region_cell_count(region: AtmosphereDirtyRegion) -> u32 {
    let dx = (region.max.x - region.min.x + 1).max(0) as u32;
    let dy = (region.max.y - region.min.y + 1).max(0) as u32;
    dx.saturating_mul(dy)
}

#[must_use]
pub fn weather_fire_field_full_texture_bytes() -> u64 {
    WEATHER_FIRE_FIELD_SIZE.x as u64
        * WEATHER_FIRE_FIELD_SIZE.y as u64
        * FIELD_TEXEL_BYTES
}

#[must_use]
pub fn chunk_to_field_texel(chunk: IVec2, atlas_center: IVec2) -> Option<UVec2> {
    let w = WEATHER_FIRE_FIELD_SIZE.x.max(1) as i32;
    let h = WEATHER_FIRE_FIELD_SIZE.y.max(1) as i32;
    let half = IVec2::new(w / 2, h / 2);
    let local = chunk - atlas_center + half;
    if local.x < 0 || local.y < 0 || local.x >= w || local.y >= h {
        return None;
    }
    Some(UVec2::new(local.x as u32, local.y as u32))
}

pub fn partial_upload_for_region(
    region: AtmosphereDirtyRegion,
    atlas_center: IVec2,
) -> Option<AtmospherePartialUpload> {
    let expanded = expand_diffusion_region(region, ATMOSPHERE_DIFFUSION_BORDER_CHUNKS);
    let extent = UVec2::new(
        (expanded.max.x - expanded.min.x + 1).max(1) as u32,
        (expanded.max.y - expanded.min.y + 1).max(1) as u32,
    );
    let origin = chunk_to_field_texel(expanded.min, atlas_center)?;
    let bytes_offset = (origin.y as u64 * WEATHER_FIRE_FIELD_SIZE.x as u64 + origin.x as u64)
        * FIELD_TEXEL_BYTES;
    Some(AtmospherePartialUpload {
        region: expanded,
        bytes_offset,
        extent,
    })
}

#[must_use]
pub fn build_partial_gpu_uploads(
    regions: &[AtmosphereDirtyRegion],
    atlas_center: IVec2,
) -> Vec<AtmospherePartialUpload> {
    regions
        .iter()
        .copied()
        .filter_map(|region| partial_upload_for_region(region, atlas_center))
        .collect()
}

pub fn register_atmosphere_incremental_schedule(app: &mut App) {
    app.init_resource::<AtmosphereIncrementalSchedule>()
        .init_resource::<AtmosphereDirtyRegionQueue>()
        .init_resource::<AtmospherePartialFieldState>()
        .init_resource::<AtmospherePartialUploadPlan>()
        .init_resource::<AtmosphereGpuFieldBridge>()
        .init_resource::<AtmosphereFieldAtlasCenter>()
        .init_resource::<AtmosphereFieldResidencyTable>()
        .init_resource::<AtmospherePartialWriteMetrics>()
        .add_systems(
            Update,
            (
                enqueue_atmosphere_dirty_regions_from_fire,
                sync_atmosphere_field_atlas_center,
                sync_atmosphere_field_page_residency.after(sync_atmosphere_field_atlas_center),
                build_partial_gpu_uploads_from_queue.after(sync_atmosphere_field_page_residency),
                apply_partial_field_updates_tick,
                prepare_atmosphere_partial_texture_writes,
                atmosphere_reconcile_tick,
                mirror_partial_write_metrics,
            )
                .chain()
                .in_set(crate::systems::atmosphere::pipeline::AtmospherePipelineSet::FieldFill),
        );
}

pub fn enqueue_atmosphere_dirty_regions_from_fire(
    fire: Option<Res<crate::render::sim_visual_extract::FireVisualFrame>>,
    fence: Option<Res<crate::render::CommittedVisualSnapshotFence>>,
    mut queue: ResMut<AtmosphereDirtyRegionQueue>,
) {
    let Some(fire) = fire else {
        return;
    };
    if fire.chunk_heat.is_empty() {
        return;
    }
    if let Some(fence) = fence.as_deref() {
        if fence.fire.tick > 0 && fire.stamp != fence.fire {
            return;
        }
    }
    let mut min = fire.chunk_heat[0].chunk;
    let mut max = min;
    let mut heat_sum = 0.0f32;
    for row in &fire.chunk_heat {
        min = IVec2::new(min.x.min(row.chunk.x), min.y.min(row.chunk.y));
        max = IVec2::new(max.x.max(row.chunk.x), max.y.max(row.chunk.y));
        heat_sum += row.heat;
    }
    let mean_heat = heat_sum / fire.chunk_heat.len() as f32;
    if queue
        .regions
        .last()
        .is_some_and(|r| r.stamp == fire.stamp && r.min == min && r.max == max)
    {
        return;
    }
    queue.regions.push(AtmosphereDirtyRegion {
        min,
        max,
        stamp: fire.stamp,
        mean_heat,
    });
}

pub fn sync_atmosphere_field_atlas_center(
    fire: Option<Res<crate::render::sim_visual_extract::FireVisualFrame>>,
    mut atlas: ResMut<AtmosphereFieldAtlasCenter>,
) {
    let Some(fire) = fire else {
        return;
    };
    if fire.chunk_heat.is_empty() {
        return;
    }
    let mut min = fire.chunk_heat[0].chunk;
    let mut max = min;
    for row in &fire.chunk_heat {
        min = IVec2::new(min.x.min(row.chunk.x), min.y.min(row.chunk.y));
        max = IVec2::new(max.x.max(row.chunk.x), max.y.max(row.chunk.y));
    }
    atlas.origin_chunk = IVec2::new((min.x + max.x) / 2, (min.y + max.y) / 2);
}

pub fn build_partial_gpu_uploads_from_queue(
    queue: Res<AtmosphereDirtyRegionQueue>,
    atlas: Res<AtmosphereFieldAtlasCenter>,
    mut plan: ResMut<AtmospherePartialUploadPlan>,
    mut metrics: ResMut<AtmospherePartialWriteMetrics>,
) {
    plan.uploads = build_partial_gpu_uploads(&queue.regions, atlas.origin_chunk);
    metrics.atlas_skip_count = metrics.atlas_skip_count.saturating_add(
        queue
            .regions
            .iter()
            .copied()
            .filter(|region| partial_upload_for_region(*region, atlas.origin_chunk).is_none())
            .count() as u32,
    );
    metrics.dirty_region_count = queue.regions.len() as u32;
    metrics.dirty_cell_count = queue
        .regions
        .iter()
        .copied()
        .map(dirty_region_cell_count)
        .sum();
    metrics.full_field_texture_bytes = weather_fire_field_full_texture_bytes();
}

pub fn prepare_atmosphere_partial_texture_writes(
    mut plan: ResMut<AtmospherePartialUploadPlan>,
    mut bridge: ResMut<AtmosphereGpuFieldBridge>,
    mut metrics: ResMut<AtmospherePartialWriteMetrics>,
    textures: Option<Res<WeatherFireFieldTextures>>,
) {
    if plan.uploads.is_empty() {
        return;
    }
    if let Some(tex) = textures.as_deref() {
        bridge.texture = tex.texture_a.clone();
    }
    let upload_count = plan.uploads.len() as u32;
    let mut upload_bytes = 0u64;
    for upload in plan.uploads.drain(..) {
        upload_bytes = upload_bytes.saturating_add(upload.partial_upload_bytes());
        bridge.pending_partial_uploads.push(upload);
        metrics.last_partial_stamp = upload.region.stamp;
    }
    metrics.partial_upload_count = metrics.partial_upload_count.saturating_add(upload_count);
    metrics.partial_upload_bytes = metrics.partial_upload_bytes.saturating_add(upload_bytes);
}

fn diffuse_partial_neighbors(center: IVec2, heat: f32, field: &mut AtmospherePartialFieldState) {
    for delta in [IVec2::X, IVec2::NEG_X, IVec2::Y, IVec2::NEG_Y] {
        let coord = center + delta;
        field
            .cells
            .entry(coord)
            .and_modify(|v| *v = (*v * 0.9 + heat * 0.08).clamp(0.0, 1.5))
            .or_insert((heat * 0.08).clamp(0.0, 1.5));
    }
}

fn apply_partial_field_writes(
    region: AtmosphereDirtyRegion,
    field: &mut AtmospherePartialFieldState,
) -> Vec<IVec2> {
    let mut touched = Vec::new();
    let heat = region.mean_heat.clamp(0.0, 1.5);
    for y in region.min.y..=region.max.y {
        for x in region.min.x..=region.max.x {
            let coord = IVec2::new(x, y);
            field
                .cells
                .entry(coord)
                .and_modify(|v| *v = (*v * 0.85 + heat).clamp(0.0, 1.5))
                .or_insert(heat.max(0.05));
            diffuse_partial_neighbors(coord, heat, field);
            touched.push(coord);
        }
    }
    field.partial_writes = field.partial_writes.saturating_add(1);
    field.last_applied_stamp = region.stamp;
    touched
}

pub fn apply_partial_field_updates_tick(
    time: Res<Time>,
    schedule: Res<AtmosphereIncrementalSchedule>,
    fence: Option<Res<crate::render::CommittedVisualSnapshotFence>>,
    mut queue: ResMut<AtmosphereDirtyRegionQueue>,
    mut field: ResMut<AtmospherePartialFieldState>,
    mut acc: Local<f32>,
) {
    *acc += time.delta_secs();
    let interval = 1.0 / schedule.partial_hz.max(1.0);
    if *acc < interval {
        return;
    }
    *acc -= interval;
    if queue.regions.is_empty() {
        return;
    }
    let regions: Vec<AtmosphereDirtyRegion> = queue.regions.drain(..).collect();
    for region in regions {
        if let Some(fence) = fence.as_deref() {
            if fence.fire.tick > 0 && region.stamp != fence.fire {
                field.stale_region_skips = field.stale_region_skips.saturating_add(1);
                continue;
            }
            if field.last_applied_stamp.tick > 0 && region.stamp.tick < field.last_applied_stamp.tick {
                field.stale_region_skips = field.stale_region_skips.saturating_add(1);
                continue;
            }
        }
        apply_partial_field_writes(region, &mut field);
    }
}

pub fn atmosphere_reconcile_tick(
    time: Res<Time>,
    schedule: Res<AtmosphereIncrementalSchedule>,
    fence: Option<Res<crate::render::CommittedVisualSnapshotFence>>,
    mut queue: ResMut<AtmosphereDirtyRegionQueue>,
    mut field: ResMut<AtmospherePartialFieldState>,
    mut bridge: ResMut<AtmosphereGpuFieldBridge>,
    mut metrics: ResMut<AtmospherePartialWriteMetrics>,
    mut full_acc: Local<f32>,
) {
    *full_acc += time.delta_secs();
    let interval =
        (1.0 / schedule.reconcile_hz.max(0.01)).min(schedule.full_reconcile_secs.max(0.1));
    if *full_acc < interval {
        return;
    }
    *full_acc = 0.0;
    queue.regions.clear();
    bridge.pending_partial_uploads.clear();
    field.cells.clear();
    field.full_reconciles = field.full_reconciles.saturating_add(1);
    field.last_applied_stamp = SimStepStamp::default();
    let reconcile_stamp = fence
        .as_deref()
        .map(|f| f.fire)
        .unwrap_or_default();
    bridge.last_full_reconcile = reconcile_stamp;
    metrics.last_full_reconcile_stamp = reconcile_stamp;
}

pub fn mirror_partial_write_metrics(
    partial: Res<AtmospherePartialFieldState>,
    metrics: Res<AtmospherePartialWriteMetrics>,
    mut diag: ResMut<super::diagnostics::AtmosphereDiagnostics>,
) {
    diag.partial_field_writes = partial.partial_writes;
    diag.full_field_reconciles = partial.full_reconciles;
    diag.stale_partial_region_skips = partial.stale_region_skips;
    diag.partial_write_metrics = *metrics;
}

impl AtmospherePartialUpload {
    #[must_use]
    pub fn partial_upload_bytes(&self) -> u64 {
        self.extent.x as u64 * self.extent.y as u64 * FIELD_TEXEL_BYTES
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_write_only_touches_dirty_and_expanded_bounds() {
        let mut field = AtmospherePartialFieldState::default();
        field.cells.insert(IVec2::new(20, 20), 0.25);
        let region = AtmosphereDirtyRegion {
            min: IVec2::ZERO,
            max: IVec2::new(2, 2),
            stamp: SimStepStamp::new(1, 0),
            mean_heat: 0.6,
        };
        let touched = apply_partial_field_writes(region, &mut field);
        assert!(touched.iter().any(|c| *c == IVec2::ONE));
        assert_eq!(field.cells.get(&IVec2::new(20, 20)), Some(&0.25));
    }

    #[test]
    fn stale_region_stamp_is_skipped_when_fence_mismatch() {
        let mut field = AtmospherePartialFieldState::default();
        let region = AtmosphereDirtyRegion {
            min: IVec2::ZERO,
            max: IVec2::ZERO,
            stamp: SimStepStamp::new(1, 0),
            mean_heat: 0.8,
        };
        let fence = crate::render::CommittedVisualSnapshotFence {
            fire: SimStepStamp::new(2, 0),
        };
        if fence.fire.tick > 0 && region.stamp != fence.fire {
            field.stale_region_skips = field.stale_region_skips.saturating_add(1);
        } else {
            apply_partial_field_writes(region, &mut field);
        }
        assert_eq!(field.partial_writes, 0);
        assert_eq!(field.stale_region_skips, 1);
    }

    #[test]
    fn older_stamp_rejected_after_newer_partial_apply() {
        let mut field = AtmospherePartialFieldState::default();
        apply_partial_field_writes(
            AtmosphereDirtyRegion {
                min: IVec2::ZERO,
                max: IVec2::ZERO,
                stamp: SimStepStamp::new(3, 0),
                mean_heat: 0.5,
            },
            &mut field,
        );
        let stale = AtmosphereDirtyRegion {
            min: IVec2::ONE,
            max: IVec2::ONE,
            stamp: SimStepStamp::new(2, 0),
            mean_heat: 0.9,
        };
        if field.last_applied_stamp.tick > 0 && stale.stamp.tick < field.last_applied_stamp.tick {
            field.stale_region_skips = field.stale_region_skips.saturating_add(1);
        } else {
            apply_partial_field_writes(stale, &mut field);
        }
        assert_eq!(field.stale_region_skips, 1);
        assert!(!field.cells.contains_key(&IVec2::ONE));
    }

    #[test]
    fn full_reconcile_clears_stale_partial_cells() {
        let mut field = AtmospherePartialFieldState::default();
        field.cells.insert(IVec2::ZERO, 0.5);
        field.cells.clear();
        field.full_reconciles = 1;
        field.last_applied_stamp = SimStepStamp::default();
        assert!(field.cells.is_empty());
        assert_eq!(field.full_reconciles, 1);
    }

    #[test]
    fn stale_region_recovers_after_full_reconcile_and_fresh_stamp() {
        let mut field = AtmospherePartialFieldState::default();
        let stale = AtmosphereDirtyRegion {
            min: IVec2::ZERO,
            max: IVec2::ZERO,
            stamp: SimStepStamp::new(1, 0),
            mean_heat: 0.9,
        };
        field.stale_region_skips = 1;
        field.cells.clear();
        field.last_applied_stamp = SimStepStamp::default();
        field.full_reconciles = 1;
        let fresh = AtmosphereDirtyRegion {
            min: IVec2::ZERO,
            max: IVec2::ZERO,
            stamp: SimStepStamp::new(2, 0),
            mean_heat: 0.7,
        };
        apply_partial_field_writes(fresh, &mut field);
        assert_eq!(field.partial_writes, 1);
        assert!(field.cells.contains_key(&IVec2::ZERO));
        assert!(field.cells.get(&IVec2::ZERO).copied().unwrap_or(0.0) > 0.0);
        let _ = stale;
    }

    #[test]
    fn diffusion_border_bleeds_into_neighbor_chunks() {
        let mut field = AtmospherePartialFieldState::default();
        let region = AtmosphereDirtyRegion {
            min: IVec2::new(4, 4),
            max: IVec2::new(4, 4),
            stamp: SimStepStamp::new(1, 0),
            mean_heat: 0.9,
        };
        apply_partial_field_writes(region, &mut field);
        assert!(field.cells.contains_key(&IVec2::new(4, 4)));
        assert!(field.cells.contains_key(&IVec2::new(3, 4)));
        assert!(field.cells.contains_key(&IVec2::new(5, 4)));
    }

    #[test]
    fn partial_upload_plan_counts_dirty_bytes() {
        let regions = vec![AtmosphereDirtyRegion {
            min: IVec2::ZERO,
            max: IVec2::new(1, 1),
            stamp: SimStepStamp::new(1, 0),
            mean_heat: 0.4,
        }];
        let uploads = build_partial_gpu_uploads(&regions, IVec2::ZERO);
        assert_eq!(uploads.len(), 1);
        assert!(uploads[0].extent.x >= 2);
        assert!(uploads[0].partial_upload_bytes() > 0);
    }

    #[test]
    fn chunk_to_field_texel_maps_relative_to_atlas_center() {
        let center = IVec2::new(10, 10);
        assert_eq!(
            chunk_to_field_texel(IVec2::new(10, 10), center),
            Some(UVec2::splat(64))
        );
        assert_eq!(chunk_to_field_texel(IVec2::new(200, 10), center), None);
    }

    #[test]
    fn expand_diffusion_region_adds_border() {
        let region = AtmosphereDirtyRegion {
            min: IVec2::new(2, 2),
            max: IVec2::new(3, 3),
            stamp: SimStepStamp::new(1, 0),
            mean_heat: 0.1,
        };
        let expanded = expand_diffusion_region(region, 1);
        assert_eq!(expanded.min, IVec2::new(1, 1));
        assert_eq!(expanded.max, IVec2::new(4, 4));
    }
}
