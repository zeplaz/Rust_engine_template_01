//! Road tool popup — segment summary, Build / Cancel / snap / upgrade (P2-05 + P9).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui;

use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool, RailType, RoadType};
use crate::construction::construction_pipeline::{ConstructionPlanQueue, ExecutedRoadNetwork};
use crate::construction::sessions::ActiveToolSession;
use crate::construction::snap::RoadSnapSettings;
use crate::construction::upgrade::enqueue_road_upgrade;
use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::commit::commit_road_path_to_queue;
use super::input::cursor_world_on_map;
use super::placement::ActiveRoadPlacement;

#[derive(Resource, Default)]
pub struct RoadToolPopupState {
    pub cancel_requested: bool,
}

pub fn draw_road_tool_popup_egui(
    mut contexts: bevy_egui::EguiContexts,
    tool: Res<ActiveBuildTool>,
    mut placement: ResMut<ActiveRoadPlacement>,
    mut popup: ResMut<RoadToolPopupState>,
    mut queue: ResMut<ConstructionPlanQueue>,
    mut session: ResMut<ActiveToolSession>,
    mut snap: ResMut<RoadSnapSettings>,
    roads: Res<ExecutedRoadNetwork>,
    params: Res<WorldGenParams>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
) -> Result {
    popup.cancel_requested = false;
    let title = match tool.tool {
        BuildTool::Road(road_type) => {
            let name = match road_type {
                RoadType::Street => "Street",
                RoadType::Highway => "Highway",
            };
            format!("Road — {name}")
        }
        BuildTool::Rail(RailType::Standard) => "Rail — Standard".to_string(),
        _ => return Ok(()),
    };

    let valid_count = placement
        .generated_segments
        .iter()
        .filter(|s| s.valid)
        .count();
    let est_cost = valid_count.saturating_mul(10);
    let can_build = valid_count > 0;

    let cursor_world = win
        .single()
        .ok()
        .and_then(|window| {
            cursor_world_on_map(
                window,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            )
        });

    egui::Window::new(title)
        .id(egui::Id::new("road_tool_popup"))
        .default_pos(egui::pos2(12.0, 200.0))
        .default_width(240.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label(format!("Control points: {}", placement.control_points.len()));
            ui.label(format!("Segments: {}", placement.generated_segments.len()));
            ui.label(format!("Valid segments: {valid_count}"));
            ui.label(format!("Estimated cost: {est_cost}"));
            ui.label(format!("Width: {:.1}", placement.width));
            ui.separator();
            ui.checkbox(&mut snap.grid_snap, "Grid snap");
            ui.checkbox(&mut snap.node_snap, "Node snap");
            ui.checkbox(&mut placement.use_curved_preview, "Curved preview");
            ui.separator();
            ui.label(egui::RichText::new("LMB add · RMB undo · Shift+LMB commit").small());
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(can_build, egui::Button::new("Build"))
                    .clicked()
                {
                    commit_road_path_to_queue(
                        placement.as_mut(),
                        queue.as_mut(),
                        params.as_ref(),
                        session.continuous_path,
                    );
                    session.record_commit();
                }
                if ui.button("Cancel").clicked() {
                    placement.control_points.clear();
                    placement.generated_segments.clear();
                    popup.cancel_requested = true;
                }
            });
            if ui.button("Upgrade nearest segment").clicked() {
                if let Some(world) = cursor_world {
                    enqueue_road_upgrade(
                        world,
                        roads.as_ref(),
                        queue.as_mut(),
                        placement.as_mut(),
                        params.as_ref(),
                    );
                }
            }
        });
    Ok(())
}
