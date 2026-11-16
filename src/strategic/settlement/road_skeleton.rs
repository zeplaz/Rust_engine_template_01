//! **TWN-PCG07 / PCG-07** — MST town skeleton + recursive ring subdivision.
//!
//! Dual-cite: NEW PCG-07 (aig3 8.4.2–8.4.3) · LEGACY `pcg` / road network.
//! Proposes road segments only — enqueues [`ConstructionPlanQueue`]; never writes
//! `TransportTopology` or `LogisticsGraph`.

use bevy::prelude::*;

use crate::ai::faction::rng::derive_keyed_seed;
use crate::construction::{ConstructionIntent, ConstructionPlanQueue, ConstructionType};
use crate::strategic::BuildSiteTile;

use super::district::{ring_bounds, DistrictBook, DistrictRecord};
use super::town::TownBook;

/// Axis-aligned or Manhattan road segment in world tiles (inclusive endpoints).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RoadSegmentProposal {
    pub head: IVec2,
    pub tail: IVec2,
}

impl RoadSegmentProposal {
    #[must_use]
    pub fn normalized(a: IVec2, b: IVec2) -> Option<Self> {
        if a == b {
            return None;
        }
        let (head, tail) = if (a.x, a.y) <= (b.x, b.y) {
            (a, b)
        } else {
            (b, a)
        };
        Some(Self { head, tail })
    }
}

/// Kruskal MST over town centers. Tie-break: seeded key so equal lengths stay deterministic.
#[must_use]
pub fn mst_town_road_segments(centers: &[IVec2], seed: u64) -> Vec<RoadSegmentProposal> {
    if centers.len() < 2 {
        return Vec::new();
    }
    let n = centers.len();
    let mut edges: Vec<(u64, u32, usize, usize)> = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let a = centers[i];
            let b = centers[j];
            let dist = (a.x - b.x).unsigned_abs() as u64 + (a.y - b.y).unsigned_abs() as u64;
            let tie = derive_keyed_seed(seed, ((i as u64) << 32) | j as u64);
            edges.push((dist, (tie & 0xffff_ffff) as u32, i, j));
        }
    }
    edges.sort_by(|l, r| l.0.cmp(&r.0).then(l.1.cmp(&r.1)).then(l.2.cmp(&r.2)).then(l.3.cmp(&r.3)));

    let mut parent: Vec<usize> = (0..n).collect();
    fn find(p: &mut [usize], mut x: usize) -> usize {
        while p[x] != x {
            p[x] = p[p[x]];
            x = p[x];
        }
        x
    }
    let mut out = Vec::with_capacity(n.saturating_sub(1));
    for (_, _, i, j) in edges {
        let ri = find(&mut parent, i);
        let rj = find(&mut parent, j);
        if ri == rj {
            continue;
        }
        parent[ri] = rj;
        if let Some(seg) = RoadSegmentProposal::normalized(centers[i], centers[j]) {
            out.push(seg);
        }
        if out.len() == n - 1 {
            break;
        }
    }
    out
}

/// Recursive AABB subdivision of a closed ring → internal cut segments (axis-aligned).
/// Stops when depth is exhausted or the ring bounds are smaller than `min_span` on both axes.
#[must_use]
pub fn subdivide_ring_edges(ring: &[IVec2], depth: u32, seed: u64, min_span: i32) -> Vec<RoadSegmentProposal> {
    let Some(bounds) = ring_bounds(ring) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    // Ring perimeter edges (skeleton of the district limit).
    let n = ring.len();
    let edge_count = if n >= 2 && ring[0] == ring[n - 1] { n - 1 } else { n };
    for i in 0..edge_count {
        if let Some(seg) = RoadSegmentProposal::normalized(ring[i], ring[(i + 1) % edge_count]) {
            out.push(seg);
        }
    }
    subdivide_bounds_into(bounds, depth, seed, min_span, &mut out);
    dedupe_segments(&mut out);
    out
}

