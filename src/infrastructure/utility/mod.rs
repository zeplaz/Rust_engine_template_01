//! Utility network types + snapshot (INFRA-E4-001).

mod authoring;
mod connection;
pub mod graph;

pub use authoring::{UtilityAuthoringMode, UtilityAuthoringTool};
pub use connection::{UtilityConnection, UtilityNetworkKind};
pub use connection::infra_e4_003_utility_connection_witness_green;
pub use authoring::infra_e4_004_authoring_witness_green;
pub use graph::{
    fixture_utility_network_snapshot, hydrate_utility_graph_from_snapshot,
    refresh_utility_network_live_witness_payload, UtilityGraph, UtilityGraphEdge,
    UtilityGraphNode, UtilityGraphPlugin,
};

use serde::{Deserialize, Serialize};

pub const UTILITY_NETWORK_SCHEMA_V1: u32 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoltageClass {
    Low,
    Medium,
    High,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UtilityLink {
    pub id: u64,
    pub head: String,
    pub tail: String,
    #[serde(default)]
    pub utility_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PowerLine {
    pub link_id: u64,
    pub voltage: VoltageClass,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WaterPipe {
    pub link_id: u64,
    pub diameter_mm: u16,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UtilityNetworkSnapshot {
    #[serde(default = "default_utility_schema")]
    pub schema_version: u32,
    #[serde(default)]
    pub nodes: Vec<String>,
    #[serde(default)]
    pub edges: Vec<UtilityLink>,
    #[serde(default)]
    pub power_lines: Vec<PowerLine>,
    #[serde(default)]
    pub water_pipes: Vec<WaterPipe>,
}

fn default_utility_schema() -> u32 {
    UTILITY_NETWORK_SCHEMA_V1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utility_network_snapshot_ron_roundtrip() {
        let snap = UtilityNetworkSnapshot {
            schema_version: UTILITY_NETWORK_SCHEMA_V1,
            nodes: vec!["n0".into()],
            edges: vec![UtilityLink {
                id: 1,
                head: "n0".into(),
                tail: "n1".into(),
                utility_type: "power".into(),
            }],
            power_lines: vec![PowerLine {
                link_id: 1,
                voltage: VoltageClass::Medium,
            }],
            water_pipes: vec![],
        };
        let ron = ron::ser::to_string(&snap).unwrap();
        let back: UtilityNetworkSnapshot = ron::from_str(&ron).unwrap();
        assert_eq!(back, snap);
    }
}
