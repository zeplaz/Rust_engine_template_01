//! Simulation session chrome: hide editor/tools panels when entering gameplay.

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::engine::AppState;
use crate::engine::WorldGenChromeLatch;
use crate::gui::diagnostics_ui::DiagnosticsUiState;
use crate::gui::editor::scenario_script_panel::ScenarioScriptPanelState;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::gui::editor::world_preview::{
    dismiss_world_gen_preview_chrome, WorldPreviewLifecycle, WorldPreviewUiState,
};
use crate::gui::hud::shell_framework::{
    simulation_floating_shells_gated, suppress_simulation_floating_shell_slots,
};
use crate::gui::hud::simulation_shell_phase2::UiShellMigrationWitness;
use crate::gui::hud::{
    seed_ux_e03_transmission_media_registry, ContextTrayState, HudCommandShellLayout,
    HudDockRegistry, HudOverlayTrayState, HudPanelState, ProductShellUpdateBudget,
    ProductShellWidgetId, TransmissionMediaProviderRegistry, TransmissionShellState,
};
use crate::gui::{MinimapPresentationSource, MinimapShellState};
use crate::engine::ActiveTestScene;
use crate::gui::map_view::MapViewInstanceId;
use crate::gui::MapViewInstances;
use crate::gui::simulation_minimap_overlay_defaults;
use crate::gui::{
    map_camera_viewport_pixels, map_zoom_limits_for_world, MapCameraDesired, SimulationMapViewport,
};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::gui::MainWorldCamera;
use crate::render::minimap_gpu_compositor_env_enabled;
use crate::render::{
    seed_minimap_m2_overlay_witness, seed_minimap_m3_fow_ew_witness, EcologyVisualSnapshot,
    FireSimulationSnapshot, MinimapOperationalSnapshot, SharedOverlayFieldBuffers,
};
use crate::render::sim_visual_extract::ClimateVisualAggregate;
use crate::strategic::CorridorConstructionBook;
use bevy::window::PrimaryWindow;

/// Collapse layout + overlay tray state that drives floating egui shells (Sprint 3.2).
pub fn collapse_simulation_floating_shell_layout(
    layout: &mut HudCommandShellLayout,
    tray: &mut HudOverlayTrayState,
    transmission: &mut TransmissionShellState,
) {
    layout.overlay_tray_state = HudPanelState::Collapsed;
    layout.status_side_panel_state = HudPanelState::Collapsed;
    layout.command_tray_state = HudPanelState::Collapsed;
    layout.intel_timeline_state = HudPanelState::Collapsed;
    layout.command_table_state = HudPanelState::Collapsed;
    tray.tray_panel_state = HudPanelState::Collapsed;
    transmission.panel_state = HudPanelState::Collapsed;
}

/// PLAY-01: collapse editor chrome; keep sim HUD shells available but not expanded by default.
pub fn apply_simulation_hud_defaults(
    mut world_gen: ResMut<WorldGenUiState>,
    mut preview_ui: ResMut<WorldPreviewUiState>,
    mut preview_lifecycle: ResMut<WorldPreviewLifecycle>,
    mut latch: ResMut<WorldGenChromeLatch>,
    mut script_panel: ResMut<ScenarioScriptPanelState>,
    mut dock: ResMut<HudDockRegistry>,
    mut layout: ResMut<HudCommandShellLayout>,
    mut tray: ResMut<HudOverlayTrayState>,
    mut transmission: ResMut<TransmissionShellState>,
    mut context_tray: ResMut<ContextTrayState>,
    mut shell_budget: ResMut<ProductShellUpdateBudget>,
    mut diagnostics: ResMut<DiagnosticsUiState>,
    mut witness: ResMut<UiShellMigrationWitness>,
    mut shell_diag: ResMut<crate::gui::hud::ProductShellDiagnostics>,
) {
    shell_diag.reset_egui_pass_count_for_simulation_session();
    shell_budget.set_bypass_throttle(false);
    diagnostics.sections_default_open = false;
    dismiss_world_gen_preview_chrome(
        &mut world_gen,
        &mut preview_ui,
        &mut preview_lifecycle,
        &mut latch,
        "enter_simulation",
    );
    script_panel.window_open = false;
    script_panel.tools_entry_visible = false;

    collapse_simulation_floating_shell_layout(&mut layout, &mut tray, &mut transmission);
    context_tray.panel_state = HudPanelState::Collapsed;

    suppress_simulation_floating_shell_slots(&mut dock);
    // Minimap dock slot stays visible for Bevy chrome; egui texture dock is editor-only (Phase 2B).
    dock.slot_mut(ProductShellWidgetId::Minimap).visible = true;
    dock.slot_mut(ProductShellWidgetId::Minimap).minimized = false;
    sync_simulation_egui_shell_gate_witness(&dock, &layout, &mut witness);
}

