//! **INFRA-E2-001** — corridor authoring session (ghost until confirm; no snapshot mutation).

use bevy::prelude::*;

use crate::infrastructure::transport::graph::{TransportEdge, TransportGraph};
use crate::infrastructure::transport::junction::{ensure_edge_endpoints, rebuild_junction_metadata};
use crate::infrastructure::transport::spline::{subdivide_edge_with_radius, SplineError};
use crate::infrastructure::profiles::RoadProfile;
use crate::systems::transport::{corridor_class_from_profile, TransportEdgeId};

/// Ghost-only polyline + profile picker state (map editor tool attaches here later).
#[derive(Resource, Clone, Debug, Default)]
pub struct CorridorAuthoringSession {
    pub control_points: Vec<Vec3>,
    pub profile_id: String,
    pub min_turn_radius_m: f32,
    pub confirmed: bool,
}

impl CorridorAuthoringSession {
    pub fn push_control_point(&mut self, p: Vec3) {
        self.control_points.push(p);
    }

    pub fn clear(&mut self) {
        self.control_points.clear();
        self.confirmed = false;
    }
}

/// Confirm ghost session → append one subdivided edge on `TransportGraph` (T-GHOST-001: only on confirm).
pub fn confirm_corridor_authoring(
    session: &CorridorAuthoringSession,
    graph: &mut TransportGraph,
    next_edge_id: TransportEdgeId,
) -> Result<TransportEdgeId, SplineError> {
    if session.control_points.len() < 2 {
        return Err(SplineError::TooFewControlPoints);
    }
    let cps: Vec<[f32; 3]> = session.control_points.iter().map(|p| p.to_array()).collect();
    let profile = RoadProfile {
        id: session.profile_id.clone(),
        road_type: "local".into(),
        lanes: 2,
        speed_limit_kmh: 50,
        surface_tags: vec![],
        turn_radius_m: session.min_turn_radius_m,
        base_cost: 1.0,
        allowed_agents: vec!["truck".into()],
    };
    let samples = subdivide_edge_with_radius(&cps, profile.turn_radius_m, 6)?;
    let head_pos = Vec3::from_array(samples.first().expect("samples").position);
    let tail_pos = Vec3::from_array(samples.last().expect("samples").position);
    let control_points: Vec<[f32; 3]> = samples.iter().map(|s| s.position).collect();

    let (head, tail) = ensure_edge_endpoints(graph, next_edge_id, head_pos, tail_pos);
    let corridor = corridor_class_from_profile(&session.profile_id);
    graph.insert_edge(
        next_edge_id,
        TransportEdge {
            head,
            tail,
            profile_id: session.profile_id.clone(),
            control_points,
            corridor,
            allowed_agents: profile.allowed_agents,
        },
    );
    graph.adjacency.insert(next_edge_id, vec![]);
    rebuild_junction_metadata(graph);
    Ok(next_edge_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infra_e2_001_confirm_writes_edge_with_subdivided_control_points() {
        let mut session = CorridorAuthoringSession {
            profile_id: "default_road".into(),
            min_turn_radius_m: 4.0,
            ..Default::default()
        };
        session.push_control_point(Vec3::new(0.0, 0.0, 0.0));
        session.push_control_point(Vec3::new(6.0, 0.0, 0.0));
        session.push_control_point(Vec3::new(12.0, 0.0, 1.0));

        let mut graph = TransportGraph::default();
        let id = confirm_corridor_authoring(&session, &mut graph, TransportEdgeId(42))
            .expect("confirm");
        let edge = graph.edges.get(&id).expect("edge");
        assert!(edge.control_points.len() >= 2);
        assert_eq!(edge.profile_id, "default_road");
    }
}
