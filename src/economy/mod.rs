//! Economy integration — construction → industrial activation, resource flow (Phase 4).

pub mod activation;
pub mod concrete_batch;
pub mod logistics;
pub mod logistics_bridge;
pub mod resource_flow;
pub mod site_placement;
pub mod spatial_district;
pub mod supply_chain;

pub use activation::IndustrialActivationPlugin;
pub use concrete_batch::{ConcreteBatchRegistered, ConcreteBatchState};
pub use logistics::LogisticsThroughputPlugin;
pub use logistics_bridge::{FacilityPortal, FacilityPortalRegistered, PortalAttachmentMap};
pub use resource_flow::{
    flow_node_from_definition, register_resource_flow_nodes_system, resource_type_from_tag,
    FacilityFlowState, ResourceFlowNode, ResourceFlowNodeRegistered, ResourceFlowPlugin,
    ResourceFlowRegistry, ResourceFlowSimWitness, ResourceRate, TransportMode,
};
pub use site_placement::{ensure_site_world_transform_system, site_world_position};
pub use spatial_district::{
    attach_district_anchors_system, measure_spatial_industrial_district_system,
    IndustrialDistrictAnchor, IndustrialDistrictSnapshot, SpatialDistrictPlugin,
};
pub use supply_chain::{
    electrical_from_power_units, insert_supply_chain_runtime,
    insert_supply_chain_runtime_for_catalog, IndustrialSupplyChainMembership,
};
