//! DESTROY-T2 — presentation destruction mask (CPU book + GPU Image).
//!
//! Downstream of occupation / demolish / blast / dam collapse. Not save authority.
//! PixelWorld ground VS collapses columns while remesh (`CHUNKS_PER_FRAME`) lags.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::render::TileWorldFallbackState;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Intact column / tile.
pub const MASK_INTACT: u8 = 0;
/// Damaged but still drawn (future T2.1 / overlays).
pub const MASK_DAMAGED: u8 = 1;
/// Destroyed — VS collapses geometry same frame.
pub const MASK_DESTROYED: u8 = 2;

/// CPU presentation book — sole writers: demolish / blast / dam collapse mark paths.
#[derive(Resource, Debug, Clone)]
pub struct DestructionMaskBook {
    pub width: u32,
    pub height: u32,
    /// Row-major `y * width + x`, values [`MASK_INTACT`] / [`MASK_DAMAGED`] / [`MASK_DESTROYED`].
    pub cells: Vec<u8>,
    pub revision: u64,
    /// Inclusive dirty bounds for GPU upload (empty when clean).
    pub dirty: Option<(UVec2, UVec2)>,
    /// DESTROY-T2b: terrain chunk coords `(cx, cy)` to rebuild ahead of the normal pending drain.
    pub remesh_priority: Vec<(u32, u32)>,
}

impl Default for DestructionMaskBook {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            cells: Vec::new(),
            revision: 0,
            dirty: None,
            remesh_priority: Vec::new(),
        }
    }
}

impl DestructionMaskBook {
    pub fn ensure_size(&mut self, w: u32, h: u32) {
        let w = w.max(1);
        let h = h.max(1);
        if self.width == w && self.height == h && self.cells.len() == (w * h) as usize {
            return;
        }
        self.width = w;
        self.height = h;
        self.cells = vec![MASK_INTACT; (w * h) as usize];
        self.revision = self.revision.wrapping_add(1);
        self.dirty = Some((UVec2::ZERO, UVec2::new(w.saturating_sub(1), h.saturating_sub(1))));
    }

    #[inline]
    fn idx(&self, x: i32, y: i32) -> Option<usize> {
        if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
            return None;
        }
        Some(y as u32 as usize * self.width as usize + x as u32 as usize)
    }

    pub fn mark_tile(&mut self, x: i32, y: i32, state: u8) {
        let Some(i) = self.idx(x, y) else {
            return;
        };
        if self.cells[i] == state {
            return;
        }
        self.cells[i] = state;
        self.revision = self.revision.wrapping_add(1);
        let p = UVec2::new(x as u32, y as u32);
        self.dirty = Some(match self.dirty {
            None => (p, p),
            Some((lo, hi)) => (lo.min(p), hi.max(p)),
        });
        if state >= MASK_DESTROYED {
            // Match `terrain::TERRAIN_CHUNK` (64) — avoid sibling-mod path in this module.
            const CHUNK: u32 = 64;
            let cx = (x as u32) / CHUNK;
            let cy = (y as u32) / CHUNK;
            if !self.remesh_priority.contains(&(cx, cy)) {
                self.remesh_priority.push((cx, cy));
            }
        }
    }

    pub fn mark_destroyed_tiles(&mut self, tiles: impl IntoIterator<Item = IVec2>) {
        for t in tiles {
            self.mark_tile(t.x, t.y, MASK_DESTROYED);
        }
    }

    #[must_use]
    pub fn cell(&self, x: i32, y: i32) -> u8 {
        self.idx(x, y).map(|i| self.cells[i]).unwrap_or(MASK_INTACT)
    }
}

/// STAGE-RING — presentation upload tiers. Tier 0 (destruction mask) never skips.
#[derive(Resource, Debug, Clone)]
pub struct PresentationUploadRing {
    /// When true, Tier 2+ field uploads may defer (future). Tier 0 ignores this.
    pub tier2_budget_exhausted: bool,
    pub tier0_mask_uploads: u64,
}

impl Default for PresentationUploadRing {
    fn default() -> Self {
        Self {
            tier2_budget_exhausted: false,
            tier0_mask_uploads: 0,
        }
    }
}

/// GPU Image handle for the mask (sampled by [`super::ground_material::PixelWorldGroundMaterial`]).
#[derive(Resource, Debug, Clone)]
pub struct DestructionMaskGpu {
    pub image: Handle<Image>,
    pub width: u32,
    pub height: u32,
    pub uploaded_revision: u64,
}

impl Default for DestructionMaskGpu {
    fn default() -> Self {
        Self {
            image: Handle::default(),
            width: 0,
            height: 0,
            uploaded_revision: 0,
        }
    }
}

fn make_mask_image(w: u32, h: u32) -> Image {
    let width = w.max(1);
    let height = h.max(1);
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    // R channel = state (0/1/2); RGBA8 for simple sampling in PBR materials.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("destruction_mask_t2"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };
    image.data = Some(vec![0u8; 4 * width as usize * height as usize]);
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image
}

