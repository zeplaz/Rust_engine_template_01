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

/// **INFRA-E4-003** — operational sites receive [`UtilityConnection`] (no `has_power` flag).
#[must_use]
pub fn infra_e4_003_utility_connection_witness_green() -> bool {
    use bevy::app::App;
    use bevy::MinimalPlugins;

    use crate::construction::BuildingDefinitionRegistry;
    use crate::economy::activation::{
        activate_industrial_facilities_system, BuildingDefinitionRef,
    };
    use crate::strategic::{
        ConstructionSite, SiteArchetype, SiteConstructionPhase, SiteConstructionBook,
        SiteConstructionStatus, SiteId,
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<BuildingDefinitionRegistry>()
        .init_resource::<SiteConstructionBook>();
    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().entity_mut(entity).insert((
        ConstructionSite {
            site_id: 99,
            owner: entity,
            archetype: SiteArchetype::Factory,
            phase: SiteConstructionPhase::Operational,
            operational_readiness: 1.0,
        },
        BuildingDefinitionRef {
            catalog_id: "builtin:factory".into(),
        },
    ));
    {
        let mut book = app.world_mut().resource_mut::<SiteConstructionBook>();
        book.by_site.insert(
            SiteId(99),
            SiteConstructionStatus {
                phase: SiteConstructionPhase::Operational,
                progress: 1.0,
            },
        );
    }
    app.add_systems(Update, activate_industrial_facilities_system);
    app.update();
    app.world()
        .get::<UtilityConnection>(entity)
        .is_some_and(|u| u.kind == UtilityNetworkKind::Power && !u.connected)
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

    #[test]
    fn infra_e4_003_witness_green() {
        assert!(infra_e4_003_utility_connection_witness_green());
    }
}
