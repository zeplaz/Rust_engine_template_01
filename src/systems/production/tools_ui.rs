use bevy::prelude::*;

/// Tooling-facing production state (editor/test controls).
#[derive(Resource, Debug, Clone)]
pub struct ProductionToolsState {
    pub enabled: bool,
    pub selected_domain: ProductionDomain,
}

impl Default for ProductionToolsState {
    fn default() -> Self {
        Self {
            enabled: false,
            selected_domain: ProductionDomain::Power,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionDomain {
    Power,
    Concrete,
    Aluminum,
    ManufacturingCore,
}

/// Tools-only plugin boundary. UI rendering systems can be attached here.
pub struct ProductionToolsUiPlugin;

impl Plugin for ProductionToolsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProductionToolsState>();
    }
}