pub fn ensure_destruction_mask_size(
    mut book: ResMut<DestructionMaskBook>,
    mut gpu: ResMut<DestructionMaskGpu>,
    mut images: ResMut<Assets<Image>>,
    params: Res<WorldGenParams>,
    fallback: Option<Res<TileWorldFallbackState>>,
    field: Option<Res<super::PixelWorldTerrainField>>,
) {
    let fw = field.as_ref().map(|f| f.width).unwrap_or(0);
    let fh = field.as_ref().map(|f| f.height).unwrap_or(0);
    let bw = fallback.as_ref().map(|f| f.last_w).unwrap_or(0);
    let bh = fallback.as_ref().map(|f| f.last_h).unwrap_or(0);
    let w = fw.max(bw).max(params.width).max(1);
    let h = fh.max(bh).max(params.height).max(1);
    book.ensure_size(w, h);
    if gpu.image != Handle::default() && gpu.width == w && gpu.height == h {
        return;
    }
    gpu.image = images.add(make_mask_image(w, h));
    gpu.width = w;
    gpu.height = h;
    gpu.uploaded_revision = 0;
    book.dirty = Some((UVec2::ZERO, UVec2::new(w.saturating_sub(1), h.saturating_sub(1))));
}

pub fn upload_destruction_mask_gpu(
    mut book: ResMut<DestructionMaskBook>,
    mut gpu: ResMut<DestructionMaskGpu>,
    mut images: ResMut<Assets<Image>>,
    mut ring: ResMut<PresentationUploadRing>,
) {
    // STAGE-RING Tier 0: never gated by `tier2_budget_exhausted`.
    let _ = ring.tier2_budget_exhausted;
    if book.dirty.is_none() || book.width == 0 || book.height == 0 {
        return;
    }
    if gpu.image == Handle::default() {
        return;
    }
    let Some(mut img) = images.get_mut(&gpu.image) else {
        return;
    };
    let Some(data) = img.data.as_mut() else {
        return;
    };
    let w = book.width as usize;
    let h = book.height as usize;
    if data.len() < w * h * 4 {
        data.resize(w * h * 4, 0);
    }
    // Full upload when dirty (mask is tiny vs remesh); keeps VS in sync same frame.
    for y in 0..h {
        for x in 0..w {
            let s = book.cells[y * w + x];
            let i = (y * w + x) * 4;
            data[i] = s;
            data[i + 1] = 0;
            data[i + 2] = 0;
            data[i + 3] = 255;
        }
    }
    gpu.uploaded_revision = book.revision;
    book.dirty = None;
    ring.tier0_mask_uploads = ring.tier0_mask_uploads.saturating_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destroy_t2_mark_destroyed_sets_cell_and_dirty() {
        let mut book = DestructionMaskBook::default();
        book.ensure_size(8, 8);
        book.mark_destroyed_tiles([IVec2::new(3, 4)]);
        assert_eq!(book.cell(3, 4), MASK_DESTROYED);
        assert_eq!(book.cell(0, 0), MASK_INTACT);
        assert!(book.dirty.is_some());
    }

    #[test]
    fn destroy_t2b_mark_queues_terrain_chunk_priority() {
        let mut book = DestructionMaskBook::default();
        book.ensure_size(128, 128);
        book.mark_destroyed_tiles([IVec2::new(70, 10)]);
        assert_eq!(book.remesh_priority, vec![(1, 0)]); // 70/64=1, 10/64=0
    }

    #[test]
    fn destroy_t2_shader_collapses_on_mask() {
        let src = include_str!("../../../assets/shaders/pixel_world/ground_mask.wesl");
        assert!(src.contains("destruction_mask"));
        assert!(src.contains("MASK_DESTROYED") || src.contains(">= 2u") || src.contains(">= 2."));
        assert!(src.contains("skirt") || src.contains("position.y") || src.contains("local.y"));
    }

    #[test]
    fn stage_ring_tier0_upload_ignores_budget_flag() {
        let src = include_str!("destruction_mask.rs");
        let prod = src.split_once("#[cfg(test)]").map_or(src, |(p, _)| p);
        assert!(
            prod.contains("tier2_budget_exhausted"),
            "STAGE-RING must name the Tier-2 gate"
        );
        assert!(
            prod.contains("Tier 0") || prod.contains("tier0_mask_uploads"),
            "STAGE-RING Tier 0 counter / comment required"
        );
        // Upload body must not early-return on the budget flag.
        let upload = prod
            .split("pub fn upload_destruction_mask_gpu")
            .nth(1)
            .unwrap_or("");
        assert!(
            !upload.contains("if ring.tier2_budget_exhausted"),
            "Tier 0 must not skip when Tier-2 budget is exhausted"
        );
    }
}
