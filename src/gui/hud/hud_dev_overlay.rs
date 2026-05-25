//! HUD / shell developer overlay — read-only diagnostics.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::{
    ActiveMapViewInput, MapFitValidationLog, MapPresentationDiagnostics, MapViewInstanceId,
    MapViewTextureCache, MinimapShellState, PerViewLodHints, ViewManager,
};
use crate::render::DebugRenderTraceConfig;
use crate::systems::weather::WeatherVisualSettings;
use crate::gui::style::{
    native_ui_pixels_per_point, resolve_ui_scale, HudDensityProfile, UiPalette,
    DEFAULT_UI_GLOBAL_SCALE, UI_GLOBAL_SCALE_STEP,
};
use super::frame_budget_diagnostics::{FrameBudgetBucket, FrameBudgetDiagnostics, FRAME_HISTORY_LEN};
use super::hud_async_task_queue::HudAsyncTaskQueue;
use super::hud_interaction_budget::HudFrameBudget;
use super::interaction_latency::InteractionLatencyMetrics;
use super::retained_widget_cache::RetainedWidgetCache;
use super::layout_store::HudLayoutStore;
use super::player_intent_panel::PlayerIntentPanelState;
use super::shell_update_budget::ProductShellUpdateBudget;
use super::shell_widget_timing::ShellWidgetDiagnostics;
use super::stage6_telemetry::Stage6HudTelemetry;
use super::shell_diagnostics::ProductShellDiagnostics;
use super::shell_framework::{HudDockRegistry, HudWidgetId};
use super::viewport_rect_sanity::ViewportRectSanity;
use super::world_interaction_diagnostics::WorldInteractionDiagnostics;

#[derive(Resource, Clone, Debug)]
pub struct HudDevOverlayState {
    pub visible: bool,
    pub show_ui_timing: bool,
    pub show_viewport_mismatch: bool,
    pub show_texture_refresh: bool,
    pub show_map_transform: bool,
}

impl Default for HudDevOverlayState {
    fn default() -> Self {
        Self {
            visible: false,
            show_ui_timing: true,
            show_viewport_mismatch: true,
            show_texture_refresh: true,
            show_map_transform: true,
        }
    }
}

pub struct HudDevOverlayPlugin;

impl Plugin for HudDevOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudDevOverlayState>()
            .add_systems(Update, (hud_dev_overlay_keyboard_toggle, hud_ui_scale_keyboard));
    }
}

fn hud_dev_overlay_keyboard_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<HudDevOverlayState>,
) {
    if keys.just_pressed(KeyCode::F4) {
        state.visible = !state.visible;
    }
}

fn hud_ui_scale_keyboard(keys: Res<ButtonInput<KeyCode>>, mut density: ResMut<HudDensityProfile>) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        density.adjust_global_scale(UI_GLOBAL_SCALE_STEP);
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        density.adjust_global_scale(-UI_GLOBAL_SCALE_STEP);
    }
}

fn draw_frame_time_graph(ui: &mut egui::Ui, history: &[f32; FRAME_HISTORY_LEN], cursor: usize) {
    let width = ui.available_width().max(120.0);
    let height = 56.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);
    let max_ms = history
        .iter()
        .copied()
        .fold(16.0f32, f32::max)
        .max(1.0);
    let step = rect.width() / FRAME_HISTORY_LEN as f32;
    for i in 0..FRAME_HISTORY_LEN {
        let idx = (cursor + i) % FRAME_HISTORY_LEN;
        let sample = history[idx];
        let bar_h = (sample / max_ms).clamp(0.0, 1.0) * rect.height();
        let x = rect.left() + i as f32 * step;
        let bar = egui::Rect::from_min_max(
            egui::pos2(x, rect.bottom() - bar_h),
            egui::pos2(x + step.max(1.0), rect.bottom()),
        );
        painter.rect_filled(bar, 0.0, ui.visuals().warn_fg_color);
    }
    painter.text(
        rect.left_top() + egui::vec2(4.0, 2.0),
        egui::Align2::LEFT_TOP,
        format!("max {:.1} ms", max_ms),
        egui::FontId::monospace(10.0),
        ui.visuals().weak_text_color(),
    );
}