fn subdivide_bounds_into(
    bounds: IRect,
    depth: u32,
    seed: u64,
    min_span: i32,
    out: &mut Vec<RoadSegmentProposal>,
) {
    if depth == 0 {
        return;
    }
    let w = bounds.width();
    let h = bounds.height();
    if w < min_span && h < min_span {
        return;
    }
    let split_x = w >= h && w >= min_span;
    let key = derive_keyed_seed(
        seed,
        ((bounds.min.x as u64) << 32) | (bounds.min.y as u64 & 0xffff_ffff),
    );
    if split_x {
        let mid = bounds.min.x + (w / 2).max(1);
        if mid > bounds.min.x && mid < bounds.max.x {
            if let Some(seg) = RoadSegmentProposal::normalized(
                IVec2::new(mid, bounds.min.y),
                IVec2::new(mid, bounds.max.y - 1),
            ) {
                out.push(seg);
            }
            let left = IRect {
                min: bounds.min,
                max: IVec2::new(mid, bounds.max.y),
            };
            let right = IRect {
                min: IVec2::new(mid, bounds.min.y),
                max: bounds.max,
            };
            subdivide_bounds_into(left, depth - 1, derive_keyed_seed(key, 1), min_span, out);
            subdivide_bounds_into(right, depth - 1, derive_keyed_seed(key, 2), min_span, out);
        }
    } else if h >= min_span {
        let mid = bounds.min.y + (h / 2).max(1);
        if mid > bounds.min.y && mid < bounds.max.y {
            if let Some(seg) = RoadSegmentProposal::normalized(
                IVec2::new(bounds.min.x, mid),
                IVec2::new(bounds.max.x - 1, mid),
            ) {
                out.push(seg);
            }
            let bottom = IRect {
                min: bounds.min,
                max: IVec2::new(bounds.max.x, mid),
            };
            let top = IRect {
                min: IVec2::new(bounds.min.x, mid),
                max: bounds.max,
            };
            subdivide_bounds_into(bottom, depth - 1, derive_keyed_seed(key, 3), min_span, out);
            subdivide_bounds_into(top, depth - 1, derive_keyed_seed(key, 4), min_span, out);
        }
    }
}

fn dedupe_segments(segs: &mut Vec<RoadSegmentProposal>) {
    segs.sort_by_key(|s| (s.head.x, s.head.y, s.tail.x, s.tail.y));
    segs.dedup();
}

/// MST between town centers + subdivision cuts inside each district ring.
#[must_use]
pub fn settlement_road_skeleton(
    towns: &TownBook,
    districts: &DistrictBook,
    seed: u64,
    subdivide_depth: u32,
    min_span: i32,
) -> Vec<RoadSegmentProposal> {
    let mut centers: Vec<IVec2> = towns.towns.values().map(|t| t.center_tile).collect();
    centers.sort_by_key(|c| (c.x, c.y));
    let mut segs = mst_town_road_segments(&centers, derive_keyed_seed(seed, 0x4d53_5400)); // "MST\0"
    let mut district_rows: Vec<&DistrictRecord> = districts.districts.values().collect();
    district_rows.sort_by_key(|d| d.id.0.clone());
    for (i, district) in district_rows.iter().enumerate() {
        if district.ring.len() < 3 {
            continue;
        }
        let d_seed = derive_keyed_seed(seed, 0x5355_4200 ^ (i as u64)); // "SUB"
        segs.extend(subdivide_ring_edges(
            &district.ring,
            subdivide_depth,
            d_seed,
            min_span,
        ));
    }
    dedupe_segments(&mut segs);
    segs
}

fn tile_to_build_site(tile: IVec2) -> Option<BuildSiteTile> {
    if tile.x < 0 || tile.y < 0 {
        return None;
    }
    Some(BuildSiteTile {
        x: tile.x as u32,
        z: tile.y as u32,
    })
}

