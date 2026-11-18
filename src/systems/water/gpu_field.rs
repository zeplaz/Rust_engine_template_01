//! DF-3 — per-cell surface water → GPU texture (sim pack; render samples only).
//!
//! Channels (Rgba8Unorm):
//! - R = depth01 (depth / [`DEPTH_SCALE_M`])
//! - G = vx01 (0.5 + 0.5·signed flow x)
//! - B = vy01
//! - A = foam01 (speed / turbulence proxy)

use std::time::Instant;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageFilterMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureUsages,
};

use super::static_depth::WorldStaticWaterDepth;
use super::surface_water::SurfaceWaterField;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Depth that maps to R=1.0 in the field texture.
pub const DEPTH_SCALE_M: f32 = 24.0;
/// Flow speed (m/s) that maps foam / velocity encoding toward 1.
pub const FLOW_SPEED_SCALE: f32 = 4.0;
/// Soft cap on texture edge so upload stays frame-budget friendly.
pub const GPU_FIELD_MAX_EDGE: u32 = 256;

/// CPU book + GPU image handle for the ROI field (sole writer: [`publish_surface_water_gpu_field_system`]).
#[derive(Resource, Debug, Clone)]
pub struct SurfaceWaterGpuField {
    pub image: Handle<Image>,
    pub origin_x: i32,
    pub origin_z: i32,
    pub width: u32,
    pub height: u32,
    pub step: u32,
    pub revision: u64,
    pub wired: bool,
    /// Last pack cost (ms) — matrix `render.water_upload_ms_*`.
    pub upload_ms: f32,
    pub wet_texels: u32,
    /// Inclusive dirty rectangle in texture space (for partial rewrite).
    pub dirty_x0: u32,
    pub dirty_z0: u32,
    pub dirty_x1: u32,
    pub dirty_z1: u32,
}

impl Default for SurfaceWaterGpuField {
    fn default() -> Self {
        Self {
            image: Handle::default(),
            origin_x: 0,
            origin_z: 0,
            width: 0,
            height: 0,
            step: 1,
            revision: 0,
            wired: false,
            upload_ms: 0.0,
            wet_texels: 0,
            dirty_x0: 0,
            dirty_z0: 0,
            dirty_x1: 0,
            dirty_z1: 0,
        }
    }
}

/// Extracted meta for the render world (no hydro query in RenderApp).
#[derive(Resource, Clone, Debug, Default, ExtractResource)]
#[extract_app(bevy::render::RenderApp)]
pub struct SurfaceWaterGpuFieldMeta {
    pub image: Handle<Image>,
    pub origin_x: i32,
    pub origin_z: i32,
    pub width: u32,
    pub height: u32,
    pub step: u32,
    pub revision: u64,
    pub wired: bool,
    pub upload_ms: f32,
    pub wet_texels: u32,
}

impl SurfaceWaterGpuFieldMeta {
    #[must_use]
    pub fn from_book(book: &SurfaceWaterGpuField) -> Self {
        Self {
            image: book.image.clone(),
            origin_x: book.origin_x,
            origin_z: book.origin_z,
            width: book.width,
            height: book.height,
            step: book.step,
            revision: book.revision,
            wired: book.wired,
            upload_ms: book.upload_ms,
            wet_texels: book.wet_texels,
        }
    }
}

#[must_use]
pub fn field_texel_step(width: u32, height: u32) -> u32 {
    let edge = width.max(height).max(1);
    if edge <= GPU_FIELD_MAX_EDGE {
        1
    } else {
        edge.div_ceil(GPU_FIELD_MAX_EDGE).max(1)
    }
}

#[inline]
fn encode_signed01(v: f32) -> u8 {
    ((0.5 + 0.5 * v.clamp(-1.0, 1.0)) * 255.0).round() as u8
}

