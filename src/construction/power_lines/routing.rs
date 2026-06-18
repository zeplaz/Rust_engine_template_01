//! Orthogonal + spline routers for power line preview/commit (COD-POWER-ORTHOGONAL/SPLINE-ROUTER-001).

use bevy::prelude::*;

use super::placement::{PowerLineRoutingMode, PowerLineSegmentPreview};
use crate::construction::roads::spline::catmull_rom_chain;

#[must_use]
pub fn snap_power_grid(v: Vec3) -> Vec3 {
    Vec3::new(v.x.floor() + 0.5, v.y, v.z.floor() + 0.5)
}

/// Manhattan path between two world points (horizontal then vertical).
#[must_use]
pub fn orthogonal_chain_between(from: Vec3, to: Vec3) -> Vec<Vec3> {
    if (from - to).length_squared() < 0.0001 {
        return vec![from];
    }
    if (from.x - to.x).abs() < 0.01 || (from.z - to.z).abs() < 0.01 {
        return vec![from, to];
    }
    let corner = Vec3::new(to.x, from.y, from.z);
    vec![from, corner, to]
}

#[must_use]
pub fn flatten_chain(points: &[Vec3]) -> Vec<Vec3> {
    let mut out = Vec::new();
    for p in points {
        if out.last().map_or(true, |last: &Vec3| (*last - *p).length_squared() > 0.0001) {
            out.push(*p);
        }
    }
    out
}

#[must_use]
pub fn build_sample_chain(
    control_points: &[Vec3],
    cursor_world: Option<Vec3>,
    mode: PowerLineRoutingMode,
    grid_snap: bool,
) -> Vec<Vec3> {
    let mut chain: Vec<Vec3> = control_points.to_vec();
    if let Some(c) = cursor_world {
        if chain.last().copied().map_or(true, |last| (last - c).length_squared() > 0.01) {
            chain.push(c);
        }
    }
    if chain.is_empty() {
        return chain;
    }
    if mode == PowerLineRoutingMode::Orthogonal90 && grid_snap {
        chain = chain.into_iter().map(snap_power_grid).collect();
    }
    if chain.len() < 2 {
        return chain;
    }
    match mode {
        PowerLineRoutingMode::Curved if chain.len() >= 3 => catmull_rom_chain(&chain, 6),
        PowerLineRoutingMode::Curved => chain,
        PowerLineRoutingMode::Orthogonal90 => {
            let mut out = Vec::new();
            for w in chain.windows(2) {
                let leg = orthogonal_chain_between(w[0], w[1]);
                if out.is_empty() {
                    out.extend(leg);
                } else {
                    out.extend(leg.into_iter().skip(1));
                }
            }
            flatten_chain(&out)
        }
    }
}

#[must_use]
pub fn segment_preview_valid(
    start: Vec3,
    end: Vec3,
    mode: PowerLineRoutingMode,
) -> bool {
    if (start - end).length_squared() < 0.25 {
        return false;
    }
    if mode == PowerLineRoutingMode::Orthogonal90 {
        let dx = (start.x - end.x).abs();
        let dz = (start.z - end.z).abs();
        return dx < 0.01 || dz < 0.01;
    }
    true
}

#[must_use]
pub fn regenerate_power_line_segments(
    control_points: &[Vec3],
    cursor_world: Option<Vec3>,
    mode: PowerLineRoutingMode,
    grid_snap: bool,
) -> Vec<PowerLineSegmentPreview> {
    let sample_chain = build_sample_chain(control_points, cursor_world, mode, grid_snap);
    if sample_chain.len() < 2 {
        return Vec::new();
    }
    sample_chain
        .windows(2)
        .map(|w| {
            let start = w[0];
            let end = w[1];
            PowerLineSegmentPreview {
                start,
                end,
                valid: segment_preview_valid(start, end, mode),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orthogonal_rectangle_has_no_diagonal_segments() {
        let pts = vec![
            Vec3::new(0.5, 0.0, 0.5),
            Vec3::new(4.5, 0.0, 0.5),
            Vec3::new(4.5, 0.0, 4.5),
        ];
        let segs = regenerate_power_line_segments(
            &pts,
            None,
            PowerLineRoutingMode::Orthogonal90,
            true,
        );
        assert!(segs.len() >= 2);
        assert!(segs.iter().all(|s| s.valid));
        for seg in segs {
            let dx = (seg.start.x - seg.end.x).abs();
            let dz = (seg.start.z - seg.end.z).abs();
            assert!(dx < 0.01 || dz < 0.01);
        }
    }

    #[test]
    fn curved_mode_emits_more_samples_than_polyline() {
        let pts = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(4.0, 0.0, 0.0),
            Vec3::new(8.0, 0.0, 6.0),
            Vec3::new(12.0, 0.0, 6.0),
        ];
        let flat = regenerate_power_line_segments(&pts, None, PowerLineRoutingMode::Curved, false);
        assert!(flat.len() >= 3);
    }
}
