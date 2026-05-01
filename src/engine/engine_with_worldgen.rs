use crate::entities::production::core::ManufacturingCorePlugin;
use crate::entities::vehicles::RoadVehicleToolsUiPlugin;
use crate::gui::{
    DiagnosticsUiPlugin, FactionToolsUiPlugin, HudQuickMenuPlugin, InGameHudPlugin,
    KeybindingsOptionsPlugin, LogisticsTargetsPanelPlugin,
};
#[cfg(feature = "bevy_tilemap_adapter")]
use crate::render::TilemapAdapterPlugin;
use crate::systems::production::{
    ProductionRuntimePlugin, ProductionSerializationPlugin, ProductionToolsUiPlugin,
};
use crate::systems::sim_control::SimControlPlugin;
use crate::systems::terrain::MaterialUnificationPlugin;
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
            .add_plugins(MaterialUnificationPlugin);
        #[cfg(feature = "bevy_tilemap_adapter")]
        app.add_plugins(TilemapAdapterPlugin);
        app.add_plugins(KeybindingsOptionsPlugin)
            .add_plugins(DiagnosticsUiPlugin)
            .add_plugins(FactionToolsUiPlugin)
            .add_plugins(InGameHudPlugin)
            .add_plugins(LogisticsTargetsPanelPlugin)
            // World generation editor + runtime.
            .add_plugins(WorldGenToolsPlugin)
            .add_plugins(HudQuickMenuPlugin)
            // Production stack.
            .add_plugins(ProductionRuntimePlugin)
            .add_plugins(ManufacturingCorePlugin)
            .add_plugins(ProductionSerializationPlugin)
            .add_plugins(ProductionToolsUiPlugin)
            // Surface logistics tools.
            .add_plugins(RoadVehicleToolsUiPlugin);

        info!(
            "Engine initialized. Key bindings default: F1 options · F3 diagnostics · F4 faction · F8 world gen · F9 logistics cycle · F10 logistics list · P pause sim · / egui scale — edit in Options; saved RON under user config path."
        );
    }
}