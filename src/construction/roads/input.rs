//! Corridor path input (roads of every tier, rail, firebreaks — one tool, PLAN-ROADS-v1 RD-6).
//!
//! Click to start, click to add a bend; double-click, Enter, or click an existing line to finish;
//! Backspace / RMB removes the last point; Esc cancels; Shift locks the stretch to 15° steps;
//! drag a placed point to move it. The preview follows the cursor from the first click (no
//! modifier) and every stretch is judged live by [`check_corridor_path`].

use bevy::prelude::*;

use crate::construction::map_egui_projection::ConstructionMapProjection;
use crate::construction::map_pick_params::ConstructionMapPick;
use crate::gui::hud::simulation_pointer_gate::{
    simulation_tool_pointer, SimulationMapPointerGate, SimulationToolPointer,
};
use crate::gui::{MapCameraDesired, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::systems::water::WaterTileProbe;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool};
use crate::construction::construction_pipeline::ConstructionPlanQueue;
use crate::construction::corridor_tool::{
    check_corridor_path, commit_corridor_path, mode_spec, summarize_path, CorridorToolKind, StretchCheck,
};
use crate::construction::path_feedback::ConstructionPathFeedback;
use crate::construction::sessions::ActiveToolSession;
use crate::construction::snap::{snap_placement, RoadSnapSettings, SnapTarget};
use crate::construction::terrain_conform::{conform_vec3, sample_height_norm};

use super::placement::{ActiveRoadPlacement, RoadSegmentPreview};
use super::spline::catmull_rom_chain;

/// The corridor kind the active tool lays, if it is a path tool.
#[must_use]
pub fn corridor_tool_kind(tool: BuildTool) -> Option<CorridorToolKind> {
    match tool {
        BuildTool::Road(tier) => Some(CorridorToolKind::Road(tier)),
        BuildTool::Rail(_) => Some(CorridorToolKind::Rail),
        BuildTool::Firebreak => Some(CorridorToolKind::Firebreak),
        _ => None,
    }
}

/// Cursor → map point `Vec3(x, 0, row)` in the presentation on screen.
///
/// PixelWorld3D: the gate's voxel ray hit ([`SimulationMapPointerGate::pixel_world_pick`]) — never the
/// top-down ortho. Top-down: the tactical ortho via [`ConstructionMapProjection`].
#[must_use]
pub fn cursor_world_on_map(
    window: &Window,
    gate: Option<&SimulationMapPointerGate>,
    authority: Option<&ViewProjectionAuthority>,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    params: &WorldGenParams,
) -> Option<Vec3> {
    if let Some(hit) = gate.and_then(SimulationMapPointerGate::pixel_world_pick) {
        return hit.map(|xy| Vec3::new(xy.x, 0.0, xy.y));
    }
    let cursor_px = window.cursor_position()?;
    let proj = ConstructionMapProjection::resolve(authority, desired, map_vp, params);
    let xy = proj.cursor_world_xy(cursor_px)?;
    Some(Vec3::new(xy.x, 0.0, xy.y))
}

/// Shift: keep the stretch from `from` to `to` on a 15° heading, same length.
fn angle_locked(from: Vec3, to: Vec3) -> Vec3 {
    let d = Vec2::new(to.x - from.x, to.z - from.z);
    let len = d.length();
    if len < 1e-3 {
        return to;
    }
    let step = 15f32.to_radians();
    let a = (d.y.atan2(d.x) / step).round() * step;
    Vec3::new(from.x + a.cos() * len, to.y, from.z + a.sin() * len)
}

/// Sampled path (tile units): straight between points, or a Catmull-Rom curve through them.
fn sample_path(points: &[Vec3], curved: bool) -> Vec<Vec2> {
    let chain = if curved && points.len() >= 3 { catmull_rom_chain(points, 6) } else { points.to_vec() };
    let mut out: Vec<Vec2> = chain.iter().map(|p| Vec2::new(p.x, p.z)).collect();
    out.dedup_by(|a, b| a.distance(*b) < 0.05);
    out
}

