//! Rail path input (parallel to roads — separate placement resource).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::gui::hud::simulation_pointer_gate::{
    simulation_tool_pointer, SimulationMapPointerGate, SimulationToolPointer,
};

use crate::gui::{MapCameraDesiredRes, SimulationMapViewport};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::terrain::generation::WorldGenDenseTerrainCache;
use crate::construction::map_pick_params::ConstructionMapPick;

use super::super::build_tool_authority::{ActiveBuildTool, BuildTool};
use super::super::construction_pipeline::ConstructionPlanQueue;
use super::super::path_feedback::ConstructionPathFeedback;
use super::super::roads::cursor_world_on_map;
use super::super::sessions::ActiveToolSession;
use super::super::snap::{snap_placement, RoadSnapSettings, SnapTarget};
use super::super::terrain_conform::conform_vec3;
use super::commit::commit_rail_path_to_queue;
use super::pathing::regenerate_rail_segments;
use super::pathing::world_xy_to_tile;
use super::placement::ActiveRailPlacement;
use super::validation::validate_rail_segment;

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
    pick: ConstructionMapPick,
    snap: Res<RoadSnapSettings>,
    roads: Res<super::super::construction_pipeline::ExecutedRoadNetwork>,
    mut placement: ResMut<ActiveRailPlacement>,
    mut feedback: ResMut<ConstructionPathFeedback>,
    pointer_gate: Option<Res<SimulationMapPointerGate>>,
) {
    if !matches!(tool.tool, BuildTool::Rail(_)) {
        placement.generated_segments.clear();
        feedback.snap_hint = None;
        feedback.required_actions.clear();
        return;
    }
    match simulation_tool_pointer(pointer_gate.as_deref()) {
        SimulationToolPointer::Unavailable => return,
        SimulationToolPointer::HeldByChrome => {
            placement.generated_segments.clear();
            return;
        }
        SimulationToolPointer::Free => {}
    }
    let ctrl_preview = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    let cursor = if ctrl_preview {
        pick.cursor_world().map(|w| {
            let placed = snap_placement(w, snap.as_ref(), roads.as_ref());
            feedback.snap_hint = placed.target.as_ref().map(SnapTarget::hint_label);
            conform_vec3(placed.world, pick.params.as_ref(), pick.height_grid())
        })
    } else {
        None
    };
    placement.generated_segments =
        regenerate_rail_segments(placement.as_ref(), cursor, pick.params.as_ref());
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
    let valid_count = placement
        .generated_segments
        .iter()
        .filter(|s| s.valid)
        .count();
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
                validate_rail_segment(
                    head,
                    tail,
                    seg.start.y,
                    seg.end.y,
                    max_slope,
                    params.as_ref(),
                )
                .required_actions
                .into_iter(),
            );
        }
    }
    feedback.required_actions = actions.clone();
    let ok = valid_count > 0 && !invalid;
    preview.report = crate::strategic::SitePlacementValidation {
        valid: ok,
        allows_commit: ok,
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
    pick: ConstructionMapPick,
    snap: Res<RoadSnapSettings>,
    roads: Res<super::super::construction_pipeline::ExecutedRoadNetwork>,
    mut placement: ResMut<ActiveRailPlacement>,
    mut queue: ResMut<ConstructionPlanQueue>,
    mut session: ResMut<ActiveToolSession>,
    mut junctions: ResMut<super::junction::RailJunctionAuthority>,
    mut feedback: ResMut<ConstructionPathFeedback>,
    pointer_gate: Option<Res<SimulationMapPointerGate>>,
) {
    if !matches!(tool.tool, BuildTool::Rail(_)) {
        return;
    }
    if simulation_tool_pointer(pointer_gate.as_deref()) != SimulationToolPointer::Free {
        return;
    }
    let Some(raw) = pick.cursor_world() else {
        return;
    };
    let placed = snap_placement(raw, snap.as_ref(), roads.as_ref());
    feedback.snap_hint = placed.target.as_ref().map(SnapTarget::hint_label);
    let world = conform_vec3(placed.world, pick.params.as_ref(), pick.height_grid());
    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    if buttons.just_pressed(MouseButton::Left) {
        if shift {
            if crate::construction::path_commit_leaves_generated_map(
                &placement.control_points,
                placement
                    .generated_segments
                    .iter()
                    .flat_map(|seg| [seg.start, seg.end]),
                pick.params.width,
                pick.params.height,
            ) {
                return;
            }
            commit_rail_path_to_queue(
                placement.as_mut(),
                queue.as_mut(),
                junctions.as_mut(),
                roads.as_ref(),
                pick.params.as_ref(),
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
            regenerate_rail_segments(placement.as_ref(), None, pick.params.as_ref());
    }
}