#[inline]
fn encode_unit01(v: f32) -> u8 {
    (v.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Pack one texel from depth + discharge.
#[must_use]
pub fn pack_water_texel(depth_m: f32, qx_m3s: f32, qy_m3s: f32, dx_m: f32) -> [u8; 4] {
    if depth_m < 0.05 {
        return [0, 128, 128, 0];
    }
    let depth01 = (depth_m / DEPTH_SCALE_M).clamp(0.0, 1.0);
    let cell = (depth_m * dx_m.max(1e-3)).max(1e-3);
    let vx = (qx_m3s / cell / FLOW_SPEED_SCALE).clamp(-1.0, 1.0);
    let vy = (qy_m3s / cell / FLOW_SPEED_SCALE).clamp(-1.0, 1.0);
    let speed = (vx * vx + vy * vy).sqrt();
    let foam = (speed * 1.35).clamp(0.0, 1.0);
    [
        encode_unit01(depth01),
        encode_signed01(vx),
        encode_signed01(vy),
        encode_unit01(foam),
    ]
}

/// Full ROI pack (possibly subsampled). Returns (tex_w, tex_h, step, rgba, wet_count).
#[must_use]
pub fn pack_surface_water_field_rgba8(field: &SurfaceWaterField) -> (u32, u32, u32, Vec<u8>, u32) {
    if !field.wired || field.width == 0 || field.height == 0 {
        return (0, 0, 1, Vec::new(), 0);
    }
    let (tw, th, step, data, wet, _, _) = pack_composite_water_field_rgba8(Some(field), None);
    (tw, th, step, data, wet)
}

/// Composite pack: world static depth ⊕ live ROI (ROI overwrites). Same Rgba8 contract.
/// Returns `(tex_w, tex_h, step, rgba, wet, origin_x, origin_z)`.
#[must_use]
pub fn pack_composite_water_field_rgba8(
    field: Option<&SurfaceWaterField>,
    static_depth: Option<&WorldStaticWaterDepth>,
) -> (u32, u32, u32, Vec<u8>, u32, i32, i32) {
    let roi = field.filter(|f| f.wired && f.width > 0 && f.height > 0);
    let st = static_depth.filter(|s| s.width > 0 && s.height > 0 && s.depth_m.len() >= (s.width * s.height) as usize);

    let (origin_x, origin_z, world_w, world_h) = match (roi, st) {
        (Some(f), Some(s)) => {
            let x0 = f.origin_x.min(0);
            let z0 = f.origin_z.min(0);
            let x1 = (f.origin_x + f.width as i32).max(s.width as i32);
            let z1 = (f.origin_z + f.height as i32).max(s.height as i32);
            // Prefer full static map extent when present (covers play world).
            if s.width >= f.width && s.height >= f.height && f.origin_x >= 0 && f.origin_z >= 0 {
                (0, 0, s.width, s.height)
            } else {
                let w = (x1 - x0).max(1) as u32;
                let h = (z1 - z0).max(1) as u32;
                (x0, z0, w, h)
            }
        }
        (Some(f), None) => (f.origin_x, f.origin_z, f.width, f.height),
        (None, Some(s)) => (0, 0, s.width, s.height),
        (None, None) => return (0, 0, 1, Vec::new(), 0, 0, 0),
    };

    let step = field_texel_step(world_w, world_h);
    let tw = world_w.div_ceil(step).max(1);
    let th = world_h.div_ceil(step).max(1);
    let mut data = vec![0u8; (tw * th * 4) as usize];
    let dx = roi
        .map(|f| f.dx_m.max(TILE_METRES_GAMEPLAY))
        .unwrap_or(TILE_METRES_GAMEPLAY);
    let mut wet = 0u32;

    for tz in 0..th {
        for tx in 0..tw {
            let wx = origin_x + (tx * step) as i32;
            let wz = origin_z + (tz * step) as i32;
            let mut d = 0.0_f32;
            let mut qx = 0.0_f32;
            let mut qy = 0.0_f32;
            if let Some(s) = st {
                if wx >= 0 && wz >= 0 {
                    if let Some(i) = s.idx(wx as u32, wz as u32) {
                        d = s.depth_m.get(i).copied().unwrap_or(0.0);
                    }
                }
            }
            if let Some(f) = roi {
                let lx = wx - f.origin_x;
                let lz = wz - f.origin_z;
                if lx >= 0 && lz >= 0 && (lx as u32) < f.width && (lz as u32) < f.height {
                    let i = f.idx(lx as u32, lz as u32);
                    d = f.depth_m.get(i).copied().unwrap_or(d);
                    qx = f.qx_m3s.get(i).copied().unwrap_or(0.0);
                    qy = f.qy_m3s.get(i).copied().unwrap_or(0.0);
                }
            }
            let px = pack_water_texel(d, qx, qy, dx);
            if px[0] > 8 {
                wet += 1;
            }
            let o = ((tz * tw + tx) * 4) as usize;
            data[o..o + 4].copy_from_slice(&px);
        }
    }
    (tw, th, step, data, wet, origin_x, origin_z)
}

fn make_field_image(w: u32, h: u32, data: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.texture_descriptor.usage |= TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST;
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        label: Some("surface_water_gpu_field".into()),
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..ImageSamplerDescriptor::default()
    });
    image
}

