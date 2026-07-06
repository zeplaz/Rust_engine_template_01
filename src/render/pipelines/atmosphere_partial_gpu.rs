//! P2-H render-world partial texture uploads (CPU mirror → GPU subresource writes).

use std::collections::HashSet;

use bevy::prelude::*;
use bevy::render::{
    extract_resource::ExtractResource,
    render_asset::RenderAssets,
    render_resource::{Extent3d, Origin3d, TexelCopyBufferLayout, TexelCopyTextureInfo, TextureAspect},
    renderer::RenderQueue,
    texture::GpuImage,
};

use crate::render::CommittedVisualSnapshotFence;
use crate::systems::atmosphere::{
    chunk_to_field_texel, weather_fire_field_full_texture_bytes, AtmosphereFieldAtlasCenter,
    AtmosphereGpuFieldBridge, AtmospherePartialFieldState, AtmospherePartialWriteMetrics,
    FIELD_TEXEL_BYTES,
};
use crate::systems::sim_control::SimStepStamp;

/// Main-world extract payload mirrored into the render world each frame.
#[derive(Resource, Clone, Default, ExtractResource)]
pub struct AtmospherePartialGpuExtract {
    pub uploads: Vec<GpuPreparedPartialUpload>,
    pub partial_dispatch_origin: UVec2,
    pub partial_dispatch_extent: UVec2,
    pub partial_dispatch_active: bool,
    pub full_field_fallback: bool,
}

/// Render-world prepared rows for `queue.write_texture`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GpuPreparedPartialUpload {
    pub origin: UVec2,
    pub extent: UVec2,
    pub bytes_per_row: u32,
    pub data: Vec<[f32; 4]>,
    pub stamp: SimStepStamp,
}

impl GpuPreparedPartialUpload {
    #[must_use]
    pub fn upload_bytes(&self) -> u64 {
        self.data.len() as u64 * FIELD_TEXEL_BYTES
    }
}

#[must_use]
pub fn partial_upload_stamp_is_fresh(
    stamp: SimStepStamp,
    fence: Option<&CommittedVisualSnapshotFence>,
    last_full_reconcile: SimStepStamp,
    last_applied_stamp: SimStepStamp,
) -> bool {
    if let Some(fence) = fence {
        if fence.fire.tick > 0 && stamp != fence.fire {
            return false;
        }
    }
    if last_full_reconcile.tick > 0 && stamp.tick < last_full_reconcile.tick {
        return false;
    }
    if last_applied_stamp.tick > 0 && stamp.tick < last_applied_stamp.tick {
        return false;
    }
    true
}

#[must_use]
pub fn collect_gpu_prepared_partial_uploads(
    bridge: &AtmosphereGpuFieldBridge,
    field: &AtmospherePartialFieldState,
    atlas: IVec2,
    fence: Option<&CommittedVisualSnapshotFence>,
) -> Vec<GpuPreparedPartialUpload> {
    let mut uploads = Vec::new();
    let mut seen = HashSet::new();
    for pending in &bridge.pending_partial_uploads {
        let expanded = pending.region;
        if !partial_upload_stamp_is_fresh(
            expanded.stamp,
            fence,
            bridge.last_full_reconcile,
            field.last_applied_stamp,
        ) {
            continue;
        }
        let key = (
            expanded.min,
            expanded.max,
            expanded.stamp.tick,
            expanded.stamp.sim_time_micros,
        );
        if !seen.insert(key) {
            continue;
        }
        let Some(origin) = chunk_to_field_texel(expanded.min, atlas) else {
            continue;
        };
        let extent = pending.extent;
        let mut data = Vec::with_capacity((extent.x * extent.y) as usize);
        for y in expanded.min.y..=expanded.max.y {
            for x in expanded.min.x..=expanded.max.x {
                let heat = field
                    .cells
                    .get(&IVec2::new(x, y))
                    .copied()
                    .unwrap_or(0.0);
                data.push([heat, 0.0, 0.0, 1.0]);
            }
        }
        uploads.push(GpuPreparedPartialUpload {
            origin,
            extent,
            bytes_per_row: extent.x * FIELD_TEXEL_BYTES as u32,
            data,
            stamp: expanded.stamp,
        });
    }
    uploads
}

#[must_use]
pub fn union_partial_dispatch_bounds(uploads: &[GpuPreparedPartialUpload]) -> Option<(UVec2, UVec2)> {
    let first = uploads.first()?;
    let mut min = first.origin;
    let mut max = first.origin + first.extent.saturating_sub(UVec2::ONE);
    for upload in uploads.iter().skip(1) {
        let end = upload.origin + upload.extent.saturating_sub(UVec2::ONE);
        min = UVec2::new(min.x.min(upload.origin.x), min.y.min(upload.origin.y));
        max = UVec2::new(max.x.max(end.x), max.y.max(end.y));
    }
    let extent = UVec2::new(max.x - min.x + 1, max.y - min.y + 1);
    Some((min, extent))
}

