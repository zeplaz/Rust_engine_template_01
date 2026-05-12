use crate::entities::production::core::ManufacturingCorePlugin;
use crate::entities::vehicles::tools_ui::RoadVehicleToolsUiPlugin;
use crate::gui::{
    editor::map_editor::MapEditorPlugin, AppShellPlugin, BaseMenuPlugin, BuildPlanningPlugin,
    DiagnosticsUiPlugin, FactionToolsUiPlugin, GameplayCapturePlugin, InGameHudPlugin,
    KeybindingsOptionsPlugin, LogisticsTargetsPanelPlugin, MainWorldCamera, MapCameraPlugin, SplashPlugin,
    StrategicToolingPlugin, UiThemePlugin,
};
#[cfg(feature = "bevy_tilemap_adapter")]
use crate::render::TilemapAdapterPlugin;
use crate::systems::production::{
    ProductionRuntimePlugin, ProductionSerializationPlugin, ProductionToolsUiPlugin,
};
use crate::systems::damage::DamageSystem;
use crate::systems::navigation::NavigationSchedulePlugin;
use crate::scenario::ScenarioScriptingPlugin;
use crate::systems::sim_control::SimControlPlugin;
use crate::systems::transport::{TransportSchedule, TransportSimulationPlugin};
use crate::strategic::StrategicFieldPipeline;
use crate::render::{GpuWeatherFireFieldPlugin, LocalLightPlugin, TileWorldFallbackPlugin};
use crate::systems::{
    configure_chunk_environment_sets,
    AtmospherePlugin, ChunkEnvironmentPersistPlugin, ChunkSimLodPlugin, EcologyPlugin, FirePlugin,
    WeatherPlugin,
};
use crate::systems::terrain::MaterialUnificationPlugin;
use crate::terrain::generation::WorldGenToolsPlugin;
use super::TestHarnessPlugin;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::window::{PresentMode, Window, WindowPlugin};
use bevy_egui::EguiPlugin;

/// Root camera for **Bevy UI** (splash, in-game HUD). Without this, the window stays clear/black.
fn spawn_primary_ui_camera(mut commands: Commands) {
    commands.spawn((MainWorldCamera, Camera2d));
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
            .add_plugins(LocalLightPlugin)
            .add_plugins(TestHarnessPlugin)
            .add_plugins(EguiPlugin::default())
            .add_plugins(UiThemePlugin)
            .add_plugins(SplashPlugin)
            .add_plugins(BaseMenuPlugin)
            .add_plugins(AppShellPlugin)
            .add_plugins(MapEditorPlugin)
            // Sim loop control (pause / step / speed / monotonic tick).
            .add_plugins(SimControlPlugin)
            // Scenario script host (Wave 1): drains one step per frame before sim tick.
            .add_plugins(ScenarioScriptingPlugin);
        configure_chunk_environment_sets(app);
        app.add_plugins(ChunkEnvironmentPersistPlugin)
            .add_plugins(ChunkSimLodPlugin)
            .add_plugins(FirePlugin)
            .add_plugins(EcologyPlugin)
            .add_plugins(AtmospherePlugin)
            .add_plugins(WeatherPlugin)
            .add_plugins(GpuWeatherFireFieldPlugin)
            .add_plugins(TransportSimulationPlugin)
            // Nav: damage/speed adjustments after transport cost cache; motion stage after damage (S2).
            .add_plugins(NavigationSchedulePlugin)
            .add_plugins(DamageSystem)
            .add_plugins(MaterialUnificationPlugin)
            .add_plugins(crate::gui::editor::editor_world_commit_bridge::EditorWorldCommitBridgePlugin)
            .add_plugins(crate::strategic::StrategicFieldsAndAiPlugin)
            .add_plugins(crate::strategic::GpuBridgePlugin);
        // Fire visual extract: one sim pass → buffer; then pooled local lights collect messages.
        app.configure_sets(
            Update,
            crate::render::extraction::FireExtractSet::BuildProfiles
                .after(crate::systems::atmosphere::AtmospherePipelineSet::VisualExtract),
        );
        app.configure_sets(
            Update,
            crate::render::LocalLightExtractSet::Collect
                .after(crate::render::extraction::FireExtractSet::EmitParticles),
        );
        app.configure_sets(
            Update,
            crate::systems::atmosphere::AtmospherePipelineSet::Diagnostics
                .before(TransportSchedule::Topology),
        );
        app.configure_sets(
            Update,
            StrategicFieldPipeline::GraphSync.after(TransportSchedule::CostCache),
        );
        #[cfg(feature = "bevy_tilemap_adapter")]
        app.add_plugins(TilemapAdapterPlugin);
        // Plugin order still matters for init; cross-simulation ordering uses SystemSet edges
        // (see `SimControlSystemSet`, `TransportSchedule` — `prompts/guides/ecs_systems_schedule_runbook_v1.md`).
        app.add_plugins(KeybindingsOptionsPlugin)
            .add_plugins(GameplayCapturePlugin)
            .add_plugins(MapCameraPlugin)
            .add_plugins(TileWorldFallbackPlugin)
            .add_plugins(DiagnosticsUiPlugin)
            .add_plugins(FactionToolsUiPlugin)
            .add_plugins(crate::gui::AiExplainabilityPlugin)
            .add_plugins(StrategicToolingPlugin)
            .add_plugins(InGameHudPlugin)
            .add_plugins(BuildPlanningPlugin)
            .add_plugins(LogisticsTargetsPanelPlugin)
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
            "Engine initialized. Optional: `--test weather` / `--test fire` / `--test atmosphere` / `--test visual` for sample worlds. Keys: F1 options · F2 pressure composer · F3 diagnostics · F7 agent perms · F11/F12 capture; RON under user config · captures under APPDATA/proc_A_dine01/captures."
        );
    }
}