/// UX-E03-CODER-A — seed transmission media registry on Simulation enter (read-only narrative lane).
pub fn seed_ux_e03_transmission_on_simulation_enter(
    mut registry: ResMut<TransmissionMediaProviderRegistry>,
    mut witness: ResMut<UiShellMigrationWitness>,
) {
    seed_ux_e03_transmission_media_registry(&mut registry);
    witness.ux_e03_media_registry_wired =
        crate::gui::hud::transmission_media::ux_e03_coder_a_green(&registry);
}

/// Minimap + sim map presentation defaults on enter (keeps [`apply_simulation_hud_defaults`] under Bevy param cap).
pub fn apply_simulation_map_presentation_defaults(
    mut minimap: ResMut<MinimapShellState>,
    mut map_views: ResMut<MapViewInstances>,
    mut tray: ResMut<HudOverlayTrayState>,
    mut presentation: ResMut<crate::gui::MapViewPresentationStates>,
    mut shared_overlay: ResMut<SharedOverlayFieldBuffers>,
    test_scene: Option<Res<ActiveTestScene>>,
    _scenario: Option<Res<crate::engine::ActivePlayScenario>>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
) {
    minimap.visible = true;
    minimap.minimized = false;
    // P1-B: main sim HUD → GPU RT. CPU layered raster remains for explicit `SharedCpuRaster` effects.
    minimap.presentation_source = if minimap_gpu_compositor_env_enabled() {
        MinimapPresentationSource::SharedRenderTargetImage
    } else {
        MinimapPresentationSource::SharedCpuRaster
    };
    let mask = simulation_minimap_overlay_defaults();
    map_views.minimap.overlays = mask;
    map_views.minimap.bump_revision();
    tray.set_minimap_overlay_mask(mask);
    let sim_pres = presentation.get_mut(MapViewInstanceId::SimulationMap);
    sim_pres.overlays.fire_heat = false;
    sim_pres.bump_revision();
    if test_scene.is_none() {
        shared_overlay.chunk_fire_heat.clear();
        shared_overlay.bump();
    }
    if let Ok(window) = primary_window.single() {
        minimap.bootstrap_simulation_layout_rect(window.width(), window.height());
    }
}

/// UI-P3-M3-001 — ecology + construction snapshots before first GPU minimap composite.
fn seed_minimap_m2_ecology_construction_on_simulation_enter(
    fire: Option<Res<FireSimulationSnapshot>>,
    book: Option<ResMut<CorridorConstructionBook>>,
    climate: Option<ResMut<ClimateVisualAggregate>>,
    ecology: Option<ResMut<EcologyVisualSnapshot>>,
) {
    let (Some(fire), Some(mut book), Some(mut climate), Some(mut ecology)) =
        (fire, book, climate, ecology)
    else {
        return;
    };
    if ecology.chunk_rows.len() >= 100 && book.rows.len() >= 18 {
        return;
    }
    seed_minimap_m2_overlay_witness(&fire, &mut book, &mut climate, &mut ecology);
}

/// **UI-P3-M4-001** — design M3 fog/EW snapshot (not **UI-P3-M3-001** M2 construction/ecology).
fn seed_minimap_m3_fow_ew_on_simulation_enter(
    operational: Option<ResMut<MinimapOperationalSnapshot>>,
) {
    let Some(mut operational) = operational else {
        return;
    };
    seed_minimap_m3_fow_ew_witness(&mut operational);
}

