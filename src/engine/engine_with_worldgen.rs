use crate::entities::production::core::ManufacturingCorePlugin;
use crate::entities::vehicles::tools_ui::RoadVehicleToolsUiPlugin;
use crate::gui::{
    editor::map_editor::MapEditorPlugin,
    editor::world_preview::{
        init_preview_render_contract_resources, WorldPreviewGpuRuntime,
    },
    AppShellPlugin, BaseMenuPlugin, BuildPlanningPlugin,
    GameplayCapturePlugin, InGameHudPlugin, InGamePauseMenuPlugin, KeybindingsOptionsPlugin,
    LogisticsTargetsPanelPlugin,
    hud::HudDockShellPlugin,
    MapCameraPlugin, CameraFocusDebugPlugin, SplashPlugin,
    ViewAuthorityPlugin,
    ViewRepresentationPlugin,
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
    Core2dOverlayOrderPlugin, FramePerfPlugin, GpuWeatherFireFieldPlugin, LocalLightPlugin,
    SharedOverlayFieldBuffersPlugin,
    StallWatchPlugin, TerrainGpuBakeSpikePlugin, TerrainInstancedDrawPlugin,
    TerrainMaterialAtlasPlugin, TerrainRenderAuthorityPlugin, TileWorldFallbackPlugin,
    ViewportPipelinePlugin,
    ViewRuntimePlugin, GpuWaterParticlesPlugin, WaterSurfaceVisualPlugin,
    register_water_surface_draw, register_world_water_particle_draw,
};
use crate::systems::{
    configure_chunk_environment_sets,
    AtmospherePlugin, ChunkEnvironmentPersistPlugin, ChunkSimLodPlugin, CollisionPlugin,
    EcologyPlugin, FirePlugin, SimAttentionPlugin, WeatherPlugin,
};
use crate::systems::terrain::MaterialUnificationPlugin;
use crate::terrain::generation::WorldGenToolsPlugin;
use super::ux_orchestration::UxOrchestrationPlugin;
use super::worldgen_chrome_debug::WorldGenChromeDebugPlugin;
use crate::render::{DebugViewportOverlayPlugin, VisualDiagnosticsPlugin};
use super::test_harness::{TestHarnessMenuPlugin, TestHarnessPlugin, TestHarnessStatePlugin};
use super::DebugManeuverPlugin;
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use bevy::winit::WinitSettings;
use bevy::window::{Window, WindowPlugin};

