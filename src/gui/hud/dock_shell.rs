//! Detachable HUD docking + overlay tray scaffolding (**BQ-123**).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::engine::states::BaseState;
use crate::gui::input_bindings::InputBindings;
use crate::gui::ui_gates::{in_simulation_or_editor, product_egui_shell_active};
use crate::gui::map_view::MapViewInstances;
use crate::gui::{simulation_minimap_overlay_defaults, MinimapOverlayMask, MinimapShellState, TransmissionShellState};
use crate::strategic::StrategicOverlayDisplayPolicy;

use super::shell_diagnostics::ProductShellDiagnostics;
use super::layout_store::{HudLayoutCollectionR8, HudLayoutStore};
use super::shell_framework::{
    capture_shell_layout, draw_minimized_shell_chip, shell_default_window_pos,
    shell_widget_runs_egui_with_budget, show_product_shell_window, HudDockRegistry,
    HudWidgetDockState, HudWidgetId, ShellWindowHost,
};
use super::shell_persistence::ProductShellPersistenceBundleR8;
use super::stage5_spine_consumer::draw_stage5_spine_consumer_panel;
use super::stage6_consumer::draw_stage6_residency_consumer_panel;
use super::panel_state::HudPanelState;
use super::stage6_telemetry::Stage6HudTelemetry;
use super::hud_async_task_queue::HudAsyncTaskQueue;
use super::interaction_latency::InteractionLatencyMetrics;
use super::pending_hud_layout_commit::PendingHudLayoutCommit;
use super::shell_update_budget::ProductShellUpdateBudget;
use super::shell_widget_timing::ShellWidgetDiagnostics;
use super::world_interaction_diagnostics::WorldInteractionDiagnostics;
use crate::gui::editor::world_preview::{PreviewPathAuthority, PreviewPresentationDebug};
use crate::gui::style::{widget_scroll_vertical_fill, UiPalette};
use crate::gui::WorldRepresentationFrame;
use crate::render::AppStage5ReadinessReport;

/// Overlay tray toggles — strategic policy + minimap compositor [`MinimapOverlayMask`] (UI-P3-M2-TRAY-OPT).
#[derive(Resource, Clone, Debug)]
pub struct HudOverlayTrayState {
    pub recon_visible: bool,
    pub logistics_stress_visible: bool,
    pub congestion_visible: bool,
    pub fow_visible: bool,
    pub ew_visible: bool,
    /// GPU minimap compositor heat channels (M2-06 — synced to [`MapViewInstances::minimap`]).
    pub fire_heat: bool,
    pub logistics_heat: bool,
    pub construction_heat: bool,
    pub ecology_heat: bool,
    pub tray_panel_state: HudPanelState,
}

impl HudOverlayTrayState {
    #[must_use]
    pub fn minimap_overlay_mask(&self) -> MinimapOverlayMask {
        let base = simulation_minimap_overlay_defaults();
        MinimapOverlayMask {
            fire_heat: self.fire_heat,
            logistics_heat: self.logistics_heat,
            construction_heat: self.construction_heat,
            ecology_heat: self.ecology_heat,
            fow: self.fow_visible,
            ew: self.ew_visible,
            units: base.units,
            replay_scrub: base.replay_scrub,
        }
    }

    pub fn set_minimap_overlay_mask(&mut self, mask: MinimapOverlayMask) {
        self.fire_heat = mask.fire_heat;
        self.logistics_heat = mask.logistics_heat;
        self.construction_heat = mask.construction_heat;
        self.ecology_heat = mask.ecology_heat;
        self.fow_visible = mask.fow;
        self.ew_visible = mask.ew;
    }
}

