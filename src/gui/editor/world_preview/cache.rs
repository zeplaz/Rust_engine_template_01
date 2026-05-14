//! Per-chunk CPU preview payloads (egui image); populated incrementally as chunk-diff preview matures.

use bevy::prelude::*;
use bevy_egui::egui;
use std::collections::HashMap;

/// Cached RGBA for one chunk’s preview tile rect (Stage-2+); today raster still writes the global texture.
#[derive(Clone)]
pub struct ChunkPreviewCache {
    pub version: u64,
    pub image: egui::ColorImage,
    pub dirty: bool,
}

#[derive(Resource, Default)]
pub struct WorldPreviewChunkCaches {
    pub chunks: HashMap<IVec2, ChunkPreviewCache>,
}

/// Copy one chunk’s tile rect from a full-world RGBA buffer into the per-chunk cache.
pub(crate) fn sync_chunk_preview_cache(
    coord: IVec2,
    size: UVec2,
    data: &[u8],
    tex_w: usize,
    tex_h: usize,
    caches: &mut WorldPreviewChunkCaches,
    version: u64,
) {
    let sx = size.x as usize;
    let sy = size.y as usize;
    let x0 = coord.x as usize * sx;
    let y0 = coord.y as usize * sy;
    let mut img = egui::ColorImage::new(
        [sx, sy],
        vec![egui::Color32::TRANSPARENT; sx * sy],
    );
    for ly in 0..sy {
        for lx in 0..sx {
            let tx = x0 + lx;
            let ty = y0 + ly;
            if tx >= tex_w || ty >= tex_h {
                continue;
            }
            let i = 4 * (ty * tex_w + tx);
            if i + 3 >= data.len() {
                continue;
            }
            img[(lx, ly)] = egui::Color32::from_rgba_unmultiplied(
                data[i],
                data[i + 1],
                data[i + 2],
                data[i + 3],
            );
        }
    }
    caches.chunks.insert(
        coord,
        ChunkPreviewCache {
            version,
            image: img,
            dirty: false,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_cache_copies_subrect_from_full_buffer() {
        let tex_w = 4usize;
        let tex_h = 4usize;
        let mut data = vec![0u8; 4 * tex_w * tex_h];
        data[4 * (1 * tex_w + 1)] = 200;
        data[4 * (1 * tex_w + 1) + 1] = 10;
        data[4 * (1 * tex_w + 1) + 2] = 20;
        data[4 * (1 * tex_w + 1) + 3] = 255;

        let mut caches = WorldPreviewChunkCaches::default();
        sync_chunk_preview_cache(
            IVec2::ZERO,
            UVec2::new(2, 2),
            &data,
            tex_w,
            tex_h,
            &mut caches,
            7,
        );
        let entry = caches.chunks.get(&IVec2::ZERO).expect("chunk cache");
        assert_eq!(entry.version, 7);
        assert_eq!(entry.image[(1, 1)], egui::Color32::from_rgba_unmultiplied(200, 10, 20, 255));
    }
}
