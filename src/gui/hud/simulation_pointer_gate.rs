//! Rect-based pointer gate for simulation Bevy chrome over the full-window map hole.
//!
//! The map viewport fill spans the window; left stack, ops strips, minimap, and context tray
//! overlay it without shrinking the measured hole. Map picks and camera wheel must respect these
//! regions so ghost placement and hover targets align with visible chrome.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::window::{CursorOptions, PrimaryWindow};

use crate::engine::states::BaseState;

use crate::gui::hud::panel_state::HudPanelState;
use crate::gui::hud::simulation_shell_phase2::{
    command_left_stack_footprint_px, sim_build_rail_submenu_block_rect, ContextTrayState,
    CONTEXT_TRAY_BODY_H_PX, CONTEXT_TRAY_PEEK_BODY_H_PX, CONTEXT_TRAY_TAB_H_PX,
};
use crate::construction::{ActiveBuildTool, BuildStripState, BuildTool, ToolContext};
use crate::gui::{
    CommandLeftStackState, MinimapShellState, SimulationMapViewport,
    SIMULATION_MAP_VIEWPORT_TOP_CHROME_PX,
};
use crate::gui::in_game_hud::CENTER_ROW_EDGE_PAD_PX;

/// Per-frame simulation map pointer routing (chrome vs play area).
#[derive(Resource, Clone, Copy, Debug)]
pub struct SimulationMapPointerGate {
    pub cursor: Vec2,
    pub window_logical: Vec2,
    pub chrome_blocks: bool,
    /// Chrome rects only (left stack / minimap / tray) — not egui `wants_pointer_input`.
    pub chrome_blocks_pre_egui: bool,
    pub in_play_area: bool,
    /// Map hole minus chrome; stable for wheel even when egui captures pointer for text focus.
    pub wheel_play_area: bool,
    /// Set after egui pass — floating HUD panels / menus over the map hole.
    pub egui_blocks: bool,
    /// Last frame OS cursor visibility (debug / witness).
    pub os_cursor_visible: bool,
    /// `Window::cursor_position()` was `Some` this frame. A missing cursor is not `(0, 0)`.
    pub cursor_known: bool,
    /// How a map pick resolves this frame (written in [`PointerGateSet::WorldPick`]).
    pub world_pick: MapWorldPick,
}

/// Which projection owns cursor → world this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum MapWorldPick {
    /// Top-down: callers resolve the cursor through the tactical ortho
    /// ([`crate::gui::input_pick::TacticalMapPickFrame`] / `ConstructionMapProjection`).
    #[default]
    TopDown,
    /// PixelWorld3D: the voxel camera ray already hit the surface at this tactical XY (tile units,
    /// x = column, y = row). `None` = the ray misses (or the voxel view is not up yet): no pick.
    PixelWorld(Option<Vec2>),
}

impl SimulationMapPointerGate {
    /// `Some(hit)` when the voxel view owns picks this frame. Callers must use it as is — never
    /// fall back to the top-down ortho, which does not describe what is on screen.
    #[inline]
    #[must_use]
    pub fn pixel_world_pick(&self) -> Option<Option<Vec2>> {
        match self.world_pick {
            MapWorldPick::PixelWorld(hit) => Some(hit),
            MapWorldPick::TopDown => None,
        }
    }
}

impl Default for SimulationMapPointerGate {
    fn default() -> Self {
        Self {
            cursor: Vec2::ZERO,
            window_logical: Vec2::ZERO,
            chrome_blocks: false,
            chrome_blocks_pre_egui: false,
            in_play_area: false,
            wheel_play_area: false,
            egui_blocks: false,
            os_cursor_visible: false,
            cursor_known: true,
            world_pick: MapWorldPick::TopDown,
        }
    }
}