impl Default for HudOverlayTrayState {
    fn default() -> Self {
        let mask = simulation_minimap_overlay_defaults();
        Self {
            recon_visible: false,
            logistics_stress_visible: false,
            congestion_visible: false,
            fow_visible: mask.fow,
            ew_visible: mask.ew,
            fire_heat: mask.fire_heat,
            logistics_heat: mask.logistics_heat,
            construction_heat: mask.construction_heat,
            ecology_heat: mask.ecology_heat,
            tray_panel_state: HudPanelState::Collapsed,
        }
    }
}

/// Command shell layout flags (overlay tray → command tray → intel timeline → command table).
#[derive(Resource, Clone, Debug)]
pub struct HudCommandShellLayout {
    pub overlay_tray_state: HudPanelState,
    /// Docked left **status rail** (egui `SidePanel`) — not the floating command window.
    pub status_side_panel_state: HudPanelState,
    /// Floating **Command shell** window (telemetry / layout tools).
    pub command_tray_state: HudPanelState,
    pub intel_timeline_state: HudPanelState,
    pub command_table_state: HudPanelState,
}

impl Default for HudCommandShellLayout {
    fn default() -> Self {
        Self {
            overlay_tray_state: HudPanelState::Collapsed,
            status_side_panel_state: HudPanelState::Collapsed,
            command_tray_state: HudPanelState::Collapsed,
            intel_timeline_state: HudPanelState::Collapsed,
            command_table_state: HudPanelState::Collapsed,
        }
    }
}

impl HudCommandShellLayout {
    #[inline]
    fn panel_open(state: HudPanelState) -> bool {
        state.shows_content()
    }
}

pub struct HudDockShellPlugin;

impl Plugin for HudDockShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(super::panel_state::HudPanelStatePlugin)
            .init_resource::<HudDockRegistry>()
            .init_resource::<HudOverlayTrayState>()
            .init_resource::<HudCommandShellLayout>()
            .init_resource::<HudLayoutStore>()
            .init_resource::<ProductShellDiagnostics>()
            .init_resource::<super::viewport_rect_sanity::ViewportRectSanity>()
            .init_resource::<super::stage6_telemetry::Stage6HudTelemetry>()
            .init_resource::<super::frame_budget_diagnostics::FrameBudgetDiagnostics>()
            .init_resource::<super::shell_update_budget::ProductShellUpdateBudget>()
            .init_resource::<super::shell_widget_timing::ShellWidgetDiagnostics>()
            .init_resource::<super::widget_presentation::WidgetPresentationPolicy>()
            .init_resource::<super::hud_async_task_queue::HudAsyncTaskQueue>()
            .init_resource::<super::pending_hud_layout_commit::PendingHudLayoutCommit>()
            .init_resource::<super::world_interaction_diagnostics::WorldInteractionDiagnostics>()
            .init_resource::<super::retained_widget_cache::RetainedWidgetCache>()
            .init_resource::<super::hud_interaction_budget::HudFrameBudget>()
            .init_resource::<super::interaction_latency::InteractionLatencyMetrics>()
            .init_resource::<super::shell_surface::ShellSurfacePolicy>()
            .init_resource::<super::widget_presentation::HudShellInteractionRouter>()
            .init_resource::<super::hud_shell_stress_harness::HudShellStressHarness>()
            .init_resource::<super::player_intent_panel::PlayerIntentPanelState>()
            .add_plugins(super::hud_dev_overlay::HudDevOverlayPlugin)
            .add_plugins(super::hud_root_tick::HudRootTickPlugin)
            .add_systems(
                Update,
                (
                    sync_hud_overlay_tray_to_policy,
                    sync_hud_overlay_tray_to_minimap_overlays,
                    sync_minimap_overlays_to_hud_tray,
                    mirror_hud_dock_registry_from_widgets,
                    sync_hud_shell_interaction_router,
                    hud_dock_shell_keyboard_toggle.run_if(product_egui_shell_active),
                    super::shell_diagnostics::product_shell_diagnostics_tick,
                    super::shell_update_budget::sync_product_shell_update_budget,
                    super::shell_update_budget::advance_product_shell_update_budget,
                    super::hud_async_task_queue::drain_hud_async_task_queue,
                    super::world_interaction_diagnostics::refresh_world_interaction_diagnostics,
                    super::interaction_latency::refresh_interaction_latency_metrics,
                )
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                PostUpdate,
                (
                    super::hud_interaction_budget::apply_hud_interaction_frame_budget,
                    super::pending_hud_layout_commit::finalize_pending_hud_layout_commits,
                    super::pending_hud_layout_commit::flush_pending_hud_layout_on_pointer_release,
                )
                    .run_if(in_simulation_or_editor),
            );
    }
}

