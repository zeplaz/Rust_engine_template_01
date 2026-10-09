//! Road path input: LMB add, RMB undo last, Shift+LMB commit (P2-02).

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::construction::map_egui_projection::ConstructionMapProjection;
use crate::gui::hud::simulation_pointer_gate::{
    simulation_tool_pointer, SimulationMapPointerGate, SimulationToolPointer,
};
use crate::gui::{MapCameraDesired, MapCameraDesiredRes, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::pathing::world_xy_to_tile;
use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool, RailType, RoadType};
use crate::construction::construction_pipeline::validate_road_segment;
use crate::construction::construction_pipeline::ConstructionPlanQueue;
use crate::construction::path_feedback::ConstructionPathFeedback;
use crate::construction::sessions::ActiveToolSession;
use crate::construction::snap::{snap_placement, RoadSnapSettings, SnapTarget};
use crate::construction::terrain_conform::conform_vec3;

use super::commit::commit_road_path_to_queue;
use super::pathing::regenerate_road_segments;
use super::placement::ActiveRoadPlacement;

#[inline]
fn path_tool_active(tool: BuildTool) -> bool {
    matches!(tool, BuildTool::Road(_))
}

#[must_use]
pub fn cursor_world_on_map(
    window: &Window,
    authority: Option<&ViewProjectionAuthority>,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    params: &WorldGenParams,
) -> Option<Vec3> {
    let cursor_px = window.cursor_position()?;
    let proj = ConstructionMapProjection::resolve(authority, desired, map_vp, params);
    let xy = proj.cursor_world_xy(cursor_px)?;
    Some(Vec3::new(xy.x, 0.0, xy.y))
}

fn resolve_placement_world(
    raw: Vec3,
    snap: &RoadSnapSettings,
    roads: &crate::construction::construction_pipeline::ExecutedRoadNetwork,
    params: &WorldGenParams,
    feedback: &mut ConstructionPathFeedback,
) -> Vec3 {
    let placed = snap_placement(raw, snap, roads);
    feedback.snap_hint = placed.target.as_ref().map(SnapTarget::hint_label);
    conform_vec3(placed.world, params)
}

pub fn sync_road_placement_width_from_tool(
    tool: Res<ActiveBuildTool>,
    mut placement: ResMut<ActiveRoadPlacement>,
) {
    placement.width = match tool.tool {
        BuildTool::Road(RoadType::Street) => 8.0,
        BuildTool::Road(RoadType::Highway) => 14.0,
        BuildTool::Rail(RailType::Standard) => 6.0,
        _ => return,
    };
}

pub fn update_road_path_preview_system(
    tool: Res<ActiveBuildTool>,
    keys: Res<ButtonInput<KeyCode>>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    snap: Res<RoadSnapSettings>,
    roads: Res<crate::construction::construction_pipeline::ExecutedRoadNetwork>,
    mut placement: ResMut<ActiveRoadPlacement>,
    mut feedback: ResMut<ConstructionPathFeedback>,
    pointer_gate: Option<Res<SimulationMapPointerGate>>,
) {
    if !path_tool_active(tool.tool) {
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
            resolve_placement_world(
                w,
                snap.as_ref(),
                roads.as_ref(),
                params.as_ref(),
                feedback.as_mut(),
            )
        })
    } else {
        None
    };
    placement.generated_segments = regenerate_road_segments(
        &placement.control_points,
        cursor,
        placement.width,
        params.as_ref(),
        placement.use_curved_preview,
    );
}

pub fn sync_road_path_build_preview(
    tool: Res<ActiveBuildTool>,
    placement: Res<ActiveRoadPlacement>,
    params: Res<WorldGenParams>,
    mut preview: ResMut<crate::construction::BuildPlacementPreview>,
    mut feedback: ResMut<ConstructionPathFeedback>,
) {
    if !path_tool_active(tool.tool) {
        feedback.required_actions.clear();
        return;
    }
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
                validate_road_segment(head, tail, params.as_ref())
                    .required_actions
                    .into_iter(),
            );
        }
    }
    feedback.required_actions = actions.clone();
    preview.report = crate::strategic::SitePlacementValidation {
        allows_commit: valid_count > 0 && !invalid,
        errors: if invalid {
            vec!["invalid road segment in path".into()]
        } else {
            Vec::new()
        },
        ..Default::default()
    };
}