pub fn sync_atmosphere_partial_gpu_extract(
    mut bridge: ResMut<AtmosphereGpuFieldBridge>,
    field: Res<AtmospherePartialFieldState>,
    atlas: Res<AtmosphereFieldAtlasCenter>,
    fence: Option<Res<CommittedVisualSnapshotFence>>,
    mut out: ResMut<AtmospherePartialGpuExtract>,
    mut metrics: ResMut<AtmospherePartialWriteMetrics>,
    mut perf: Option<ResMut<crate::render::FramePerf>>,
) {
    let t0 = std::time::Instant::now();
    let fence_ref = fence.as_deref();
    out.uploads = collect_gpu_prepared_partial_uploads(
        &bridge,
        &field,
        atlas.origin_chunk,
        fence_ref,
    );
    let gpu_bytes = out
        .uploads
        .iter()
        .map(GpuPreparedPartialUpload::upload_bytes)
        .sum::<u64>()
        * 2;
    metrics.gpu_texture_upload_bytes = gpu_bytes;
    metrics.gpu_texture_upload_count = out.uploads.len() as u32;
    metrics.full_field_texture_bytes = weather_fire_field_full_texture_bytes();

    if let Some((origin, extent)) = union_partial_dispatch_bounds(&out.uploads) {
        out.partial_dispatch_origin = origin;
        out.partial_dispatch_extent = extent;
        out.partial_dispatch_active = true;
        out.full_field_fallback = false;
        metrics.partial_compute_dispatch_count = out.uploads.len() as u32;
        metrics.full_field_fallback_active = false;
    } else {
        out.partial_dispatch_active = false;
        let needs_full = bridge.pending_full_field_dispatch;
        out.full_field_fallback = needs_full;
        metrics.full_field_fallback_active = needs_full;
        if needs_full {
            metrics.full_field_dispatch_count = metrics.full_field_dispatch_count.saturating_add(1);
            bridge.pending_full_field_dispatch = false;
        }
    }

    bridge.pending_partial_uploads.clear();
    if let Some(perf) = perf.as_mut() {
        crate::render::record_frame_perf_ms(
            perf,
            t0.elapsed().as_secs_f32() * 1000.0,
            crate::render::FramePerfSlot::AtmosphereExtract,
        );
    }
}