fn sync_hud_overlay_tray_to_policy(
    tray: Res<HudOverlayTrayState>,
    mut policy: ResMut<StrategicOverlayDisplayPolicy>,
) {
    if !tray.is_changed() {
        return;
    }
    policy.apply_routing_congestion = tray.congestion_visible;
    policy.apply_ew_denial = tray.ew_visible;
}

/// UI-P3-M2-TRAY-OPT — overlay tray checkboxes drive [`MapViewInstances::minimap`] compositor mask.
fn sync_hud_overlay_tray_to_minimap_overlays(
    base: Res<State<BaseState>>,
    tray: Res<HudOverlayTrayState>,
    mut map_views: ResMut<MapViewInstances>,
) {
    if !matches!(*base.get(), BaseState::Simulation) || !tray.is_changed() {
        return;
    }
    let mask = tray.minimap_overlay_mask();
    if map_views.minimap.overlays == mask {
        return;
    }
    map_views.minimap.overlays = mask;
    map_views.minimap.bump_revision();
}

/// Keep tray toggles aligned when minimap toolbar edits the same mask.
fn sync_minimap_overlays_to_hud_tray(
    base: Res<State<BaseState>>,
    map_views: Res<MapViewInstances>,
    mut tray: ResMut<HudOverlayTrayState>,
) {
    if !matches!(*base.get(), BaseState::Simulation) || !map_views.is_changed() {
        return;
    }
    let mask = map_views.minimap.overlays;
    if tray.minimap_overlay_mask() == mask {
        return;
    }
    tray.set_minimap_overlay_mask(mask);
}

fn sync_hud_shell_interaction_router(
    dock: Res<HudDockRegistry>,
    mut router: ResMut<super::widget_presentation::HudShellInteractionRouter>,
) {
    if dock.is_changed() {
        router.sync_from_dock(&dock);
    }
}

fn mirror_hud_dock_registry_from_widgets(
    base: Res<State<BaseState>>,
    minimap: Res<MinimapShellState>,
    transmission: Res<TransmissionShellState>,
    layout: Res<HudCommandShellLayout>,
    tray: Res<HudOverlayTrayState>,
    mut dock: ResMut<HudDockRegistry>,
) {
    if !minimap.is_changed()
        && !transmission.is_changed()
        && !layout.is_changed()
        && !tray.is_changed()
        && !base.is_changed()
    {
        return;
    }
    let sim = matches!(*base.get(), BaseState::Simulation);
    let next_minimap = HudWidgetDockState {
        visible: minimap.visible,
        minimized: minimap.minimized,
        detached: minimap.detached,
        z_order: dock.slot(HudWidgetId::Minimap).z_order,
    };
    let next_transmission = HudWidgetDockState {
        visible: transmission.active,
        minimized: transmission.minimized,
        detached: false,
        z_order: dock.slot(HudWidgetId::Transmission).z_order,
    };
    let next_overlay = HudWidgetDockState {
        visible: if sim {
            false
        } else {
            HudCommandShellLayout::panel_open(tray.tray_panel_state)
        },
        minimized: dock.slot(HudWidgetId::OverlayTray).minimized,
        detached: false,
        z_order: dock.slot(HudWidgetId::OverlayTray).z_order,
    };
    let next_command = HudWidgetDockState {
        visible: if sim {
            false
        } else {
            HudCommandShellLayout::panel_open(layout.command_tray_state)
        },
        minimized: dock.slot(HudWidgetId::CommandShell).minimized,
        detached: false,
        z_order: dock.slot(HudWidgetId::CommandShell).z_order,
    };
    if dock.slot(HudWidgetId::Minimap) != next_minimap {
        dock.set_state(HudWidgetId::Minimap, next_minimap);
    }
    if dock.slot(HudWidgetId::Transmission) != next_transmission {
        dock.set_state(HudWidgetId::Transmission, next_transmission);
    }
    if dock.slot(HudWidgetId::OverlayTray).visible != next_overlay.visible {
        dock.slot_mut(HudWidgetId::OverlayTray).visible = next_overlay.visible;
    }
    if dock.slot(HudWidgetId::CommandShell).visible != next_command.visible {
        dock.slot_mut(HudWidgetId::CommandShell).visible = next_command.visible;
    }
}