/// Frame the whole world on normal sim enter (not CLI test scenes — those set tactical zoom).
pub fn refit_simulation_map_camera_on_enter(
    test_scene: Option<Res<ActiveTestScene>>,
    params: Res<WorldGenParams>,
    sim_viewport: Res<SimulationMapViewport>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut cam: Query<&mut Transform, With<MainWorldCamera>>,
    mut authority: ResMut<crate::render::view_runtime::ViewProjectionAuthority>,
    mut trace: ResMut<crate::render::view_runtime::ViewRuntimeTrace>,
) {
    if test_scene.is_some() {
        return;
    }
    if params.width == 0 || params.height == 0 {
        return;
    }
    let world_w = params.width as f32;
    let world_h = params.height as f32;
    let window_px = windows
        .single()
        .map(|w| Vec2::new(w.width().max(1.0), w.height().max(1.0)))
        .unwrap_or(Vec2::ONE);
    let viewport = map_camera_viewport_pixels(window_px, Some(sim_viewport.as_ref()));
    let (zoom_lo, zoom_hi) = map_zoom_limits_for_world(world_w, world_h, viewport);
    let margin = 0.9;
    let fit = margin * (viewport.x / world_w).min(viewport.y / world_h);
    let zoom = fit.clamp(zoom_lo, zoom_hi);
    let center = Vec3::new(world_w * 0.5, world_h * 0.5, 0.0);
    let pose = MapCameraDesired {
        translation: center,
        scale: Vec3::splat(zoom),
        rotation: Quat::IDENTITY,
    };
    crate::gui::view_authority::commit_map_camera_pose_to_view_authority(
        authority.as_mut(),
        trace.as_mut(),
        &pose,
    );
    for mut t in cam.iter_mut() {
        t.translation = center;
        t.translation.z = 999.0;
        t.scale = Vec3::splat(zoom);
        t.rotation = Quat::IDENTITY;
    }
}

/// PLAY-01 Phase 2B: keep egui product shells closed in sim (Bevy rail is authoritative).
pub fn enforce_simulation_product_egui_gates(
    base: Res<State<BaseState>>,
    mut dock: ResMut<HudDockRegistry>,
    mut layout: ResMut<HudCommandShellLayout>,
    mut tray: ResMut<HudOverlayTrayState>,
    mut transmission: ResMut<TransmissionShellState>,
    mut witness: ResMut<UiShellMigrationWitness>,
) {
    if !matches!(*base.get(), BaseState::Simulation) {
        return;
    }
    suppress_simulation_floating_shell_slots(&mut dock);
    collapse_simulation_floating_shell_layout(&mut layout, &mut tray, &mut transmission);
    sync_simulation_egui_shell_gate_witness(&dock, &layout, &mut witness);
}

/// **UI-P2B-001 / UI-P2B-CODER-B** — sync gate flags for live proof (`phase2b_closed`).
pub(crate) fn sync_simulation_egui_shell_gate_witness(
    dock: &HudDockRegistry,
    layout: &HudCommandShellLayout,
    witness: &mut UiShellMigrationWitness,
) {
    let build = dock.slot(ProductShellWidgetId::BuildToolbox);
    witness.build_toolbox_egui_gated =
        !build.visible && build.minimized && !build.detached;
    witness.side_status_rail_egui_gated =
        layout.status_side_panel_state == HudPanelState::Collapsed;
    witness.floating_egui_shells_gated = simulation_floating_shells_gated(dock);
}

/// Re-close generator + preview if something reopens them while in gameplay (stops panel flicker).
pub fn enforce_world_gen_chrome_closed_in_simulation(
    base: Res<State<BaseState>>,
    app: Res<State<AppState>>,
    mut world_gen: ResMut<WorldGenUiState>,
    mut preview_ui: ResMut<WorldPreviewUiState>,
    mut preview_lifecycle: ResMut<WorldPreviewLifecycle>,
    mut latch: ResMut<WorldGenChromeLatch>,
) {
    let in_play = matches!(*base.get(), BaseState::Simulation)
        || matches!(*app.get(), AppState::InGame | AppState::Paused);
    if !in_play {
        return;
    }
    if !world_gen.visible && !preview_ui.window_open {
        return;
    }
    dismiss_world_gen_preview_chrome(
        &mut world_gen,
        &mut preview_ui,
        &mut preview_lifecycle,
        &mut latch,
        "simulation_enforce_closed",
    );
}

