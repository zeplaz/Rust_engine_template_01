//! Corridor grading (PLAN-ROADS-v1 R-6): a placed road or railway is cut and filled into the
//! terrain, not just labelled. Between two nodes the line's bed runs straight from one node's
//! ground height to the next; every dry tile on the way is cut down or filled up to that bed, at
//! most the mode's cut/fill limit (`corridor_tool::mode_spec`). Bridge spans and water are left
//! alone. The change goes through the terrain's own queue (`GroundDeformOp::Grade`), so terrain
//! stays the single writer of ground height, and water, fire and the voxel field all see it.
//!
//! Only lines built to order are graded (`CorridorSegment::charged`); lines the world starts
//! with sit on the ground as generated.

use std::collections::HashSet;

use bevy::prelude::*;

use super::bridges::BridgeBook;
use super::corridor_network::{CorridorKind, CorridorNetwork};
use super::corridor_tool::{mode_spec, CorridorToolKind};
use crate::systems::water::WaterTileProbe;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::WorldGenDenseTerrainCache;
use crate::terrain::ground_deform::{GroundDeformOp, GroundDeformQueue};

/// Height changes smaller than this (m) are left to the road surface.
const MIN_GRADE_M: f32 = 0.25;
/// Water deeper than this (m) is not graded (a bridge or ford carries the line).
const WET_M: f32 = 0.3;

fn tool_kind(kind: &CorridorKind) -> CorridorToolKind {
    match kind {
        CorridorKind::Road { tier, .. } => CorridorToolKind::Road(*tier),
        CorridorKind::Rail { .. } => CorridorToolKind::Rail,
    }
}

/// Cut (negative) / fill (positive) per tile for a line through `nodes` (tile centres).
/// `ground(tile)` = height m; `skip(tile)` = water or bridge.
#[must_use]
pub fn grade_cells(
    nodes: &[Vec2],
    max_cut_m: f32,
    ground: &dyn Fn(IVec2) -> Option<f32>,
    skip: &dyn Fn(IVec2) -> bool,
) -> Vec<(IVec2, f32)> {
    let mut seen: HashSet<IVec2> = HashSet::new();
    let mut out = Vec::new();
    for w in nodes.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (ta, tb) = (a.floor().as_ivec2(), b.floor().as_ivec2());
        let (Some(ha), Some(hb)) = (ground(ta), ground(tb)) else {
            continue;
        };
        let n = (a.distance(b) / 0.5).ceil().max(1.0) as usize;
        for k in 0..=n {
            let t = k as f32 / n as f32;
            let tile = (a + (b - a) * t).floor().as_ivec2();
            if !seen.insert(tile) || skip(tile) {
                continue;
            }
            let Some(h) = ground(tile) else {
                continue;
            };
            let bed = ha + (hb - ha) * t;
            let d = (bed - h).clamp(-max_cut_m, max_cut_m);
            if d.abs() >= MIN_GRADE_M {
                out.push((tile, d));
            }
        }
    }
    out
}

/// Grades each new line built to order once (needs live terrain heights).
pub fn grade_new_corridors_system(
    network: Res<CorridorNetwork>,
    bridges: Res<BridgeBook>,
    water: WaterTileProbe,
    params: Option<Res<WorldGenParams>>,
    dense: Option<Res<WorldGenDenseTerrainCache>>,
    mut queue: ResMut<GroundDeformQueue>,
    mut graded: Local<HashSet<u32>>,
) {
    if !network.is_changed() && !bridges.is_changed() {
        return;
    }
    let (Some(params), Some(dense)) = (params, dense) else {
        return;
    };
    if dense.elevation.is_empty() {
        return;
    }
    graded.retain(|id| network.segment(*id).is_some());
    let range = params.elevation_range_m;
    let (w, h) = (dense.width as i32, dense.height as i32);
    let ground = |t: IVec2| {
        (t.x >= 0 && t.y >= 0 && t.x < w && t.y < h).then(|| dense.elevation[(t.y * w + t.x) as usize] * range)
    };
    let fresh: Vec<&super::corridor_network::CorridorSegment> =
        network.segments.iter().filter(|s| s.charged && !graded.contains(&s.id)).collect();
    for seg in fresh {
        graded.insert(seg.id);
        let nodes: Vec<Vec2> = seg.nodes.iter().map(|t| Vec2::new(t.x as f32 + 0.5, t.z as f32 + 0.5)).collect();
        let spans: Vec<(Vec2, Vec2)> =
            bridges.bridges.iter().filter(|b| b.segment_id == seg.id).map(|b| (b.a, b.b)).collect();
        let on_bridge = |t: IVec2| {
            let p = t.as_vec2() + Vec2::splat(0.5);
            spans.iter().any(|(a, b)| {
                let d = *b - *a;
                let s = ((p - *a).dot(d) / d.length_squared().max(1e-6)).clamp(0.0, 1.0);
                (*a + d * s).distance(p) < 1.0
            })
        };
        let skip = |t: IVec2| water.depth_m(t) > WET_M || on_bridge(t);
        let cells = grade_cells(&nodes, mode_spec(tool_kind(&seg.kind)).max_cut_m, &ground, &skip);
        if !cells.is_empty() {
            bevy::log::debug!(target: "construction::grading", segment = seg.id, tiles = cells.len(), "line graded");
            queue.push(GroundDeformOp::Grade { cells });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bump_is_cut_a_dip_is_filled_water_is_left_alone() {
        // Flat at 100 m with a 3 m bump at x = 5 and a 2 m dip at x = 8; water at x = 10.
        let ground = |t: IVec2| {
            Some(match t.x {
                5 => 103.0,
                8 => 98.0,
                _ => 100.0,
            })
        };
        let skip = |t: IVec2| t.x == 10;
        let cells = grade_cells(&[Vec2::new(0.5, 0.5), Vec2::new(14.5, 0.5)], 8.0, &ground, &skip);
        let at = |x: i32| cells.iter().find(|c| c.0.x == x).map(|c| c.1);
        assert_eq!(at(5), Some(-3.0), "cut the bump");
        assert_eq!(at(8), Some(2.0), "fill the dip");
        assert_eq!(at(10), None, "water is bridged, not filled");
        assert_eq!(at(3), None, "already on the bed");
    }

    #[test]
    fn grading_stops_at_the_modes_cut_limit() {
        let ground = |t: IVec2| Some(if t.x == 4 { 130.0 } else { 100.0 });
        let cells = grade_cells(&[Vec2::new(0.5, 0.5), Vec2::new(8.5, 0.5)], 8.0, &ground, &|_| false);
        assert_eq!(cells.iter().find(|c| c.0.x == 4).map(|c| c.1), Some(-8.0));
    }
}