fn hud_dock_shell_keyboard_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut layout: ResMut<HudCommandShellLayout>,
    mut dock: ResMut<HudDockRegistry>,
) {
    if keys.just_pressed(bindings.toggle_command_left_stack) {
        layout.command_tray_state = if layout.command_tray_state.shows_content() {
            HudPanelState::Collapsed
        } else {
            HudPanelState::Expanded
        };
        dock.slot_mut(HudWidgetId::CommandShell).visible =
            HudCommandShellLayout::panel_open(layout.command_tray_state);
    }
}

pub fn draw_hud_overlay_tray_egui(
    ctx: &mut egui::Context,
    tray: &mut HudOverlayTrayState,
    dock: &mut HudDockRegistry,
    layout_store: &mut HudLayoutStore,
    update_budget: &mut ProductShellUpdateBudget,
    now_secs: f32,
    widget_timing: Option<&mut ShellWidgetDiagnostics>,
    pending_layout: &mut PendingHudLayoutCommit,
) {
    if !tray.tray_panel_state.shows_content() {
        return;
    }
    let mut open = dock.slot(HudWidgetId::OverlayTray).visible;
    if !shell_widget_runs_egui_with_budget(
        dock,
        HudWidgetId::OverlayTray,
        open,
        Some(update_budget),
        now_secs,
    ) {
        return;
    }
    if let Some(response) = show_product_shell_window(
        ctx,
        ShellWindowHost {
            id: HudWidgetId::OverlayTray,
            title: "Overlay tray",
            default_pos: shell_default_window_pos(ctx, HudWidgetId::OverlayTray, [200.0, 120.0]),
            default_size: [200.0, 120.0],
            min_size: [150.0, 100.0],
        },
        layout_store,
        dock,
        &mut open,
        |ui| {
            ui.checkbox(&mut tray.recon_visible, "Recon");
            ui.checkbox(&mut tray.logistics_stress_visible, "Logistics stress");
            ui.checkbox(&mut tray.congestion_visible, "Congestion");
            ui.checkbox(&mut tray.ew_visible, "EW / denial");
            ui.separator();
            ui.label(egui::RichText::new("Minimap heat (GPU compositor)").strong());
            ui.checkbox(&mut tray.fire_heat, "Fire heat");
            ui.checkbox(&mut tray.logistics_heat, "Logistics heat");
            ui.checkbox(&mut tray.construction_heat, "Construction heat");
            ui.checkbox(&mut tray.ecology_heat, "Ecology heat");
        },
        widget_timing,
    ) {
        capture_shell_layout(layout_store, HudWidgetId::OverlayTray, &response, Some(pending_layout));
    }
    dock.slot_mut(HudWidgetId::OverlayTray).visible = open;
}

