//! **COD-SIM-HUD-POPUP-MIGRATE-001** — rail-anchored road tool sheet (sim).

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui;

use crate::construction::{
    commit_road_path_to_queue, cursor_world_on_map, draw_road_tool_popup_egui,
    enqueue_road_upgrade, ActiveBuildTool, ActiveRoadPlacement, ActiveToolSession,
    BuildStripState, BuildTool, ConstructionPlanQueue, ExecutedRoadNetwork, RailType, RoadType,
    RoadSnapSettings, RoadToolPopupState, ToolContext,
};
use crate::engine::states::BaseState;
use crate::gui::hud::sim_build_picker_sheet::{
    build_rail_slot_anchor_y, BUILD_PICKER_RAIL_GAP_PX,
};
use crate::gui::hud::simulation_shell_phase2::{
    BUILD_RAIL_W_PX, COMMAND_LEFT_STACK_COLUMN_GAP_PX, CONTEXT_RAIL_W_PX,
};
use crate::gui::{MapCameraDesired, SimulationMapViewport, UiPalette};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::sim_hud_copy::{ROAD_SHEET_BUILD, ROAD_SHEET_CANCEL, ROAD_SHEET_HINT_INPUT, ROAD_SHEET_UPGRADE};
use super::sim_hud_egui_theme::{
    apply_sim_hud_egui_theme, body_text, caption_text, data_text, picker_header_frame,
    picker_sheet_frame, title_text,
};

pub const ROAD_TOOL_SHEET_W_PX: f32 = 280.0;

/// Sim uses rail-anchored sheet; editor keeps legacy floating window.
pub const ROAD_POPUP_FLOATING_IN_SIM: bool = false;

#[derive(Resource, Debug, Clone, Default)]
pub struct SimRoadToolSheetState {
    pub open: bool,
}

impl SimRoadToolSheetState {
    pub fn sync_from_strip(&mut self, strip: &BuildStripState) {
        self.open = matches!(strip.active, ToolContext::Roads | ToolContext::Rail);
    }

    pub fn close(&mut self) {
        self.open = false;
    }
}

#[derive(SystemParam)]
pub struct SimRoadToolSheetDrawParams<'w> {
    pub strip: Res<'w, BuildStripState>,
    pub palette: Res<'w, UiPalette>,
    pub tool: Res<'w, ActiveBuildTool>,
    pub road_sheet: ResMut<'w, SimRoadToolSheetState>,
    pub placement: ResMut<'w, ActiveRoadPlacement>,
    pub popup: ResMut<'w, RoadToolPopupState>,
    pub queue: ResMut<'w, ConstructionPlanQueue>,
    pub session: ResMut<'w, ActiveToolSession>,
    pub snap: ResMut<'w, RoadSnapSettings>,
    pub roads: Res<'w, ExecutedRoadNetwork>,
    pub params: Res<'w, WorldGenParams>,
    pub desired: Res<'w, MapCameraDesired>,
    pub map_vp: Res<'w, SimulationMapViewport>,
}

#[must_use]
pub fn road_tool_sheet_anchor(strip: &BuildStripState) -> egui::Pos2 {
    let slot = match strip.active {
        ToolContext::Rail => ToolContext::Rail,
        _ => ToolContext::Roads,
    };
    let x = CONTEXT_RAIL_W_PX
        + COMMAND_LEFT_STACK_COLUMN_GAP_PX
        + BUILD_RAIL_W_PX
        + BUILD_PICKER_RAIL_GAP_PX;
    egui::pos2(x, build_rail_slot_anchor_y(slot))
}