/// Sole writer of [`SurfaceWaterGpuField`] + its [`Image`] bytes.
pub fn publish_surface_water_gpu_field_system(
    field: Res<SurfaceWaterField>,
    static_depth: Option<Res<WorldStaticWaterDepth>>,
    mut book: ResMut<SurfaceWaterGpuField>,
    mut meta: ResMut<SurfaceWaterGpuFieldMeta>,
    mut images: ResMut<Assets<Image>>,
) {
    let t0 = Instant::now();
    let st = static_depth.as_deref();
    let (tw, th, step, data, wet, ox, oz) =
        pack_composite_water_field_rgba8(Some(field.as_ref()), st);
    if tw == 0 || th == 0 || data.is_empty() {
        if book.wired {
            book.wired = false;
            book.width = 0;
            book.height = 0;
            book.wet_texels = 0;
            book.revision = book.revision.wrapping_add(1);
            *meta = SurfaceWaterGpuFieldMeta::from_book(&book);
        }
        book.upload_ms = t0.elapsed().as_secs_f32() * 1000.0;
        meta.upload_ms = book.upload_ms;
        return;
    }

    let size_changed = book.width != tw || book.height != th || book.image == Handle::default();
    let handle = book.image.clone();
    let reused = !size_changed && images.contains(&handle);
    if reused {
        if let Some(mut img) = images.get_mut(&handle) {
            // Replace data + retouch asset_usage so RenderAssets re-uploads (tile_world pattern).
            img.data = Some(data);
            img.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
        }
    } else {
        book.image = images.add(make_field_image(tw, th, data));
    }

    book.origin_x = ox;
    book.origin_z = oz;
    book.width = tw;
    book.height = th;
    book.step = step;
    book.wired = true;
    book.wet_texels = wet;
    book.dirty_x0 = 0;
    book.dirty_z0 = 0;
    book.dirty_x1 = tw.saturating_sub(1);
    book.dirty_z1 = th.saturating_sub(1);
    book.revision = book.revision.wrapping_add(1);
    book.upload_ms = t0.elapsed().as_secs_f32() * 1000.0;
    *meta = SurfaceWaterGpuFieldMeta::from_book(&book);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_texel_encodes_depth_and_flow() {
        let dry = pack_water_texel(0.0, 0.0, 0.0, 16.0);
        assert_eq!(dry[0], 0);
        let deep = pack_water_texel(12.0, 80.0, 0.0, 16.0);
        assert!(deep[0] > 100, "deep water elevates R");
        assert!(deep[1] != 128, "eastward discharge shifts G");
        assert!(deep[3] > 0, "moving water carries foam");
    }

    #[test]
    fn pack_field_respects_max_edge() {
        let mut field = SurfaceWaterField::default();
        let w = 320_u32;
        let h = 320_u32;
        let n = (w * h) as usize;
        field.allocate(0, 0, w, h, vec![0.0; n], 0.0);
        for d in &mut field.depth_m {
            *d = 1.0;
        }
        let (tw, th, step, data, wet) = pack_surface_water_field_rgba8(&field);
        assert!(step >= 2);
        assert!(tw <= GPU_FIELD_MAX_EDGE && th <= GPU_FIELD_MAX_EDGE);
        assert_eq!(data.len(), (tw * th * 4) as usize);
        assert!(wet > 0);
    }
}
