//! Built corridor network (PLAN-ROADS-v1 RD-0/RD-1): every committed road or rail line is its own
//! segment with nodes along it, never a flat tile list (a flat list paired road A's end with road
//! B's start into a phantom segment). Sole writer: the construction execute funnel (and undo/redo).
//!
//! - **Nodes along the way.** A segment carries a node at least every [`NODE_SPACING_TILES`] so a
//!   truck can join or leave it anywhere, not only at the two ends.
//! - **Junctions.** A new line whose end lies on an existing line of the same class, or that
//!   crosses it, shares a node there. A motorway (limited access) is joined only at its ends;
//!   other crossings of it are grade separated (no shared node). Road and rail never share nodes
//!   (level crossings are RD-7).
//! - **Graph.** [`CorridorNetwork::bake_snapshot`] turns the spans into transport edges (one per
//!   span, profile = road tier or rail profile) whose successors are the edges meeting them at a
//!   shared node.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::infrastructure::RoadTier;
use crate::strategic::BuildSiteTile;
use crate::systems::transport::{
    TransportEdgeRecord, TransportNetworkSnapshot, TransportNodeRecord, TRANSPORT_NETWORK_SCHEMA_V1,
};

/// Longest span (tiles) between two nodes of one segment.
pub const NODE_SPACING_TILES: f32 = 8.0;
/// A line end this close (tiles) to another line joins it.
const JOIN_SNAP_TILES: f32 = 0.75;

/// What a segment is.
#[derive(Clone, Debug, PartialEq)]
pub enum CorridorKind {
    Road { tier: RoadTier, lanes: u8 },
    Rail { profile: String },
}

impl CorridorKind {
    #[must_use]
    pub fn is_rail(&self) -> bool {
        matches!(self, CorridorKind::Rail { .. })
    }

    #[must_use]
    pub fn road_tier(&self) -> Option<RoadTier> {
        match self {
            CorridorKind::Road { tier, .. } => Some(*tier),
            CorridorKind::Rail { .. } => None,
        }
    }

    fn limited_access(&self) -> bool {
        self.road_tier().is_some_and(RoadTier::limited_access)
    }

    fn profile_id(&self) -> String {
        match self {
            CorridorKind::Road { tier, .. } => tier.profile_id().to_owned(),
            CorridorKind::Rail { profile } => profile.clone(),
        }
    }

    fn agents(&self) -> Vec<String> {
        if self.is_rail() {
            vec!["rail_train".into(), "train".into()]
        } else {
            vec!["road_vehicle".into()]
        }
    }
}

/// One committed line: ordered nodes head → tail.
#[derive(Clone, Debug, PartialEq)]
pub struct CorridorSegment {
    pub id: u32,
    pub kind: CorridorKind,
    pub nodes: Vec<BuildSiteTile>,
    /// Bridge type the builder picked for this line's water crossings (None = cheapest that fits).
    pub bridge_pref: Option<crate::infrastructure::bridge_types::BridgeType>,
}

impl CorridorSegment {
    #[must_use]
    pub fn head(&self) -> BuildSiteTile {
        self.nodes[0]
    }

    #[must_use]
    pub fn tail(&self) -> BuildSiteTile {
        *self.nodes.last().expect("segment has nodes")
    }

    /// Consecutive node pairs.
    pub fn spans(&self) -> impl Iterator<Item = (BuildSiteTile, BuildSiteTile)> + '_ {
        self.nodes.windows(2).map(|w| (w[0], w[1]))
    }
}

/// The built network (roads + rail). Sole writer: construction execute + undo/redo.
#[derive(Resource, Debug, Default, Clone)]
pub struct CorridorNetwork {
    pub segments: Vec<CorridorSegment>,
    pub marker_seq: u32,
    next_id: u32,
    /// Bumps on every change (readers re-derive on change).
    pub revision: u64,
}

fn centre(t: BuildSiteTile) -> Vec2 {
    Vec2::new(t.x as f32 + 0.5, t.z as f32 + 0.5)
}

fn tile_at(p: Vec2) -> BuildSiteTile {
    BuildSiteTile { x: p.x.floor().max(0.0) as u32, z: p.y.floor().max(0.0) as u32 }
}

/// Parameter (0..1) of the closest point on `a→b` to `p`, and the distance to it.
fn project(p: Vec2, a: Vec2, b: Vec2) -> (f32, f32) {
    let d = b - a;
    let l2 = d.length_squared();
    if l2 < 1e-6 {
        return (0.0, p.distance(a));
    }
    let t = ((p - a).dot(d) / l2).clamp(0.0, 1.0);
    (t, p.distance(a + d * t))
}