pub fn draw_sim_road_tool_sheet_egui(
    mut contexts: bevy_egui::EguiContexts,
    base: Res<State<BaseState>>,
    mut draw: SimRoadToolSheetDrawParams,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<ViewProjectionAuthority>>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) {
        return draw_road_tool_popup_egui(
            contexts,
            draw.tool,
            draw.placement,
            draw.popup,
            draw.queue,
            draw.session,
            draw.snap,
            draw.roads,
            draw.params,
            win,
            authority,
            draw.desired,
            draw.map_vp,
        );
    }

    draw.road_sheet.sync_from_strip(draw.strip.as_ref());
    if !draw.road_sheet.open {
        return Ok(());
    }
    if !matches!(draw.tool.tool, BuildTool::Road(_) | BuildTool::Rail(_)) {
        return Ok(());
    }

    draw.popup.cancel_requested = false;
    let sheet_title = match draw.tool.tool {
        BuildTool::Road(RoadType::Street) => "Road — Street",
        BuildTool::Road(RoadType::Highway) => "Road — Highway",
        BuildTool::Rail(RailType::Standard) => "Rail — Standard",
        _ => return Ok(()),
    };

    let valid_count = draw
        .placement
        .generated_segments
        .iter()
        .filter(|s| s.valid)
        .count();
    let est_cost = valid_count.saturating_mul(10);
    let can_build = valid_count > 0;

    let cursor_world = win.single().ok().and_then(|window| {
        cursor_world_on_map(
            window,
            authority.as_deref(),
            draw.desired.as_ref(),
            draw.map_vp.as_ref(),
            draw.params.as_ref(),
        )
    });

    let ctx = contexts.ctx_mut()?;
    apply_sim_hud_egui_theme(ctx, &draw.palette);
    let anchor = road_tool_sheet_anchor(draw.strip.as_ref());

    egui::Area::new(egui::Id::new("sim_road_tool_sheet"))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor)
        .show(ctx, |ui| {
            ui.set_width(ROAD_TOOL_SHEET_W_PX);
            picker_sheet_frame(&draw.palette).show(ui, |ui| {
                picker_header_frame(&draw.palette).show(ui, |ui| {
                    ui.label(title_text(&draw.palette, sheet_title));
                });
                ui.label(data_text(
                    &draw.palette,
                    &format!("Control points: {}", draw.placement.control_points.len()),
                ));
                ui.label(data_text(
                    &draw.palette,
                    &format!("Valid segments: {valid_count}"),
                ));
                ui.label(data_text(
                    &draw.palette,
                    &format!("Estimated cost: {est_cost}"),
                ));
                ui.label(data_text(
                    &draw.palette,
                    &format!("Width: {:.1}", draw.placement.width),
                ));
                ui.separator();
                ui.checkbox(&mut draw.snap.grid_snap, "Grid snap");
                ui.checkbox(&mut draw.snap.node_snap, "Node snap");
                ui.checkbox(&mut draw.placement.use_curved_preview, "Curved preview");
                ui.separator();
                ui.label(caption_text(&draw.palette, ROAD_SHEET_HINT_INPUT));
                ui.horizontal(|ui| {
                    if ui
                        .add_enabled(
                            can_build,
                            egui::Button::new(
                                body_text(&draw.palette, ROAD_SHEET_BUILD)
                                    .color(draw.palette.accent_action),
                            ),
                        )
                        .clicked()
                    {
                        commit_road_path_to_queue(
                            draw.placement.as_mut(),
                            draw.queue.as_mut(),
                            draw.params.as_ref(),
                            draw.session.continuous_path,
                        );
                        draw.session.record_commit();
                    }
                    if ui.button(body_text(&draw.palette, ROAD_SHEET_CANCEL)).clicked() {
                        draw.placement.control_points.clear();
                        draw.placement.generated_segments.clear();
                        draw.popup.cancel_requested = true;
                    }
                });
                if ui.button(body_text(&draw.palette, ROAD_SHEET_UPGRADE)).clicked() {
                    if let Some(world) = cursor_world {
                        enqueue_road_upgrade(
                            world,
                            draw.roads.as_ref(),
                            draw.queue.as_mut(),
                            draw.placement.as_mut(),
                            draw.params.as_ref(),
                        );
                    }
                }
            });
        });
    Ok(())
}