/// Width the active kind draws at.
pub fn sync_road_placement_width_from_tool(tool: Res<ActiveBuildTool>, mut placement: ResMut<ActiveRoadPlacement>) {
    if let Some(kind) = corridor_tool_kind(tool.tool) {
        let w = mode_spec(kind).width;
        if placement.width != w {
            placement.width = w;
        }
    }
}

/// Live preview: snapped cursor, the placed path and the stretch to the cursor, each judged.
#[allow(clippy::too_many_arguments)]
pub fn update_road_path_preview_system(
    tool: Res<ActiveBuildTool>,
    keys: Res<ButtonInput<KeyCode>>,
    pick: ConstructionMapPick,
    snap: Res<RoadSnapSettings>,
    roads: Res<crate::construction::CorridorNetwork>,
    mut placement: ResMut<ActiveRoadPlacement>,
    mut feedback: ResMut<ConstructionPathFeedback>,
    water: WaterTileProbe,
) {
    let Some(kind) = corridor_tool_kind(tool.tool) else {
        if !placement.generated_segments.is_empty() || placement.cursor.is_some() {
            placement.clear_path();
            placement.cursor = None;
        }
        return;
    };
    let rail = kind == CorridorToolKind::Rail;
    let cursor = pick.cursor_world().map(|raw| {
        let placed = snap_placement(raw, snap.as_ref(), roads.as_ref(), rail);
        feedback.snap_hint = placed.target.as_ref().map(SnapTarget::hint_label);
        let joins = placed.target.as_ref().is_some_and(SnapTarget::joins_line);
        let mut w = placed.world;
        let shift = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
        if shift && !joins {
            if let Some(&last) = placement.control_points.last() {
                w = angle_locked(last, w);
            }
        }
        (conform_vec3(w, pick.params.as_ref(), pick.height_grid()), joins)
    });
    placement.cursor = cursor.map(|c| c.0);
    placement.cursor_joins_line = cursor.is_some_and(|c| c.1);

    let params = pick.params.as_ref();
    let grid = pick.height_grid();
    let ground = |p: Vec2| sample_height_norm(p.x, p.y, params, grid) * params.elevation_range_m;
    let wet = |t: IVec2| water.depth_m(t);
    let map = UVec2::new(params.width, params.height);
    let curved = placement.use_curved_preview && kind != CorridorToolKind::Firebreak;

    // G: route assist from the last point to the cursor (adds the bends as control points).
    if std::mem::take(&mut placement.assist_requested) {
        if let (Some(last), Some(c)) = (placement.control_points.last().copied(), placement.cursor) {
            let route = crate::construction::corridor_tool::assist_route(
                Vec2::new(last.x, last.z),
                Vec2::new(c.x, c.z),
                kind,
                &ground,
                &wet,
                roads.as_ref(),
                map,
            );
            placement.assist_note = match route {
                Some(pts) => {
                    for p in pts {
                        placement.control_points.push(conform_vec3(Vec3::new(p.x, 0.0, p.y), params, grid));
                    }
                    None
                }
                None => Some("route assist: no way within this line's grade found".into()),
            };
        }
    }
    let placed = sample_path(&placement.control_points, curved);
    let mut checks = if placed.len() >= 2 {
        check_corridor_path(&placed, kind, &ground, &wet, roads.as_ref(), map)
    } else {
        Vec::new()
    };
    crate::construction::corridor_tool::apply_bridge_choice(&mut checks, placement.bridge_choice);
    let mut live_points = placement.control_points.clone();
    if let (Some(c), None) = (placement.cursor, placement.dragging) {
        if live_points.last().is_none_or(|l| Vec2::new(l.x - c.x, l.z - c.z).length() > 0.3) && !live_points.is_empty() {
            live_points.push(c);
        }
    }
    let live = sample_path(&live_points, curved);
    let mut live_checks = if live.len() >= 2 {
        check_corridor_path(&live, kind, &ground, &wet, roads.as_ref(), map)
    } else {
        Vec::new()
    };
    crate::construction::corridor_tool::apply_bridge_choice(&mut live_checks, placement.bridge_choice);
    let width = placement.width;
    let segs = live_checks
        .iter()
        .map(|c| RoadSegmentPreview {
            start: conform_vec3(Vec3::new(c.a.x, 0.0, c.a.y), params, grid),
            end: conform_vec3(Vec3::new(c.b.x, 0.0, c.b.y), params, grid),
            width,
            valid: c.verdict.buildable(),
            works: matches!(c.verdict, crate::construction::corridor_tool::StretchVerdict::Works(_)),
        })
        .collect();
    placement.summary = summarize_path(&live_checks, kind);
    if let Some(note) = placement.assist_note.clone() {
        placement.summary.notes.insert(0, note);
    }
    placement.checks = checks;
    placement.live_checks = live_checks;
    placement.generated_segments = segs;
}

