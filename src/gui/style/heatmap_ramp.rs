//! RND-20 — perceptual heatmap ramp (lightness-monotonic, built in linear RGB).

use bevy::prelude::Color;

pub const HEATMAP_RAMP_STOPS: usize = 16;

#[inline]
pub fn srgb_channel_to_linear(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[inline]
pub fn linear_channel_to_srgb(c: f32) -> f32 {
    let c = c.clamp(0.0, 1.0);
    if c <= 0.0031308 {
        12.92 * c
    } else {
        1.055 * c.powf(1.0 / 2.4) - 0.055
    }
}

#[inline]
pub fn linear_rgb_to_lightness(rgb: [f32; 3]) -> f32 {
    0.2126 * rgb[0] + 0.7152 * rgb[1] + 0.0722 * rgb[2]
}

/// Deterministic overlay heat ramp in **linear** space (low → high scalar).
#[must_use]
pub fn default_heatmap_ramp_linear() -> [[f32; 3]; HEATMAP_RAMP_STOPS] {
    let mut ramp = [[0.0_f32; 3]; HEATMAP_RAMP_STOPS];
    for (i, row) in ramp.iter_mut().enumerate() {
        let t = i as f32 / (HEATMAP_RAMP_STOPS - 1) as f32;
        row[0] = srgb_channel_to_linear(0.05 + 0.82 * t);
        row[1] = srgb_channel_to_linear(0.07 + 0.58 * t.powf(1.08));
        row[2] = srgb_channel_to_linear(0.20 * (1.0 - 0.65 * t));
    }
    ramp
}

#[must_use]
pub fn sample_heatmap_ramp_linear(ramp: &[[f32; 3]; HEATMAP_RAMP_STOPS], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    let f = t * (HEATMAP_RAMP_STOPS - 1) as f32;
    let i0 = f.floor() as usize;
    let i1 = (i0 + 1).min(HEATMAP_RAMP_STOPS - 1);
    let u = f - i0 as f32;
    let a = ramp[i0];
    let b = ramp[i1];
    [
        a[0] + (b[0] - a[0]) * u,
        a[1] + (b[1] - a[1]) * u,
        a[2] + (b[2] - a[2]) * u,
    ]
}

/// Blend two heatmap samples in linear RGB, return display-ready sRGB [`Color`].
#[must_use]
pub fn blend_heatmap_linear(
    ramp: &[[f32; 3]; HEATMAP_RAMP_STOPS],
    base_t: f32,
    overlay_t: f32,
    overlay_alpha: f32,
) -> Color {
    let a = overlay_alpha.clamp(0.0, 1.0);
    let base = sample_heatmap_ramp_linear(ramp, base_t);
    let over = sample_heatmap_ramp_linear(ramp, overlay_t);
    let mixed = [
        base[0] + (over[0] - base[0]) * a,
        base[1] + (over[1] - base[1]) * a,
        base[2] + (over[2] - base[2]) * a,
    ];
    Color::srgba(
        linear_channel_to_srgb(mixed[0]),
        linear_channel_to_srgb(mixed[1]),
        linear_channel_to_srgb(mixed[2]),
        1.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ramp_lightness_strictly_increases() {
        let ramp = default_heatmap_ramp_linear();
        let mut prev = -1.0_f32;
        for (i, stop) in ramp.iter().enumerate() {
            let l = linear_rgb_to_lightness(*stop);
            assert!(
                l > prev + 1e-5,
                "stop {i} lightness {l} not above previous {prev}"
            );
            prev = l;
        }
    }

    #[test]
    fn linear_blend_midpoint_between_stops() {
        let ramp = default_heatmap_ramp_linear();
        let c0 = sample_heatmap_ramp_linear(&ramp, 0.0);
        let c1 = sample_heatmap_ramp_linear(&ramp, 1.0);
        let mid = sample_heatmap_ramp_linear(&ramp, 0.5);
        let l0 = linear_rgb_to_lightness(c0);
        let l1 = linear_rgb_to_lightness(c1);
        let lmid = linear_rgb_to_lightness(mid);
        // Multi-stop ramp: mid sample sits between ends in lightness.
        assert!(lmid > l0 && lmid < l1);
        // blend(0,1,0.5) is lerp of endpoint *samples*, not sample(0.5).
        let mixed = [
            c0[0] + (c1[0] - c0[0]) * 0.5,
            c0[1] + (c1[1] - c0[1]) * 0.5,
            c0[2] + (c1[2] - c0[2]) * 0.5,
        ];
        let blended = blend_heatmap_linear(&ramp, 0.0, 1.0, 0.5).to_srgba();
        let back = [
            srgb_channel_to_linear(blended.red),
            srgb_channel_to_linear(blended.green),
            srgb_channel_to_linear(blended.blue),
        ];
        for i in 0..3 {
            assert!((back[i] - mixed[i]).abs() < 1e-4, "blend channel {i}");
        }
    }
}