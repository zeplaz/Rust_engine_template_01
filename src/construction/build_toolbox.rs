//! Movable construction toolbox (floating product-shell window — not left-anchored).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::engine::states::BaseState;
use crate::gui::InputBindings;
use crate::gui::hud::{
    capture_shell_layout, draw_shell_window_chrome, floating_unanchored_default_pos,
    shell_widget_runs_egui_with_budget, HudDockRegistry, HudLayoutStore, HudWidgetId,
    PendingHudLayoutCommit, ProductShellUpdateBudget,
};
use crate::gui::std_floating;

use super::build_state::{BuildGhostState, BuildPlacementPreview};
use super::build_strip::{BuildStripState, ToolContext};
use super::build_tool_authority::{
    ActiveBuildTool, BuildTool, RailType, RoadType, ZoneTool,
};
use super::commercial_menu::draw_commercial_submenu;
use super::industrial_menu::draw_industrial_submenu;
use super::building_definitions::BuildingDefinitionRegistry;
use super::residential_menu::{draw_intent_preview, draw_residential_submenu};
use super::mock_shapes_menu::draw_mock_shapes_submenu;
use super::utilities_menu::draw_utilities_submenu;
use crate::strategic::CorridorConstructionBook;

pub fn draw_build_toolbox_egui(
    mut contexts: bevy_egui::EguiContexts,
    mut tool: ResMut<ActiveBuildTool>,
    registry: Res<BuildingDefinitionRegistry>,
    corridor_book: Option<Res<CorridorConstructionBook>>,
    mut dock: ResMut<HudDockRegistry>,
    mut layout_store: ResMut<HudLayoutStore>,
    mut update_budget: ResMut<ProductShellUpdateBudget>,
    mut pending_layout: ResMut<PendingHudLayoutCommit>,
    ghost: Res<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    bindings: Res<InputBindings>,
    time: Res<Time>,
) -> Result {
    let ctx = contexts.ctx_mut()?;
    let now_secs = time.elapsed_secs();
    let mut open = dock.slot(HudWidgetId::BuildToolbox).visible;
    if !shell_widget_runs_egui_with_budget(
        &dock,
        HudWidgetId::BuildToolbox,
        open,
        Some(&mut *update_budget),
        now_secs,
    ) {
        return Ok(());
    }

    let default_size = [240.0, 420.0];
    let default_pos = floating_unanchored_default_pos(ctx, HudWidgetId::BuildToolbox, default_size);
    let frame = layout_store.frame(HudWidgetId::BuildToolbox);
    let window = if frame.initialized {
        std_floating(egui::Window::new("Construction"))
            .id(HudWidgetId::BuildToolbox.egui_window_id())
            .default_pos(egui::pos2(frame.pos.x, frame.pos.y))
            .default_size([frame.size.x, frame.size.y])
            .min_size([180.0, 240.0])
            .resizable(true)
    } else {
        std_floating(egui::Window::new("Construction"))
            .id(HudWidgetId::BuildToolbox.egui_window_id())
            .default_pos(default_pos)
            .default_size(default_size)
            .min_size([180.0, 240.0])
            .resizable(true)
    };

    let mut minimized = dock.slot(HudWidgetId::BuildToolbox).minimized;
    let mut detached = dock.slot(HudWidgetId::BuildToolbox).detached;
    if let Some(inner) = window.open(&mut open).show(ctx, |ui| {
        let response = ui.response();
        let lightweight = response.dragged() || response.drag_started();
        draw_shell_window_chrome(ui, &mut minimized, &mut detached, lightweight);
        ui.label(
            egui::RichText::new("Drag title bar to move · drag edges to resize")
                .small()
                .weak(),
        );
        ui.separator();
        if matches!(tool.tool, BuildTool::Building(_)) {
            let can_place = ghost.origin.is_some() && preview.report.allows_commit;
            let place_label = format!(
                "Place on map ({})",
                InputBindings::format_key(bindings.confirm_build_placement)
            );
            ui.add_enabled_ui(can_place, |ui| {
                let _ = ui.button(place_label);
            });
            if !can_place {
                ui.label(
                    egui::RichText::new("Pick a catalog building below, then LMB on the map.")
                        .small()
                        .weak(),
                );
            }
            ui.separator();
        }
        ui.collapsing("Zoning", |ui| {
            if ui.button("Residential…").clicked() {
                tool.close_submenus();
                tool.residential_menu_open = true;
                tool.tool = BuildTool::Zone(ZoneTool::ResidentialLow);
            }
        });
        ui.collapsing("Buildings", |ui| {
            if ui.button("Commercial…").clicked() {
                tool.close_submenus();
                tool.commercial_menu_open = true;
                tool.clear_building_intent();
                tool.tool = BuildTool::Building(super::build_tool_authority::BuildingArchetypeId::Office);
            }
            if ui.button("Industrial…").clicked() {
                tool.close_submenus();
                tool.industrial_menu_open = true;
                tool.clear_building_intent();
                tool.tool = BuildTool::Building(super::build_tool_authority::BuildingArchetypeId::Factory);
            }
            if ui.button("Utilities…").clicked() {
                tool.close_submenus();
                tool.utilities_menu_open = true;
                tool.clear_building_intent();
                tool.tool =
                    BuildTool::Building(super::build_tool_authority::BuildingArchetypeId::WaterPlant);
            }
            if ui.button("Mock shapes (T/O/L)…").clicked() {
                tool.close_submenus();
                tool.mock_shapes_menu_open = true;
                tool.clear_building_intent();
                tool.tool = BuildTool::Building(super::build_tool_authority::BuildingArchetypeId::Factory);
            }
        });
        ui.collapsing("Infrastructure", |ui| {
            if ui.button("Roads").clicked() {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Road(RoadType::Street);
            }
            if ui.button("Rail").clicked() {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Rail(RailType::Standard);
            }
        });
        ui.collapsing("Editing", |ui| {
            if ui.button("Demolish").clicked() {
                tool.close_submenus();
                tool.clear_building_intent();
                tool.tool = BuildTool::Demolish;
            }
            if ui.button("Clear tool").clicked() {
                tool.tool = BuildTool::None;
                tool.close_submenus();
                tool.clear_building_intent();
            }
        });
        if tool.residential_menu_open {
            ui.separator();
            draw_residential_submenu(ui, &mut tool, &registry);
        }
        if tool.commercial_menu_open {
            ui.separator();
            draw_commercial_submenu(ui, &mut tool, &registry);
        }
        if tool.industrial_menu_open {
            ui.separator();
            draw_industrial_submenu(ui, &mut tool, &registry);
        }
        if tool.utilities_menu_open {
            ui.separator();
            draw_utilities_submenu(ui, &mut tool, &registry);
        }
        if tool.mock_shapes_menu_open {
            ui.separator();
            draw_mock_shapes_submenu(ui, &mut tool, &registry);
        }
        if let Some(intent) = tool.building_intent.as_ref() {
            ui.separator();
            draw_intent_preview(ui, intent);
        }
        if let Some(book) = corridor_book.as_ref() {
            super::round4_corridor::draw_r4_corridor_tray_legend(ui, &tool, book);
        }
    }) {
        if pending_layout.can_emit_layout_capture() {
            capture_shell_layout(
                &mut layout_store,
                HudWidgetId::BuildToolbox,
                &inner.response,
                Some(&mut *pending_layout),
            );
        }
        if inner.response.hovered() || inner.response.has_focus() {
            dock.focus(HudWidgetId::BuildToolbox);
        }
    }

    let slot = dock.slot_mut(HudWidgetId::BuildToolbox);
    slot.minimized = minimized;
    slot.detached = detached;
    Ok(())
}