/// HUD feedback + placement report from the live checks.
pub fn sync_road_path_build_preview(
    tool: Res<ActiveBuildTool>,
    placement: Res<ActiveRoadPlacement>,
    mut preview: ResMut<crate::construction::BuildPlacementPreview>,
    mut feedback: ResMut<ConstructionPathFeedback>,
) {
    if corridor_tool_kind(tool.tool).is_none() {
        if !feedback.required_actions.is_empty() {
            feedback.required_actions.clear();
        }
        return;
    }
    if feedback.required_actions != placement.summary.notes {
        feedback.required_actions = placement.summary.notes.clone();
    }
    let ok = placement.checks.iter().any(|c| c.verdict.buildable());
    preview.report = crate::strategic::SitePlacementValidation {
        valid: ok,
        allows_commit: ok,
        errors: placement
            .live_checks
            .iter()
            .filter_map(|c| match &c.verdict {
                crate::construction::corridor_tool::StretchVerdict::Blocked(why) => Some(why.clone()),
                _ => None,
            })
            .take(2)
            .collect(),
        ..Default::default()
    };
}

/// Commit the placed path's buildable stretches; keep the last point as the next start when the
/// session draws continuous paths.
pub fn finish_corridor_path(
    placement: &mut ActiveRoadPlacement,
    kind: CorridorToolKind,
    queue: &mut ConstructionPlanQueue,
    session: &mut ActiveToolSession,
) -> usize {
    let checks: Vec<StretchCheck> = std::mem::take(&mut placement.checks);
    let n = commit_corridor_path(&checks, kind, queue);
    let last = placement.control_points.last().copied();
    placement.clear_path();
    if n > 0 {
        session.record_commit();
        if session.continuous_path {
            if let Some(p) = last {
                placement.control_points.push(p);
            }
        }
    }
    n
}

/// A second click this soon (s) and this close (tiles) finishes the path.
const DOUBLE_CLICK_S: f64 = 0.35;
const DOUBLE_CLICK_TILES: f32 = 1.0;
/// A press this close (tiles) to a placed point grabs it for dragging.
const GRAB_TILES: f32 = 0.8;

