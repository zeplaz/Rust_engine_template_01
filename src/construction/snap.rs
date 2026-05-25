//! Grid + executed-road node snap for path placement (PHASE2-BUILD-18).

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;

use super::construction_pipeline::ExecutedRoadNetwork;

/// Snap target kinds (Round 2 — node magnetism today; intersection entity later).
#[derive(Clone, Debug)]
pub enum SnapTarget {
    RoadNode(BuildSiteTile),
    Grid(Vec3),
}

impl SnapTarget {
    #[must_use]
    pub fn hint_label(&self) -> String {
        match self {
            SnapTarget::RoadNode(t) => format!("snap: road node ({},{})", t.x, t.z),
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
    roads: &ExecutedRoadNetwork,
) -> SnapPlacement {
    let mut out = world;
    let mut target = None;
    if settings.grid_snap && settings.grid_step > 0.0 {
        let step = settings.grid_step;
        let snapped = Vec3::new(
            (out.x / step).round() * step,
            out.y,
            (out.z / step).round() * step,
        );
        if (snapped.x - out.x).abs() > 1e-4 || (snapped.z - out.z).abs() > 1e-4 {
            target = Some(SnapTarget::Grid(snapped));
        }
        out = snapped;
    }
    if settings.node_snap {
        if let Some(node) = nearest_road_node(world, roads, settings.node_radius) {
            out.x = node.x as f32 + 0.5;
            out.z = node.z as f32 + 0.5;
            target = Some(SnapTarget::RoadNode(node));
        }
    }
    SnapPlacement { world: out, target }
}

#[must_use]
pub fn nearest_road_node(
    world: Vec3,
    roads: &ExecutedRoadNetwork,
    radius: f32,
) -> Option<BuildSiteTile> {
    let mut best: Option<(f32, BuildSiteTile)> = None;
    for tile in &roads.tiles {
        let dx = world.x - tile.x as f32 - 0.5;
        let dz = world.z - tile.z as f32 - 0.5;
        let d2 = dx * dx + dz * dz;
        if d2 <= radius * radius {
            if best.map_or(true, |(bd, _)| d2 < bd) {
                best = Some((d2, *tile));
            }
        }
    }
    best.map(|(_, t)| t)
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
        let out = snap_placement(Vec3::new(3.2, 0.0, 5.7), &s, &ExecutedRoadNetwork::default()).world;
        assert_eq!(out.x, 4.0);
        assert_eq!(out.z, 6.0);
    }
}