/// Simulation-only build rail submenu panel (does not touch product-shell egui pass counter).
pub fn draw_sim_build_rail_submenus_egui(
    mut contexts: bevy_egui::EguiContexts,
    mut tool: ResMut<ActiveBuildTool>,
    strip: Res<BuildStripState>,
    registry: Res<BuildingDefinitionRegistry>,
    base: Res<State<BaseState>>,
) -> Result {
    if !matches!(*base.get(), BaseState::Simulation) {
        return Ok(());
    }
    if strip.active == ToolContext::None {
        return Ok(());
    }
    if !tool.residential_menu_open
        && !tool.commercial_menu_open
        && !tool.industrial_menu_open
        && !tool.utilities_menu_open
        && !tool.mock_shapes_menu_open
    {
        return Ok(());
    }

    use crate::gui::hud::simulation_shell_phase2::{
        BUILD_RAIL_W_PX, COMMAND_LEFT_STACK_COLUMN_GAP_PX, CONTEXT_RAIL_W_PX,
    };

    let ctx = contexts.ctx_mut()?;
    let anchor_x = CONTEXT_RAIL_W_PX + COMMAND_LEFT_STACK_COLUMN_GAP_PX + BUILD_RAIL_W_PX + 8.0;
    let anchor_y = 96.0;

    egui::Area::new(egui::Id::new("sim_build_rail_submenus"))
        .fixed_pos(egui::pos2(anchor_x, anchor_y))
        .show(ctx, |ui| {
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_min_width(220.0);
                ui.label(egui::RichText::new(strip.active.label()).strong());
                ui.separator();
                if tool.residential_menu_open {
                    draw_residential_submenu(ui, &mut tool, &registry);
                }
                if tool.commercial_menu_open {
                    draw_commercial_submenu(ui, &mut tool, &registry);
                }
                if tool.industrial_menu_open {
                    draw_industrial_submenu(ui, &mut tool, &registry);
                }
                if tool.utilities_menu_open {
                    draw_utilities_submenu(ui, &mut tool, &registry);
                }
                if tool.mock_shapes_menu_open {
                    draw_mock_shapes_submenu(ui, &mut tool, &registry);
                }
                if let Some(intent) = tool.building_intent.as_ref() {
                    ui.separator();
                    draw_intent_preview(ui, intent);
                }
            });
        });
    Ok(())
}