pub fn draw_hud_command_shell_egui(
    ctx: &mut egui::Context,
    layout: &mut HudCommandShellLayout,
    dock: &mut HudDockRegistry,
    layout_store: &mut HudLayoutStore,
    palette: &UiPalette,
    world: &WorldRepresentationFrame,
    readiness: Option<&AppStage5ReadinessReport>,
    preview_authority: Option<&PreviewPathAuthority>,
    preview_debug: Option<&PreviewPresentationDebug>,
    stage6: Option<&Stage6HudTelemetry>,
    update_budget: &mut ProductShellUpdateBudget,
    now_secs: f32,
    widget_timing: Option<&mut ShellWidgetDiagnostics>,
    world_interaction: Option<&WorldInteractionDiagnostics>,
    async_queue: &HudAsyncTaskQueue,
    interaction_latency: &InteractionLatencyMetrics,
    pending_layout: &mut PendingHudLayoutCommit,
    wave_s_capture: &mut crate::io::save::WaveSShellCapturePending,
    wave_s_restore: &mut crate::io::save::WaveSShellRestorePending,
    wave_s_hydrate: &crate::io::save::WaveSShellHydrateWitness,
    wave_s_imported: &crate::io::save::WaveSImportedBlueprints,
) {
    if !layout.command_tray_state.shows_content() {
        return;
    }

    let mut open = HudCommandShellLayout::panel_open(layout.command_tray_state);
    if !shell_widget_runs_egui_with_budget(
        dock,
        HudWidgetId::CommandShell,
        open,
        Some(update_budget),
        now_secs,
    ) {
        return;
    }
    let layout_slot_count = layout_store.to_collection(dock).widgets.len();
    let mut capture_layout = false;
    let mut restore_layout = false;
    let mut restore_wave_s = false;
    let timing_rows = widget_timing.as_ref().map(|timing| timing.sorted_rows());
    let defer_heavy = pending_layout.drag_active;
    if let Some(response) = show_product_shell_window(
        ctx,
        ShellWindowHost {
            id: HudWidgetId::CommandShell,
            title: "Command shell",
            default_pos: shell_default_window_pos(ctx, HudWidgetId::CommandShell, [250.0, 190.0]),
            default_size: [250.0, 190.0],
            min_size: [190.0, 140.0],
        },
        layout_store,
        dock,
        &mut open,
        |ui| {
            panel_state_checkbox(ui, "Overlay tray", &mut layout.overlay_tray_state);
            panel_state_checkbox(ui, "Status side rail", &mut layout.status_side_panel_state);
            panel_state_checkbox(ui, "Command window", &mut layout.command_tray_state);
            panel_state_checkbox(ui, "Intel timeline (stub)", &mut layout.intel_timeline_state);
            panel_state_checkbox(ui, "Command table (stub)", &mut layout.command_table_state);
            if ui
                .button(
                    egui::RichText::new("Pin window")
                        .small()
                        .monospace(),
                )
                .clicked()
            {
                layout.command_tray_state.toggle_pin();
            }
            ui.separator();
            if defer_heavy {
                ui.label(egui::RichText::new("Layout drag active — telemetry deferred").small().weak());
            } else {
                widget_scroll_vertical_fill("hud_command_shell_body_scroll", ui.available_height())
                    .show(ui, |ui| {
                    draw_stage5_spine_consumer_panel(
                        ui,
                        palette,
                        readiness,
                        Some(world),
                        preview_authority,
                        preview_debug,
                    );
                    ui.separator();
                    let residency = stage6
                        .map(|telemetry| telemetry.residency.clone())
                        .unwrap_or_default();
                    draw_stage6_residency_consumer_panel(ui, palette, &residency);
                    ui.separator();
                    ui.label(egui::RichText::new("Telemetry tab").strong());
                    if let Some(stage6) = stage6 {
                        ui.label(format!(
                            "GPU est {} KB · textures {} · RT {} · upload {:.1} KB/s",
                            stage6.gpu.gpu_memory_estimate_bytes / 1024,
                            stage6.gpu.texture_count_estimate,
                            stage6.gpu.render_target_count_estimate,
                            stage6.gpu.upload_throughput_bytes_per_sec / 1024.0
                        ));
                        ui.label(format!(
                            "buffer residency {} KB · rebuilds {} · egui tex regs {}",
                            stage6.gpu.buffer_residency_bytes / 1024,
                            stage6.gpu.texture_rebuild_count,
                            stage6.gpu.egui_texture_registrations_frame
                        ));
                    }
                    if let Some(snapshot) = async_queue.cache.telemetry_snapshot.as_ref() {
                        ui.label(snapshot.gpu_summary.clone());
                        ui.label(snapshot.shell_metrics_line.clone());
                        ui.label(format!(
                            "async avg {:.2} ms · spikes {}",
                            snapshot.avg_frame_ms, snapshot.spike_count
                        ));
                    }
                    if let Some(summary) = async_queue.cache.telemetry_summary.as_deref() {
                        ui.label(summary.to_string());
                    }
                    if let Some(world) = world_interaction {
                        ui.label(format!(
                            "Construction throughput {:.0}% · queue latency {:.1} ms · depth {}",
                            world.construction_throughput_hint * 100.0,
                            world.construction_queue_latency_ms,
                            world.pending_queue_depth
                        ));
                        ui.label(format!(
                            "Map interaction latency {:.1} ms · hover {}",
                            world.map_interaction_latency_ms,
                            world.hover_diagnostics_active
                        ));
                    }
                    ui.label(format!(
                        "UI click {:.1} ms · drag {:.1} ms · resize {:.1} ms · tooltip {:.1} ms · hover {:.1} ms",
                        interaction_latency.click_to_response_ms,
                        interaction_latency.drag_latency_ms,
                        interaction_latency.panel_resize_latency_ms,
                        interaction_latency.tooltip_resolve_ms,
                        interaction_latency.hover_resolve_ms
                    ));
                    if let Some(rows) = timing_rows.as_ref() {
                        egui::Grid::new("shell_widget_timing_grid")
                            .striped(true)
                            .show(ui, |ui| {
                                ui.label("Widget");
                                ui.label("Layout ms");
                                ui.label("Paint ms");
                                ui.label("Uploads");
                                ui.label("Cause");
                                ui.end_row();
                                for (id, row) in rows.iter() {
                                    if row.layout_us + row.paint_us == 0 && row.texture_uploads == 0 {
                                        continue;
                                    }
                                    ui.label(id.label());
                                    ui.label(format!("{:.2}", row.layout_us as f32 / 1000.0));
                                    ui.label(format!("{:.2}", row.paint_us as f32 / 1000.0));
                                    ui.label(row.texture_uploads.to_string());
                                    ui.label(format!("{:?}", row.rebuild_reason));
                                    ui.end_row();
                                }
                            });
                    }
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!(
                            "Layout slots persisted: {} · bundle schema {}",
                            layout_slot_count,
                            ProductShellPersistenceBundleR8::CURRENT_SCHEMA
                        ))
                        .small()
                        .weak(),
                    );
                    if ui.button("Capture layout to Wave S DTO").clicked() {
                        capture_layout = true;
                    }
                    if ui.button("Restore layout from save bundle").clicked() {
                        restore_wave_s = true;
                    }
                    if wave_s_hydrate.shell_loaded {
                        ui.label(
                            egui::RichText::new(format!(
                                "Wave S loaded: {} widgets · {} blueprint presets",
                                wave_s_hydrate.layout_widget_count, wave_s_hydrate.blueprint_count
                            ))
                            .small()
                            .weak(),
                        );
                    } else if let Some(count) = wave_s_imported
                        .collection
                        .as_ref()
                        .map(|c| c.presets.len())
                    {
                        ui.label(
                            egui::RichText::new(format!("Wave S blueprints cached: {count}"))
                                .small()
                                .weak(),
                        );
                    }
                    if let Some(path) = wave_s_capture.last_written_path.as_deref() {
                        ui.label(
                            egui::RichText::new(format!("Last Wave S write: {path}"))
                                .small()
                                .weak(),
                        );
                    } else if let Some(err) = wave_s_capture.last_error.as_deref() {
                        ui.label(
                            egui::RichText::new(format!("Wave S write: {err}"))
                                .small()
                                .color(palette.warn),
                        );
                    }
                    if ui.button("Restore default layout DTO").clicked() {
                        restore_layout = true;
                    }
                });
            }
        },
        widget_timing,
    ) {
        capture_shell_layout(layout_store, HudWidgetId::CommandShell, &response, Some(pending_layout));
    }
    if capture_layout {
        let captured = layout_store.to_collection(dock);
        layout_store.apply_collection(&captured);
        wave_s_capture.requested = true;
    }
    if restore_wave_s {
        wave_s_restore.requested = true;
    }
    if restore_layout {
        layout_store.apply_collection(&HudLayoutCollectionR8::new());
    }
    layout.command_tray_state = if open {
        HudPanelState::Expanded
    } else {
        HudPanelState::Collapsed
    };
}

