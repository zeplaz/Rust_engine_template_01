//! Power line path input: LMB add, RMB undo, Shift+LMB commit, O cycle mode.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::gui::{MapCameraDesired, SimulationMapViewport};
use crate::infrastructure::utility::graph::{UtilityGraph, UtilityNetworkSnapshotResource};
use crate::infrastructure::UtilityAuthoringMode;
use crate::infrastructure::UtilityAuthoringTool;
use crate::infrastructure::UtilityNetworkKind;
use crate::render::PowerMapOverlayPresentation;
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool};
use crate::construction::path_feedback::ConstructionPathFeedback;
use crate::construction::roads::cursor_world_on_map;

use super::commit::commit_power_line_to_utility_graph;
use super::placement::{ActivePowerLinePlacement, PowerLineRoutingMode};
use super::routing::{regenerate_power_line_segments, snap_power_grid};

#[inline]
fn power_line_tool_active(tool: BuildTool) -> bool {
    matches!(tool, BuildTool::PowerLine(_))
}

fn resolve_voltage(tool: BuildTool) -> crate::infrastructure::VoltageClass {
    match tool {
        BuildTool::PowerLine(v) => v,
        _ => crate::infrastructure::VoltageClass::Medium,
    }
}

pub fn sync_power_line_from_build_tool(
    tool: Res<ActiveBuildTool>,
    mut placement: ResMut<ActivePowerLinePlacement>,
    mut authoring: ResMut<UtilityAuthoringTool>,
) {
    if !power_line_tool_active(tool.tool) {
        if authoring.mode == UtilityAuthoringMode::PlacePower {
            authoring.mode = UtilityAuthoringMode::Idle;
        }
        return;
    }
    placement.voltage = resolve_voltage(tool.tool);
    placement.grid_snap = placement.routing_mode == PowerLineRoutingMode::Orthogonal90;
    authoring.mode = UtilityAuthoringMode::PlacePower;
    authoring.active_kind = UtilityNetworkKind::Power;
}

pub fn power_line_routing_mode_hotkey_system(
    keys: Res<ButtonInput<KeyCode>>,
    tool: Res<ActiveBuildTool>,
    mut placement: ResMut<ActivePowerLinePlacement>,
) {
    if !power_line_tool_active(tool.tool) {
        return;
    }
    if keys.just_pressed(KeyCode::KeyO)
        || keys.just_pressed(KeyCode::BracketLeft)
        || keys.just_pressed(KeyCode::BracketRight)
    {
        placement.routing_mode = if keys.just_pressed(KeyCode::BracketLeft) {
            PowerLineRoutingMode::Curved
        } else if keys.just_pressed(KeyCode::BracketRight) {
            PowerLineRoutingMode::Orthogonal90
        } else {
            placement.routing_mode.cycle()
        };
        placement.grid_snap = placement.routing_mode == PowerLineRoutingMode::Orthogonal90;
        placement.generated_segments = regenerate_power_line_segments(
            &placement.control_points,
            None,
            placement.routing_mode,
            placement.grid_snap,
        );
    }
}

pub fn update_power_line_path_preview_system(
    tool: Res<ActiveBuildTool>,
    keys: Res<ButtonInput<KeyCode>>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    mut placement: ResMut<ActivePowerLinePlacement>,
    mut egui_ctx: EguiContexts,
) {
    if !power_line_tool_active(tool.tool) {
        placement.generated_segments.clear();
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
            if placement.routing_mode == PowerLineRoutingMode::Orthogonal90 && placement.grid_snap {
                snap_power_grid(w)
            } else {
                w
            }
        })
    } else {
        None
    };
    placement.generated_segments = regenerate_power_line_segments(
        &placement.control_points,
        cursor,
        placement.routing_mode,
        placement.grid_snap,
    );
}

pub fn sync_power_line_build_preview(
    tool: Res<ActiveBuildTool>,
    placement: Res<ActivePowerLinePlacement>,
    mut preview: ResMut<crate::construction::BuildPlacementPreview>,
    mut feedback: ResMut<ConstructionPathFeedback>,
) {
    if !power_line_tool_active(tool.tool) {
        return;
    }
    let valid_count = placement.generated_segments.iter().filter(|s| s.valid).count();
    let invalid = placement
        .generated_segments
        .iter()
        .any(|s| !s.valid && placement.generated_segments.len() > 1);
    preview.report = crate::strategic::SitePlacementValidation {
        allows_commit: valid_count > 0 && !invalid,
        errors: if invalid {
            vec!["invalid power line segment in path".into()]
        } else {
            Vec::new()
        },
        ..Default::default()
    };
    feedback.required_actions.clear();
    if invalid {
        feedback
            .required_actions
            .push("blocked: diagonal not allowed in 90° mode".into());
    }
}

pub fn sync_power_line_preview_overlay_system(
    tool: Res<ActiveBuildTool>,
    placement: Res<ActivePowerLinePlacement>,
    mut presentation: ResMut<PowerMapOverlayPresentation>,
) {
    presentation.preview_segments.clear();
    if !power_line_tool_active(tool.tool) {
        return;
    }
    for seg in &placement.generated_segments {
        if !seg.valid {
            continue;
        }
        presentation.preview_segments.push((
            Vec2::new(seg.start.x, seg.start.z),
            Vec2::new(seg.end.x, seg.end.z),
            placement.voltage,
        ));
    }
}

pub fn power_line_path_input_system(
    tool: Res<ActiveBuildTool>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    mut placement: ResMut<ActivePowerLinePlacement>,
    mut snap_res: ResMut<UtilityNetworkSnapshotResource>,
    mut graph: ResMut<UtilityGraph>,
    mut egui_ctx: EguiContexts,
) {
    if !power_line_tool_active(tool.tool) {
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
    let Some(raw) = cursor_world_on_map(
        &window,
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    ) else {
        return;
    };
    let world = if placement.routing_mode == PowerLineRoutingMode::Orthogonal90 && placement.grid_snap
    {
        snap_power_grid(raw)
    } else {
        raw
    };

    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
    let voltage = resolve_voltage(tool.tool);

    if buttons.just_pressed(MouseButton::Left) {
        if shift {
            commit_power_line_to_utility_graph(
                placement.as_mut(),
                &mut snap_res.0,
                graph.as_mut(),
                voltage,
            );
        } else {
            placement.control_points.push(world);
            placement.generated_segments = regenerate_power_line_segments(
                &placement.control_points,
                None,
                placement.routing_mode,
                placement.grid_snap,
            );
        }
    }

    if buttons.just_pressed(MouseButton::Right) {
        placement.control_points.pop();
        placement.generated_segments = regenerate_power_line_segments(
            &placement.control_points,
            None,
            placement.routing_mode,
            placement.grid_snap,
        );
    }
}
