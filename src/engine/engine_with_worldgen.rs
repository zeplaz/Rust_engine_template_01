use crate::entities::production::core::ManufacturingCorePlugin;
use crate::entities::vehicles::RoadVehicleToolsUiPlugin;
use crate::gui::{
    BaseMenuPlugin, DiagnosticsUiPlugin, FactionToolsUiPlugin, HudQuickMenuPlugin,
    InGameHudPlugin, KeybindingsOptionsPlugin, LogisticsTargetsPanelPlugin, SplashPlugin,
};
#[cfg(feature = "bevy_tilemap_adapter")]
use crate::render::TilemapAdapterPlugin;
use crate::systems::production::{
    ProductionRuntimePlugin, ProductionSerializationPlugin, ProductionToolsUiPlugin,
};
use crate::systems::sim_control::SimControlPlugin;
use crate::systems::terrain::MaterialUnificationPlugin;
use crate::terrain::generation::WorldGenToolsPlugin;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin};
use bevy_egui::EguiPlugin;

/// Root camera for **Bevy UI** (splash, in-game HUD). Without this, the window stays clear/black.
fn spawn_primary_ui_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        // Resolve `assets/` from the crate root so running `target/debug/proc_A_dine01.exe` from any CWD still finds files.
        let assets_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let asset_file_path = assets_root.to_string_lossy().into_owned();

        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_file_path,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                }),
        )
            .add_systems(Startup, spawn_primary_ui_camera)
            .add_plugins(EguiPlugin::default())
            .add_plugins(SplashPlugin)
            .add_plugins(BaseMenuPlugin)
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