fn panel_state_checkbox(ui: &mut egui::Ui, label: &str, state: &mut HudPanelState) {
    let mut open = state.shows_content();
    if ui.checkbox(&mut open, label).changed() {
        *state = if open {
            HudPanelState::Expanded
        } else {
            HudPanelState::Collapsed
        };
    }
}

pub fn draw_hud_dock_minimized_strip_egui(
    ctx: &mut egui::Context,
    policy: &super::widget_presentation::WidgetPresentationPolicy,
    dock: &mut HudDockRegistry,
    minimap: &mut MinimapShellState,
    transmission: &mut TransmissionShellState,
    tray: &mut HudOverlayTrayState,
    layout: &mut HudCommandShellLayout,
    update_budget: &mut ProductShellUpdateBudget,
) {
    if !HudWidgetId::ALL.iter().any(|id| dock.slot(*id).minimized) {
        return;
    }
    egui::TopBottomPanel::bottom("hud_dock_minimized_strip")
        .frame(egui::Frame::NONE)
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                for id in HudWidgetId::ALL {
                    if draw_minimized_shell_chip(ui, id, dock) {
                        update_budget.bump_widget_event(id);
                        match id {
                            HudWidgetId::Minimap => {
                                minimap.visible = true;
                                minimap.minimized = false;
                            }
                            HudWidgetId::Transmission => {
                                if policy.widget_enabled(HudWidgetId::Transmission) {
                                    transmission.active = true;
                                    transmission.minimized = false;
                                }
                            }
                            HudWidgetId::OverlayTray => {
                                tray.tray_panel_state = HudPanelState::Expanded;
                            }
                            HudWidgetId::CommandShell => {
                                layout.command_tray_state = HudPanelState::Expanded;
                            }
                            _ => {}
                        }
                    }
                }
            });
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_shell_layout_defaults_closed_on_load() {
        let layout = HudCommandShellLayout::default();
        assert_eq!(layout.overlay_tray_state, HudPanelState::Collapsed);
        assert_eq!(layout.command_tray_state, HudPanelState::Collapsed);
        assert_eq!(layout.command_table_state, HudPanelState::Collapsed);
    }

    #[test]
    fn overlay_tray_default_matches_simulation_minimap_mask() {
        let tray = HudOverlayTrayState::default();
        let expected = simulation_minimap_overlay_defaults();
        assert_eq!(tray.minimap_overlay_mask(), expected);
    }

    #[test]
    fn overlay_tray_minimap_mask_roundtrip() {
        let mut tray = HudOverlayTrayState::default();
        tray.logistics_heat = false;
        tray.construction_heat = true;
        let mask = tray.minimap_overlay_mask();
        assert!(!mask.logistics_heat);
        assert!(mask.construction_heat);
        tray.set_minimap_overlay_mask(simulation_minimap_overlay_defaults());
        assert!(tray.ecology_heat);
    }
}