pub struct SimulationSessionPlugin;

impl Plugin for SimulationSessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(BaseState::Simulation), apply_simulation_hud_defaults)
            .add_systems(
                OnEnter(BaseState::Simulation),
                seed_ux_e03_transmission_on_simulation_enter.after(apply_simulation_hud_defaults),
            )
            .add_systems(
                OnEnter(BaseState::Simulation),
                apply_simulation_map_presentation_defaults.after(seed_ux_e03_transmission_on_simulation_enter),
            )
            .add_systems(
                OnEnter(BaseState::Simulation),
                seed_minimap_m2_ecology_construction_on_simulation_enter
                    .after(apply_simulation_map_presentation_defaults),
            )
            .add_systems(
                OnEnter(BaseState::Simulation),
                seed_minimap_m3_fow_ew_on_simulation_enter
                    .after(seed_minimap_m2_ecology_construction_on_simulation_enter),
            )
            .add_systems(
                OnEnter(BaseState::Simulation),
                refit_simulation_map_camera_on_enter
                    .after(seed_minimap_m3_fow_ew_on_simulation_enter),
            )
            .add_systems(
                Update,
                (
                    enforce_world_gen_chrome_closed_in_simulation,
                    enforce_simulation_product_egui_gates,
                )
                    .chain()
                    .after(crate::gui::editor::world_gen_ui::toggle_world_gen_ui_system),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **VX-P0-01** — operator Simulation enters with fire CPU tint off (tray + sim map + overlay buffer).
    #[test]
    fn vx_p0_01_operator_simulation_fire_heat_off_by_default() {
        assert!(!simulation_minimap_overlay_defaults().fire_heat);
        let mask = simulation_minimap_overlay_defaults();
        assert!(mask.logistics_heat);
        assert!(!mask.fire_heat);
    }

    #[test]
    fn ui_p2b_coder_b_resets_egui_pass_count_on_sim_enter() {
        let mut diag = crate::gui::hud::ProductShellDiagnostics::default();
        diag.record_egui_pass();
        diag.record_egui_pass();
        assert_eq!(diag.egui_pass_count, 2);
        diag.reset_egui_pass_count_for_simulation_session();
        assert_eq!(diag.egui_pass_count, 0);
        assert_eq!(diag.egui_pass_count_sim_session, 0);
    }

    #[test]
    fn simulation_egui_gate_witness_sync() {
        let mut dock = HudDockRegistry::default();
        dock.slot_mut(ProductShellWidgetId::BuildToolbox).visible = true;
        dock.slot_mut(ProductShellWidgetId::OverlaysPanel).visible = true;
        let mut layout = HudCommandShellLayout::default();
        layout.status_side_panel_state = HudPanelState::Expanded;
        let mut witness = UiShellMigrationWitness::default();
        sync_simulation_egui_shell_gate_witness(&dock, &layout, &mut witness);
        assert!(!witness.build_toolbox_egui_gated);
        assert!(!witness.side_status_rail_egui_gated);
        assert!(!witness.floating_egui_shells_gated);

        suppress_simulation_floating_shell_slots(&mut dock);
        layout.status_side_panel_state = HudPanelState::Collapsed;
        sync_simulation_egui_shell_gate_witness(&dock, &layout, &mut witness);
        assert!(witness.build_toolbox_egui_gated);
        assert!(witness.side_status_rail_egui_gated);
        assert!(witness.floating_egui_shells_gated);
    }

    #[test]
    fn collapse_floating_shell_layout_resets_tray_panel_state() {
        let mut layout = HudCommandShellLayout::default();
        layout.command_tray_state = HudPanelState::Expanded;
        let mut tray = HudOverlayTrayState::default();
        tray.tray_panel_state = HudPanelState::Expanded;
        let mut transmission = TransmissionShellState::default();
        transmission.panel_state = HudPanelState::Expanded;
        collapse_simulation_floating_shell_layout(&mut layout, &mut tray, &mut transmission);
        assert_eq!(tray.tray_panel_state, HudPanelState::Collapsed);
        assert_eq!(layout.command_tray_state, HudPanelState::Collapsed);
        assert_eq!(transmission.panel_state, HudPanelState::Collapsed);
    }
}
