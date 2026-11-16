//! **PCG-03** — thermal then hydraulic erosion before D8 (aig3 8.3.5–8.3.6).
//!
//! Deterministic height edits. Callers pass a DEM copy; hydrology D8 runs on the result.
//! Transfers accumulate into a delta buffer then apply (CQ-04) so same-pass deposits are not
//! overwritten when a receiver is later processed as a source.

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

/// Fraction of hydraulic transport that lands on the receiver. The remainder is intentional
/// valley-incision loss (not the CQ-04 overwrite bug).
pub const HYDRAULIC_DEPOSIT_FRACTION: f32 = 0.85;

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

fn apply_deltas(dem: &mut [f32], prev: &[f32], delta: &[f32]) {
    for i in 0..dem.len() {
        dem[i] = prev[i] + delta[i];
    }
}

/// Thermal (talus) erosion: move excess slope material downhill.
/// Mass is conserved (sum of heights unchanged modulo float error).
pub fn apply_thermal_erosion(dem: &mut [f32], w: usize, h: usize, params: &HydrologyParams) {
    if params.thermal_iters == 0 || w == 0 || h == 0 || dem.len() != w * h {
        return;
    }
    let talus = params.thermal_talus.max(1e-6);
    let rate = params.thermal_rate.clamp(0.0, 1.0);
    let mut delta = vec![0.0f32; dem.len()];
    for _ in 0..params.thermal_iters {
        let prev = dem.to_vec();
        delta.fill(0.0);
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
                delta[i] -= excess;
                delta[ni] += excess;
            }
        }
        apply_deltas(dem, &prev, &delta);
    }
}

/// Hydraulic erosion: transport a fraction of steepest-descent drop along the flow.
/// Moved mass deposits at [`HYDRAULIC_DEPOSIT_FRACTION`]; the rest is intentional incision loss.
pub fn apply_hydraulic_erosion(dem: &mut [f32], w: usize, h: usize, params: &HydrologyParams) {
    if params.hydraulic_iters == 0 || w == 0 || h == 0 || dem.len() != w * h {
        return;
    }
    let cap = params.hydraulic_capacity.clamp(0.0, 1.0);
    let mut delta = vec![0.0f32; dem.len()];
    for _ in 0..params.hydraulic_iters {
        let prev = dem.to_vec();
        delta.fill(0.0);
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
                delta[i] -= move_amt;
                delta[ni] += move_amt * HYDRAULIC_DEPOSIT_FRACTION;
            }
        }
        apply_deltas(dem, &prev, &delta);
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

    fn ridge_dem(w: usize, h: usize) -> Vec<f32> {
        let mut dem = vec![0.3f32; w * h];
        for y in 0..h {
            for x in 0..w {
                let d = ((x as i32 - 4).abs() + (y as i32 - 4).abs()) as f32;
                dem[y * w + x] = 0.9 - d * 0.08;
            }
        }
        dem
    }

    fn sum(dem: &[f32]) -> f64 {
        dem.iter().map(|v| f64::from(*v)).sum()
    }

    #[test]
    fn pcg03() {
        let w = 8usize;
        let h = 8usize;
        let dem = ridge_dem(w, h);
        let mut p = HydrologyParams::default();
        p.apply_erosion = true;
        p.thermal_iters = 3;
        p.hydraulic_iters = 3;
        let a = erode_dem(&dem, w, h, &p);
        let b = erode_dem(&dem, w, h, &p);
        assert_eq!(a, b, "deterministic");
        assert!(a[4 * w + 4] < dem[4 * w + 4]);
        p.apply_erosion = false;
        let id = erode_dem(&dem, w, h, &p);
        assert_eq!(id, dem);
    }

    #[test]
    fn thermal_conserves_mass() {
        let w = 8usize;
        let h = 8usize;
        let dem = ridge_dem(w, h);
        let before = sum(&dem);
        let mut p = HydrologyParams::default();
        p.thermal_iters = 5;
        p.hydraulic_iters = 0;
        let mut out = dem.clone();
        apply_thermal_erosion(&mut out, w, h, &p);
        let after = sum(&out);
        assert!(
            (after - before).abs() < 1e-4,
            "thermal must conserve mass: before={before} after={after}"
        );
    }

    #[test]
    fn hydraulic_mass_matches_intentional_deposit_loss() {
        let w = 8usize;
        let h = 8usize;
        let dem = ridge_dem(w, h);
        let before = sum(&dem);
        let mut p = HydrologyParams::default();
        p.thermal_iters = 0;
        p.hydraulic_iters = 1;
        p.hydraulic_capacity = 1.0;

        // Replay one pass to account for intentional (1 - deposit) loss.
        let prev = dem.clone();
        let mut expected_loss = 0.0f64;
        for y in 0..h {
            for x in 0..w {
                let Some((_, drop)) = steepest_down(&prev, w, h, x, y) else {
                    continue;
                };
                let move_amt = (drop * 0.5).min(drop * 0.5);
                if move_amt <= 1e-8 {
                    continue;
                }
                expected_loss += f64::from(move_amt) * f64::from(1.0 - HYDRAULIC_DEPOSIT_FRACTION);
            }
        }

        let mut out = dem.clone();
        apply_hydraulic_erosion(&mut out, w, h, &p);
        let after = sum(&out);
        let lost = before - after;
        assert!(
            (lost - expected_loss).abs() < 1e-4,
            "hydraulic loss must equal intentional deposit fraction only: lost={lost} expected={expected_loss}"
        );
    }
}