pub fn road_path_input_system(
    tool: Res<ActiveBuildTool>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    win: Query<&Window, With<PrimaryWindow>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    snap: Res<RoadSnapSettings>,
    roads: Res<crate::construction::construction_pipeline::ExecutedRoadNetwork>,
    mut placement: ResMut<ActiveRoadPlacement>,
    mut queue: ResMut<ConstructionPlanQueue>,
    mut session: ResMut<ActiveToolSession>,
    mut feedback: ResMut<ConstructionPathFeedback>,
    pointer_gate: Option<Res<SimulationMapPointerGate>>,
) {
    if !path_tool_active(tool.tool) {
        return;
    }
    if simulation_tool_pointer(pointer_gate.as_deref()) != SimulationToolPointer::Free {
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
    let world = resolve_placement_world(
        raw,
        snap.as_ref(),
        roads.as_ref(),
        params.as_ref(),
        feedback.as_mut(),
    );

    let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);

    if buttons.just_pressed(MouseButton::Left) {
        if shift {
            if crate::construction::path_commit_leaves_generated_map(
                &placement.control_points,
                placement
                    .generated_segments
                    .iter()
                    .flat_map(|seg| [seg.start, seg.end]),
                params.width,
                params.height,
            ) {
                return;
            }
            commit_road_path_to_queue(
                placement.as_mut(),
                queue.as_mut(),
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
        placement.generated_segments = regenerate_road_segments(
            &placement.control_points,
            None,
            placement.width,
            params.as_ref(),
            placement.use_curved_preview,
        );
    }
}

#[cfg(test)]
mod egr_tests {
    use super::update_road_path_preview_system;
    use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool, RoadType};
    use crate::engine::states::BaseState;
    use crate::gui::hud::shell_diagnostics::ProductShellDiagnostics;
    use crate::gui::hud::simulation_pointer_gate::SimulationMapPointerGate;
    use bevy::input::ButtonInput;
    use bevy::prelude::*;

    #[test]
    fn egr_armed_road_preview_in_simulation_does_not_acquire_context() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin));
        app.init_state::<BaseState>();
        app.init_resource::<ProductShellDiagnostics>();
        app.init_resource::<bevy_egui::EguiUserTextures>();
        app.world_mut().spawn((
            bevy_egui::EguiContext::default(),
            bevy_egui::PrimaryEguiContext,
        ));
        app.init_resource::<ActiveBuildTool>();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.init_resource::<crate::gui::MapCameraDesiredRes>();
        app.init_resource::<crate::gui::SimulationMapViewport>();
        app.init_resource::<crate::terrain::generation::world_generator_enhanced::WorldGenParams>();
        app.init_resource::<crate::construction::snap::RoadSnapSettings>();
        app.init_resource::<crate::construction::construction_pipeline::ExecutedRoadNetwork>();
        app.init_resource::<super::super::placement::ActiveRoadPlacement>();
        app.init_resource::<crate::construction::path_feedback::ConstructionPathFeedback>();
        app.insert_resource(SimulationMapPointerGate {
            in_play_area: true,
            chrome_blocks: false,
            egui_blocks: false,
            ..Default::default()
        });
        {
            let mut tool = app.world_mut().resource_mut::<ActiveBuildTool>();
            tool.tool = BuildTool::Road(RoadType::Street);
        }
        app.add_systems(Update, update_road_path_preview_system);

        app.world_mut()
            .resource_mut::<NextState<BaseState>>()
            .set(BaseState::Simulation);
        app.update();
        app.update();

        assert!(
            matches!(
                app.world().resource::<State<BaseState>>().get(),
                BaseState::Simulation
            ),
            "tool tick must run while Simulation is current"
        );
        assert_eq!(
            app.world()
                .resource::<ProductShellDiagnostics>()
                .sim_egui_context_acquires,
            0,
            "an armed simulation road tick must not acquire an egui context"
        );
    }
}