pub fn sync_simulation_map_pointer_gate_system(
    window: Query<&Window, With<PrimaryWindow>>,
    map_vp: Res<SimulationMapViewport>,
    left_stack: Res<CommandLeftStackState>,
    minimap: Res<MinimapShellState>,
    context_tray: Res<ContextTrayState>,
    base: Option<Res<State<BaseState>>>,
    strip: Option<Res<BuildStripState>>,
    tool: Option<Res<ActiveBuildTool>>,
    picker: Option<Res<crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState>>,
    power_sheet: Option<Res<crate::gui::hud::sim_power_tool_sheet::SimPowerToolSheetState>>,
    road_sheet: Option<Res<crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState>>,
    mut gate: ResMut<SimulationMapPointerGate>,
) {
    let Ok(w) = window.single() else {
        *gate = SimulationMapPointerGate {
            cursor_known: false,
            ..Default::default()
        };
        return;
    };
    let Some(cursor) = w.cursor_position() else {
        gate.cursor_known = false;
        gate.chrome_blocks = false;
        gate.chrome_blocks_pre_egui = false;
        gate.in_play_area = false;
        gate.egui_blocks = false;
        return;
    };
    gate.cursor_known = true;
    let window_logical = Vec2::new(w.width(), w.height());
    gate.cursor = cursor;
    gate.window_logical = window_logical;
    gate.chrome_blocks = simulation_chrome_blocks_map_pointer(
        cursor,
        window_logical,
        left_stack.as_ref(),
        minimap.as_ref(),
        context_tray.as_ref(),
    );
    gate.chrome_blocks_pre_egui = gate.chrome_blocks;
    // Play clears the editor's last egui capture. Editor keeps it so tools can
    // see the previous finalize pass without calling `ctx_mut` themselves.
    let in_simulation = base
        .as_ref()
        .is_none_or(|state| matches!(state.get(), BaseState::Simulation));
    if in_simulation {
        gate.egui_blocks = false;
        if open_play_sheet_holds_pointer(
            cursor,
            left_stack.collapsed,
            strip.as_deref(),
            tool.as_deref(),
            picker.as_deref(),
            power_sheet.as_deref(),
            road_sheet.as_deref(),
        ) {
            gate.chrome_blocks = true;
        }
        // Floating Bevy panels (F1 / F3 / logistics / dev) are rect-tested in
        // `apply_bevy_panel_pointer_blocks_system` (`PointerGateSet::Block`).
    } else if gate.egui_blocks {
        gate.chrome_blocks = true;
    }
    gate.in_play_area = if map_vp.is_adequate_for_camera() {
        (!map_vp.valid || map_vp.contains_cursor(cursor)) && !gate.chrome_blocks
    } else {
        false
    };
    gate.wheel_play_area = map_wheel_play_area_allowed(
        cursor,
        window_logical,
        map_vp.as_ref(),
        gate.chrome_blocks_pre_egui,
    );
    // `os_cursor_visible` is written only by `apply_simulation_unified_cursor_system`.
    // Forcing it true here showed the OS cursor in the same frame the crosshair policy hid it.
}

/// Ordered stages of the per-frame map pointer gate. Every consumer of
/// [`SimulationMapPointerGate`] (placement, demolish, draw tools, hover picks) must run
/// `.after(PointerGateSet::Publish)` so a click can never reach the map before the chrome and
/// panel blockers have finalized `chrome_blocks` / `in_play_area` for the frame (R-H5).
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerGateSet {
    /// Cursor + chrome rect sync ([`sync_simulation_map_pointer_gate_system`]).
    Sync,
    /// Bevy panel root rect hit-test ([`apply_bevy_panel_pointer_blocks_system`]).
    Block,
    /// Cursor → world for the active presentation ([`resolve_map_world_pick_system`]).
    WorldPick,
    /// Final gate state published (OS cursor policy). Consumers run after this set.
    Publish,
}

/// Chain [`PointerGateSet`] in `Update`. Idempotent per app only if called once.
pub fn configure_pointer_gate_sets(app: &mut App) {
    app.configure_sets(
        Update,
        (
            PointerGateSet::Sync,
            PointerGateSet::Block,
            PointerGateSet::WorldPick,
            PointerGateSet::Publish,
        )
            .chain(),
    );
}

/// Register the three gate systems into their [`PointerGateSet`] stages (single source of truth
/// for the production plugin and the interaction regression harness).
pub fn add_pointer_gate_systems(app: &mut App) {
    configure_pointer_gate_sets(app);
    app.add_systems(
        Update,
        (
            sync_simulation_map_pointer_gate_system.in_set(PointerGateSet::Sync),
            apply_bevy_panel_pointer_blocks_system.in_set(PointerGateSet::Block),
            resolve_map_world_pick_system.in_set(PointerGateSet::WorldPick),
            apply_simulation_unified_cursor_system.in_set(PointerGateSet::Publish),
        ),
    );
}

