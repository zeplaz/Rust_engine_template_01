//! **COD-ART-HUD-ICON-ATLAS-001** — rail-anchored power line tool sheet with HUD icons.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_egui::egui;

use crate::construction::{
    commit_power_line_to_utility_graph, ActivePowerLinePlacement, ActiveBuildTool, BuildStripState,
    BuildTool, PowerLineRoutingMode, ToolContext,
};
use crate::engine::states::BaseState;
use crate::gui::hud::power_hud_icon_atlas::{
    draw_power_hud_icon_labeled, PowerHudEguiTextureCache, PowerHudIconAtlasManifest,
    PowerHudIconAtlasUi, PowerHudIconId,
};
use crate::gui::UiPalette;
use crate::infrastructure::utility::graph::{UtilityGraph, UtilityNetworkSnapshotResource};
use crate::infrastructure::VoltageClass;

use super::sim_hud_egui_theme::{
    apply_sim_hud_egui_theme, body_text, caption_text, data_text, picker_header_frame,
    picker_sheet_frame, title_text,
};

pub const POWER_TOOL_SHEET_W_PX: f32 = 300.0;

#[derive(Resource, Debug, Clone, Default)]
pub struct SimPowerToolSheetState {
    pub open: bool,
}

impl SimPowerToolSheetState {
    pub fn sync_from_tool(&mut self, tool: &ActiveBuildTool) {
        self.open = matches!(tool.tool, BuildTool::PowerLine(_));
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}

#[derive(SystemParam)]
pub struct SimPowerToolSheetDrawParams<'w> {
    pub strip: Res<'w, BuildStripState>,
    pub palette: Res<'w, UiPalette>,
    pub tool: ResMut<'w, ActiveBuildTool>,
    pub power_sheet: ResMut<'w, SimPowerToolSheetState>,
    pub placement: ResMut<'w, ActivePowerLinePlacement>,
    pub snap_res: ResMut<'w, UtilityNetworkSnapshotResource>,
    pub graph: ResMut<'w, UtilityGraph>,
    pub atlas_ui: Option<Res<'w, PowerHudIconAtlasUi>>,
    pub manifests: Res<'w, Assets<PowerHudIconAtlasManifest>>,
    pub tex_cache: ResMut<'w, PowerHudEguiTextureCache>,
}

#[must_use]
pub fn power_tool_sheet_anchor(strip: &BuildStripState, left_stack_collapsed: bool) -> egui::Pos2 {
    let _ = strip;
    super::simulation_shell_phase2::build_rail_slot_anchor_xy(
        ToolContext::Utilities,
        left_stack_collapsed,
    )
}

fn voltage_label(v: VoltageClass) -> &'static str {
    match v {
        VoltageClass::Low => "Distribution",
        VoltageClass::Medium => "Medium",
        VoltageClass::High => "Transmission",
    }
}

