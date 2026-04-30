//! Tooling / editor UI boundary for road vehicles (no direct ECS mutation here).

use bevy::prelude::*;

#[derive(Resource, Debug, Clone, Default)]
pub struct RoadVehicleToolsState {
    pub inspector_open: bool,
    pub selected_config_name: Option<String>,
}

pub struct RoadVehicleToolsUiPlugin;

impl Plugin for RoadVehicleToolsUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RoadVehicleToolsState>();
    }
}