/// Resolve cursor → world for the voxel view: the ray of the frame on screen
/// ([`crate::render::pixel_world::PixelWorldCameraFrame`], published in PostUpdate) against the voxel
/// surface. Top-down leaves picks to the tactical ortho path. Sole writer of
/// [`SimulationMapPointerGate::world_pick`].
pub fn resolve_map_world_pick_system(
    mode: Option<Res<crate::gui::WorldPresentationMode>>,
    camera: Option<Res<crate::render::pixel_world::PixelWorldCameraFrame>>,
    field: Option<Res<crate::render::pixel_world::PixelWorldTerrainField>>,
    site_boxes: Option<Res<crate::render::pixel_world::PixelWorldSitePickBoxes>>,
    mut gate: ResMut<SimulationMapPointerGate>,
) {
    let pick = if mode.as_deref().is_some_and(|m| m.is_pixel_world()) {
        let hit = match (camera.as_deref(), field.as_deref()) {
            (Some(cam), Some(field)) if gate.cursor_known => {
                let ground = cam.pick(field, gate.cursor);
                // OP-1: sites stand above the ground (a dam over a river); the nearer of the drawn
                // site box and the ground wins, so clicking the dam selects it.
                let boxed = site_boxes.as_deref().zip(cam.ray(gate.cursor)).and_then(|(b, (o, d))| {
                    crate::render::pixel_world::ray_hit_site_boxes(&b.0, o, d)
                });
                match (boxed, ground) {
                    (Some((tb, xy)), Some(g)) => {
                        let gs = field.surface_at(g).unwrap_or(field.min_h);
                        let gp = Vec3::new(g.x, gs, -g.y);
                        if tb <= (gp - cam.eye).length() { Some(xy) } else { Some(g) }
                    }
                    (Some((_, xy)), None) => Some(xy),
                    (None, g) => g,
                }
            }
            _ => None,
        };
        MapWorldPick::PixelWorld(hit)
    } else {
        MapWorldPick::TopDown
    };
    if gate.world_pick != pick {
        gate.world_pick = pick;
    }
}

/// Bevy UI panel roots that block map picks while the cursor is **over their on-screen rect**
/// (editor dock / toolbox / map-editor shell / road popup, floating dev panels, F1 / F3 and
/// logistics panels). Presence alone never blocks (R-H4).
type PointerBlockingPanelRoots = Or<(
    With<crate::construction::BuildToolboxBevyRoot>,
    With<crate::gui::editor::map_editor::MapEditorBevyShell>,
    With<crate::construction::RoadToolPopupBevyRoot>,
    With<crate::gui::ai_explainability_ui::AiExplainabilityBevyRoot>,
    With<crate::gui::options_keybindings_ui::KeybindingsBevyRoot>,
    With<crate::gui::logistics_orders_panel::LogisticsOrdersBevyRoot>,
    With<crate::gui::logistics_targets_panel::LogisticsTargetsBevyRoot>,
    With<crate::gui::world_presentation_mode::WorldPresentationToggleRoot>,
    With<crate::gui::front_controls::FrontSelectionPanelRoot>,
    With<crate::gui::hud::sim_build_picker_panel::SimBuildPickerPanelRoot>,
)>;

/// True when `physical_cursor` (window physical px — the space `ComputedNode` /
/// `UiGlobalTransform` use, independent of window scale factor and `UiScale`) is inside a laid-out,
/// non-empty node.
#[inline]
#[must_use]
pub fn cursor_over_ui_node(
    node: &ComputedNode,
    transform: &UiGlobalTransform,
    physical_cursor: Vec2,
) -> bool {
    !node.is_empty() && node.contains_point(*transform, physical_cursor)
}

