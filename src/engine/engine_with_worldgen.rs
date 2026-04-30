use crate::entities::production::core::ManufacturingCorePlugin;
use crate::entities::vehicles::RoadVehicleToolsUiPlugin;
use crate::gui::{DiagnosticsUiPlugin, FactionToolsUiPlugin};
use crate::systems::production::{
    ProductionRuntimePlugin, ProductionSerializationPlugin, ProductionToolsUiPlugin,
};
use crate::systems::sim_control::SimControlPlugin;
use crate::terrain::generation::WorldGenToolsPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins)
           .add_plugins(EguiPlugin::default())
           // Sim loop control (pause / step / speed / monotonic tick).
           .add_plugins(SimControlPlugin)
           // Devtools UI for iteration loop.
           .add_plugins(DiagnosticsUiPlugin)
           .add_plugins(FactionToolsUiPlugin)
           // World generation editor + runtime.
           .add_plugins(WorldGenToolsPlugin)
           // Production stack.
           .add_plugins(ProductionRuntimePlugin)
           .add_plugins(ManufacturingCorePlugin)
           .add_plugins(ProductionSerializationPlugin)
           .add_plugins(ProductionToolsUiPlugin)
           // Surface logistics tools.
           .add_plugins(RoadVehicleToolsUiPlugin);

        info!(
            "Engine initialized. Hotkeys: F3 Diagnostics · F4 Faction Tools · F7 Agents · F8 World Generator."
        );
    }
}