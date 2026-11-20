//! WM-5 -- per-tile wet/depth mask of the map-wide water field, for the strategic map views.
//!
//! One byte per tile (0 = dry, else depth on a coarse scale). Patched per dirty chunk from the
//! field, never rebuilt per change. `changed_chunks` names the chunks whose bytes actually moved,
//! so a map raster repaints only those.

use bevy::prelude::*;

use super::field_feed::WaterFieldDirtyFeed;
use super::surface_water::SurfaceWaterField;
use super::water_runtime::WET_DEPTH_M;
use crate::terrain::world_scale_contract::CHUNK_TILES_SIM;

/// Depth (m) that maps to the top of the mask scale.
pub const WET_MASK_DEPTH_M: f32 = 8.0;

#[derive(Resource, Debug, Default, Clone)]
pub struct WaterFieldWetMask {
    pub width: u32,
    pub height: u32,
    /// Row-major `width x height`; 0 = dry, 1..=255 = depth / [`WET_MASK_DEPTH_M`].
    pub level: Vec<u8>,
    /// Bumps whenever any byte changed.
    pub revision: u64,
    /// Chunks whose bytes changed in the latest update (cleared each update).
    pub changed_chunks: Vec<IVec2>,
    feed_seq: u64,
}

#[inline]
fn level_of(depth_m: f32) -> u8 {
    if depth_m < WET_DEPTH_M {
        0
    } else {
        1 + ((depth_m / WET_MASK_DEPTH_M).clamp(0.0, 1.0) * 254.0).round() as u8
    }
}

impl WaterFieldWetMask {
    #[must_use]
    pub fn is_ready(&self) -> bool {
        self.width > 0 && self.level.len() == (self.width * self.height) as usize
    }

    #[must_use]
    pub fn level_at(&self, x: u32, y: u32) -> u8 {
        if x < self.width && y < self.height {
            self.level.get((y * self.width + x) as usize).copied().unwrap_or(0)
        } else {
            0
        }
    }

    fn rebuild(&mut self, field: &SurfaceWaterField) {
        self.width = field.origin_x.max(0) as u32 + field.width;
        self.height = field.origin_z.max(0) as u32 + field.height;
        self.level = vec![0; (self.width * self.height) as usize];
        self.patch_rect(field, 0, 0, self.width, self.height);
        let cs = CHUNK_TILES_SIM;
        self.changed_chunks.clear();
        for cz in 0..self.height.div_ceil(cs) {
            for cx in 0..self.width.div_ceil(cs) {
                self.changed_chunks.push(IVec2::new(cx as i32, cz as i32));
            }
        }
        self.revision = self.revision.wrapping_add(1);
    }

    /// Re-read tiles `[x0,x1) x [z0,z1)` from the field; true when a byte changed.
    fn patch_rect(&mut self, field: &SurfaceWaterField, x0: u32, z0: u32, x1: u32, z1: u32) -> bool {
        let mut changed = false;
        for z in z0..z1.min(self.height) {
            for x in x0..x1.min(self.width) {
                let d = field.depth_at_world(x as i32, z as i32).unwrap_or(0.0);
                let l = level_of(d);
                let slot = &mut self.level[(z * self.width + x) as usize];
                if *slot != l {
                    *slot = l;
                    changed = true;
                }
            }
        }
        changed
    }

    /// Follow the field through the dirty feed. A skipped feed frame or a resized field resyncs.
    pub fn follow(&mut self, field: &SurfaceWaterField, feed: &WaterFieldDirtyFeed) {
        self.changed_chunks.clear();
        let want_w = field.origin_x.max(0) as u32 + field.width;
        let want_h = field.origin_z.max(0) as u32 + field.height;
        let chunks = feed.chunks_since(self.feed_seq);
        self.feed_seq = feed.seq;
        let Some(chunks) = chunks.filter(|_| self.is_ready() && (self.width, self.height) == (want_w, want_h))
        else {
            self.rebuild(field);
            return;
        };
        let cs = CHUNK_TILES_SIM as i32;
        let mut any = false;
        for c in chunks {
            if c.x < 0 || c.y < 0 {
                continue;
            }
            let (x0, z0) = ((c.x * cs) as u32, (c.y * cs) as u32);
            if self.patch_rect(field, x0, z0, x0 + cs as u32, z0 + cs as u32) {
                self.changed_chunks.push(*c);
                any = true;
            }
        }
        if any {
            self.revision = self.revision.wrapping_add(1);
        }
    }
}

/// Sole writer of [`WaterFieldWetMask`].
pub fn update_water_wet_mask_system(
    field: Res<SurfaceWaterField>,
    feed: Res<WaterFieldDirtyFeed>,
    mut mask: ResMut<WaterFieldWetMask>,
) {
    if !field.wired || field.width == 0 {
        return;
    }
    if mask.is_ready() && feed.chunks_since(mask.feed_seq) == Some(&[][..]) {
        return;
    }
    mask.follow(&field, &feed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mask_patches_only_the_moved_chunk() {
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, 96, 96, vec![0.0; 96 * 96], 0.0);
        let mut feed = WaterFieldDirtyFeed { seq: 1, chunks: f.take_dirty_chunks() };
        let mut m = WaterFieldWetMask::default();
        m.follow(&f, &feed);
        assert!(m.is_ready() && m.level.iter().all(|l| *l == 0));

        let i = f.idx(40, 5); // chunk (1, 0)
        f.depth_m[i] = 2.0;
        feed = WaterFieldDirtyFeed { seq: 2, chunks: vec![IVec2::new(1, 0)] };
        m.follow(&f, &feed);
        assert!(m.level_at(40, 5) > 0);
        assert_eq!(m.changed_chunks, vec![IVec2::new(1, 0)]);
    }
}