/// Block map pick when the cursor is over any registered Bevy panel root's actual rect.
/// Runs in [`PointerGateSet::Block`] (after the gate sync, before consumers).
pub fn apply_bevy_panel_pointer_blocks_system(
    window: Query<&Window, With<PrimaryWindow>>,
    panels: Query<(&ComputedNode, &UiGlobalTransform), PointerBlockingPanelRoots>,
    fresh_panels: Query<&ComputedNode, (PointerBlockingPanelRoots, Added<Node>)>,
    mut gate: ResMut<SimulationMapPointerGate>,
) {
    if !gate.cursor_known {
        return;
    }
    // A panel root spawned this frame has no layout yet (`ComputedNode` is empty until
    // `PostUpdate`), so its rect cannot be tested: hold the pointer for that one frame.
    let layout_pending = fresh_panels.iter().any(ComputedNode::is_empty);
    if layout_pending {
        gate.chrome_blocks = true;
        gate.in_play_area = false;
        gate.wheel_play_area = false;
        return;
    }
    // `Window::physical_cursor_position` is the exact source of the logical `gate.cursor`
    // (`cursor_position` = physical / scale_factor), so both stay in one coordinate story.
    let Some(physical) = window.single().ok().and_then(Window::physical_cursor_position) else {
        return;
    };
    let over_panel = panels
        .iter()
        .any(|(node, tf)| cursor_over_ui_node(node, tf, physical));
    if !over_panel {
        return;
    }
    gate.chrome_blocks = true;
    gate.in_play_area = false;
    gate.wheel_play_area = false;
}

/// Placement, demolish, and power-cut tools read this instead of `ctx_mut`.
///
/// `Unavailable` matches a failed context read: the click does not commit.
/// `HeldByChrome` matches `egui_wants_pointer_input`: preview clears, commit does not run.
/// `Free` is the pointer that already reached today's commit function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimulationToolPointer {
    Unavailable,
    HeldByChrome,
    Free,
}

#[inline]
#[must_use]
pub fn simulation_tool_pointer(gate: Option<&SimulationMapPointerGate>) -> SimulationToolPointer {
    match gate {
        None => SimulationToolPointer::Unavailable,
        Some(g) if !g.cursor_known => SimulationToolPointer::Unavailable,
        Some(g) if g.chrome_blocks || g.egui_blocks => SimulationToolPointer::HeldByChrome,
        Some(_) => SimulationToolPointer::Free,
    }
}

/// Open Bevy picker / power / road sheet. The map commit path stays held while
/// the cursor is on that sheet. Editor egui still uses the finalize pass.
fn open_play_sheet_holds_pointer(
    cursor: Vec2,
    left_stack_collapsed: bool,
    strip: Option<&BuildStripState>,
    tool: Option<&ActiveBuildTool>,
    picker: Option<&crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState>,
    power_sheet: Option<&crate::gui::hud::sim_power_tool_sheet::SimPowerToolSheetState>,
    road_sheet: Option<&crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState>,
) -> bool {
    let pt = bevy::prelude::Vec2::new(cursor.x, cursor.y);
    let tool = tool.map(|t| t.tool);
    let power_open = power_sheet.is_some_and(|s| s.open)
        && matches!(tool, Some(BuildTool::PowerLine(_)));
    if let Some(picker) = picker {
        let strip_live = strip.is_some_and(|s| s.active != ToolContext::None);
        if picker.open && strip_live && !power_open {
            if sim_build_rail_submenu_block_rect(picker, left_stack_collapsed).contains(pt) {
                return true;
            }
        }
    }
    if power_open {
        let anchor = crate::gui::hud::simulation_shell_phase2::build_rail_slot_anchor_xy(
            ToolContext::Utilities,
            left_stack_collapsed,
        );
        let rect = bevy::prelude::Rect::from_corners(anchor, anchor + bevy::prelude::Vec2::new(
                crate::gui::hud::sim_power_tool_sheet::POWER_TOOL_SHEET_W_PX,
                crate::gui::hud::sim_power_tool_sheet::POWER_TOOL_SHEET_BLOCK_H_PX,
            ),);
        if rect.contains(pt) {
            return true;
        }
    }
    let road_open = road_sheet.is_some_and(|s| s.open)
        && matches!(tool, Some(BuildTool::Road(_) | BuildTool::Rail(_)));
    if road_open {
        let slot = strip.map(|s| s.active).unwrap_or(ToolContext::Roads);
        let anchor = crate::gui::hud::simulation_shell_phase2::build_rail_slot_anchor_xy(
            if slot == ToolContext::Rail {
                ToolContext::Rail
            } else {
                ToolContext::Roads
            },
            left_stack_collapsed,
        );
        let rect = bevy::prelude::Rect::from_corners(anchor, anchor + bevy::prelude::Vec2::new(
                crate::gui::hud::sim_road_tool_sheet::ROAD_TOOL_SHEET_W_PX,
                crate::gui::hud::sim_road_tool_sheet::ROAD_TOOL_SHEET_BLOCK_H_PX,
            ),);
        if rect.contains(pt) {
            return true;
        }
    }
    false
}