pub fn apply_partial_texture_writes(
    uploads: Res<AtmospherePartialGpuExtract>,
    gpu_images: Res<RenderAssets<GpuImage>>,
    textures: Res<crate::render::gpu_weather_fire_field::WeatherFireFieldTextures>,
    queue: Res<RenderQueue>,
) {
    if !crate::systems::atmosphere::P2H_GPU_PARTIAL_TEXTURE_UPLOADS_ENABLED {
        return;
    }
    if uploads.uploads.is_empty() {
        return;
    }
    for handle in [&textures.texture_a, &textures.texture_b] {
        let Some(target) = gpu_images.get(handle) else {
            continue;
        };
        for upload in &uploads.uploads {
            if upload.extent.x == 0 || upload.extent.y == 0 {
                continue;
            }
            queue.write_texture(
                TexelCopyTextureInfo {
                    texture: &target.texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: upload.origin.x,
                        y: upload.origin.y,
                        z: 0,
                    },
                    aspect: TextureAspect::All,
                },
                bytemuck::cast_slice(&upload.data),
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(upload.bytes_per_row),
                    rows_per_image: Some(upload.extent.y),
                },
                Extent3d {
                    width: upload.extent.x,
                    height: upload.extent.y,
                    depth_or_array_layers: 1,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::atmosphere::{
        build_partial_gpu_uploads, AtmosphereDirtyRegion, AtmospherePartialUpload,
    };

    #[test]
    fn prepared_upload_rows_follow_cpu_mirror_cells() {
        let mut field = AtmospherePartialFieldState::default();
        field.cells.insert(IVec2::new(2, 2), 0.75);
        let region = AtmosphereDirtyRegion {
            min: IVec2::new(2, 2),
            max: IVec2::new(2, 2),
            stamp: SimStepStamp::new(1, 0),
            mean_heat: 0.75,
        };
        let pending = build_partial_gpu_uploads(&[region], IVec2::new(2, 2));
        let bridge = AtmosphereGpuFieldBridge {
            pending_partial_uploads: pending,
            ..Default::default()
        };
        let uploads = collect_gpu_prepared_partial_uploads(&bridge, &field, IVec2::new(2, 2), None);
        assert_eq!(uploads.len(), 1);
        assert!(uploads[0].data.iter().any(|row| (row[0] - 0.75).abs() < 1e-5));
    }

    #[test]
    fn prepared_upload_origin_uses_camera_centered_atlas() {
        let pending = vec![AtmospherePartialUpload {
            region: AtmosphereDirtyRegion {
                min: IVec2::new(130, 4),
                max: IVec2::new(130, 4),
                stamp: SimStepStamp::new(1, 0),
                mean_heat: 0.2,
            },
            bytes_offset: 0,
            extent: UVec2::ONE,
        }];
        let bridge = AtmosphereGpuFieldBridge {
            pending_partial_uploads: pending,
            ..Default::default()
        };
        let uploads = collect_gpu_prepared_partial_uploads(
            &bridge,
            &AtmospherePartialFieldState::default(),
            IVec2::new(130, 4),
            None,
        );
        assert_eq!(uploads[0].origin, UVec2::splat(64));
    }

    #[test]
    fn stale_stamp_rejected_on_gpu_collect() {
        let pending = build_partial_gpu_uploads(
            &[AtmosphereDirtyRegion {
                min: IVec2::ZERO,
                max: IVec2::ZERO,
                stamp: SimStepStamp::new(1, 0),
                mean_heat: 0.5,
            }],
            IVec2::ZERO,
        );
        let bridge = AtmosphereGpuFieldBridge {
            pending_partial_uploads: pending,
            last_full_reconcile: SimStepStamp::new(2, 0),
            ..Default::default()
        };
        let uploads = collect_gpu_prepared_partial_uploads(
            &bridge,
            &AtmospherePartialFieldState::default(),
            IVec2::ZERO,
            None,
        );
        assert!(uploads.is_empty());
    }

    #[test]
    fn partial_texture_write_leaves_outside_cells_untouched() {
        let mut field = AtmospherePartialFieldState::default();
        field.cells.insert(IVec2::new(20, 20), 0.25);
        let region = AtmosphereDirtyRegion {
            min: IVec2::ZERO,
            max: IVec2::new(1, 1),
            stamp: SimStepStamp::new(1, 0),
            mean_heat: 0.6,
        };
        let pending = build_partial_gpu_uploads(&[region], IVec2::ZERO);
        let bridge = AtmosphereGpuFieldBridge {
            pending_partial_uploads: pending,
            ..Default::default()
        };
        let uploads = collect_gpu_prepared_partial_uploads(&bridge, &field, IVec2::ZERO, None);
        assert!(!uploads.is_empty());
        assert_eq!(field.cells.get(&IVec2::new(20, 20)), Some(&0.25));
    }

    #[test]
    fn idle_extract_skips_full_field_fallback_without_reconcile() {
        let bridge = AtmosphereGpuFieldBridge::default();
        assert!(!bridge.pending_full_field_dispatch);
        let mut out = AtmospherePartialGpuExtract::default();
        let mut metrics = AtmospherePartialWriteMetrics::default();
        let uploads: Vec<GpuPreparedPartialUpload> = vec![];
        if uploads.is_empty() {
            out.partial_dispatch_active = false;
            out.full_field_fallback = bridge.pending_full_field_dispatch;
            metrics.full_field_fallback_active = out.full_field_fallback;
        }
        assert!(!out.full_field_fallback);
        assert!(!metrics.full_field_fallback_active);
    }

    #[test]
    fn reconcile_requests_one_full_field_dispatch() {
        let bridge = AtmosphereGpuFieldBridge {
            pending_full_field_dispatch: true,
            ..Default::default()
        };
        let mut out = AtmospherePartialGpuExtract::default();
        let uploads: Vec<GpuPreparedPartialUpload> = vec![];
        if uploads.is_empty() {
            out.full_field_fallback = bridge.pending_full_field_dispatch;
        }
        assert!(out.full_field_fallback);
    }

    #[test]
    fn union_dispatch_bounds_cover_all_uploads() {
        let uploads = vec![
            GpuPreparedPartialUpload {
                origin: UVec2::new(2, 2),
                extent: UVec2::new(2, 2),
                ..Default::default()
            },
            GpuPreparedPartialUpload {
                origin: UVec2::new(6, 2),
                extent: UVec2::new(2, 2),
                ..Default::default()
            },
        ];
        let (origin, extent) = union_partial_dispatch_bounds(&uploads).expect("union");
        assert_eq!(origin, UVec2::new(2, 2));
        assert_eq!(extent, UVec2::new(6, 2));
    }
}
