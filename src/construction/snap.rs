//! Grid + executed-road node snap for path placement (PHASE2-BUILD-18).

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;

use super::corridor_network::CorridorNetwork;

/// Snap target kinds (Round 2 — node magnetism today; intersection entity later).
#[derive(Clone, Debug)]
pub enum SnapTarget {
    RoadNode(BuildSiteTile),
    /// A point along an existing line (finishing here makes a junction).
    OnLine(BuildSiteTile),
    Grid(Vec3),
}

impl SnapTarget {
    /// Snapped onto an existing line (node or along it): a click here finishes the path.
    #[must_use]
    pub fn joins_line(&self) -> bool {
        matches!(self, SnapTarget::RoadNode(_) | SnapTarget::OnLine(_))
    }

    #[must_use]
    pub fn hint_label(&self) -> String {
        match self {
            SnapTarget::RoadNode(t) => format!("snap: joins the line at ({},{})", t.x, t.z),
            SnapTarget::OnLine(t) => format!("snap: joins along the line at ({},{})", t.x, t.z),
            SnapTarget::Grid(p) => format!("snap: grid ({:.1}, {:.1})", p.x, p.z),
        }
    }
}

#[derive(Clone, Debug)]
pub struct SnapPlacement {
    pub world: Vec3,
    pub target: Option<SnapTarget>,
}

#[derive(Resource, Debug, Clone)]
pub struct RoadSnapSettings {
    pub grid_snap: bool,
    pub node_snap: bool,
    pub grid_step: f32,
    pub node_radius: f32,
}

impl Default for RoadSnapSettings {
    fn default() -> Self {
        Self {
            grid_snap: true,
            node_snap: true,
            grid_step: 1.0,
            node_radius: 1.25,
        }
    }
}

#[must_use]
pub fn snap_placement(
    world: Vec3,
    settings: &RoadSnapSettings,
    roads: &CorridorNetwork,
    rail: bool,
) -> SnapPlacement {
    let mut out = world;
    let mut target = None;
    if settings.grid_snap && settings.grid_step > 0.0 {
        let step = settings.grid_step;
        // Tile centres (+0.5): the same place a node snap and the built road use.
        let snapped = Vec3::new(
            ((out.x - 0.5) / step).round() * step + 0.5,
            out.y,
            ((out.z - 0.5) / step).round() * step + 0.5,
        );
        if (snapped.x - out.x).abs() > 1e-4 || (snapped.z - out.z).abs() > 1e-4 {
            target = Some(SnapTarget::Grid(snapped));
        }
        out = snapped;
    }
    if settings.node_snap {
        if let Some(node) = nearest_road_node(world, roads, settings.node_radius, rail) {
            out.x = node.x as f32 + 0.5;
            out.z = node.z as f32 + 0.5;
            target = Some(SnapTarget::RoadNode(node));
        } else if let Some(tile) = nearest_point_on_line(world, roads, settings.node_radius, rail) {
            out.x = tile.x as f32 + 0.5;
            out.z = tile.z as f32 + 0.5;
            target = Some(SnapTarget::OnLine(tile));
        }
    }
    SnapPlacement { world: out, target }
}

#[must_use]
pub fn nearest_road_node(
    world: Vec3,
    roads: &CorridorNetwork,
    radius: f32,
    rail: bool,
) -> Option<BuildSiteTile> {
    let mut best: Option<(f32, BuildSiteTile)> = None;
    let tiles = roads.segments.iter().filter(|s| s.kind.is_rail() == rail).flat_map(|s| s.nodes.iter().copied());
    for tile in tiles {
        let dx = world.x - tile.x as f32 - 0.5;
        let dz = world.z - tile.z as f32 - 0.5;
        let d2 = dx * dx + dz * dz;
        if d2 <= radius * radius {
            if best.map_or(true, |(bd, _)| d2 < bd) {
                best = Some((d2, tile));
            }
        }
    }
    best.map(|(_, t)| t)
}

/// Tile on the nearest line of the same class within `radius` of `world` (projection).
#[must_use]
pub fn nearest_point_on_line(world: Vec3, roads: &CorridorNetwork, radius: f32, rail: bool) -> Option<BuildSiteTile> {
    let p = Vec2::new(world.x, world.z);
    let c = |t: BuildSiteTile| Vec2::new(t.x as f32 + 0.5, t.z as f32 + 0.5);
    let mut best: Option<(f32, Vec2)> = None;
    for seg in roads.segments.iter().filter(|s| s.kind.is_rail() == rail) {
        for (a, b) in seg.spans() {
            let (a, b) = (c(a), c(b));
            let d = b - a;
            let t = if d.length_squared() < 1e-6 { 0.0 } else { ((p - a).dot(d) / d.length_squared()).clamp(0.0, 1.0) };
            let q = a + d * t;
            let dist = p.distance(q);
            if dist <= radius && best.is_none_or(|(bd, _)| dist < bd) {
                best = Some((dist, q));
            }
        }
    }
    best.map(|(_, q)| BuildSiteTile { x: q.x.floor().max(0.0) as u32, z: q.y.floor().max(0.0) as u32 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_snap_quantizes_to_step() {
        let s = RoadSnapSettings {
            grid_snap: true,
            node_snap: false,
            grid_step: 2.0,
            node_radius: 1.0,
        };
        let out = snap_placement(
            Vec3::new(3.2, 0.0, 5.7),
            &s,
            &CorridorNetwork::default(),
            false,
        )
        .world;
        // Tile centres on the step grid (x.5), where nodes and the built road sit.
        assert_eq!(out.x, 2.5);
        assert_eq!(out.z, 6.5);
    }
}