/// Enqueue road plans through the construction funnel. Returns number of plans enqueued.
/// Does **not** mutate topology.
pub fn enqueue_settlement_road_skeleton(
    queue: &mut ConstructionPlanQueue,
    segments: &[RoadSegmentProposal],
) -> usize {
    let mut n = 0;
    for seg in segments {
        let Some(head) = tile_to_build_site(seg.head) else {
            continue;
        };
        let Some(tail) = tile_to_build_site(seg.tail) else {
            continue;
        };
        if head == tail {
            continue;
        }
        let mid = Vec2::new(
            (head.x as f32 + tail.x as f32) * 0.5,
            (head.z as f32 + tail.z as f32) * 0.5,
        );
        queue.enqueue(ConstructionIntent {
            entity_type: ConstructionType::RoadSegment { head, tail },
            world_position: mid,
            rotation: 0.0,
        });
        n += 1;
    }
    n
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::settlement::district::{core_ring_for_rect, DistrictRecord};
    use crate::strategic::settlement::ids::{DistrictId, TownId};
    use crate::strategic::settlement::town::{TownAccessPolicy, TownRecord};
    use bevy::math::IRect;

    #[test]
    fn twn_pcg07() {
        // MST: 3 towns → exactly 2 segments, deterministic.
        let centers = [
            IVec2::new(10, 10),
            IVec2::new(40, 10),
            IVec2::new(10, 40),
        ];
        let a = mst_town_road_segments(&centers, 99_001);
        let b = mst_town_road_segments(&centers, 99_001);
        assert_eq!(a, b);
        assert_eq!(a.len(), 2);

        // Subdivision: rectangular ring → perimeter + internal cuts.
        let rect = IRect::from_corners(IVec2::new(0, 0), IVec2::new(16, 16));
        let ring = core_ring_for_rect(rect);
        let cuts = subdivide_ring_edges(&ring, 2, 7, 4);
        assert!(cuts.len() > 4, "expected perimeter + cuts, got {}", cuts.len());

        // Full skeleton + enqueue-only (no topology resource touched).
        let mut towns = TownBook::default();
        towns.towns.insert(
            TownId("a".into()),
            TownRecord {
                id: TownId("a".into()),
                name: "A".into(),
                region_name: "A".into(),
                center_tile: IVec2::new(8, 8),
                population: 100,
                jobs: 20,
                housing: 40,
                faction: 1,
                access: TownAccessPolicy::FreeLabor,
                required_clearance: 0,
            },
        );
        towns.towns.insert(
            TownId("b".into()),
            TownRecord {
                id: TownId("b".into()),
                name: "B".into(),
                region_name: "B".into(),
                center_tile: IVec2::new(48, 8),
                population: 100,
                jobs: 20,
                housing: 40,
                faction: 1,
                access: TownAccessPolicy::FreeLabor,
                required_clearance: 0,
            },
        );
        let mut districts = DistrictBook::default();
        let d_rect = IRect::from_corners(IVec2::new(0, 0), IVec2::new(17, 17));
        districts.districts.insert(
            DistrictId("d0".into()),
            DistrictRecord {
                id: DistrictId("d0".into()),
                town_id: TownId("a".into()),
                name: "core".into(),
                tile_rect: d_rect,
                zoning_default: super::super::zoning::ZoningClass::MixedUse,
                ring: core_ring_for_rect(d_rect),
                style_rules: Default::default(),
                site_ids: Vec::new(),
            },
        );
        let segs = settlement_road_skeleton(&towns, &districts, 42, 2, 4);
        assert!(!segs.is_empty());
        let mut queue = ConstructionPlanQueue::default();
        let n = enqueue_settlement_road_skeleton(&mut queue, &segs);
        assert_eq!(n, queue.plans.len());
        assert!(n >= 1);
        // Dual-cite / authority: every plan is Requested RoadSegment — funnel owns topology later.
        assert!(queue.plans.iter().all(|p| matches!(
            p.intent.entity_type,
            ConstructionType::RoadSegment { .. }
        )));
    }
}
