//! World-layer infrastructure (transport graph, utilities, profiles).
//!
//! Epic slices: [`src/dev/plan_infrastructure_world_layers_exec_001_v1.md`](../dev/plan_infrastructure_world_layers_exec_001_v1.md).

pub mod authoring;
pub mod profiles;
pub mod settlement;
pub mod transport;
pub mod utility;

pub use authoring::CorridorAuthoringSession;
pub use profiles::{
    CorridorProfileKind, InfrastructureProfilesPlugin, ProfileRegistry, RailProfile, RoadProfile,
    DEFAULT_CORRIDOR_PROFILES_RON,
};
pub use transport::InfrastructureTransportPlugin;
pub use settlement::{
    attach_settlement_to_nearest_transport_node, settlement_node_for_town, SettlementId,
    SettlementKind, SettlementNode,
};
pub use utility::{
    infra_e4_003_utility_connection_witness_green, infra_e4_004_authoring_witness_green,
    PowerLine, UtilityAuthoringMode, UtilityAuthoringTool, UtilityConnection, UtilityLink,
    UtilityNetworkKind, UtilityNetworkSnapshot, UTILITY_NETWORK_SCHEMA_V1, VoltageClass, WaterPipe,
};

/// **INFRA-E0-003** — legacy `entities::structure::Road` / `Rrails` stubs gated off default build.
#[must_use]
pub fn infra_e0_003_legacy_transport_stubs_gated_witness_green() -> bool {
    !cfg!(feature = "legacy_transport_ecs_stubs")
}

/// Rollup for INFRA-E1 spine (lib tests + sim witness fields).
#[must_use]
pub fn infra_e1_transport_spine_witness_green() -> bool {
    transport::infra_e1_001_transport_graph_sync_witness_green()
        && transport::infra_e1_002_spline_subdivide_witness_green()
}