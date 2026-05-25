//! Catmull-Rom spline sampling for road preview (PHASE2-BUILD-17).

use bevy::prelude::*;

/// Sample a smooth chain through control points (uniform Catmull-Rom, `tension = 0.5`).
#[must_use]
pub fn catmull_rom_chain(control_points: &[Vec3], samples_per_span: usize) -> Vec<Vec3> {
    let n = control_points.len();
    if n < 2 {
        return control_points.to_vec();
    }
    if n == 2 {
        return control_points.to_vec();
    }
    let samples_per_span = samples_per_span.max(2);
    let mut out = Vec::new();
    for i in 0..n.saturating_sub(1) {
        let p0 = control_points[i.saturating_sub(1)];
        let p1 = control_points[i];
        let p2 = control_points[(i + 1).min(n - 1)];
        let p3 = control_points[(i + 2).min(n - 1)];
        let steps = if i + 1 == n - 1 {
            samples_per_span + 1
        } else {
            samples_per_span
        };
        for s in 0..steps {
            if i > 0 && s == 0 {
                continue;
            }
            let t = s as f32 / samples_per_span as f32;
            out.push(catmull_rom_point(p0, p1, p2, p3, t));
        }
    }
    if out.is_empty() {
        return control_points.to_vec();
    }
    out
}

#[inline]
fn catmull_rom_point(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5
        * ((2.0 * p1)
            + (-p0 + p2) * t
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
            + (-p0 + 3.0 * p1 - 3.0 * p2 + p3) * t3)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catmull_rom_produces_more_samples_than_control_count() {
        let pts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(8.0, 0.0, 4.0),
            Vec3::new(12.0, 0.0, 4.0),
        ];
        let chain = catmull_rom_chain(&pts, 6);
        assert!(chain.len() > pts.len());
    }
}
