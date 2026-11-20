//! DF-3 / WM-5 -- the map-wide surface-water field -> GPU texture (sim pack; render samples only).
//!
//! The texture is packed from [`SurfaceWaterField`] alone (depth and flux; the static seed is gone
//! by the time anything is drawn), one texel per tile up to [`GPU_FIELD_MAX_EDGE`]. After the first
//! full pack only the texels of the field's dirty chunks are rewritten.
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

use super::field_feed::WaterFieldDirtyFeed;
use super::surface_water::SurfaceWaterField;
use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

/// Depth that maps to R=1.0 in the field texture.
/// Depth at encoded R = 1. Encoding is `sqrt(depth / DEPTH_SCALE_M)` (shaders square it back):
/// fine 8-bit steps near zero for rivers and films, and deep reservoirs no longer saturate (24 m
/// linear capped a 40 m reservoir, so its surface drew 16 m low and a breach drawdown was
/// invisible until it fell under 24 m).
pub const DEPTH_SCALE_M: f32 = 96.0;
/// Flow speed (m/s) that maps foam / velocity encoding toward 1.
pub const FLOW_SPEED_SCALE: f32 = 4.0;
/// Largest texture edge packed at full resolution (one texel per tile). Bigger maps subsample.
pub const GPU_FIELD_MAX_EDGE: u32 = 1024;

/// A texel shows water once its R reaches this (matches the shaders' discard, depth01 < 0.02).
const WET_TEXEL_R: u8 = 5;

/// CPU book + GPU image handle for the field (sole writer: [`publish_surface_water_gpu_field_system`]).
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
    /// Inclusive rectangle (texture space) rewritten by the latest publish.
    pub dirty_x0: u32,
    pub dirty_z0: u32,
    pub dirty_x1: u32,
    pub dirty_z1: u32,
    /// Last [`WaterFieldDirtyFeed`] frame applied.
    feed_seq: u64,
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
            feed_seq: 0,
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

/// Tiles per texel along each edge: 1 up to [`GPU_FIELD_MAX_EDGE`], coarser beyond it.
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
    let depth01 = (depth_m / DEPTH_SCALE_M).clamp(0.0, 1.0).sqrt();
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

#[inline]
fn field_texel(field: &SurfaceWaterField, lx: u32, lz: u32, dx_m: f32) -> [u8; 4] {
    let i = field.idx(lx.min(field.width - 1), lz.min(field.height - 1));
    pack_water_texel(
        field.depth_m.get(i).copied().unwrap_or(0.0),
        field.qx_m3s.get(i).copied().unwrap_or(0.0),
        field.qy_m3s.get(i).copied().unwrap_or(0.0),
        dx_m,
    )
}

/// Texture size and tile-per-texel step for a field.
#[must_use]
pub fn field_texture_dims(field: &SurfaceWaterField) -> (u32, u32, u32) {
    let step = field_texel_step(field.width, field.height);
    (field.width.div_ceil(step).max(1), field.height.div_ceil(step).max(1), step)
}

/// Pack the whole field (full resolution up to [`GPU_FIELD_MAX_EDGE`]).
/// Returns `(tex_w, tex_h, step, rgba, wet_texels)`; empty when the field is not wired.
#[must_use]
pub fn pack_surface_water_field_rgba8(field: &SurfaceWaterField) -> (u32, u32, u32, Vec<u8>, u32) {
    if !field.wired || field.width == 0 || field.height == 0 {
        return (0, 0, 1, Vec::new(), 0);
    }
    let (tw, th, step) = field_texture_dims(field);
    let dx = field.dx_m.max(TILE_METRES_GAMEPLAY);
    let mut data = vec![0u8; (tw * th * 4) as usize];
    let mut wet = 0u32;
    for tz in 0..th {
        for tx in 0..tw {
            let px = field_texel(field, tx * step, tz * step, dx);
            wet += u32::from(px[0] >= WET_TEXEL_R);
            let o = ((tz * tw + tx) * 4) as usize;
            data[o..o + 4].copy_from_slice(&px);
        }
    }
    (tw, th, step, data, wet)
}

