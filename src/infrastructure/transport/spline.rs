//! **INFRA-E1-002** — Catmull-Rom subdivision with profile turn-radius gate.

use bevy::prelude::Vec3;

use crate::infrastructure::profiles::{CorridorProfileKind, RoadProfile};

fn catmull_rom_chain(control_points: &[Vec3], samples_per_span: usize) -> Vec<Vec3> {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SubEdgeSample {
    pub position: [f32; 3],
    /// Normalized arc parameter along the subdivided polyline.
    pub t: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SplineError {
    TooFewControlPoints,
    RadiusViolation { min_radius_m: f32, observed_radius_m: f32 },
}

/// Subdivide a corridor polyline; rejects when any interior turn is tighter than `profile.turn_radius_m`.
pub fn subdivide_edge(
    control_points: &[[f32; 3]],
    profile: &RoadProfile,
    samples_per_span: usize,
) -> Result<Vec<SubEdgeSample>, SplineError> {
    subdivide_edge_with_radius(control_points, profile.turn_radius_m, samples_per_span)
}

/// Subdivide using an explicit minimum turn radius (tests + rail callers).
pub fn subdivide_edge_with_radius(
    control_points: &[[f32; 3]],
    min_turn_radius_m: f32,
    samples_per_span: usize,
) -> Result<Vec<SubEdgeSample>, SplineError> {
    if control_points.len() < 2 {
        return Err(SplineError::TooFewControlPoints);
    }
    let pts: Vec<Vec3> = control_points.iter().map(|p| Vec3::from_array(*p)).collect();
    let chain = catmull_rom_chain(&pts, samples_per_span.max(4));
    let observed = min_observed_turn_radius_m(&chain);
    if observed < min_turn_radius_m {
        return Err(SplineError::RadiusViolation {
            min_radius_m: min_turn_radius_m,
            observed_radius_m: observed,
        });
    }
    let len = chain.len().max(1);
    Ok(chain
        .into_iter()
        .enumerate()
        .map(|(i, p)| SubEdgeSample {
            position: p.to_array(),
            t: i as f32 / (len - 1).max(1) as f32,
        })
        .collect())
}

/// Profile-aware wrapper for [`CorridorProfileKind`].
pub fn subdivide_edge_for_profile(
    control_points: &[[f32; 3]],
    profile: &CorridorProfileKind,
    samples_per_span: usize,
) -> Result<Vec<SubEdgeSample>, SplineError> {
    let radius = profile.turn_radius_m();
    subdivide_edge_with_radius(control_points, radius, samples_per_span)
}

fn min_observed_turn_radius_m(samples: &[Vec3]) -> f32 {
    if samples.len() < 3 {
        return f32::INFINITY;
    }
    let mut min_r = f32::INFINITY;
    for w in samples.windows(3) {
        let r = circumradius_m(w[0], w[1], w[2]);
        if r.is_finite() && r > 0. {
            min_r = min_r.min(r);
        }
    }
    min_r
}

fn circumradius_m(a: Vec3, b: Vec3, c: Vec3) -> f32 {
    let ab = b - a;
    let ac = c - a;
    let cross_len = ab.cross(ac).length();
    if cross_len < 1e-6 {
        return f32::INFINITY;
    }
    let ab_len = ab.length();
    let bc_len = (c - b).length();
    let ca_len = (a - c).length();
    (ab_len * bc_len * ca_len) / (2.0 * cross_len)
}

#[must_use]
pub fn infra_e1_002_spline_subdivide_witness_green() -> bool {
    let gentle = [
        [0.0_f32, 0.0, 0.0],
        [5.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [15.0, 0.0, 0.0],
    ];
    let profile = RoadProfile {
        id: "road_local".into(),
        road_type: "local".into(),
        lanes: 2,
        speed_limit_kmh: 50,
        surface_tags: vec![],
        turn_radius_m: 8.0,
        base_cost: 1.0,
        allowed_agents: vec!["truck".into()],
    };
    let ok = subdivide_edge(&gentle, &profile, 6)
        .map(|s| s.len() >= gentle.len())
        .unwrap_or(false);
    let sharp = [
        [0.0_f32, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [10.0, 0.0, 0.0],
        [10.0, 0.0, 10.0],
    ];
    let rejected = matches!(
        subdivide_edge(&sharp, &profile, 8),
        Err(SplineError::RadiusViolation { .. })
    );
    ok && rejected
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infra_e1_002_gentle_curve_produces_many_samples() {
        let profile = RoadProfile {
            id: "road_local".into(),
            road_type: "local".into(),
            lanes: 2,
            speed_limit_kmh: 50,
            surface_tags: vec![],
            turn_radius_m: 4.0,
            base_cost: 1.0,
            allowed_agents: vec!["truck".into()],
        };
        let pts = [
            [0.0_f32, 0.0, 0.0],
            [4.0, 0.0, 0.0],
            [8.0, 0.0, 0.5],
            [12.0, 0.0, 0.5],
        ];
        let samples = subdivide_edge(&pts, &profile, 6).expect("gentle subdivide");
        assert!(samples.len() > pts.len());
    }

    #[test]
    fn infra_e1_002_sharp_corner_rejected() {
        let profile = RoadProfile {
            id: "road_local".into(),
            road_type: "local".into(),
            lanes: 2,
            speed_limit_kmh: 50,
            surface_tags: vec![],
            turn_radius_m: 12.0,
            base_cost: 1.0,
            allowed_agents: vec![],
        };
        let pts = [
            [0.0_f32, 0.0, 0.0],
            [0.0, 0.0, 0.0],
            [8.0, 0.0, 0.0],
            [8.0, 0.0, 8.0],
        ];
        assert!(matches!(
            subdivide_edge(&pts, &profile, 8),
            Err(SplineError::RadiusViolation { .. })
        ));
    }

    #[test]
    fn infra_e1_002_witness_green() {
        assert!(infra_e1_002_spline_subdivide_witness_green());
    }
}
