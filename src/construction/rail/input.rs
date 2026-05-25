//! Rail path input (parallel to roads — separate placement resource).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::gui::{MapCameraDesired, SimulationMapViewport};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::commit::commit_rail_path_to_queue;
use super::pathing::regenerate_rail_segments;
use super::placement::ActiveRailPlacement;
use super::super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::super::construction_pipeline::ConstructionPlanQueue;
use super::super::roads::cursor_world_on_map;
use super::super::sessions::ActiveToolSession;
use super::super::path_feedback::ConstructionPathFeedback;
use super::super::snap::{snap_placement, RoadSnapSettings, SnapTarget};
use super::pathing::world_xy_to_tile;
use super::validation::validate_rail_segment;
use super::super::terrain_conform::conform_vec3;

pub fn sync_rail_placement_from_tool(
    tool: Res<ActiveBuildTool>,
    mut placement: ResMut<ActiveRailPlacement>,
) {
    if matches!(tool.tool, BuildTool::Rail(_)) {
        placement.width = 6.0;
    }
}

pub fn update_rail_path_preview_system(
    tool: Res<ActiveBuildTool>,
    keys: Res<ButtonInput<KeyCode>>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    snap: Res<RoadSnapSettings>,
    roads: Res<super::super::construction_pipeline::ExecutedRoadNetwork>,
    mut placement: ResMut<ActiveRailPlacement>,
    mut feedback: ResMut<ConstructionPathFeedback>,
    mut egui_ctx: EguiContexts,
) {
    if !matches!(tool.tool, BuildTool::Rail(_)) {
        placement.generated_segments.clear();
        feedback.snap_hint = None;
        feedback.required_actions.clear();
        return;
    }
    let Ok(ctx) = egui_ctx.ctx_mut() else {
        return;
    };
    if ctx.wants_pointer_input() {
        placement.generated_segments.clear();
        return;
    }
    let Ok(window) = win.single() else {
        return;
    };
    let ctrl_preview = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let cursor = if ctrl_preview {
        cursor_world_on_map(
            &window,
            authority.as_deref(),
            desired.as_ref(),
            map_vp.as_ref(),
            params.as_ref(),
        )
        .map(|w| {
        let placed = snap_placement(w, snap.as_ref(), roads.as_ref());
        feedback.snap_hint = placed.target.as_ref().map(SnapTarget::hint_label);
        conform_vec3(placed.world, params.as_ref())
        })
    } else {
        None
    };
    placement.generated_segments = regenerate_rail_segments(placement.as_ref(), cursor, params.as_ref());
}

pub fn sync_rail_path_build_preview(
    tool: Res<ActiveBuildTool>,
    placement: Res<ActiveRailPlacement>,
    params: Res<WorldGenParams>,
    mut preview: ResMut<super::super::build_state::BuildPlacementPreview>,
    mut feedback: ResMut<ConstructionPathFeedback>,
) {
    if !matches!(tool.tool, BuildTool::Rail(_)) {
        feedback.required_actions.clear();
        return;
    }
    let max_slope = placement.max_slope;
    let valid_count = placement.generated_segments.iter().filter(|s| s.valid).count();
    let invalid = placement
        .generated_segments
        .iter()
        .any(|s| !s.valid && placement.generated_segments.len() > 1);
    let mut actions = Vec::new();
    if invalid {
        for seg in placement.generated_segments.iter().filter(|s| !s.valid) {
            let head = world_xy_to_tile(seg.start);
            let tail = world_xy_to_tile(seg.end);
            actions.extend(
                validate_rail_segment(head, tail, seg.start.y, seg.end.y, max_slope, params.as_ref())
                    .required_actions
                    .into_iter(),
            );
        }
    }
    feedback.required_actions = actions.clone();
    preview.report = crate::strategic::SitePlacementValidation {
        allows_commit: valid_count > 0 && !invalid,
        errors: if invalid {
            vec!["invalid rail segment (grade or curve)".into()]
        } else {
            Vec::new()
        },
        ..Default::default()
    };
}

pub fn rail_path_input_system(
    tool: Res<ActiveBuildTool>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    snap: Res<RoadSnapSettings>,
    roads: Res<super::super::construction_pipeline::ExecutedRoadNetwork>,
    mut placement: ResMut<ActiveRailPlacement>,
    mut queue: ResMut<ConstructionPlanQueue>,
    mut session: ResMut<ActiveToolSession>,
    mut junctions: ResMut<super::junction::RailJunctionAuthority>,
    mut feedback: ResMut<ConstructionPathFeedback>,
    mut egui_ctx: EguiContexts,
) {
    if !matches!(tool.tool, BuildTool::Rail(_)) {
        return;
    }
    let Ok(ctx) = egui_ctx.ctx_mut() else {
        return;
    };
    if ctx.wants_pointer_input() {
        return;
    }
    let Ok(window) = win.single() else {
        return;
    };
    let Some(raw) =
        cursor_world_on_map(
            &window,
            authority.as_deref(),
            desired.as_ref(),
            map_vp.as_ref(),
            params.as_ref(),
        )
    else {
        return;
    };
    let placed = snap_placement(raw, snap.as_ref(), roads.as_ref());
    feedback.snap_hint = placed.target.as_ref().map(SnapTarget::hint_label);
    let world = conform_vec3(placed.world, params.as_ref());
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if buttons.just_pressed(MouseButton::Left) {
        if shift {
            commit_rail_path_to_queue(
                placement.as_mut(),
                queue.as_mut(),
                junctions.as_mut(),
                roads.as_ref(),
                params.as_ref(),
                session.continuous_path,
            );
            session.record_commit();
        } else {
            placement.control_points.push(world);
        }
    }
    if buttons.just_pressed(MouseButton::Right) {
        placement.control_points.pop();
        placement.generated_segments =
            regenerate_rail_segments(placement.as_ref(), None, params.as_ref());
    }
}
