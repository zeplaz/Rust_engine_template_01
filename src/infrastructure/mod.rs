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
    initial_utility_power_connected, sync_utility_connection_power_system,
    PowerLine, UtilityAuthoringMode, UtilityAuthoringTool,
    UtilityConnection, UtilityLink, UtilityNetworkKind, UtilityNetworkSnapshot,
    UTILITY_NETWORK_SCHEMA_V1, VoltageClass, WaterPipe,
};

