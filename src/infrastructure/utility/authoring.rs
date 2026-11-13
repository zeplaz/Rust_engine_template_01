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

#[cfg(test)]
mod tests {
    

}
