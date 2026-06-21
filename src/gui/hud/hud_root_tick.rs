//! Single egui pass for product-shell HUD panels (suspended widgets skip body work).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPrimaryContextPass};

use crate::construction::{
    draw_pending_construction_queue_egui, ConstructionBlueprintImportUi, ConstructionQueueIntent,
    ConstructionQueuePanelView, PendingConstructionQueue,
};
use crate::gui::construction_growth_inspector::{
    draw_organic_growth_inspector_egui, sync_ecology_growth_hint, EcologyGrowthHint,
    GrowthInspectorUiState,
};
use crate::strategic::settlement::{AutoBuildPolicyBook, GrowthProposalQueue};
use crate::gui::MapFitValidationLog;
use crate::gui::ui_gates::product_egui_shell_active;
use crate::gui::{
    minimap::resolve_minimap_egui_texture, MapCameraDesired, MapPresentationDiagnostics,
    MapViewTextureCache, MinimapShellState, ResolvedMapViewFrames,
};
use crate::render::{
    draw_simulation_minimap_egui, AppStage5ReadinessReport,
    DebugRenderTraceConfig, FireAtmosphereAggregate, SimMinimapUiState, TileWorldFallbackRasterDirty,
    TileWorldFallbackState,
};

use super::dock_shell::{
    draw_hud_command_shell_egui, draw_hud_dock_minimized_strip_egui, draw_hud_overlay_tray_egui,
    HudCommandShellLayout, HudOverlayTrayState,
};
use super::hud_side_status_panel::draw_hud_side_status_panel_egui;
use crate::construction::{ActiveBuildTool, BuildGhostState, BuildPlacementPreview};
use crate::gui::InputBindings;
use crate::gui::CommandLeftStackState;
use crate::systems::sim_control::{SimControlState, SimTick};
use crate::strategic::OperationalTheaterSummary;
use super::hud_dev_overlay::{draw_hud_dev_overlay_egui, HudDevOverlayState};
use super::hud_async_task_queue::{HudAsyncTask, HudAsyncTaskQueue};
use super::layout_store::HudLayoutStore;
use super::pending_hud_layout_commit::PendingHudLayoutCommit;
use super::info_tabs::{HudInfoLiveData, HudInfoTabState};
use super::overlay_shell::{draw_overlay_shell_egui, OverlayShellState};
use super::overlay_framework::OverlayFrameworkState;
use super::player_intent_panel::PlayerIntentPanelState;
use super::shell_diagnostics::ProductShellDiagnostics;
use super::shell_update_budget::ProductShellUpdateBudget;
use super::shell_widget_timing::ShellWidgetDiagnostics;
use super::stage6_telemetry::Stage6HudTelemetry;
use super::world_interaction_diagnostics::WorldInteractionDiagnostics;
use super::viewport_rect_sanity::ViewportRectSanity;
use super::frame_budget_diagnostics::{FrameBudgetBucket, FrameBudgetDiagnostics, FrameBudgetTimer};
use super::widget_presentation::WidgetPresentationPolicy;
use super::hud_interaction_budget::HudFrameBudget;
use super::interaction_latency::InteractionLatencyMetrics;
use super::retained_widget_cache::RetainedWidgetCache;
use super::shell_framework::{
    raise_focused_product_shell_window, HudDockRegistry, HudWidgetId, ProductShellWidgetId,
};
use super::stage7_ui_shell::{draw_stage7_ui_shell_egui, Stage7UiShellState};
use super::transmission::{draw_transmission_shell_egui, TransmissionMediaProvider, TransmissionShellState};
use super::explainability_viewer::ExplainabilityViewerState;
use crate::gui::editor::world_preview::{PreviewPathAuthority, PreviewPresentationDebug};
use crate::gui::style::{HudDensityProfile, UiPalette};
use crate::gui::WorldRepresentationFrame;
use crate::systems::weather::WeatherVisualSettings;

pub struct HudRootTickPlugin;

impl Plugin for HudRootTickPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GrowthInspectorUiState>()
            .init_resource::<EcologyGrowthHint>()
            .add_systems(Update, sync_ecology_growth_hint)
            .add_systems(
            EguiPrimaryContextPass,
            hud_product_shell_egui_root
                .after(crate::gui::sync_shell_layout_drag_gate)
                .run_if(product_egui_shell_active),
        );
    }
}