/// After egui HUD: block map picks / hide unified cursor when floating panels capture the pointer.

/// Whether the cursor is over the tactical map hole (excluding chrome), for wheel zoom routing.
#[inline]
pub fn map_wheel_play_area_allowed(
    cursor: Vec2,
    window_logical: Vec2,
    map_vp: &SimulationMapViewport,
    chrome_blocks: bool,
) -> bool {
    if chrome_blocks {
        return false;
    }
    if map_vp.is_adequate_for_camera() {
        !map_vp.valid || map_vp.contains_cursor(cursor)
    } else {
        cursor.x >= 0.0
            && cursor.y >= 0.0
            && cursor.x <= window_logical.x.max(1.0)
            && cursor.y <= window_logical.y.max(1.0)
    }
}

/// **TRIAGE-CURSOR-UNIFY-001** / design_build_ux §5 — placement active when strip or tool selected.
#[inline]
#[must_use]
pub fn simulation_build_placement_active(
    strip: &BuildStripState,
    tool: Option<&ActiveBuildTool>,
) -> bool {
    strip.active != ToolContext::None
        || tool.is_some_and(|t| !matches!(t.tool, BuildTool::None))
}

/// OS cursor policy: hidden **only** while an armed placement tool is over the free map play area,
/// where [`crate::gui::hud::sim_play_bevy_chrome::sync_sim_play_crosshair`] draws the world
/// crosshair in its place. Over any chrome (build rail, left stack, ops strip, tray, minimap,
/// panels) the OS cursor must stay visible: the crosshair is never drawn there, so hiding it left
/// no cursor at all above clickable buttons (the "funny cursor" over the side bar).
/// Menu/chrome flags must not flip this back on — that alternates OS cursor and crosshair.
#[inline]
#[must_use]
pub fn armed_play_area_hides_os_cursor(
    base: BaseState,
    build_placement_active: bool,
    in_play_area: bool,
    chrome_blocks: bool,
) -> bool {
    matches!(base, BaseState::Simulation)
        && build_placement_active
        && in_play_area
        && !chrome_blocks
}

pub fn apply_simulation_unified_cursor_system(
    base: Res<State<BaseState>>,
    mut gate: ResMut<SimulationMapPointerGate>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
    strip: Option<Res<BuildStripState>>,
    tool: Option<Res<ActiveBuildTool>>,
) {
    let build_active = strip
        .as_ref()
        .map(|s| simulation_build_placement_active(s, tool.as_deref()))
        .unwrap_or_else(|| tool.as_ref().is_some_and(|t| !matches!(t.tool, BuildTool::None)));
    let hide_os = armed_play_area_hides_os_cursor(
        *base.get(),
        build_active,
        gate.in_play_area,
        gate.chrome_blocks,
    );
    gate.os_cursor_visible = !hide_os;
    for mut cursor in &mut cursors {
        cursor.visible = !hide_os;
    }
}

/// Draw crosshair when OS cursor is hidden (TRIAGE-CURSOR-UNIFY-001).
/// Position = gate.cursor (same logical px as map pick) converted to egui points.

#[inline]
fn rect_contains(rect: bevy::prelude::Rect, cursor: Vec2) -> bool {
    rect.contains(bevy::prelude::Vec2::new(cursor.x, cursor.y))
}

/// Bottom context tray height when visible (logical px).
#[must_use]
pub fn context_tray_chrome_height(tray: &ContextTrayState) -> f32 {
    if !tray.panel_state.shows_content() {
        return 0.0;
    }
    let body = match tray.panel_state {
        HudPanelState::Peek => CONTEXT_TRAY_PEEK_BODY_H_PX,
        HudPanelState::Expanded | HudPanelState::Pinned => CONTEXT_TRAY_BODY_H_PX,
        HudPanelState::Collapsed => 0.0,
    };
    CONTEXT_TRAY_TAB_H_PX + body
}

