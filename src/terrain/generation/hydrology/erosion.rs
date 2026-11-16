//! **PCG-03** — thermal then hydraulic erosion before D8 (aig3 8.3.5–8.3.6).
//!
//! Deterministic, in-place height edits. Callers pass a DEM copy; hydrology D8 runs on the result.

use super::HydrologyParams;

const D8: [(i32, i32); 8] = [
    (-1, -1),
    (0, -1),
    (1, -1),
    (-1, 0),
    (1, 0),
    (-1, 1),
    (0, 1),
    (1, 1),
];

#[inline]
fn idx(w: usize, x: usize, y: usize) -> usize {
    y * w + x
}

/// Steepest downhill neighbor index, or `None` if flat / border-isolated.
fn steepest_down(dem: &[f32], w: usize, h: usize, x: usize, y: usize) -> Option<(usize, f32)> {
    let i = idx(w, x, y);
    let h0 = dem[i];
    let mut best: Option<(usize, f32)> = None;
    for (dx, dy) in D8 {
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
            continue;
        }
        let ni = idx(w, nx as usize, ny as usize);
        let drop = h0 - dem[ni];
        if drop <= 0.0 {
            continue;
        }
        if best.is_none_or(|(_, bd)| drop > bd) {
            best = Some((ni, drop));
        }
    }
    best
}

/// Thermal (talus) erosion: move excess slope material downhill.
pub fn apply_thermal_erosion(dem: &mut [f32], w: usize, h: usize, params: &HydrologyParams) {
    if params.thermal_iters == 0 || w == 0 || h == 0 || dem.len() != w * h {
        return;
    }
    let talus = params.thermal_talus.max(1e-6);
    let rate = params.thermal_rate.clamp(0.0, 1.0);
    for _ in 0..params.thermal_iters {
        // Double-buffer for determinism (no in-pass dependency on update order).
        let prev = dem.to_vec();
        for y in 0..h {
            for x in 0..w {
                let Some((ni, drop)) = steepest_down(&prev, w, h, x, y) else {
                    continue;
                };
                if drop <= talus {
                    continue;
                }
                let excess = (drop - talus) * rate;
                let i = idx(w, x, y);
                dem[i] = prev[i] - excess;
                dem[ni] += excess;
            }
        }
    }
}

/// Hydraulic erosion: transport a fraction of steepest-descent drop along the flow.
pub fn apply_hydraulic_erosion(dem: &mut [f32], w: usize, h: usize, params: &HydrologyParams) {
    if params.hydraulic_iters == 0 || w == 0 || h == 0 || dem.len() != w * h {
        return;
    }
    let cap = params.hydraulic_capacity.clamp(0.0, 1.0);
    for _ in 0..params.hydraulic_iters {
        let prev = dem.to_vec();
        for y in 0..h {
            for x in 0..w {
                let Some((ni, drop)) = steepest_down(&prev, w, h, x, y) else {
                    continue;
                };
                let move_amt = (drop * cap * 0.5).min(drop * 0.5);
                if move_amt <= 1e-8 {
                    continue;
                }
                let i = idx(w, x, y);
                dem[i] = prev[i] - move_amt;
                dem[ni] += move_amt * 0.85; // slight loss → valley incision bias
            }
        }
    }
}

/// Thermal → hydraulic on a DEM copy; returns eroded heights (same length as input).
#[must_use]
pub fn erode_dem(dem: &[f32], w: usize, h: usize, params: &HydrologyParams) -> Vec<f32> {
    let mut out = dem.to_vec();
    if !params.apply_erosion || out.len() != w.saturating_mul(h) {
        return out;
    }
    apply_thermal_erosion(&mut out, w, h, params);
    apply_hydraulic_erosion(&mut out, w, h, params);
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::HydrologyParams;

    #[test]
    fn pcg03() {
        let w = 8usize;
        let h = 8usize;
        // Peak ridge → should incise after erosion.
        let mut dem = vec![0.3f32; w * h];
        for y in 0..h {
            for x in 0..w {
                let d = ((x as i32 - 4).abs() + (y as i32 - 4).abs()) as f32;
                dem[y * w + x] = 0.9 - d * 0.08;
            }
        }
        let mut p = HydrologyParams::default();
        p.apply_erosion = true;
        p.thermal_iters = 3;
        p.hydraulic_iters = 3;
        let a = erode_dem(&dem, w, h, &p);
        let b = erode_dem(&dem, w, h, &p);
        assert_eq!(a, b, "deterministic");
        // Center should drop vs raw peak (material moved downhill).
        assert!(a[4 * w + 4] < dem[4 * w + 4]);
        // Off with flag → identity.
        p.apply_erosion = false;
        let id = erode_dem(&dem, w, h, &p);
        assert_eq!(id, dem);
    }
}