/// Rewrite the texels of one sim chunk in place. Returns the signed change of the wet-texel count
/// and the inclusive texture rect touched (`None` when the chunk is outside the texture).
fn patch_chunk_texels(
    field: &SurfaceWaterField,
    data: &mut [u8],
    tw: u32,
    th: u32,
    step: u32,
    chunk: IVec2,
) -> Option<(i32, [u32; 4])> {
    let cs = CHUNK_TILES_SIM as i32;
    // Chunk cell range in field-local cells.
    let lx0 = (chunk.x * cs - field.origin_x).max(0);
    let lz0 = (chunk.y * cs - field.origin_z).max(0);
    let lx1 = ((chunk.x + 1) * cs - field.origin_x).min(field.width as i32);
    let lz1 = ((chunk.y + 1) * cs - field.origin_z).min(field.height as i32);
    if lx0 >= lx1 || lz0 >= lz1 {
        return None;
    }
    let s = step as i32;
    // Texels whose sampled cell (t * step) falls inside the chunk range.
    let tx0 = (lx0 + s - 1) / s;
    let tz0 = (lz0 + s - 1) / s;
    let tx1 = ((lx1 + s - 1) / s).min(tw as i32); // exclusive
    let tz1 = ((lz1 + s - 1) / s).min(th as i32);
    if tx0 >= tx1 || tz0 >= tz1 {
        return None;
    }
    let dx = field.dx_m.max(TILE_METRES_GAMEPLAY);
    let mut wet_delta = 0i32;
    for tz in tz0..tz1 {
        for tx in tx0..tx1 {
            let px = field_texel(field, tx as u32 * step, tz as u32 * step, dx);
            let o = ((tz as u32 * tw + tx as u32) * 4) as usize;
            let was = data[o] >= WET_TEXEL_R;
            wet_delta += i32::from(px[0] >= WET_TEXEL_R) - i32::from(was);
            data[o..o + 4].copy_from_slice(&px);
        }
    }
    Some((wet_delta, [tx0 as u32, tz0 as u32, tx1 as u32 - 1, tz1 as u32 - 1]))
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
///
/// First sight (or a resized field, or a skipped feed frame) packs the whole field; afterwards
/// only the texels of the chunks the field reported as moved are rewritten.
pub fn publish_surface_water_gpu_field_system(
    field: Res<SurfaceWaterField>,
    feed: Res<WaterFieldDirtyFeed>,
    mut book: ResMut<SurfaceWaterGpuField>,
    mut meta: ResMut<SurfaceWaterGpuFieldMeta>,
    mut images: ResMut<Assets<Image>>,
) {
    if !field.wired || field.width == 0 || field.height == 0 {
        if book.wired {
            book.wired = false;
            book.width = 0;
            book.height = 0;
            book.wet_texels = 0;
            book.revision = book.revision.wrapping_add(1);
            *meta = SurfaceWaterGpuFieldMeta::from_book(&book);
        }
        return;
    }
    let (tw, th, step) = field_texture_dims(&field);
    let size_ok = book.wired
        && book.width == tw
        && book.height == th
        && book.step == step
        && (book.origin_x, book.origin_z) == (field.origin_x, field.origin_z)
        && images.contains(&book.image);
    let delta = if size_ok { feed.chunks_since(book.feed_seq) } else { None };
    if size_ok && delta.is_some_and(<[IVec2]>::is_empty) {
        return;
    }
    let t0 = Instant::now();
    book.feed_seq = feed.seq;

    if let Some(chunks) = delta {
        let handle = book.image.clone();
        let mut wet = book.wet_texels as i32;
        let mut rect: Option<[u32; 4]> = None;
        if let Some(mut img) = images.get_mut(&handle) {
            if let Some(data) = img.data.as_mut() {
                for &c in chunks {
                    if let Some((d, r)) = patch_chunk_texels(&field, data, tw, th, step, c) {
                        wet += d;
                        rect = Some(rect.map_or(r, |u| [u[0].min(r[0]), u[1].min(r[1]), u[2].max(r[2]), u[3].max(r[3])]));
                    }
                }
            }
            if rect.is_some() {
                // Retouch asset usage so the render world re-uploads the image.
                img.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
            }
        }
        if let Some(r) = rect {
            book.wet_texels = wet.max(0) as u32;
            book.dirty_x0 = r[0];
            book.dirty_z0 = r[1];
            book.dirty_x1 = r[2];
            book.dirty_z1 = r[3];
            book.revision = book.revision.wrapping_add(1);
            book.upload_ms = t0.elapsed().as_secs_f32() * 1000.0;
            *meta = SurfaceWaterGpuFieldMeta::from_book(&book);
        }
        return;
    }

    let (tw, th, step, data, wet) = pack_surface_water_field_rgba8(&field);
    if size_ok {
        if let Some(mut img) = images.get_mut(&book.image.clone()) {
            img.data = Some(data);
            img.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
        }
    } else {
        book.image = images.add(make_field_image(tw, th, data));
    }
    book.origin_x = field.origin_x;
    book.origin_z = field.origin_z;
    book.width = tw;
    book.height = th;
    book.step = step;
    book.wired = true;
    book.wet_texels = wet;
    book.dirty_x0 = 0;
    book.dirty_z0 = 0;
    book.dirty_x1 = tw - 1;
    book.dirty_z1 = th - 1;
    book.revision = book.revision.wrapping_add(1);
    book.upload_ms = t0.elapsed().as_secs_f32() * 1000.0;
    *meta = SurfaceWaterGpuFieldMeta::from_book(&book);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_field(w: u32, h: u32) -> SurfaceWaterField {
        let mut field = SurfaceWaterField::default();
        field.allocate(0, 0, w, h, vec![0.0; (w * h) as usize], 0.0);
        field
    }

    #[test]
    fn pack_texel_encodes_depth_and_flow() {
        let dry = pack_water_texel(0.0, 0.0, 0.0, 16.0);
        assert_eq!(dry[0], 0);
        let deep = pack_water_texel(12.0, 80.0, 0.0, 16.0);
        assert!(deep[0] > 80, "deep water elevates R");
        // A reservoir draining from 40 m to 30 m must read as a change (24 m linear saturated).
        assert!(pack_water_texel(40.0, 0.0, 0.0, 16.0)[0] > pack_water_texel(30.0, 0.0, 0.0, 16.0)[0] + 5);
        assert!(deep[1] != 128, "eastward discharge shifts G");
        assert!(deep[3] > 0, "moving water carries foam");
    }

    /// WM-5: a 320 map packs one texel per tile (it used to drop to every second tile).
    #[test]
    fn pack_is_full_res_at_320() {
        let mut field = flat_field(320, 320);
        let marked = field.idx(319, 301);
        field.depth_m[marked] = 3.0;
        let (tw, th, step, data, wet) = pack_surface_water_field_rgba8(&field);
        assert_eq!((tw, th, step), (320, 320, 1));
        assert_eq!(data.len(), 320 * 320 * 4);
        assert_eq!(wet, 1, "the odd-coordinate cell survives (a 2x subsample would drop it)");
        assert!(data[((301 * 320 + 319) * 4) as usize] > 0);
        assert_eq!(field_texel_step(2048, 100), 2, "past the cap the pack still subsamples");
    }

    /// WM-5: a river cell carries the field's own flux into the texture (static texels had none).
    #[test]
    fn pack_carries_field_flow_on_a_river_cell() {
        let mut field = flat_field(64, 64);
        let i = field.idx(10, 20);
        field.depth_m[i] = 2.0;
        field.qx_m3s[i] = 60.0;
        let (tw, _, _, data, _) = pack_surface_water_field_rgba8(&field);
        let o = ((20 * tw + 10) * 4) as usize;
        assert!(data[o] > 0, "depth packed");
        assert!(data[o + 1] > 128, "eastward flux shifts G above rest");
        assert_eq!(data[o + 2], 128, "no north-south flux stays at rest");
        assert!(data[o + 3] > 0, "moving water foams");
    }

    #[test]
    fn chunk_patch_matches_a_full_pack_and_tracks_wet_count() {
        let mut field = flat_field(96, 96);
        let (tw, th, step, mut data, wet0) = pack_surface_water_field_rgba8(&field);
        assert_eq!(wet0, 0);
        for (x, z) in [(40, 5), (41, 5), (70, 70)] {
            let i = field.idx(x, z);
            field.depth_m[i] = 1.5;
            field.qy_m3s[i] = -20.0;
        }
        let mut wet = 0i32;
        for c in [IVec2::new(1, 0), IVec2::new(2, 2)] {
            let (d, r) = patch_chunk_texels(&field, &mut data, tw, th, step, c).expect("inside");
            wet += d;
            assert!(r[0] <= r[2] && r[1] <= r[3]);
        }
        let (_, _, _, full, full_wet) = pack_surface_water_field_rgba8(&field);
        assert_eq!(data, full, "in-place patch equals a full repack");
        assert_eq!(wet as u32, full_wet);
    }
}
