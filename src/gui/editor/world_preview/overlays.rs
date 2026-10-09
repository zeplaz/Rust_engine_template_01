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

/// Darken hollows and steep ground, brighten ridges. Alpha is left alone.
pub fn landform_relief_rgba(rgba: [u8; 4], slope: f32, curvature: f32) -> [u8; 4] {
    if rgba[3] == 0 {
        return rgba;
    }
    let steep = (slope / 0.12).clamp(0.0, 1.0);
    let curve = (curvature / 0.08).clamp(-1.0, 1.0);
    let light = (1.0 - curve * 0.16 - steep * 0.22).clamp(0.68, 1.16);
    let scale = |c: u8| ((c as f32) * light).round().clamp(0.0, 255.0) as u8;
    [scale(rgba[0]), scale(rgba[1]), scale(rgba[2]), rgba[3]]
}

/// A pit (positive curvature) reads wetter and darker than a peak.
pub fn preview_hydro_rgba(rgba: [u8; 4], curvature: f32) -> [u8; 4] {
    if rgba[3] == 0 {
        return rgba;
    }
    let wet = (curvature / 0.08).clamp(0.0, 1.0);
    let mut r = rgba[0] as f32;
    let mut g = rgba[1] as f32;
    let b = rgba[2] as f32 * (1.0 - wet * 0.15) + wet * 210.0;
    if wet > 0.45 {
        r *= 0.7;
        g *= 0.7;
    }
    [
        r.round().clamp(0.0, 255.0) as u8,
        g.round().clamp(0.0, 255.0) as u8,
        b.round().clamp(0.0, 255.0) as u8,
        rgba[3],
    ]
}

/// Flat ground stays brighter. Steep ground dims.
pub fn preview_support_rgba(rgba: [u8; 4], slope: f32) -> [u8; 4] {
    if rgba[3] == 0 {
        return rgba;
    }
    let support = crate::terrain::generation::support_from_slope(slope);
    let light = 0.45 + 0.55 * support;
    let scale = |c: u8| ((c as f32) * light).round().clamp(0.0, 255.0) as u8;
    [scale(rgba[0]), scale(rgba[1]), scale(rgba[2]), rgba[3]]
}

/// Editor green drops with slope times drainage. Curvature is not an input.
#[must_use]
pub fn preview_erosion_rgba(rgba: [u8; 4], slope: f32, drainage: f32) -> [u8; 4] {
    if rgba[3] == 0 {
        return rgba;
    }
    let erode = crate::terrain::generation::erosion_from_slope_drainage(slope, drainage);
    let g = ((rgba[1] as f32) * (1.0 - erode * 0.4))
        .round()
        .clamp(0.0, 255.0) as u8;
    [rgba[0], g, rgba[2], rgba[3]]
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

#[cfg(test)]
mod tests {
    use super::landform_relief_rgba;

    #[test]
    fn relief_curvature_brightens_ridges_and_darkens_hollows() {
        let base = [80u8, 120, 60, 255];
        let ridge = landform_relief_rgba(base, 0.0, -0.08);
        let hollow = landform_relief_rgba(base, 0.02, 0.08);
        assert!(ridge[1] > base[1], "ridge {} vs base {}", ridge[1], base[1]);
        assert!(hollow[1] < base[1], "hollow {} vs base {}", hollow[1], base[1]);
        assert!(ridge[1] > hollow[1]);
    }

    #[test]
    fn editor_preview_pit_is_bluer_than_peak() {
        let base = [80u8, 120, 60, 255];
        let pit = super::preview_hydro_rgba(base, 0.1);
        let peak = super::preview_hydro_rgba(base, -0.1);
        assert!(pit[2] > peak[2], "pit blue {} peak blue {}", pit[2], peak[2]);
    }

    #[test]
    fn editor_preview_basin_is_darker_than_peak() {
        let base = [80u8, 120, 60, 255];
        let pit = super::preview_hydro_rgba(base, 0.1);
        let peak = super::preview_hydro_rgba(base, -0.1);
        assert!(pit[0] < peak[0], "pit red {} peak red {}", pit[0], peak[0]);
    }

    #[test]
    fn editor_preview_steep_is_dimmer_than_flat() {
        let base = [180u8, 180, 180, 255];
        let steep = super::preview_support_rgba(base, 0.4);
        let flat = super::preview_support_rgba(base, 0.0);
        assert!(steep[0] < flat[0], "steep {} flat {}", steep[0], flat[0]);
    }

    #[test]
    fn push_editor_erosion_dims_a_drained_slope() {
        let base = [80, 160, 90, 255];
        let flat = super::preview_erosion_rgba(base, 0.0, 4.0);
        let drained = super::preview_erosion_rgba(base, 0.4, 4.0);
        let dry = super::preview_erosion_rgba(base, 0.4, 0.0);
        assert_eq!(flat[1], base[1]);
        assert!(drained[1] < flat[1]);
        assert_eq!(dry[1], base[1]);
        assert_eq!(drained[0], base[0]);
    }

    #[test]
    fn cont_clear_pixel_ignores_hydro_tint() {
        let clear = [12, 40, 20, 0];
        assert_eq!(super::preview_hydro_rgba(clear, 0.2), clear);
        let opaque = super::preview_hydro_rgba([12, 40, 20, 255], 0.2);
        assert!(opaque[2] > 20);
    }

    #[test]
    fn cont_clear_pixel_keeps_its_support_color() {
        let clear = [180, 180, 180, 0];
        assert_eq!(super::preview_support_rgba(clear, 0.4), clear);
        let steep = super::preview_support_rgba([180, 180, 180, 255], 0.4);
        assert!(steep[0] < 180);
    }

    #[test]
    fn fold_clear_pixel_ignores_erosion_tint() {
        let clear = [80, 160, 90, 0];
        assert_eq!(super::preview_erosion_rgba(clear, 0.4, 8.0), clear);
        let drained = super::preview_erosion_rgba([80, 160, 90, 255], 0.4, 8.0);
        assert!(drained[1] < 160);
    }

    #[test]
    fn fold_dry_peak_matches_the_source() {
        let base = [80u8, 120, 60, 255];
        assert_eq!(super::preview_hydro_rgba(base, 0.0), base);
        assert_eq!(super::preview_hydro_rgba(base, -0.1), base);
        let pit = super::preview_hydro_rgba(base, 0.1);
        assert!(pit[0] < base[0]);
    }
}
