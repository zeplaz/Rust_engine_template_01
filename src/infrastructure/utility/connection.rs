//! Building ↔ utility graph attachment (INFRA-E4-003).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UtilityNetworkKind {
    Power,
    Water,
    Sewer,
    Gas,
    Telecom,
}

#[derive(Component, Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UtilityConnection {
    pub network_id: u64,
    pub kind: UtilityNetworkKind,
    /// Normalized demand 0..1 for activation gates.
    pub demand: f32,
    pub connected: bool,
}

impl UtilityConnection {
    #[must_use]
    pub fn power(network_id: u64, demand: f32, connected: bool) -> Self {
        Self {
            network_id,
            kind: UtilityNetworkKind::Power,
            demand: demand.clamp(0.0, 1.0),
            connected,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utility_connection_serde_roundtrip() {
        let c = UtilityConnection::power(42, 0.75, true);
        let json = serde_json::to_string(&c).unwrap();
        let back: UtilityConnection = serde_json::from_str(&json).unwrap();
        assert_eq!(back, c);
    }

}