/// Clicks and keys for the corridor tool.
#[allow(clippy::too_many_arguments)]
pub fn road_path_input_system(
    tool: Res<ActiveBuildTool>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    mut placement: ResMut<ActiveRoadPlacement>,
    mut queue: ResMut<ConstructionPlanQueue>,
    mut session: ResMut<ActiveToolSession>,
    pointer_gate: Option<Res<SimulationMapPointerGate>>,
    mut last_click: Local<Option<(f64, Vec2)>>,
) {
    let Some(kind) = corridor_tool_kind(tool.tool) else {
        return;
    };
    if keys.just_pressed(KeyCode::Escape) {
        placement.clear_path();
        return;
    }
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) {
        finish_corridor_path(&mut placement, kind, &mut queue, &mut session);
        return;
    }
    if keys.just_pressed(KeyCode::Backspace) {
        placement.control_points.pop();
        return;
    }
    if keys.just_pressed(KeyCode::KeyG) {
        placement.assist_requested = true;
        return;
    }
    // T: next bridge type that spans every crossing of the path (wraps back to the cheapest).
    if keys.just_pressed(KeyCode::KeyT) {
        let choices = crate::construction::corridor_tool::bridge_choices(&placement.live_checks);
        let now = placement.live_checks.iter().find_map(|c| c.bridge.map(|b| b.kind));
        let at = now.and_then(|t| choices.iter().position(|c| *c == t));
        placement.bridge_choice = match at {
            Some(i) if i + 1 < choices.len() => Some(choices[i + 1]),
            _ => choices.first().copied(),
        };
        return;
    }
    if simulation_tool_pointer(pointer_gate.as_deref()) != SimulationToolPointer::Free {
        return;
    }
    if buttons.just_pressed(MouseButton::Right) {
        placement.control_points.pop();
        return;
    }
    let Some(cursor) = placement.cursor else {
        return;
    };
    let c2 = Vec2::new(cursor.x, cursor.z);
    if let Some(i) = placement.dragging {
        if buttons.pressed(MouseButton::Left) {
            if let Some(p) = placement.control_points.get_mut(i) {
                *p = cursor;
            }
        } else {
            placement.dragging = None;
        }
        return;
    }
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let now = time.elapsed_secs_f64();
    let double = last_click.is_some_and(|(t, p)| now - t < DOUBLE_CLICK_S && p.distance(c2) < DOUBLE_CLICK_TILES);
    *last_click = Some((now, c2));
    if double && placement.control_points.len() >= 2 {
        finish_corridor_path(&mut placement, kind, &mut queue, &mut session);
        *last_click = None;
        return;
    }
    // Grab a placed point (not the one just placed by the first click of a double-click).
    let grab = placement
        .control_points
        .iter()
        .position(|p| Vec2::new(p.x, p.z).distance(c2) < GRAB_TILES);
    if let Some(i) = grab {
        placement.dragging = Some(i);
        return;
    }
    let joins = placement.cursor_joins_line && !placement.control_points.is_empty();
    placement.control_points.push(cursor);
    if joins {
        // Ending on an existing line finishes the path there (a junction). The checks are
        // recomputed next frame, so finish from fresh checks then.
        placement.dragging = None;
        placement.finish_requested = true;
    }
}

/// Finish a path that ended on an existing line once its checks include the last point.
pub fn finish_pending_corridor_path(
    tool: Res<ActiveBuildTool>,
    mut placement: ResMut<ActiveRoadPlacement>,
    mut queue: ResMut<ConstructionPlanQueue>,
    mut session: ResMut<ActiveToolSession>,
) {
    if !placement.finish_requested {
        return;
    }
    placement.finish_requested = false;
    if let Some(kind) = corridor_tool_kind(tool.tool) {
        finish_corridor_path(&mut placement, kind, &mut queue, &mut session);
    }
}

#[cfg(test)]
mod egr_tests {
    use super::update_road_path_preview_system;
    use crate::construction::build_tool_authority::{ActiveBuildTool, BuildTool, RoadTier};
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
        /* no egui textures */
        /* no primary egui */
        app.init_resource::<ActiveBuildTool>();
        app.insert_resource(ButtonInput::<KeyCode>::default());
        app.init_resource::<crate::gui::MapCameraDesiredRes>();
        app.init_resource::<crate::gui::SimulationMapViewport>();
        app.init_resource::<crate::gui::MainWorldCameraOrthoTrace>();
        app.init_resource::<crate::terrain::generation::world_generator_enhanced::WorldGenParams>();
        app.init_resource::<crate::construction::snap::RoadSnapSettings>();
        app.init_resource::<crate::construction::CorridorNetwork>();
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
            tool.tool = BuildTool::Road(RoadTier::Gravel);
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