/// Root camera for tactical map — renders to [`crate::gui::SimulationMapTexture`].
fn spawn_primary_ui_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    palette: Res<crate::gui::UiPalette>,
    launch: Option<Res<crate::engine::EngineLaunchArgs>>,
) {
    let handle = crate::gui::insert_simulation_map_texture(
        images.as_mut(),
        &mut commands,
        1280,
        720,
    );
    let mode = crate::gui::rtt_diag_camera_mode(launch.as_deref());
    crate::gui::spawn_main_world_rtt_camera(&mut commands, handle, palette.as_ref(), mode);
    crate::gui::spawn_simulation_hud_ui_camera(&mut commands, palette.as_ref());
}

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        // DES-03: registration order is platform → shell → sim → world → render → transport → strategic.
        app.init_resource::<crate::render::DebugRenderTraceConfig>();
        // Resolve `assets/` from the crate root so running `target/debug/proc_A_dine01.exe` from any CWD still finds files.
        let assets_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets");
        let asset_file_path = assets_root.to_string_lossy().into_owned();
        // RND-19 / V4-RND19: sole primary-window present + buffer-count authority.
        let present = crate::render::view_runtime::resolve_present_surface_config_from_env();

        app.add_plugins(
            DefaultPlugins
                .set(AssetPlugin {
                    file_path: asset_file_path,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        present_mode: present.present_mode,
                        desired_maximum_frame_latency: Some(
                            present.desired_maximum_frame_latency(),
                        ),
                        ..default()
                    }),
                    ..default()
                }),
        )
            .add_plugins(crate::dev::MigAAdoptionPlugin)
            // PERF-PLAY: keep simulation in continuous mode; reactive low-power mode can sleep
            // near 1s between frames and makes camera/minimap feel "broken" under active play.
            .insert_resource(WinitSettings::game())
            .add_systems(Startup, spawn_primary_ui_camera)
            .add_plugins(LocalLightPlugin)
            .add_plugins(DebugManeuverPlugin)
            .add_plugins(UiThemePlugin)
            .add_plugins(SplashPlugin)
            .add_plugins(BaseMenuPlugin)
            .add_plugins(AppShellPlugin)
            .add_plugins(UxOrchestrationPlugin)
            .add_plugins((
                WorldGenChromeDebugPlugin,
                DebugViewportOverlayPlugin,
                VisualDiagnosticsPlugin,
                crate::render::GpuSurfaceTeardownPlugin,
            ))
            .add_plugins(MapEditorPlugin)
            // Sim loop control (pause / step / speed / monotonic tick).
            .add_plugins(SimControlPlugin)
            .add_plugins(SimAttentionPlugin)
            // Scenario script host (Wave 1): drains one step per frame before sim tick.
            .add_plugins(crate::sim::SimEffectsPlugin)
            .add_plugins(ScenarioScriptingPlugin);
        configure_chunk_environment_sets(app);
        #[cfg(any(debug_assertions, feature = "sim_dump"))]
        app.add_plugins(crate::dev::SimRunDumpPlugin);
        app.add_plugins(crate::substrate::SubstratePlugin)
            .add_plugins(ChunkEnvironmentPersistPlugin)
            .add_plugins(crate::io::save::WorldSaveSpinePlugin)
            .add_plugins(crate::render::Stage6VirtualizationPlugin)
            .add_plugins(ChunkSimLodPlugin)
            .add_plugins(crate::render::ExtractedCameraMetricsPlugin)
            .add_plugins(FirePlugin)
            .add_plugins(EcologyPlugin)
            .add_plugins(AtmospherePlugin)
            .add_plugins(WeatherPlugin)
            .add_plugins(CollisionPlugin)
            .add_plugins(FramePerfPlugin)
            .add_plugins(crate::render::probes::rtt_sprite_trace::RttSpriteTracePlugin)
            .add_plugins(crate::dev::diagnostics::DevDiagnosticsPlugin)
            .add_plugins(Core2dOverlayOrderPlugin)
            .add_plugins(GpuWeatherFireFieldPlugin)
            .add_plugins(crate::infrastructure::InfrastructureProfilesPlugin)
            .add_plugins(crate::infrastructure::InfrastructureTransportPlugin)
            .add_plugins(crate::infrastructure::utility::UtilityGraphPlugin)
            .add_plugins(crate::render::InfrastructureOverlayPlugin)
            .add_plugins(crate::render::WorldMapNetworkMeshPlugin)
            .add_plugins(crate::render::WorldMapVegetationMeshPlugin)
            .add_plugins(crate::render::PixelWorldViewPlugin)
            .add_plugins(crate::gui::FrontControlsPlugin)
            .add_plugins(crate::render::WorldMapFrontMeshPlugin)
            .add_plugins(TransportSimulationPlugin)
            .add_plugins(crate::engine::PlayScenarioPlugin)
            // Nav: damage/speed adjustments after transport cost cache; motion stage after damage (S2).
            .add_plugins(NavigationSchedulePlugin)
            .add_plugins(DamageSystem)
            .add_plugins(MaterialUnificationPlugin)
            .add_plugins(crate::gui::editor::editor_world_commit_bridge::EditorWorldCommitBridgePlugin)
            .add_plugins(crate::strategic::StrategicFieldsAndAiPlugin)
            .add_plugins(crate::strategic::GpuBridgePlugin);
        // Hanabi L3 double gate: cargo feature +  env.
        #[cfg(feature = "hanabi_l3")]
        if std::env::var_os("RUST_ENGINE_HANABI_L3")
            .is_some_and(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
        {
            app.add_plugins(crate::render::fx_spine::hanabi_embellishment::HanabiEmbellishmentPlugin);
        }
        // Fire visual extract: one sim pass → buffer; then pooled local lights collect messages.
        app.configure_sets(
            Update,
            crate::render::extraction::FireVisualFrameSet::BuildProfiles
                .after(crate::systems::chunk_environment_set::ChunkEnvironmentSet::Fire)
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
        app.add_plugins(crate::gui::PanelPresencePlugin)
            .add_plugins(KeybindingsOptionsPlugin)
            .add_plugins(GameplayCapturePlugin)
            .add_plugins(CameraFocusDebugPlugin)
            .add_plugins(crate::gui::GpuTileDebugPlugin)
            .add_plugins(crate::gui::TileReadabilityPlugin)
            .add_plugins(crate::gui::StrategicIconInstancesPlugin)
            .add_plugins(ViewRepresentationPlugin)
            .add_plugins(StallWatchPlugin)
            .add_plugins(ComputeDispatchPlugin);
        app.add_plugins((TestHarnessStatePlugin, TestHarnessMenuPlugin));
        let test_mode = app
            .world()
            .get_resource::<crate::engine::EngineLaunchArgs>()
            .is_some_and(|launch| launch.test_mode());
        if test_mode {
            // Seeds, proof boards, FULL_APP readiness — never mixed into the product plugin body.
            app.add_plugins(TestHarnessPlugin);
            app.add_plugins(crate::gui::VfxFireTestFocusPlugin);
        }
        init_preview_render_contract_resources(app);
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
            .add_plugins(TerrainRenderAuthorityPlugin)
            .add_plugins(TerrainMaterialAtlasPlugin)
            .add_plugins(TerrainInstancedDrawPlugin)
            .add_plugins(TerrainGpuBakeSpikePlugin)
            .add_plugins(crate::render::TacticalVectorOverlayPlugin)
            .add_plugins(WaterSurfaceVisualPlugin)
            .add_plugins(GpuWaterParticlesPlugin);
        register_water_surface_draw(app);
        register_world_water_particle_draw(app);
        app.add_plugins(ViewportPipelinePlugin)
            .add_plugins(ViewRuntimePlugin)
            .add_plugins(ViewAuthorityPlugin)
            .add_plugins(crate::audio::AudioMixPlugin)
            .add_plugins(crate::presentation::PresentationAsmPlugin)
            .add_plugins(SharedOverlayFieldBuffersPlugin)
            .add_plugins(crate::gui::AiExplainabilityPlugin)
            .add_plugins(InGameHudPlugin)
            .add_plugins(crate::gui::hud::SimulationSessionPlugin)
            .add_plugins(crate::gui::WorldPresentationModePlugin)
            .add_plugins(crate::gui::ArtPresentationTargetPlugin)
            .add_plugins(crate::render::VfxCaptureHookPlugin)
            .add_plugins(MapCameraPlugin)
            .add_plugins(InGamePauseMenuPlugin)
            .add_plugins(HudDockShellPlugin)
            .add_plugins(BuildPlanningPlugin)
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

        // PERF-VFX-002: serialize the hot Update spine so stall probes and extract don't interleave.
        app.configure_sets(
            Update,
            (
                crate::gui::MapCameraSystemSet::ApplyInput,
                crate::gui::MapCameraSystemSet::DeriveDesired,
                crate::gui::MapCameraSystemSet::Smooth,
                crate::gui::ViewAuthoritySystemSet::SyncViewManager,
                crate::render::extraction::FireVisualFrameSet::BuildProfiles,
                crate::gui::WorldRepresentationSystemSet::ComputeFrame,
            )
                .chain(),
        );

        // SCH-W1-E1-001: ambiguity warnings on request (`SCH_AMBIGUITY_WARN=1`). Not on by default in
        // debug: the report is ~14k lines per launch and buries the run log.
        let sch_ambiguity_warn = std::env::var("SCH_AMBIGUITY_WARN")
                .ok()
                .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"));
        if sch_ambiguity_warn {
            use bevy::ecs::schedule::{LogLevel, ScheduleBuildSettings};
            app.configure_schedules(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..default()
            });
        }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn des03_plugin_build_cites_registration_order() {
        let src = include_str!("engine_with_worldgen.rs");
        assert!(
            src.contains("DES-03: registration order"),
            "EnginePlugin build must cite DES-03 phase order"
        );
    }
}