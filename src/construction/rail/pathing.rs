//! Rail path preview — curved samples + turn-radius checks.

use bevy::prelude::*;

use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::placement::{ActiveRailPlacement, RailSegmentPreview};
use super::super::roads::spline::catmull_rom_chain;
use super::validation::{segment_slope_ok, turn_radius_ok, validate_rail_segment};

#[inline]
pub fn world_xy_to_tile(p: Vec3) -> crate::strategic::BuildSiteTile {
    crate::strategic::BuildSiteTile {
        x: p.x.floor().max(0.0) as u32,
        z: p.z.floor().max(0.0) as u32,
    }
}

pub fn regenerate_rail_segments(
    placement: &ActiveRailPlacement,
    cursor_world: Option<Vec3>,
    params: &WorldGenParams,
) -> Vec<RailSegmentPreview> {
    let mut chain: Vec<Vec3> = placement.control_points.to_vec();
    if let Some(c) = cursor_world {
        if chain.last().copied().map_or(true, |last| (last - c).length_squared() > 0.01) {
            chain.push(c);
        }
    }
    if chain.len() < 2 {
        return Vec::new();
    }
    let sample_chain = if chain.len() >= 3 {
        catmull_rom_chain(&chain, 10)
    } else {
        chain
    };
    let max_slope = placement.max_slope;
    let min_r = placement.min_curve_radius;
    sample_chain
        .windows(2)
        .enumerate()
        .map(|(i, w)| {
            let start = w[0];
            let end = w[1];
            let head = world_xy_to_tile(start);
            let tail = world_xy_to_tile(end);
            let slope_ok = segment_slope_ok(start, end, max_slope);
            let curve_ok = if i > 0 && i + 1 < sample_chain.len() {
                turn_radius_ok(sample_chain[i - 1], start, end, min_r)
            } else {
                true
            };
            let validation =
                validate_rail_segment(head, tail, start.y, end.y, max_slope, params);
            RailSegmentPreview {
                start,
                end,
                width: placement.width,
                valid: validation.valid && slope_ok && curve_ok,
                slope_ok,
            }
        })
        .collect()
}