#[derive(SystemParam)]
pub struct HudProductShellEguiParams<'w> {
    tray: ResMut<'w, HudOverlayTrayState>,
    layout: ResMut<'w, HudCommandShellLayout>,
    dock: ResMut<'w, HudDockRegistry>,
    layout_store: ResMut<'w, HudLayoutStore>,
    palette: Res<'w, UiPalette>,
    world: Res<'w, WorldRepresentationFrame>,
    readiness: Option<Res<'w, AppStage5ReadinessReport>>,
    preview_authority: Option<Res<'w, PreviewPathAuthority>>,
    preview_debug: Option<Res<'w, PreviewPresentationDebug>>,
    minimap: ResMut<'w, MinimapShellState>,
    legacy_minimap: ResMut<'w, SimMinimapUiState>,
    map_desired: Res<'w, MapCameraDesired>,
    map_views: ResMut<'w, crate::gui::MapViewInstances>,
    map_ready: ResMut<'w, crate::gui::MapViewReadyStates>,
    map_view_interaction: ResMut<'w, crate::gui::MapViewInteractionByView>,
    active_map_input: ResMut<'w, crate::gui::ActiveMapViewInput>,
    map_frames: Res<'w, ResolvedMapViewFrames>,
    map_presentation_diag: ResMut<'w, MapPresentationDiagnostics>,
    fallback: Res<'w, TileWorldFallbackState>,
    raster_dirty: Res<'w, TileWorldFallbackRasterDirty>,
    fire_atm: Option<Res<'w, FireAtmosphereAggregate>>,
    transmission: ResMut<'w, TransmissionShellState>,
    transmission_media: ResMut<'w, TransmissionMediaProvider>,
    overlay_shell: ResMut<'w, OverlayShellState>,
    overlay_framework: ResMut<'w, OverlayFrameworkState>,
    info_tabs: ResMut<'w, HudInfoTabState>,
    info_live: Res<'w, HudInfoLiveData>,
    stage7: ResMut<'w, Stage7UiShellState>,
    explainability: ResMut<'w, ExplainabilityViewerState>,
    construction_view: Res<'w, ConstructionQueuePanelView>,
    dev_overlay: ResMut<'w, HudDevOverlayState>,
    shell_diag: ResMut<'w, ProductShellDiagnostics>,
    viewport_rect_sanity: ResMut<'w, ViewportRectSanity>,
    stage6_telemetry: Option<Res<'w, Stage6HudTelemetry>>,
    frame_budget: ResMut<'w, FrameBudgetDiagnostics>,
    update_budget: ResMut<'w, ProductShellUpdateBudget>,
    widget_timing: ResMut<'w, ShellWidgetDiagnostics>,
    world_interaction: Option<Res<'w, WorldInteractionDiagnostics>>,
    presentation: Res<'w, WidgetPresentationPolicy>,
    async_queue: ResMut<'w, HudAsyncTaskQueue>,
    pending_layout: ResMut<'w, PendingHudLayoutCommit>,
    wave_s_capture: ResMut<'w, crate::io::save::WaveSShellCapturePending>,
    wave_s_restore: ResMut<'w, crate::io::save::WaveSShellRestorePending>,
    wave_s_hydrate: Res<'w, crate::io::save::WaveSShellHydrateWitness>,
    wave_s_imported: Res<'w, crate::io::save::WaveSImportedBlueprints>,
    texture_cache: ResMut<'w, MapViewTextureCache>,
    interaction_budget: ResMut<'w, HudFrameBudget>,
    retained: ResMut<'w, RetainedWidgetCache>,
    interaction_latency: Res<'w, InteractionLatencyMetrics>,
    map_fit_log: Res<'w, MapFitValidationLog>,
    density: ResMut<'w, HudDensityProfile>,
    time: Res<'w, Time>,
    trace: Res<'w, DebugRenderTraceConfig>,
    weather: Res<'w, WeatherVisualSettings>,
    intent: Res<'w, PlayerIntentPanelState>,
    view_manager: Res<'w, crate::gui::ViewManager>,
    sim_map_viewport: Res<'w, crate::gui::SimulationMapViewport>,
    per_view_lod: Res<'w, crate::gui::PerViewLodHints>,
    view_isolation: Res<'w, crate::gui::ViewIsolationDiagnostics>,
    update_attrib: Option<ResMut<'w, crate::render::FrameUpdateAttrib>>,
    infra_overlay: ResMut<'w, crate::render::InfrastructureOverlaySettings>,
    infra_draw: Res<'w, crate::render::InfrastructureOverlayDrawRequests>,
    power_presentation: Res<'w, crate::render::PowerMapOverlayPresentation>,
    growth_ui: ResMut<'w, GrowthInspectorUiState>,
    growth_queue: ResMut<'w, GrowthProposalQueue>,
    growth_policy: Option<Res<'w, AutoBuildPolicyBook>>,
    ecology_growth_hint: Option<Res<'w, crate::gui::construction_growth_inspector::EcologyGrowthHint>>,
    base: Res<'w, State<crate::engine::BaseState>>,
    compositor: Option<Res<'w, crate::render::MinimapCompositorState>>,
}

