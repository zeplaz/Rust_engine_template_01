//! Left **status rail** — docked egui side panel (`HudCommandShellLayout::status_side_panel_state`).
//!
//! **2B-03:** Editor-only — simulation uses Bevy left context rail; gated via
//! [`crate::gui::ui_gates::side_status_rail_egui_active`] and
//! [`super::shell_framework::side_status_rail_egui_dock_active`].
use bevy::prelude::*;
use bevy_egui::egui;

use crate::construction::{ActiveBuildTool, BuildPlacementPreview, BuildGhostState};
use crate::gui::input_bindings::InputBindings;
use crate::gui::style::widget_scroll_vertical_fill;
use crate::gui::CommandLeftStackState;
use crate::render::AppStage5ReadinessReport;
use crate::systems::sim_control::{SimControlState, SimTick};
use crate::strategic::OperationalTheaterSummary;

use super::dock_shell::HudCommandShellLayout;
use super::hud_chrome::{
    draw_collapsed_side_rail, section_rule, side_panel_header, stat_chip, hud_side_rail_frame,
    HudChromeIcon,
};
use super::panel_state::HudPanelState;
use super::stage5_spine_consumer::draw_stage5_spine_consumer_panel;
use super::stage6_consumer::draw_stage6_residency_consumer_panel;
use super::stage6_telemetry::Stage6HudTelemetry;
use super::hud_async_task_queue::HudAsyncTaskQueue;
use super::interaction_latency::InteractionLatencyMetrics;
use super::world_interaction_diagnostics::WorldInteractionDiagnostics;
use crate::gui::editor::world_preview::{PreviewPathAuthority, PreviewPresentationDebug};
use crate::gui::style::UiPalette;
use crate::gui::WorldRepresentationFrame;

/// Draw the docked left status panel (call early in the egui pass).
pub fn draw_hud_side_status_panel_egui(
    ui: &mut egui::Ui,
    layout: &mut HudCommandShellLayout,
    palette: &UiPalette,
    bindings: &InputBindings,
    world: &WorldRepresentationFrame,
    readiness: Option<&AppStage5ReadinessReport>,
    preview_authority: Option<&PreviewPathAuthority>,
    preview_debug: Option<&PreviewPresentationDebug>,
    stage6: Option<&Stage6HudTelemetry>,
    sim: Option<&SimControlState>,
    tick: Option<&SimTick>,
    theater: Option<&OperationalTheaterSummary>,
    tool: Option<&ActiveBuildTool>,
    ghost: Option<&BuildGhostState>,
    preview: Option<&BuildPlacementPreview>,
    left_stack: Option<&CommandLeftStackState>,
    async_queue: &HudAsyncTaskQueue,
    interaction_latency: &InteractionLatencyMetrics,
    world_interaction: Option<&WorldInteractionDiagnostics>,
) {
    let width = layout.status_side_panel_state.target_width();
    let mut panel_state = layout.status_side_panel_state;

    let panel_response = egui::Panel::left("hud_status_side_panel")
        .exact_size(width)
        .resizable(false)
        .frame(hud_side_rail_frame(palette))
        .show(ui, |ui| {
            side_panel_header(ui, palette, "STATUS", &mut panel_state);

            if !panel_state.shows_content() {
                draw_collapsed_side_rail(ui, palette);
                return;
            }

            ui.add_space(4.0);

            if let Some(sim) = sim {
                let tick_n = tick.map(|t| t.0).unwrap_or(0);
                let run = if sim.paused { "paused" } else { "running" };
                stat_chip(ui, palette, HudChromeIcon::Sim, format!("tick {tick_n} · {run}"));
            }
            if let Some(t) = tool {
                stat_chip(ui, palette, HudChromeIcon::Build, t.tool.label());
                if let Some(p) = preview {
                    let commit = if p.report.allows_commit { "ready" } else { "blocked" };
                    stat_chip(ui, palette, HudChromeIcon::Build, format!("place · {commit}"));
                }
                if ghost.is_some_and(|g| g.origin.is_some()) {
                    let key = InputBindings::format_key(bindings.confirm_build_placement);
                    stat_chip(ui, palette, HudChromeIcon::Build, format!("ghost · {key}"));
                }
            }
            if let Some(stack) = left_stack {
                stat_chip(
                    ui,
                    palette,
                    HudChromeIcon::Stack,
                    if stack.collapsed {
                        "context · folded"
                    } else {
                        "context · open"
                    },
                );
            }
            if let Some(th) = theater {
                stat_chip(
                    ui,
                    palette,
                    HudChromeIcon::Theater,
                    format!(
                        "threat {:.2} · {} factions",
                        th.mean_threat_by_slot[0],
                        th.active_faction_slots
                    ),
                );
            }

            stat_chip(
                ui,
                palette,
                HudChromeIcon::Lod,
                format!("{:?} · zoom {:.2}", world.global_band(), world.zoom),
            );

            section_rule(ui, palette);

            ui.collapsing(
                egui::RichText::new("Shortcuts")
                    .small()
                    .monospace()
                    .color(palette.fg_muted),
                |ui| {
                    let key = |k: KeyCode| InputBindings::format_key(k);
                    ui.label(
                        egui::RichText::new(format!(
                            "Panel {} · Left stack {} · Command win {}",
                            key(bindings.toggle_hud_status_side_panel),
                            key(bindings.toggle_command_left_stack),
                            key(bindings.toggle_diagnostics),
                        ))
                        .small()
                        .weak(),
                    );
                },
            );

            section_rule(ui, palette);

            widget_scroll_vertical_fill("hud_status_side_body", ui.available_height()).show(ui, |ui| {
                draw_stage5_spine_consumer_panel(
                    ui,
                    palette,
                    readiness,
                    Some(world),
                    preview_authority,
                    preview_debug,
                );
                if let Some(telemetry) = stage6 {
                    draw_stage6_residency_consumer_panel(ui, palette, &telemetry.residency);
                } else if let Some(dto) = async_queue.cache.residency_dto.as_ref() {
                    draw_stage6_residency_consumer_panel(ui, palette, dto);
                } else {
                    ui.label(egui::RichText::new("Stage 6 — residency pending").small().weak());
                }
                if let Some(wi) = world_interaction {
                    ui.label(
                        egui::RichText::new(format!(
                            "queue {} · hover {}",
                            wi.pending_queue_depth,
                            if wi.hover_diagnostics_active { "on" } else { "off" }
                        ))
                        .small()
                        .weak(),
                    );
                }
                ui.label(
                    egui::RichText::new(format!(
                        "click {:.0} ms · async {}",
                        interaction_latency.click_to_response_ms,
                        async_queue.pending.len()
                    ))
                    .small()
                    .weak(),
                );
            });
        });

    if panel_response.response.hovered() && panel_state == HudPanelState::Collapsed {
        panel_state.hover_peek();
    }
    layout.status_side_panel_state = panel_state;
}

/// Keyboard: cycle status side panel width state.
pub fn hud_status_side_panel_toggle_system(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut layout: ResMut<HudCommandShellLayout>,
) {
    if !keys.just_pressed(bindings.toggle_hud_status_side_panel) {
        return;
    }
    layout.status_side_panel_state = match layout.status_side_panel_state {
        HudPanelState::Collapsed => HudPanelState::Expanded,
        HudPanelState::Peek => HudPanelState::Expanded,
        HudPanelState::Expanded => HudPanelState::Collapsed,
        HudPanelState::Pinned => HudPanelState::Collapsed,
    };
}
