use crate::entities::production::core::ManufacturingCorePlugin;
use crate::entities::vehicles::tools_ui::RoadVehicleToolsUiPlugin;
use crate::gui::{
    editor::map_editor::MapEditorPlugin,
    editor::world_preview::{
        init_preview_render_contract_resources, WorldPreviewGpuRuntime,
    },
    AppShellPlugin, BaseMenuPlugin, BuildPlanningPlugin, DiagnosticsUiPlugin, FactionToolsUiPlugin,
    GameplayCapturePlugin, InGameHudPlugin, InGamePauseMenuPlugin, KeybindingsOptionsPlugin,
    LogisticsTargetsPanelPlugin,
    hud::TransmissionShellPlugin,
    hud::HudDockShellPlugin,
    MainWorldCamera, MapCameraPlugin, CameraFocusDebugPlugin, SplashPlugin, TileDebugRenderHost,
    ViewAuthorityPlugin,
    ViewRepresentationPlugin, StrategicToolingPlugin,
    UiThemePlugin,
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
use crate::compute::ComputeDispatchPlugin;
use crate::render::{
    FramePerfPlugin, GpuWeatherFireFieldPlugin, LocalLightPlugin, SharedOverlayFieldBuffersPlugin,
    StallWatchPlugin, Stage5ReadinessProfile, TileWorldFallbackPlugin, ViewportPipelinePlugin,
    ViewRuntimePlugin, GpuWaterParticlesPlugin, WaterSurfaceVisualPlugin,
    register_water_surface_draw, register_world_water_particle_draw,
};
use crate::systems::{
    configure_chunk_environment_sets,
    AtmospherePlugin, ChunkEnvironmentPersistPlugin, ChunkSimLodPlugin, EcologyPlugin, FirePlugin,
    WeatherPlugin,
};
use crate::systems::terrain::MaterialUnificationPlugin;
use crate::terrain::generation::WorldGenToolsPlugin;
use super::ux_orchestration::UxOrchestrationPlugin;
use super::worldgen_chrome_debug::WorldGenChromeDebugPlugin;
use crate::gui::debug::UiLayoutTreeDebugPlugin;
use crate::gui::hud::SimViewSyncDebugPlugin;
use crate::render::{DebugViewportOverlayPlugin, VisualDiagnosticsPlugin};
use super::test_harness::{TestHarnessMenuPlugin, TestHarnessPlugin, TestHarnessStatePlugin};
use super::DebugManeuverPlugin;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};
use bevy::window::{PresentMode, Window, WindowPlugin};
use bevy_egui::EguiPlugin;