/// One egui context pass for docked HUD widgets; individual drawers early-out when suspended.
pub fn hud_product_shell_egui_root(
    mut contexts: EguiContexts,
    mut panels: HudProductShellEguiParams,
    bindings: Res<InputBindings>,
    sim_control: Option<Res<SimControlState>>,
    sim_tick: Option<Res<SimTick>>,
    theater: Option<Res<OperationalTheaterSummary>>,
    tool: Option<Res<ActiveBuildTool>>,
    ghost: Option<Res<BuildGhostState>>,
    placement_preview: Option<Res<BuildPlacementPreview>>,
    left_stack: Option<Res<CommandLeftStackState>>,
    mut construction_intents: MessageWriter<ConstructionQueueIntent>,
    pending: Res<PendingConstructionQueue>,
    mut construction_import_ui: ResMut<ConstructionBlueprintImportUi>,
    mut construction_preset_ron: Local<Option<String>>,
    mut minimap_legend_revision: Local<u64>,
) -> Result {
    let hud_started = FrameBudgetTimer::start();
    panels.layout_store.begin_frame();
    if panels.info_tabs.request_layout_reset {
        panels.layout_store.reset_all_frames();
        panels.info_tabs.request_layout_reset = false;
    }
    panels.pending_layout.begin_frame();
    panels.shell_diag.record_egui_pass();
    panels.widget_timing.begin_frame();
    panels.interaction_budget.begin_frame();
    let now_secs = panels.time.elapsed_secs();
    let minimap_legend = panels.async_queue.cache.minimap_legend.clone();
    let interaction_frozen = !panels.pending_layout.can_emit_layout_capture();
    let minimap_tex = {
        let before = panels
            .texture_cache
            .binding(crate::gui::MapViewInstanceId::Minimap)
            .rebinds_frame;
        let tex = resolve_minimap_egui_texture(
            &mut contexts,
            &mut panels.minimap,
            &mut panels.legacy_minimap,
            &mut panels.dock,
            &panels.fallback,
            &panels.map_frames,
            &mut panels.texture_cache,
            &mut panels.map_ready,
            interaction_frozen,
        );
        if panels
            .texture_cache
            .binding(crate::gui::MapViewInstanceId::Minimap)
            .rebinds_frame
            > before
        {
            panels.shell_diag.bump_texture_rebuild(HudWidgetId::Minimap);
        }
        tex
    };
    let ctx = contexts.ctx_mut()?;
    draw_hud_side_status_panel_egui(
        ctx,
        &mut panels.layout,
        &panels.palette,
        &bindings,
        &panels.world,
        panels.readiness.as_deref(),
        panels.preview_authority.as_deref(),
        panels.preview_debug.as_deref(),
        panels.stage6_telemetry.as_deref(),
        sim_control.as_deref(),
        sim_tick.as_deref(),
        theater.as_deref(),
        tool.as_deref(),
        ghost.as_deref(),
        placement_preview.as_deref(),
        left_stack.as_deref(),
        &panels.async_queue,
        &panels.interaction_latency,
        panels.world_interaction.as_deref(),
    );
    panels
        .update_budget
        .set_bypass_throttle(panels.pending_layout.drag_active || panels.minimap.diagnostic_ui_wrote_camera);
    if let Some(tex_id) = minimap_tex {
        let minimap_overlays = panels.map_views.minimap.overlays;
        let ecology_rows = panels.compositor.as_ref().map(|c| c.ecology_rows).unwrap_or(0);
        let veg_burn_rows = panels.compositor.as_ref().map(|c| c.veg_burn_rows).unwrap_or(0);
        let base_state = *panels.base.get();
        let minimap_presentation = &mut panels.map_views.minimap;
        draw_simulation_minimap_egui(
            ctx,
            tex_id,
            &mut panels.minimap,
            &mut panels.legacy_minimap,
            &panels.view_manager,
            &panels.map_desired,
            &panels.sim_map_viewport,
            minimap_presentation,
            &mut panels.dock,
            &mut panels.layout_store,
            &panels.palette,
            &mut panels.shell_diag,
            &mut panels.viewport_rect_sanity,
            &panels.fallback,
            &panels.raster_dirty,
            panels.fire_atm.as_deref(),
            &panels.map_frames,
            &mut panels.map_presentation_diag,
            panels.pending_layout.drag_active,
            &mut panels.pending_layout,
            &mut panels.map_view_interaction.minimap,
            &mut panels.active_map_input,
            base_state,
            &minimap_overlays,
            ecology_rows,
            veg_burn_rows,
            Some(panels.infra_overlay.as_ref()),
            Some(panels.infra_draw.as_ref()),
            Some(panels.power_presentation.as_ref()),
        );
        if panels.minimap.cached_texture_revision != *minimap_legend_revision {
            *minimap_legend_revision = panels.minimap.cached_texture_revision;
            panels.async_queue.enqueue(HudAsyncTask::MinimapLegend {
                zoom: crate::gui::camera_zoom(&panels.view_manager, crate::gui::ViewId::WorldMain)
                    .unwrap_or(panels.map_desired.scale.x),
                revision: panels.minimap.cached_texture_revision,
            });
        }
    }
    if !panels
        .presentation
        .uses_egui(panels.presentation.default_backend)
    {
        panels
            .frame_budget
            .record_bucket_ms(FrameBudgetBucket::HudShell, hud_started.elapsed_ms());
        return Ok(());
    }
    draw_hud_dock_minimized_strip_egui(
        ctx,
        &panels.presentation,
        &mut panels.dock,
        &mut panels.minimap,
        &mut panels.transmission,
        &mut panels.tray,
        &mut panels.layout,
        &mut panels.update_budget,
    );
    draw_hud_overlay_tray_egui(
        ctx,
        &mut panels.tray,
        &mut panels.dock,
        &mut panels.layout_store,
        &mut panels.update_budget,
        now_secs,
        Some(&mut panels.widget_timing),
        &mut panels.pending_layout,
    );
    let hud_frame_elapsed = hud_started.elapsed_ms_now();
    if panels
        .interaction_budget
        .should_defer(ProductShellWidgetId::CommandShell, hud_frame_elapsed)
    {
        panels
            .interaction_budget
            .note_deferred(ProductShellWidgetId::CommandShell);
    } else {
        draw_hud_command_shell_egui(
            ctx,
            &mut panels.layout,
            &mut panels.dock,
            &mut panels.layout_store,
            &panels.palette,
            &panels.world,
            panels.readiness.as_deref(),
            panels.preview_authority.as_deref(),
            panels.preview_debug.as_deref(),
            panels.stage6_telemetry.as_deref(),
            &mut panels.update_budget,
            now_secs,
            Some(&mut panels.widget_timing),
            panels.world_interaction.as_deref(),
            &panels.async_queue,
            &panels.interaction_latency,
            &mut panels.pending_layout,
            &mut panels.wave_s_capture,
            &mut panels.wave_s_restore,
            panels.wave_s_hydrate.as_ref(),
            panels.wave_s_imported.as_ref(),
        );
    }
    draw_transmission_shell_egui(
        ctx,
        &panels.palette,
        &panels.presentation,
        &mut panels.transmission,
        &mut panels.transmission_media,
        &mut panels.dock,
        &mut panels.layout_store,
        &mut panels.update_budget,
        now_secs,
        Some(&mut panels.widget_timing),
        &mut panels.retained,
        &mut panels.async_queue,
        &mut panels.pending_layout,
    );
    let overlay_started = FrameBudgetTimer::start();
    draw_overlay_shell_egui(
        ctx,
        &panels.palette,
        &mut panels.overlay_shell,
        &mut panels.overlay_framework,
        &mut panels.info_tabs,
        &mut panels.dock,
        &mut panels.layout_store,
        &mut panels.update_budget,
        now_secs,
        Some(&mut panels.widget_timing),
        minimap_legend.as_deref(),
        &mut panels.retained,
        *minimap_legend_revision,
        &mut panels.pending_layout,
        Some(panels.world.as_ref()),
        panels.readiness.as_deref(),
        Some(panels.info_live.as_ref()),
        panels.infra_overlay.as_mut(),
    );
    panels
        .frame_budget
        .record_bucket_ms(FrameBudgetBucket::OverlayComposition, overlay_started.elapsed_ms());
    let frame_elapsed = hud_started.elapsed_ms_now();
    if panels
        .interaction_budget
        .should_defer(ProductShellWidgetId::IntelTimeline, frame_elapsed)
    {
        panels
            .interaction_budget
            .note_deferred(ProductShellWidgetId::IntelTimeline);
    } else {
        draw_stage7_ui_shell_egui(
            ctx,
            &mut panels.stage7,
            &mut panels.explainability,
            &mut panels.async_queue,
        );
    }
    draw_pending_construction_queue_egui(
        ctx,
        &panels.construction_view,
        &mut panels.dock,
        &mut panels.layout_store,
        &mut construction_intents,
        &pending,
        &mut construction_preset_ron,
        &mut panels.update_budget,
        now_secs,
        Some(&mut panels.widget_timing),
        panels.world_interaction.as_deref(),
        &mut panels.pending_layout,
        Some(panels.wave_s_imported.as_ref()),
        &mut construction_import_ui,
    );
    draw_organic_growth_inspector_egui(
        ctx,
        panels.growth_ui,
        panels.growth_queue,
        panels.growth_policy,
        panels.ecology_growth_hint,
    );
    draw_hud_dev_overlay_egui(
        ctx,
        &panels.palette,
        &panels.dev_overlay,
        &panels.trace,
        &panels.weather,
        &panels.shell_diag,
        &panels.minimap,
        &panels.dock,
        &panels.layout_store,
        &panels.intent,
        panels.stage6_telemetry.as_deref(),
        &panels.viewport_rect_sanity,
        &panels.frame_budget,
        &panels.update_budget,
        &panels.widget_timing,
        panels.world_interaction.as_deref(),
        &panels.async_queue,
        &panels.texture_cache,
        &panels.retained,
        &panels.interaction_budget,
        &panels.interaction_latency,
        &panels.map_presentation_diag,
        &panels.map_fit_log,
        &panels.active_map_input,
        &panels.per_view_lod,
        &panels.view_manager,
        &panels.view_isolation,
        &mut panels.density,
    );
    let hud_ms = hud_started.elapsed_ms();
    if let Some(attrib) = panels.update_attrib.as_mut() {
        attrib.hud_egui_ms = hud_ms;
        crate::render::intra_update_stall_log("egui_hud_shell", hud_ms);
    }
    panels
        .frame_budget
        .record_bucket_ms(FrameBudgetBucket::HudShell, hud_ms);
    panels.interaction_budget.finalize_frame(hud_ms);
    raise_focused_product_shell_window(ctx, &panels.dock);
    panels.async_queue.enqueue(HudAsyncTask::SpikeAnalysis {
        avg_frame_ms: panels.frame_budget.avg_frame_ms,
        max_frame_ms: panels.frame_budget.max_frame_ms,
    });
    panels.async_queue.enqueue(HudAsyncTask::ShellMetricsReduce {
        widget_count: ProductShellWidgetId::ALL.len() as u32,
        spike_markers: panels.widget_timing.frame_spike_markers,
    });
    Ok(())
}
