//! Transport graph core (INFRA-E1-*).
//!
//! **ECS-15 (gea4v2 16.2.2) — tool-side vs runtime:** [`TransportGraph`] is the
//! **tool/authoring** conditioned asset (editor bake / IF-TRG / R8 hydrate). Runtime
//! topology, field, and cost caches live under `systems/transport` and are rebuilt only
//! via [`sync_transport_runtime_from_graph`] — never by Studio/UI writing sim edges
//! directly. Same split as Studio ParamFrame → presentation massing (occupation stays
//! construction-funnel only).

pub mod graph;
pub mod junction;
pub mod plugin;
pub mod snapshot_bridge;
pub mod spline;
pub mod sync;

pub use graph::{TransportEdge, TransportGraph, TransportNode, TransportNodeId};
pub use junction::{rebuild_junction_metadata, JunctionKind, JUNCTION_MERGE_EPSILON};
pub use plugin::InfrastructureTransportPlugin;
pub use snapshot_bridge::{
    bake_transport_graph_from_ordered_markers, hydrate_transport_graph_from_snapshot,
    transport_network_snapshot_from_graph,
};
pub use spline::{
    subdivide_edge, subdivide_edge_for_profile,
    subdivide_edge_with_radius, SplineError, SubEdgeSample,
};
pub use sync::{
    sync_transport_runtime_from_graph,
};