pub fn draw_sim_power_tool_sheet_egui(
    mut contexts: bevy_egui::EguiContexts,
    base: Res<State<BaseState>>,
    left_stack: Res<crate::gui::CommandLeftStackState>,
    mut draw: SimPowerToolSheetDrawParams,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) {
        return Ok(());
    }
    draw.power_sheet.sync_from_tool(draw.tool.as_ref());
    if !draw.power_sheet.open {
        return Ok(());
    }
    let BuildTool::PowerLine(active_voltage) = draw.tool.tool else {
        return Ok(());
    };

    let valid_count = draw
        .placement
        .generated_segments
        .iter()
        .filter(|s| s.valid)
        .count();
    let est_cost = valid_count.saturating_mul(12);
    let can_build = valid_count > 0;

    let texture_id = draw
        .atlas_ui
        .as_ref()
        .and_then(|atlas| draw.tex_cache.resolve(&mut contexts, &atlas.atlas));
    let manifest = draw
        .atlas_ui
        .as_ref()
        .and_then(|atlas| draw.manifests.get(&atlas.manifest));
    let ctx = contexts.ctx_mut()?;
    apply_sim_hud_egui_theme(ctx, &draw.palette);
    let anchor = power_tool_sheet_anchor(draw.strip.as_ref(), left_stack.collapsed);
    let icon_tint = draw.palette.accent_terminal;
    let selected_tint = draw.palette.accent_action;

    egui::Area::new(egui::Id::new("sim_power_tool_sheet"))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor)
        .show(ctx, |ui| {
            ui.set_width(POWER_TOOL_SHEET_W_PX);
            picker_sheet_frame(&draw.palette).show(ui, |ui| {
                picker_header_frame(&draw.palette).show(ui, |ui| {
                    ui.label(title_text(
                        &draw.palette,
                        &format!("Power line — {}", voltage_label(active_voltage)),
                    ));
                });

                ui.label(caption_text(&draw.palette, "Mode"));
                ui.horizontal(|ui| {
                    for (mode, label) in [
                        (PowerLineRoutingMode::Curved, "Curved"),
                        (PowerLineRoutingMode::Orthogonal90, "90°"),
                    ] {
                        let selected = draw.placement.routing_mode == mode;
                        let tint = if selected { selected_tint } else { icon_tint };
                        let clicked = if let (Some(tex), Some(manifest)) = (texture_id, manifest) {
                            draw_power_hud_icon_labeled(
                                ui,
                                tex,
                                manifest,
                                PowerHudIconId::for_routing_mode(mode),
                                16.0,
                                tint,
                                label,
                                selected,
                            )
                            .clicked()
                        } else {
                            ui.selectable_label(selected, body_text(&draw.palette, label).strong())
                                .clicked()
                        };
                        if clicked {
                            draw.placement.routing_mode = mode;
                            draw.placement.grid_snap =
                                mode == PowerLineRoutingMode::Orthogonal90;
                        }
                    }
                });

                ui.label(caption_text(&draw.palette, "Type"));
                ui.horizontal(|ui| {
                    for voltage in [
                        VoltageClass::Low,
                        VoltageClass::Medium,
                        VoltageClass::High,
                    ] {
                        let selected = active_voltage == voltage;
                        let tint = if selected { selected_tint } else { icon_tint };
                        let clicked = if let (Some(tex), Some(manifest)) = (texture_id, manifest) {
                            draw_power_hud_icon_labeled(
                                ui,
                                tex,
                                manifest,
                                PowerHudIconId::for_voltage(voltage),
                                16.0,
                                tint,
                                voltage_label(voltage),
                                selected,
                            )
                            .clicked()
                        } else {
                            ui.selectable_label(
                                selected,
                                body_text(&draw.palette, voltage_label(voltage)),
                            )
                            .clicked()
                        };
                        if clicked {
                            draw.placement.voltage = voltage;
                            draw.tool.tool = BuildTool::PowerLine(voltage);
                        }
                    }
                });

                ui.separator();
                ui.label(data_text(
                    &draw.palette,
                    &format!("Points: {}", draw.placement.control_points.len()),
                ));
                ui.label(data_text(
                    &draw.palette,
                    &format!("Valid: {valid_count}"),
                ));
                ui.label(data_text(
                    &draw.palette,
                    &format!("Est. cost: {est_cost}"),
                ));
                ui.label(caption_text(
                    &draw.palette,
                    "LMB add · RMB undo · Shift+LMB commit",
                ));
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            can_build,
                            egui::Button::new(
                                body_text(&draw.palette, "Build line")
                                    .color(draw.palette.accent_action),
                            ),
                        )
                        .clicked()
                    {
                        let voltage = draw.placement.voltage;
                        commit_power_line_to_utility_graph(
                            draw.placement.as_mut(),
                            &mut draw.snap_res.0,
                            draw.graph.as_mut(),
                            voltage,
                        );
                    }
                    if ui.button(body_text(&draw.palette, "Cancel")).clicked() {
                        draw.placement.clear_path();
                    }
                });
            });
        });
    Ok(())
}

#[must_use]
pub fn sim_power_tool_sheet_icons_wired() -> bool {
    crate::gui::hud::power_hud_icon_atlas::power_hud_atlas_assets_on_disk()
        && PowerHudIconId::for_voltage(VoltageClass::Medium) == PowerHudIconId::VoltageMedium
        && PowerHudIconId::for_routing_mode(PowerLineRoutingMode::Curved) == PowerHudIconId::RouteCurved
}
