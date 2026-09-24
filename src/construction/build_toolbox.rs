//! Movable construction toolbox (floating product-shell window — not left-anchored).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::gui::InputBindings;
use crate::gui::hud::{
    capture_shell_layout, draw_shell_window_chrome, floating_unanchored_default_pos,
    shell_widget_runs_egui_with_budget, HudDockRegistry, HudLayoutStore, HudWidgetId,
    PendingHudLayoutCommit, ProductShellUpdateBudget,
};
use crate::gui::std_floating;

use super::build_state::{BuildGhostState, BuildPlacementPreview};
use super::build_tool_authority::{
    ActiveBuildTool, BuildTool, DefenseKind, RailType, RoadType, ZoneTool,
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
        if tool.tool.uses_two_click_place() {
            let in_adjust = ghost.placement_mode
                == crate::construction::BuildPlacementMode::Adjust
                && ghost.origin.is_some();
            let can_place = in_adjust && preview.report.allows_commit;
            let row = match tool.tool {
                BuildTool::Defense(kind) => kind.player_label(),
                _ => "building",
            };
            let place_label = if !in_adjust {
                format!("Lock {row} — click map")
            } else if can_place {
                format!("Place {row} — click map again")
            } else {
                "Cannot place — fix validation".to_string()
            };
            ui.add_enabled_ui(can_place || !in_adjust, |ui| {
                let _ = ui.button(place_label);
            });
            ui.label(
                egui::RichText::new(format!(
                    "Shortcut: {}",
                    InputBindings::format_key(bindings.confirm_build_placement)
                ))
                .small()
                .weak(),
            );
            if !in_adjust {
                let hint = if matches!(tool.tool, BuildTool::Defense(_)) {
                    "Pick a defense row in the Military picker, then click the map to lock."
                } else {
                    "Pick a catalog building below, then click the map to lock."
                };
                ui.label(egui::RichText::new(hint).small().weak());
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
        ui.collapsing("Defense (Military)", |ui| {
            for (kind, label) in [
                (DefenseKind::DefensiveWall, "Defensive wall"),
                (DefenseKind::TrenchLine, "Trench line"),
                (DefenseKind::Bunker, "Bunker"),
                (DefenseKind::DragonTeeth, "Dragon's teeth"),
                (DefenseKind::Minefield, "Minefield"),
            ] {
                if ui.button(label).clicked() {
                    tool.close_submenus();
                    tool.clear_building_intent();
                    tool.tool = BuildTool::Defense(kind);
                }
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
            draw_utilities_submenu(ui, &mut tool, &registry, None);
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
