//! Transport simulation data: topology, field per edge, cost weights (rulebooks A–B).
//! See `prompts/designer_questions/transport/transport_code_implementation_plan_v1.md`.

use bevy::prelude::*;
use std::collections::HashMap;

/// Stable edge key for the transport graph (bake / save mapping TBD in R8).
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, Debug, Default, serde::Serialize, serde::Deserialize,
)]
pub struct TransportEdgeId(pub u64);

/// Continuous pressure on one directed edge (Rulebook A).
#[derive(Clone, Debug)]
pub struct EdgeFieldState {
    pub congestion: f32,
    pub damage: f32,
    pub danger: f32,
    pub heat: f32,
    /// Baseline traversal time before field penalties (distance/speed abstraction).
    pub travel_time_base: f32,
}

impl Default for EdgeFieldState {
    fn default() -> Self {
        Self {
            congestion: 0.,
            damage: 0.,
            danger: 0.,
            heat: 0.,
            travel_time_base: 1.,
        }
    }
}

/// Directed adjacency: predecessor lane/edge → successors (Topology step).
#[derive(Resource, Default)]
pub struct TransportTopology {
    pub neighbors: HashMap<TransportEdgeId, Vec<TransportEdgeId>>,
}

#[derive(Resource, Default)]
pub struct TransportFieldStore {
    pub by_edge: HashMap<TransportEdgeId, EdgeFieldState>,
}

/// Tunable weights (RON / assets later).
#[derive(Resource, Clone, Debug)]
pub struct TransportCostWeights {
    pub congestion: f32,
    pub damage: f32,
    pub danger: f32,
    pub heat: f32,
}

impl Default for TransportCostWeights {
    fn default() -> Self {
        Self {
            congestion: 3.,
            damage: 5.,
            danger: 10.,
            heat: 1.,
        }
    }
}

/// Read-only traversal cost per edge for pathfinding (Rulebook B).
#[derive(Resource, Default)]
pub struct TransportCostCache {
    pub by_edge: HashMap<TransportEdgeId, f32>,
}

/// Authoring / bake metadata per edge (profiles, **R7** `allowed_agents`); not mutated by field integrate.
#[derive(Resource, Default)]
pub struct TransportEdgeDirectory {
    pub by_edge: HashMap<TransportEdgeId, TransportEdgeMeta>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransportEdgeMeta {
    pub profile: String,
    pub allowed_agents: Vec<String>,
}

/// **R7** coarse nav export: topology + cached costs + agent masks (G5 adapter reads this).
#[derive(Resource, Default, Clone, Debug)]
pub struct TransportNavExport {
    pub edges: Vec<NavExportEdge>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct NavExportEdge {
    pub id: TransportEdgeId,
    pub cost: f32,
    pub successors: Vec<TransportEdgeId>,
    pub allowed_agents: Vec<String>,
    pub profile: String,
}

/// Pure function: navigation reads this — does not mutate field.
#[inline]
pub fn edge_traversal_cost(
    field: &EdgeFieldState,
    w: &TransportCostWeights,
    base_distance: f32,
) -> f32 {
    let mut c = base_distance;
    c += field.congestion * w.congestion;
    c += field.damage * w.damage;
    c += field.danger * w.danger;
    c += field.heat * w.heat;
    c.max(0.001)
}
