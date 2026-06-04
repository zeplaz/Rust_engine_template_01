//! Transport graph core (INFRA-E1-*).

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
    infra_e1_002_spline_subdivide_witness_green, subdivide_edge, subdivide_edge_for_profile,
    subdivide_edge_with_radius, SplineError, SubEdgeSample,
};
pub use sync::{
    infra_e1_001_transport_graph_sync_witness_green, sync_transport_runtime_from_graph,
};