/// Root camera for **Bevy UI** (splash, in-game HUD). Without this, the window stays clear/black.
fn spawn_primary_ui_camera(mut commands: Commands) {
    commands.spawn((MainWorldCamera, Camera2d, TileDebugRenderHost));
}

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::render::DebugRenderTraceConfig>();
        // Resolve `assets/` from the crate root so running `target/debug/proc_A_dine01.exe` from any CWD still finds files.
        let assets_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let asset_file_path = assets_root.to_string_lossy().into_owned();
        let present_mode = if std::env::var("PERF_NO_VSYNC")
            .ok()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        {
            PresentMode::AutoNoVsync
        } else {
            PresentMode::AutoVsync
        };

        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_file_path,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode,
                        ..default()
                    }),
                    ..default()
                }),
        )
            // PERF-PLAY: keep simulation in continuous mode; reactive low-power mode can sleep
            // near 1s between frames and makes camera/minimap feel "broken" under active play.
            .insert_resource(WinitSettings {
                focused_mode: UpdateMode::Continuous,
                unfocused_mode: UpdateMode::Continuous,
            })
            .add_systems(Startup, spawn_primary_ui_camera)
            .add_plugins(LocalLightPlugin)
            .add_plugins(DebugManeuverPlugin)
            .add_plugins(EguiPlugin::default())
            .add_plugins(UiThemePlugin)
            .add_plugins(SplashPlugin)
            .add_plugins(BaseMenuPlugin)
            .add_plugins(AppShellPlugin)
            .add_plugins(UxOrchestrationPlugin)
            .add_plugins((
                WorldGenChromeDebugPlugin,
                UiLayoutTreeDebugPlugin,
                SimViewSyncDebugPlugin,
                DebugViewportOverlayPlugin,
                VisualDiagnosticsPlugin,
                crate::render::GpuSurfaceTeardownPlugin,
            ))
            .add_plugins(MapEditorPlugin)
            // Sim loop control (pause / step / speed / monotonic tick).
            .add_plugins(SimControlPlugin)
            // Scenario script host (Wave 1): drains one step per frame before sim tick.
            .add_plugins(crate::sim::SimEffectsPlugin)
            .add_plugins(ScenarioScriptingPlugin);
        configure_chunk_environment_sets(app);
        app.add_plugins(crate::substrate::SubstratePlugin)
            .add_plugins(ChunkEnvironmentPersistPlugin)
            .add_plugins(crate::io::save::WorldSaveSpinePlugin)
            .add_plugins(crate::render::Stage6VirtualizationPlugin)
            .add_plugins(ChunkSimLodPlugin)
            .add_plugins(FirePlugin)
            .add_plugins(EcologyPlugin)
            .add_plugins(AtmospherePlugin)
            .add_plugins(WeatherPlugin)
            .add_plugins(FramePerfPlugin)
            .add_plugins(GpuWeatherFireFieldPlugin)
            .add_plugins(crate::infrastructure::InfrastructureProfilesPlugin)
            .add_plugins(crate::infrastructure::InfrastructureTransportPlugin)
            .add_plugins(crate::infrastructure::utility::UtilityGraphPlugin)
            .add_plugins(crate::render::InfrastructureOverlayPlugin)
            .add_plugins(TransportSimulationPlugin)
            .add_plugins(crate::engine::PlayScenarioPlugin)
            // Nav: damage/speed adjustments after transport cost cache; motion stage after damage (S2).
            .add_plugins(NavigationSchedulePlugin)
            .add_plugins(DamageSystem)
            .add_plugins(MaterialUnificationPlugin)
            .add_plugins(crate::gui::editor::editor_world_commit_bridge::EditorWorldCommitBridgePlugin)
            .add_plugins(crate::strategic::StrategicFieldsAndAiPlugin)
            .add_plugins(crate::strategic::GpuBridgePlugin);
        if crate::render::hanabi_witness::hanabi_l3_plugin_wired() {
            #[cfg(feature = "hanabi_l3")]
            app.add_plugins(crate::render::hanabi_embellishment::HanabiEmbellishmentPlugin);
        }
        // Fire visual extract: one sim pass → buffer; then pooled local lights collect messages.
        app.configure_sets(
            Update,
            crate::render::extraction::FireVisualFrameSet::BuildProfiles
                .after(crate::systems::atmosphere::AtmospherePipelineSet::VisualExtract),
        );
        app.configure_sets(
            Update,
            crate::render::LocalLightExtractSet::Collect
                .after(crate::render::extraction::FireVisualFrameSet::ProjectGpu),
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
            .add_plugins(CameraFocusDebugPlugin)
            .add_plugins(crate::gui::GpuTileDebugPlugin)
            .add_plugins(crate::gui::TileReadabilityPlugin)
            .add_plugins(crate::gui::StrategicIconInstancesPlugin)
            .add_plugins(ViewRepresentationPlugin)
            .add_plugins(StallWatchPlugin)
            .add_plugins(crate::render::VisualReadinessWitnessPlugin)
            .add_plugins(crate::dev::TestRunInstrumentationPlugin)
            .add_plugins(crate::dev::SimSpectrumAnalyticsPlugin)
            .add_plugins(ComputeDispatchPlugin);
        app.add_plugins((TestHarnessStatePlugin, TestHarnessMenuPlugin));
        let test_mode = app
            .world()
            .get_resource::<crate::engine::EngineLaunchArgs>()
            .is_some_and(|launch| launch.test_mode());
        if test_mode {
            app.add_plugins(TestHarnessPlugin);
        }
        init_preview_render_contract_resources(app);
        app.insert_resource(Stage5ReadinessProfile::FULL_APP);
        app.configure_sets(
            Update,
            (
                crate::compute::ComputeDispatchSystemSet::Dispatch
                    .after(crate::gui::WorldRepresentationSystemSet::ComputeFrame)
                    .after(crate::render::extraction::FireVisualFrameSet::BuildProfiles),
                crate::render::extraction::FireVisualFrameSet::ProjectGpu
                    .after(crate::compute::ComputeDispatchSystemSet::Dispatch),
            ),
        );
        app.add_plugins(TileWorldFallbackPlugin)
            .add_plugins(crate::render::TacticalVectorOverlayPlugin)
            .add_plugins(WaterSurfaceVisualPlugin)
            .add_plugins(GpuWaterParticlesPlugin);
        register_water_surface_draw(app);
        register_world_water_particle_draw(app);
        app.add_plugins(ViewportPipelinePlugin)
            .add_plugins(ViewRuntimePlugin)
            .add_plugins(ViewAuthorityPlugin)
            .add_plugins(SharedOverlayFieldBuffersPlugin)
            .add_plugins(DiagnosticsUiPlugin)
            .add_plugins(FactionToolsUiPlugin)
            .add_plugins(crate::gui::AiExplainabilityPlugin)
            .add_plugins(StrategicToolingPlugin)
            .add_plugins(InGameHudPlugin)
            .add_plugins(crate::gui::hud::SimulationSessionPlugin)
            .add_plugins(crate::render::VfxCaptureHookPlugin)
            .add_plugins(MapCameraPlugin)
            .add_plugins(InGamePauseMenuPlugin)
            .add_plugins(TransmissionShellPlugin)
            .add_plugins(HudDockShellPlugin)
            .add_plugins(crate::gui::hud::OverlayShellPlugin)
            .add_plugins(crate::gui::hud::Stage7UiShellPlugin)
            .add_plugins(BuildPlanningPlugin)
            .add_plugins(crate::gui::AssemblySnapshotQcUiPlugin)
            .add_plugins(crate::gui::VfxFireTestHighlightPlugin)
            .add_plugins(LogisticsTargetsPanelPlugin)
            // World generation editor + runtime.
            .add_plugins(WorldGenToolsPlugin)
            // Production stack.
            .add_plugins(ProductionRuntimePlugin)
            .add_plugins(crate::economy::IndustrialActivationPlugin)
            .add_plugins(ManufacturingCorePlugin)
            .add_plugins(ProductionSerializationPlugin)
            .add_plugins(ProductionToolsUiPlugin)
            // Surface logistics tools.
            .add_plugins(RoadVehicleToolsUiPlugin);

        // World preview GPU offscreen camera (`gpu_preview.rs`) only when full renderer is present.
        app.insert_resource(WorldPreviewGpuRuntime {
            offscreen_renderer_ready: true,
            ..default()
        });

        app.add_plugins(crate::dev::OrchestratorHealthPlugin);
        crate::dev::stage5_live_todos::register_stage5_todo_runtime_hooks(app);
        crate::dev::visual_aidv2_live_todos::register_visual_aidv2_runtime_hooks(app);
        crate::dev::construction_live_todos::register_construction_todo_runtime_hooks(app);
        crate::dev::construction_finish_todos::register_construction_finish_todo_hooks(app);
        crate::dev::construction_phase2_todos::register_construction_phase2_todo_hooks(app);
        crate::dev::construction_round2_todos::register_construction_round2_todo_hooks(app);
        crate::dev::construction_round3_todos::register_construction_round3_todo_hooks(app);
        crate::dev::construction_operational_todos::register_construction_operational_todo_hooks(app);
        crate::dev::industrial_activation_todos::register_industrial_activation_todo_hooks(app);
        crate::dev::logistics_throughput_todos::register_logistics_throughput_todo_hooks(app);
        crate::dev::replay_editor_parity::register_replay_editor_parity_hooks(app);

        info!(
            "Engine initialized. Debug maneuvers: \
             ① `--test frame` (layout, stay open) · \
             ② `--test visual`/`capture` (proof + exit) · \
             ③ menu Demo world · \
             `unittest` fixture · `--test weather`/`fire`/`atmosphere`. \
             Keys: F1 options · F2 pressure · F3 diagnostics · F7 agent perms · F11/F12 capture."
        );
    }
}