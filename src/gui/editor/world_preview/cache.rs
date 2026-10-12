//! Per-chunk CPU preview payloads (RGBA8 bytes); egui ColorImage retired.

use bevy::prelude::*;
use std::collections::HashMap;

/// Cached RGBA for one chunk’s preview tile rect.
#[derive(Clone)]
pub struct ChunkPreviewCache {
    pub version: u64,
    /// Row-major RGBA8, length = width * height * 4.
    pub image: Vec<u8>,
    pub width: u32,
    pub height: u32,
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
    let mut img = vec![0u8; sx * sy * 4];
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
            let o = 4 * (ly * sx + lx);
            img[o..o + 4].copy_from_slice(&data[i..i + 4]);
        }
    }
    caches.chunks.insert(
        coord,
        ChunkPreviewCache {
            version,
            image: img,
            width: size.x,
            height: size.y,
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
        let o = 4 * (1 * 2 + 1);
        assert_eq!(&entry.image[o..o + 4], &[200, 10, 20, 255]);
    }
}
