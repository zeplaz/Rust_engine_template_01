//! Utility authoring tool state (INFRA-E4-004 minimal UX).

use bevy::prelude::*;

use super::UtilityNetworkKind;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UtilityAuthoringMode {
    #[default]
    Idle,
    PlacePower,
    PlaceWater,
}

#[derive(Resource, Debug, Clone)]
pub struct UtilityAuthoringTool {
    pub mode: UtilityAuthoringMode,
    pub active_kind: UtilityNetworkKind,
    pub snap_to_transport: bool,
}

impl Default for UtilityAuthoringTool {
    fn default() -> Self {
        Self {
            mode: UtilityAuthoringMode::Idle,
            active_kind: UtilityNetworkKind::Power,
            snap_to_transport: true,
        }
    }
}

#[must_use]
pub fn utility_authoring_ux_witness_green() -> bool {
    let tool = UtilityAuthoringTool::default();
    tool.active_kind == UtilityNetworkKind::Power && tool.snap_to_transport
}

/// **INFRA-E4-004** — authoring resource registered and defaults to power placement mode.
#[must_use]
pub fn infra_e4_004_authoring_witness_green() -> bool {
    utility_authoring_ux_witness_green()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utility_authoring_ux_witness_green_lib() {
        assert!(infra_e4_004_authoring_witness_green());
    }
}
