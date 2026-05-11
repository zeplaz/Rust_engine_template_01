//! Overlay tints, alpha blend, and diagnostic colors for the preview raster.

use crate::terrain::material::{TagId, TagSet};
use crate::terrain::mobility::MovementHint;

/// Highlight color for tag-overlay preview mode (U5).
pub const TAG_OVERLAY_HIGHLIGHT: [u8; 4] = [255, 220, 0, 255];

#[inline]
pub fn voronoi_region_preview_rgba(region_index: u32) -> [u8; 4] {
    let u = region_index
        .wrapping_mul(1103515245)
        .wrapping_add(12345)
        .wrapping_mul(1103515245)
        .wrapping_add(region_index);
    [
        (u & 0xff) as u8,
        ((u >> 8) & 0xff) as u8,
        ((u >> 16) & 0xff) as u8,
        255,
    ]
}

pub fn movement_hint_rgba(hint: &MovementHint) -> [u8; 4] {
    if hint.blocked {
        [220, 50, 50, 255]
    } else {
        let stress = ((hint.cost_mul - 1.0).max(0.0) / 2.0).min(1.0);
        let g = ((1.0 - stress) * 220.0) as u8;
        let r = (stress * 200.0) as u8;
        [r, g, 70, 255]
    }
}

pub fn slope_grade_to_color(s: f32) -> [u8; 4] {
    let u = (s.clamp(0.0, 1.0) * 255.0) as u8;
    [u, 255u8.saturating_sub(u), 120, 255]
}

pub fn tag_overlay_rgba(tag_target: TagId, cell_tags: &TagSet) -> [u8; 4] {
    if cell_tags.contains(tag_target) {
        TAG_OVERLAY_HIGHLIGHT
    } else {
        [0, 0, 0, 0]
    }
}

#[inline]
pub fn tag_overlay_rgba_pool(cell_tags: &TagSet, pool: &TagSet) -> [u8; 4] {
    if pool.intersects(cell_tags) {
        TAG_OVERLAY_HIGHLIGHT
    } else {
        [0, 0, 0, 0]
    }
}

/// `over_strength` scales overlay alpha before blending onto `base`.
pub fn blend_overlay(base: [u8; 4], over: [u8; 4], over_strength: f32) -> [u8; 4] {
    if over[3] == 0 {
        return base;
    }
    let t = (over[3] as f32 / 255.0) * over_strength.clamp(0.0, 1.0);
    if t <= 1e-6 {
        return base;
    }
    if base[3] == 0 {
        return [
            over[0],
            over[1],
            over[2],
            (t * 255.0).min(255.0) as u8,
        ];
    }
    let inv = 1.0 - t;
    [
        (over[0] as f32 * t + base[0] as f32 * inv) as u8,
        (over[1] as f32 * t + base[1] as f32 * inv) as u8,
        (over[2] as f32 * t + base[2] as f32 * inv) as u8,
        255,
    ]
}
