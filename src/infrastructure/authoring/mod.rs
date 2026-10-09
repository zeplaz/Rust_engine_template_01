//! **INFRA-E2-001** — corridor authoring session (ghost until confirm; no snapshot mutation).

use bevy::prelude::*;

use crate::infrastructure::transport::graph::{TransportEdge, TransportGraph};
use crate::infrastructure::transport::junction::{ensure_edge_endpoints, rebuild_junction_metadata};
use crate::infrastructure::transport::spline::{subdivide_edge_with_radius, SplineError};
use crate::infrastructure::profiles::{CorridorProfileKind, ProfileRegistry};
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
/// Without a registry the edge uses the built-in local-road defaults.
pub fn confirm_corridor_authoring(
    session: &CorridorAuthoringSession,
    graph: &mut TransportGraph,
    next_edge_id: TransportEdgeId,
) -> Result<TransportEdgeId, SplineError> {
    confirm_corridor_authoring_with_profiles(session, None, graph, next_edge_id)
}

/// Same as [`confirm_corridor_authoring`], resolving `session.profile_id` through the loaded
/// [`ProfileRegistry`]: corridor class, allowed agents and the profile's minimum turn radius
/// (the session radius can only tighten the curve limit upward, never below the profile).
pub fn confirm_corridor_authoring_with_profiles(
    session: &CorridorAuthoringSession,
    registry: Option<&ProfileRegistry>,
    graph: &mut TransportGraph,
    next_edge_id: TransportEdgeId,
) -> Result<TransportEdgeId, SplineError> {
    if session.control_points.len() < 2 {
        return Err(SplineError::TooFewControlPoints);
    }
    let resolved = registry.and_then(|r| r.resolve(&session.profile_id));
    let (turn_radius_m, allowed_agents, corridor) = match &resolved {
        Some(kind) => (
            kind.turn_radius_m().max(session.min_turn_radius_m),
            match kind {
                CorridorProfileKind::Road(p) => p.allowed_agents.clone(),
                CorridorProfileKind::Rail(p) => p.allowed_agents.clone(),
            },
            kind.corridor_class(),
        ),
        None => (
            session.min_turn_radius_m,
            vec!["truck".into()],
            corridor_class_from_profile(&session.profile_id),
        ),
    };
    let cps: Vec<[f32; 3]> = session.control_points.iter().map(|p| p.to_array()).collect();
    let samples = subdivide_edge_with_radius(&cps, turn_radius_m, 6)?;
    let head_pos = Vec3::from_array(samples.first().expect("samples").position);
    let tail_pos = Vec3::from_array(samples.last().expect("samples").position);
    let control_points: Vec<[f32; 3]> = samples.iter().map(|s| s.position).collect();

    let (head, tail) = ensure_edge_endpoints(graph, next_edge_id, head_pos, tail_pos);
    graph.insert_edge(
        next_edge_id,
        TransportEdge {
            head,
            tail,
            profile_id: session.profile_id.clone(),
            control_points,
            corridor,
            allowed_agents,
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

    #[test]
    fn confirm_with_registry_uses_rail_profile_class_and_agents() {
        let reg = ProfileRegistry::load_default_example().expect("example profiles");
        let mut session = CorridorAuthoringSession {
            profile_id: "standard_gauge_freight".into(),
            min_turn_radius_m: 1.0,
            ..Default::default()
        };
        session.push_control_point(Vec3::new(0.0, 0.0, 0.0));
        session.push_control_point(Vec3::new(400.0, 0.0, 0.0));
        let mut graph = TransportGraph::default();
        let id = confirm_corridor_authoring_with_profiles(
            &session,
            Some(&reg),
            &mut graph,
            TransportEdgeId(7),
        )
        .expect("confirm");
        let edge = graph.edges.get(&id).expect("edge");
        assert_eq!(edge.corridor, crate::systems::transport::CorridorClass::Rail);
        assert!(edge.allowed_agents.iter().any(|a| a == "freight"));
    }
}