pub fn draw_hud_dev_overlay_egui(
    ctx: &mut egui::Context,
    palette: &UiPalette,
    overlay: &HudDevOverlayState,
    trace: &DebugRenderTraceConfig,
    weather: &WeatherVisualSettings,
    diag: &ProductShellDiagnostics,
    minimap: &MinimapShellState,
    dock: &HudDockRegistry,
    layout: &HudLayoutStore,
    intent: &PlayerIntentPanelState,
    stage6: Option<&Stage6HudTelemetry>,
    viewport_rect: &ViewportRectSanity,
    budget: &FrameBudgetDiagnostics,
    shell_budget: &ProductShellUpdateBudget,
    widget_timing: &ShellWidgetDiagnostics,
    world_interaction: Option<&WorldInteractionDiagnostics>,
    async_queue: &HudAsyncTaskQueue,
    texture_cache: &MapViewTextureCache,
    retained: &RetainedWidgetCache,
    interaction_budget: &HudFrameBudget,
    interaction_latency: &InteractionLatencyMetrics,
    map_presentation: &MapPresentationDiagnostics,
    map_fit_log: &MapFitValidationLog,
    active_map_input: &ActiveMapViewInput,
    per_view_lod: &PerViewLodHints,
    view_manager: &ViewManager,
    view_isolation: &crate::gui::ViewIsolationDiagnostics,
    density: &mut HudDensityProfile,
) {
    if !overlay.visible {
        return;
    }

    egui::Window::new("HUD dev overlay (F4)")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 56.0))
        .default_size([360.0, 520.0])
        .resizable(true)
        .constrain(true)
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::VisibleWhenNeeded)
        .show(ctx, |ui| {
            ui.label(egui::RichText::new("egui widget scale").strong());
            ui.add(
                egui::Slider::new(&mut density.global_scale, 0.65..=1.25)
                    .step_by(UI_GLOBAL_SCALE_STEP as f64)
                    .text("global"),
            );
            if ui.button("Reset to default").clicked() {
                density.global_scale = DEFAULT_UI_GLOBAL_SCALE;
            }
            ui.label(
                egui::RichText::new("Ctrl + / Ctrl - adjust scale while the app is focused.")
                    .small()
                    .weak(),
            );
            ui.separator();
            ui.label(egui::RichText::new("Frame budget").strong());
            ui.label(format!(
                "frame {:.2} ms · avg {:.2} ms · max {:.2} ms",
                budget.frame_time_ms, budget.avg_frame_ms, budget.max_frame_ms
            ));
            draw_frame_time_graph(ui, &budget.frame_history, budget.history_cursor);
            ui.label(format!(
                "egui {:.2} ms · extraction {:.2} ms · upload {:.1} KB/frame ({:.1} KB/s)",
                budget.egui_frame_ms,
                budget.render_extraction_ms,
                budget.upload_bytes_frame as f32 / 1024.0,
                budget.upload_bytes_per_sec / 1024.0
            ));
            ui.label(format!(
                "layout invalidations/frame: {} · texture rebinds/frame: {} · viewport mutations/frame: {} · drag mutation attempts: {}",
                budget.layout_invalidations_frame,
                budget.texture_rebinds_frame,
                budget.viewport_mutations_frame,
                budget.drag_frame_mutation_attempts,
            ));
            if let Some(id) = budget.layout_spam_source {
                ui.label(format!("layout capture spam source: {}", id.label()));
            }
            if let Some(id) = budget.drag_mutation_source {
                ui.label(format!("drag mutation source: {}", id.label()));
            }
            ui.separator();
            ui.label(egui::RichText::new("Top subsystems (last frame)").strong());
            for (bucket, ms) in budget.top_buckets_by_last_ms(4) {
                ui.label(format!("{}: {:.2} ms", bucket.label(), ms));
            }
            ui.separator();
            ui.label(egui::RichText::new("Render trace groups (read-only)").strong());
            ui.label(format!("Viewport trace: {}", trace.viewport_trace));
            ui.label(format!("Camera sync trace: {}", trace.camera_sync_trace));
            ui.label(format!("Render-target trace: {}", trace.render_target_trace));
            ui.label(format!("Particle routing trace: {}", trace.particle_routing_trace));
            ui.separator();
            ui.label(egui::RichText::new("Particle visibility (read-only)").strong());
            ui.label(format!("Precipitation particles: {}", weather.particles));
            ui.label(format!("Precipitation overlay: {}", weather.overlay));
            ui.separator();
            ui.label(format!("Show UI timing: {}", overlay.show_ui_timing));
            ui.label(format!("Show viewport mismatch: {}", overlay.show_viewport_mismatch));
            ui.label(format!("Show texture refresh: {}", overlay.show_texture_refresh));
            if overlay.show_ui_timing {
                ui.label(format!(
                    "egui passes: {} · frame Δ {:.2} ms",
                    diag.egui_pass_count,
                    diag.last_frame_delta_secs * 1000.0
                ));
                for bucket in FrameBudgetBucket::ALL {
                    let stats = budget.buckets[bucket.index()];
                    if stats.last_ms > 0.0 || stats.events_last_frame > 0 {
                        ui.label(format!(
                            "{}: {:.2} ms avg {:.2} ms · events {}",
                            bucket.label(),
                            stats.last_ms,
                            stats.avg_ms,
                            stats.events_last_frame
                        ));
                    }
                }
            }
            if overlay.show_texture_refresh {
                for id in HudWidgetId::ALL {
                    let rebuilds = diag.texture_rebuild_count(id);
                    if rebuilds > 0 {
                        ui.label(format!("{} texture rebuilds: {rebuilds}", id.label()));
                    }
                }
                for (id, count) in budget.rebuild_spike_widgets(diag) {
                    ui.label(
                        egui::RichText::new(format!("{} rebuild spike: {count}", id.label()))
                            .color(palette.warn),
                    );
                }
                ui.label(format!(
                    "Minimap raster revision: {}",
                    minimap.cached_texture_revision
                ));
            }
            if overlay.show_viewport_mismatch {
                let window = minimap
                    .last_window_rect
                    .map(|rect| format!("{rect:?}"))
                    .unwrap_or_else(|| "none".into());
                let image = minimap
                    .last_image_rect
                    .map(|rect| format!("{rect:?}"))
                    .unwrap_or_else(|| "none".into());
                ui.label(format!("Minimap window rect: {window}"));
                ui.label(format!("Minimap image rect: {image}"));
                ui.label(format!(
                    "UI wrote camera: {} · camera drove UI: {}",
                    minimap.diagnostic_ui_wrote_camera, minimap.diagnostic_camera_drove_ui
                ));
                if let Some((source, kind)) = diag.last_viewport_rect_issue {
                    ui.label(format!("Last rect issue: {source:?} · {kind:?}"));
                }
                for (source, count) in &diag.viewport_rect_issues {
                    if *count > 0 {
                        ui.label(format!("{source:?} rect issues: {count}"));
                    }
                }
                if viewport_rect.suppressed_logs > 0 {
                    ui.label(format!(
                        "Suppressed duplicate rect warnings: {}",
                        viewport_rect.suppressed_logs
                    ));
                }
            }
            if let Some(stage6) = stage6 {
                ui.separator();
                ui.label(egui::RichText::new("Stage 6 telemetry").strong());
                ui.label(format!(
                    "resident={} ghost={} atlas_pages={} rev {}",
                    stage6.residency.resident_chunks,
                    stage6.residency.ghost_chunks,
                    stage6.residency.paged_atlas_pages,
                    stage6.frame_revision
                ));
                ui.label(format!(
                    "GPU est {} KB · textures {} · RT {} · buffer {} KB",
                    stage6.gpu.gpu_memory_estimate_bytes / 1024,
                    stage6.gpu.texture_count_estimate,
                    stage6.gpu.render_target_count_estimate,
                    stage6.gpu.buffer_residency_bytes / 1024
                ));
                ui.label(format!(
                    "upload {:.1} KB/s · rebuilds {} · egui tex regs {}",
                    stage6.gpu.upload_throughput_bytes_per_sec / 1024.0,
                    stage6.gpu.texture_rebuild_count,
                    stage6.gpu.egui_texture_registrations_frame
                ));
                ui.label(format!(
                    "GPU upload={} reserved={} high_water={} rows={} dispatch={}",
                    stage6.gpu.upload_bytes,
                    stage6.gpu.reserved_bytes,
                    stage6.gpu.high_watermark_bytes,
                    stage6.gpu.active_rows,
                    stage6.gpu.dispatch_count
                ));
                ui.label(format!(
                    "virt cells={} upload/frame={} atlas pressure {:.0}% dirty={} overlay updates={}",
                    budget.stage6.active_residency_cells,
                    budget.stage6.upload_bytes_frame,
                    budget.stage6.atlas_pressure * 100.0,
                    budget.stage6.dirty_region_count,
                    budget.stage6.overlay_update_count
                ));
            }
            ui.separator();
            ui.label(egui::RichText::new("Shell widget timing (sorted)").strong());
            egui::Grid::new("f4_shell_widget_timing")
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Widget");
                    ui.label("Layout ms");
                    ui.label("Paint ms");
                    ui.label("Uploads");
                    ui.label("Cause");
                    ui.end_row();
                    for (id, row) in widget_timing.sorted_rows() {
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
            ui.label(format!(
                "Background refresh {:.1} Hz · async completed {} dropped {}",
                shell_budget.background_hz,
                async_queue.completed,
                async_queue.dropped
            ));
            if let Some(line) = async_queue.cache.log_line.as_deref() {
                ui.label(format!("Deferred log: {line}"));
            }
            if let Some(legend) = async_queue.cache.minimap_legend.as_deref() {
                ui.label(format!("Deferred legend: {legend}"));
            }
            let minimap_binding = texture_cache.binding(MapViewInstanceId::Minimap);
            ui.label(format!(
                "Texture rebind/frame {} · upload/frame {} · stale-cache/frame {}",
                minimap_binding.rebinds_frame,
                minimap_binding.uploads_frame,
                minimap_binding.stale_cache_frame
            ));
            ui.separator();
            ui.label(egui::RichText::new("Retained widget cache").strong());
            ui.label(format!(
                "hit {:.0}% · lookups {} · skipped layout {} · skipped paint {}",
                retained.cache_hit_rate() * 100.0,
                retained.lookups,
                retained.skipped_layout,
                retained.skipped_paint
            ));
            ui.separator();
            ui.label(egui::RichText::new("Interaction budget").strong());
            ui.label(format!(
                "frame {:.2} ms · budget {:.2} ms · overruns/frame {} · deferred {}",
                interaction_budget.last_frame_ms,
                interaction_budget.ui_budget_ms,
                interaction_budget.overruns_frame,
                interaction_budget.deferred_widget_count_frame
            ));
            if let Some(offender) = interaction_budget.worst_offender {
                ui.label(format!(
                    "Worst offender {} · {:.2} ms",
                    offender.label(),
                    interaction_budget.worst_offender_ms
                ));
            }
            ui.separator();
            ui.label(egui::RichText::new("Interaction latency").strong());
            ui.label(format!(
                "click {:.1} ms · drag {:.1} ms · resize {:.1} ms · tooltip {:.1} ms · hover {:.1} ms · scroll {:.1} ms",
                interaction_latency.click_to_response_ms,
                interaction_latency.drag_latency_ms,
                interaction_latency.panel_resize_latency_ms,
                interaction_latency.tooltip_resolve_ms,
                interaction_latency.hover_resolve_ms,
                interaction_latency.scroll_latency_ms
            ));
            ui.label(format!(
                "map redraw preview {:.1} ms · minimap {:.1} ms · last widget {}",
                interaction_latency.map_preview_redraw_ms,
                interaction_latency.map_minimap_redraw_ms,
                interaction_latency.last_widget_label
            ));
            ui.separator();
            ui.label(egui::RichText::new("MAP SCALE DEBUG").strong());
            let preview_delta = map_fit_log
                .world_preview
                .as_ref()
                .map(|sample| sample.delta_pixels)
                .unwrap_or(0.0);
            let minimap_delta = map_fit_log
                .minimap
                .as_ref()
                .map(|sample| sample.delta_pixels)
                .unwrap_or(0.0);
            ui.label(format!(
                "preview scale: {:.4} · minimap scale: {:.4} · expected scale: {:.4}",
                map_fit_log.preview_scale,
                map_fit_log.minimap_scale,
                map_fit_log.expected_scale
            ));
            ui.label(format!(
                "delta px: preview {:.2} · minimap {:.2} · mismatch frames {}",
                preview_delta,
                minimap_delta,
                map_fit_log.mismatch_frames
            ));
            ui.label(format!(
                "fit mode mismatch: {} · ui global scale: {:.3} · native dpi: {:.3}",
                map_fit_log.fit_mode_mismatch,
                density.clamped_global_scale(),
                native_ui_pixels_per_point(ctx)
            ));
            ui.label(format!(
                "effective ui scale: {:.3}",
                resolve_ui_scale(ctx, density)
            ));
            ui.separator();
            ui.label(egui::RichText::new("Map input routing").strong());
            ui.label(format!("ActiveMapViewInput: {:?}", active_map_input.0));
            ui.label(format!(
                "blocks main MapCameraDesired input (vm-07): {}",
                active_map_input.blocks_main_world_map_camera_input()
            ));
            ui.separator();
            ui.label(egui::RichText::new("View authority (read-only)").strong());
            egui::CollapsingHeader::new("PerViewLodHints")
                .default_open(false)
                .show(ui, |ui| {
                    if per_view_lod.by_view.is_empty() {
                        ui.label("none");
                    } else {
                        for (id, band) in per_view_lod.by_view.iter() {
                            ui.monospace(format!("{id:?}: {band:?}"));
                        }
                    }
                });
            egui::CollapsingHeader::new("ViewManager · viewport_rect")
                .default_open(false)
                .show(ui, |ui| {
                    if view_manager.views.is_empty() {
                        ui.label("no entries");
                    } else {
                        for (id, inst) in view_manager.views.iter() {
                            let r = inst.viewport_rect;
                            ui.monospace(format!(
                                "{id:?}: {:.0}×{:.0} @ ({:.0},{:.0})",
                                r.width(),
                                r.height(),
                                r.min.x,
                                r.min.y
                            ));
                        }
                    }
                });
            ui.label(format!(
                "View isolation — minimap lockstep suspect: {} · preview lockstep suspect: {} · sim_map shares main: {}",
                view_isolation.minimap_main_lockstep_suspect,
                view_isolation.preview_main_lockstep_suspect,
                view_isolation.simulation_map_shares_main_camera
            ));
            if view_isolation.minimap_main_lockstep_suspect {
                ui.label(
                    egui::RichText::new("Minimap matches main camera while follow is not FollowCamera — audit cross-view writes.")
                        .small()
                        .color(palette.warn),
                );
            }
            ui.label(format!(
                "Overlay fire_heat (vm-08) — preview {} · minimap {}",
                view_isolation.preview_overlay_fire_heat,
                view_isolation.minimap_overlay_fire_heat
            ));
            ui.separator();
            ui.label(egui::RichText::new("Map view diagnostics").strong());
            for (label, slot) in [
                ("World preview", &map_presentation.world_preview),
                ("Minimap", &map_presentation.minimap),
            ] {
                let viewport = slot
                    .allocated_rect
                    .map(|rect| format!("{rect:?}"))
                    .unwrap_or_else(|| "none".into());
                let image = slot
                    .image_rect
                    .map(|rect| format!("{rect:?}"))
                    .unwrap_or_else(|| "none".into());
                ui.label(format!("{label} viewport: {viewport}"));
                ui.label(format!("{label} image: {image}"));
                ui.label(format!(
                    "{label} uv {:?} · fit {} · tex aspect {:.2} · panel aspect {:.2} · zoom {:.2}",
                    slot.uv_rect,
                    slot.fit_mode.label(),
                    slot.aspect_texture,
                    slot.aspect_panel,
                    slot.camera_zoom
                ));
            }
            ui.label(format!(
                "logical dpi {:.2} · density global {:.2}",
                ctx.pixels_per_point(),
                density.global_scale
            ));
            ui.label(format!(
                "density padding {:.1} · spacing {:.1} · icon {:.1} · compact {}",
                density.window_padding,
                density.item_spacing,
                density.icon_size,
                density.compact_mode
            ));
            if let Some(worst) = widget_timing.worst_offender {
                ui.label(format!(
                    "Worst offender {} · {:.2} ms · spikes {}",
                    worst.label(),
                    widget_timing.worst_offender_us as f32 / 1000.0,
                    widget_timing.frame_spike_markers
                ));
            }
            if let Some(world) = world_interaction {
                ui.separator();
                ui.label(egui::RichText::new("World interaction").strong());
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
            if let Some(anomaly) = budget.last_anomaly.as_ref() {
                ui.separator();
                ui.label(egui::RichText::new("Latest anomaly").strong());
                ui.label(format!("{:?}: {}", anomaly.kind, anomaly.detail));
                if anomaly.suppressed > 0 {
                    ui.label(format!("Suppressed repeats: {}", anomaly.suppressed));
                }
                if budget.anomaly_suppressed_total > 0 {
                    ui.label(format!(
                        "Total suppressed anomaly logs: {}",
                        budget.anomaly_suppressed_total
                    ));
                }
            }
            ui.separator();
            ui.label(egui::RichText::new("Shell visibility").strong());
            for id in HudWidgetId::ALL {
                let slot = dock.slot(id);
                let frame = layout.frame(id);
                ui.label(format!(
                    "{} · vis {} · min {} · detached {} · layout {}",
                    id.label(),
                    slot.visible,
                    slot.minimized,
                    slot.detached,
                    frame.initialized
                ));
            }
            ui.separator();
            ui.label(egui::RichText::new("Intent drafts (display only)").strong());
            if intent.drafts.is_empty() {
                ui.label(egui::RichText::new("No staged intents.").weak());
            } else {
                for draft in &intent.drafts {
                    ui.label(format!("{} — {}", draft.label, draft.summary));
                }
            }
        });
}