/// True when the cursor is over Bevy simulation chrome that overlays the map hole.
#[must_use]
pub fn simulation_chrome_blocks_map_pointer(
    cursor: Vec2,
    window_logical: Vec2,
    left_stack: &CommandLeftStackState,
    minimap: &MinimapShellState,
    context_tray: &ContextTrayState,
) -> bool {
    if cursor.y < SIMULATION_MAP_VIEWPORT_TOP_CHROME_PX {
        return true;
    }

    let left_block_w =
        CENTER_ROW_EDGE_PAD_PX + command_left_stack_footprint_px(left_stack.collapsed);
    if cursor.x < left_block_w {
        return true;
    }

    if minimap.visible && !minimap.minimized {
        if let Some(r) = minimap.last_window_rect {
            if rect_contains(r, cursor) {
                return true;
            }
        }
    }

    let tray_h = context_tray_chrome_height(context_tray);
    if tray_h > 0.0 && cursor.y >= window_logical.y - tray_h {
        return true;
    }

    false
}

/// Map play area: inside the measured hole and not under simulation chrome overlays.
#[must_use]
pub fn cursor_in_simulation_map_play_area(
    cursor: Vec2,
    window_logical: Vec2,
    map_vp: &SimulationMapViewport,
    left_stack: &CommandLeftStackState,
    minimap: &MinimapShellState,
    context_tray: &ContextTrayState,
) -> bool {
    if !map_vp.is_adequate_for_camera() {
        return false;
    }
    if map_vp.valid && !map_vp.contains_cursor(cursor) {
        return false;
    }
    !simulation_chrome_blocks_map_pointer(cursor, window_logical, left_stack, minimap, context_tray)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn left_stack_blocks_map_pointer() {
        let left = CommandLeftStackState { collapsed: true };
        let minimap = MinimapShellState::default();
        let tray = ContextTrayState::default();
        let cursor = Vec2::new(20.0, 200.0);
        assert!(simulation_chrome_blocks_map_pointer(
            cursor,
            Vec2::new(1280.0, 720.0),
            &left,
            &minimap,
            &tray,
        ));
    }

    #[test]
    fn map_center_is_play_area_when_hole_valid() {
        let vp = SimulationMapViewport {
            valid: true,
            min: Vec2::ZERO,
            max: Vec2::new(1280.0, 720.0),
            ..Default::default()
        };
        let left = CommandLeftStackState { collapsed: true };
        let minimap = MinimapShellState::default();
        let tray = ContextTrayState::default();
        let cursor = Vec2::new(640.0, 400.0);
        assert!(cursor_in_simulation_map_play_area(
            cursor,
            Vec2::new(1280.0, 720.0),
            &vp,
            &left,
            &minimap,
            &tray,
        ));
    }

    #[test]
    fn simulation_unified_cursor_hides_over_play_area() {
        use crate::engine::states::BaseState;

        assert!(armed_play_area_hides_os_cursor(
            BaseState::Simulation,
            true,
            true,
            false
        ));
        assert!(!armed_play_area_hides_os_cursor(
            BaseState::Simulation,
            false,
            true,
            false
        ));
        assert!(!armed_play_area_hides_os_cursor(
            BaseState::MainMenu,
            true,
            true,
            false
        ));
    }

    /// The OS cursor must stay visible over the build rail / any chrome even while a tool is armed
    /// (the crosshair is never drawn there).
    #[test]
    fn cursor_stays_visible_over_chrome_while_tool_armed() {
        let left = CommandLeftStackState { collapsed: true };
        let rail_cursor = Vec2::new(40.0, 200.0);
        let chrome = simulation_chrome_blocks_map_pointer(
            rail_cursor,
            Vec2::new(1280.0, 720.0),
            &left,
            &MinimapShellState::default(),
            &ContextTrayState::default(),
        );
        assert!(chrome, "rail position is chrome");
        // in_play_area is false over chrome, and chrome_blocks wins even if a caller disagrees.
        assert!(!armed_play_area_hides_os_cursor(BaseState::Simulation, true, false, chrome));
        assert!(!armed_play_area_hides_os_cursor(BaseState::Simulation, true, true, chrome));
    }

    #[test]
    fn cursor_hide_does_not_oscillate_over_armed_play_area() {
        let mut seen = [false; 4];
        for slot in &mut seen {
            *slot = armed_play_area_hides_os_cursor(BaseState::Simulation, true, true, false);
        }
        assert!(seen.iter().all(|hide| *hide));
        assert_eq!(seen[0], seen[3]);
        assert!(!armed_play_area_hides_os_cursor(
            BaseState::Simulation,
            true,
            false,
            false
        ));
        assert!(!armed_play_area_hides_os_cursor(
            BaseState::Simulation,
            false,
            true,
            false
        ));
    }
}