/// Proper crossing of `a→b` and `c→d` (parameters on each), ends excluded.
fn crossing(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> Option<(f32, f32)> {
    let r = b - a;
    let s = d - c;
    let den = r.perp_dot(s);
    if den.abs() < 1e-6 {
        return None;
    }
    let t = (c - a).perp_dot(s) / den;
    let u = (c - a).perp_dot(r) / den;
    let inner = |x: f32| (0.02..=0.98).contains(&x);
    (inner(t) && inner(u)).then_some((t, u))
}

/// Straight line head → tail with a node at least every [`NODE_SPACING_TILES`].
fn subdivide(head: BuildSiteTile, tail: BuildSiteTile) -> Vec<(f32, BuildSiteTile)> {
    let (a, b) = (centre(head), centre(tail));
    let n = (a.distance(b) / NODE_SPACING_TILES).ceil().max(1.0) as usize;
    let mut out: Vec<(f32, BuildSiteTile)> = (0..=n)
        .map(|k| {
            let t = k as f32 / n as f32;
            (t, if k == 0 { head } else if k == n { tail } else { tile_at(a + (b - a) * t) })
        })
        .collect();
    out.dedup_by(|x, y| x.1 == y.1);
    out
}

/// Insert `tile` into `nodes` between span `i` and `i + 1` unless it already is a node.
fn insert_node(nodes: &mut Vec<BuildSiteTile>, span: usize, tile: BuildSiteTile) {
    if !nodes.contains(&tile) {
        nodes.insert(span + 1, tile);
    }
}

impl CorridorNetwork {
    /// Adds a straight line head → tail, joined to existing lines of the same class. Returns the id.
    pub fn add_segment(&mut self, kind: CorridorKind, head: BuildSiteTile, tail: BuildSiteTile) -> Option<u32> {
        if head == tail {
            return None;
        }
        let (a, b) = (centre(head), centre(tail));
        // New line's nodes with their parameter along it (joins add more).
        let mut new_nodes = subdivide(head, tail);
        let new_limited = kind.limited_access();
        for seg in &mut self.segments {
            if seg.kind.is_rail() != kind.is_rail() {
                continue;
            }
            let old_limited = seg.kind.limited_access();
            let (seg_head, seg_tail) = (seg.head(), seg.tail());
            // Walk the existing spans back to front so insertions keep earlier indices valid.
            let spans: Vec<(usize, BuildSiteTile, BuildSiteTile)> =
                seg.spans().enumerate().map(|(i, (p, q))| (i, p, q)).collect();
            for &(i, p, q) in spans.iter().rev() {
                let (cp, cq) = (centre(p), centre(q));
                // New line ends on this span → the old line gets a node there.
                for end in [head, tail] {
                    let (_, dist) = project(centre(end), cp, cq);
                    if dist <= JOIN_SNAP_TILES {
                        let at_old_end = end == seg_head || end == seg_tail;
                        if !old_limited || at_old_end {
                            insert_node(&mut seg.nodes, i, end);
                        }
                    }
                }
                // Old line ends on the new line → the new line gets a node there.
                for end in [seg_head, seg_tail] {
                    if end != p && end != q {
                        continue;
                    }
                    let (t, dist) = project(centre(end), a, b);
                    let at_new_end = end == head || end == tail;
                    if dist <= JOIN_SNAP_TILES && (!new_limited || at_new_end) {
                        new_nodes.push((t, end));
                    }
                }
                // Crossing mid-span → shared node (grade separated when either is a motorway).
                if !new_limited && !old_limited {
                    if let Some((t, _u)) = crossing(a, b, cp, cq) {
                        let x = tile_at(a + (b - a) * t);
                        insert_node(&mut seg.nodes, i, x);
                        new_nodes.push((t, x));
                    }
                }
            }
        }
        new_nodes.sort_by(|x, y| x.0.total_cmp(&y.0));
        let mut nodes: Vec<BuildSiteTile> = Vec::with_capacity(new_nodes.len());
        for (_, t) in new_nodes {
            if nodes.last() != Some(&t) && !nodes.contains(&t) {
                nodes.push(t);
            }
        }
        // Ends stay ends.
        nodes.retain(|t| *t != head && *t != tail);
        nodes.insert(0, head);
        nodes.push(tail);
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.segments.push(CorridorSegment { id, kind, nodes, bridge_pref: None });
        self.revision = self.revision.wrapping_add(1);
        Some(id)
    }

    /// Records the bridge type picked for a line (read when its bridges register).
    pub fn set_bridge_pref(&mut self, id: u32, t: crate::infrastructure::bridge_types::BridgeType) {
        if let Some(s) = self.segments.iter_mut().find(|s| s.id == id) {
            s.bridge_pref = Some(t);
        }
    }

    /// Removes a segment (undo). Junction nodes it added to other lines stay as plain nodes.
    pub fn remove_segment(&mut self, id: u32) -> Option<CorridorSegment> {
        let i = self.segments.iter().position(|s| s.id == id)?;
        self.revision = self.revision.wrapping_add(1);
        Some(self.segments.remove(i))
    }

    #[must_use]
    pub fn segment(&self, id: u32) -> Option<&CorridorSegment> {
        self.segments.iter().find(|s| s.id == id)
    }

    /// Any line has a node on `tile`.
    #[must_use]
    pub fn has_node(&self, tile: BuildSiteTile) -> bool {
        self.segments.iter().any(|s| s.nodes.contains(&tile))
    }

    /// A road (not rail) has a node on `tile`.
    #[must_use]
    pub fn has_road_node(&self, tile: BuildSiteTile) -> bool {
        self.segments.iter().any(|s| !s.kind.is_rail() && s.nodes.contains(&tile))
    }

    /// Every node tile of every line.
    pub fn node_tiles(&self) -> impl Iterator<Item = BuildSiteTile> + '_ {
        self.segments.iter().flat_map(|s| s.nodes.iter().copied())
    }

    /// Road spans with their segment.
    pub fn road_spans(&self) -> impl Iterator<Item = (&CorridorSegment, BuildSiteTile, BuildSiteTile)> + '_ {
        self.segments
            .iter()
            .filter(|s| !s.kind.is_rail())
            .flat_map(|s| s.spans().map(move |(a, b)| (s, a, b)))
    }

    /// Rail spans with their segment.
    pub fn rail_spans(&self) -> impl Iterator<Item = (&CorridorSegment, BuildSiteTile, BuildSiteTile)> + '_ {
        self.segments
            .iter()
            .filter(|s| s.kind.is_rail())
            .flat_map(|s| s.spans().map(move |(a, b)| (s, a, b)))
    }

    /// Transport snapshot: one edge per span; successors = edges of the same class meeting it at a
    /// node (a motorway's inner nodes meet only motorway edges of the same segment).
    #[must_use]
    pub fn bake_snapshot(&self, marker_y: f32) -> TransportNetworkSnapshot {
        let key = |t: BuildSiteTile| format!("t{}_{}", t.x, t.z);
        struct E {
            seg: usize,
            head: BuildSiteTile,
            tail: BuildSiteTile,
            head_open: bool,
            tail_open: bool,
        }
        let mut edges: Vec<E> = Vec::new();
        let mut seen: std::collections::HashSet<(String, (u32, u32), (u32, u32))> = Default::default();
        for (si, s) in self.segments.iter().enumerate() {
            let last = s.nodes.len().saturating_sub(1);
            for (k, (p, q)) in s.spans().enumerate() {
                let pair = if (p.x, p.z) <= (q.x, q.z) { (p, q) } else { (q, p) };
                if !seen.insert((s.kind.profile_id(), (pair.0.x, pair.0.z), (pair.1.x, pair.1.z))) {
                    continue;
                }
                // A limited-access line is open to other lines only at its ends.
                let lim = s.kind.limited_access();
                edges.push(E {
                    seg: si,
                    head: p,
                    tail: q,
                    head_open: !lim || k == 0,
                    tail_open: !lim || k + 1 == last,
                });
            }
        }
        // node tile → edges touching it
        let mut at: HashMap<(u32, u32), Vec<usize>> = HashMap::new();
        for (i, e) in edges.iter().enumerate() {
            at.entry((e.head.x, e.head.z)).or_default().push(i);
            at.entry((e.tail.x, e.tail.z)).or_default().push(i);
        }
        let open_at = |e: &E, t: BuildSiteTile| if e.head == t { e.head_open } else { e.tail_open };
        let mut nodes: Vec<TransportNodeRecord> = Vec::new();
        let mut node_seen = std::collections::HashSet::new();
        let mut records = Vec::with_capacity(edges.len());
        for (i, e) in edges.iter().enumerate() {
            for t in [e.head, e.tail] {
                if node_seen.insert((t.x, t.z)) {
                    nodes.push(TransportNodeRecord { key: key(t), position: [t.x as f32, marker_y, t.z as f32] });
                }
            }
            let seg = &self.segments[e.seg];
            let mut successors: Vec<u64> = Vec::new();
            for t in [e.head, e.tail] {
                for &j in at.get(&(t.x, t.z)).into_iter().flatten() {
                    if j == i {
                        continue;
                    }
                    let o = &edges[j];
                    let other = &self.segments[o.seg];
                    if other.kind.is_rail() != seg.kind.is_rail() {
                        continue;
                    }
                    let same_line = o.seg == e.seg;
                    if same_line || (open_at(e, t) && open_at(o, t)) {
                        successors.push(j as u64);
                    }
                }
            }
            successors.sort_unstable();
            successors.dedup();
            records.push(TransportEdgeRecord {
                id: i as u64,
                head: key(e.head),
                tail: key(e.tail),
                successors,
                control_points: vec![
                    [e.head.x as f32, marker_y, e.head.z as f32],
                    [e.tail.x as f32, marker_y, e.tail.z as f32],
                ],
                profile: seg.kind.profile_id(),
                allowed_agents: seg.kind.agents(),
                ..Default::default()
            });
        }
        TransportNetworkSnapshot {
            schema_version: TRANSPORT_NETWORK_SCHEMA_V1,
            nodes,
            edges: records,
            construction: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(x: u32, z: u32) -> BuildSiteTile {
        BuildSiteTile { x, z }
    }

    fn gravel() -> CorridorKind {
        CorridorKind::Road { tier: RoadTier::Gravel, lanes: 2 }
    }

    #[test]
    fn two_separate_roads_do_not_join_into_a_phantom_segment() {
        let mut net = CorridorNetwork::default();
        net.add_segment(gravel(), t(0, 0), t(10, 0));
        net.add_segment(gravel(), t(40, 40), t(50, 40));
        let snap = net.bake_snapshot(0.0);
        let phantom = snap.edges.iter().any(|e| {
            (e.head == "t10_0" && e.tail == "t40_40") || (e.head == "t40_40" && e.tail == "t10_0")
        });
        assert!(!phantom);
        assert!(snap.edges.iter().all(|e| e.control_points[0][0] < 11.0 || e.control_points[0][0] >= 40.0));
    }

    #[test]
    fn a_long_road_has_nodes_along_it() {
        let mut net = CorridorNetwork::default();
        let id = net.add_segment(gravel(), t(0, 0), t(40, 0)).unwrap();
        let seg = net.segment(id).unwrap();
        assert!(seg.nodes.len() >= 6, "{:?}", seg.nodes);
        assert!(seg.spans().all(|(a, b)| centre(a).distance(centre(b)) <= NODE_SPACING_TILES + 1.0));
    }

    #[test]
    fn a_road_ending_on_another_road_joins_it() {
        let mut net = CorridorNetwork::default();
        net.add_segment(gravel(), t(0, 10), t(40, 10));
        net.add_segment(gravel(), t(20, 30), t(21, 10));
        let snap = net.bake_snapshot(0.0);
        let at = |k: &str| snap.edges.iter().filter(|e| e.head == k || e.tail == k).count();
        assert!(at("t21_10") >= 3, "T junction shares the node");
    }

    #[test]
    fn crossing_roads_share_a_node_but_cross_a_motorway_without_one() {
        let mut net = CorridorNetwork::default();
        net.add_segment(gravel(), t(0, 20), t(40, 20));
        net.add_segment(gravel(), t(20, 0), t(20, 40));
        assert!(net.segments.iter().all(|s| s.nodes.contains(&t(20, 20))));

        let mut net = CorridorNetwork::default();
        net.add_segment(CorridorKind::Road { tier: RoadTier::Motorway, lanes: 4 }, t(0, 20), t(40, 20));
        net.add_segment(gravel(), t(20, 0), t(20, 40));
        let snap = net.bake_snapshot(0.0);
        let shared = snap.edges.iter().any(|e| {
            e.profile == "road_gravel"
                && e.successors.iter().any(|&s| snap.edges[s as usize].profile == "road_motorway")
        });
        assert!(!shared, "overpass: no access to the motorway mid-span");
    }

    #[test]
    fn rail_and_road_never_share_graph_edges() {
        let mut net = CorridorNetwork::default();
        net.add_segment(gravel(), t(0, 20), t(40, 20));
        net.add_segment(CorridorKind::Rail { profile: "standard_gauge_freight".into() }, t(20, 0), t(20, 40));
        let snap = net.bake_snapshot(0.0);
        for e in &snap.edges {
            for &s in &e.successors {
                assert_eq!(e.profile.starts_with("road_"), snap.edges[s as usize].profile.starts_with("road_"));
            }
        }
    }

    #[test]
    fn undo_removes_only_its_own_line() {
        let mut net = CorridorNetwork::default();
        let a = net.add_segment(gravel(), t(0, 0), t(10, 0)).unwrap();
        let b = net.add_segment(gravel(), t(10, 0), t(10, 10)).unwrap();
        net.remove_segment(b);
        assert!(net.segment(a).is_some());
        assert!(net.has_node(t(10, 0)));
        assert!(!net.has_node(t(10, 10)));
    }
}
