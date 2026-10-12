//! Rect-based pointer gate for simulation Bevy chrome over the full-window map hole.
//!
//! The map viewport fill spans the window; left stack, ops strips, minimap, and context tray
//! overlay it without shrinking the measured hole. Map picks and camera wheel must respect these
//! regions so ghost placement and hover targets align with visible chrome.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::window::{CursorOptions, PrimaryWindow};

use crate::engine::states::BaseState;
use crate::engine::AppState;

use crate::gui::hud::layout_store::HudLayoutStore;
use crate::gui::hud::panel_state::HudPanelState;
use crate::gui::hud::shell_framework::{HudDockRegistry, ProductShellWidgetId};
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
    diagnostics: Option<Res<crate::gui::DiagnosticsUiState>>,
    keybindings: Option<Res<crate::gui::KeybindingsUiState>>,
    logistics_orders: Option<Res<crate::gui::LogisticsOrdersPanelState>>,
    logistics_targets: Option<Res<crate::gui::LogisticsTargetsPanelState>>,
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
        // Bevy floating panels block map pick while open (F1 / F3 / logistics).
        if diagnostics.as_ref().is_some_and(|d| d.visible)
            || keybindings.as_ref().is_some_and(|k| k.visible)
            || logistics_orders.as_ref().is_some_and(|p| p.visible)
            || logistics_targets.as_ref().is_some_and(|p| p.visible)
        {
            gate.chrome_blocks = true;
        }
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

/// Editor Bevy roots (dock / toolbox / map-editor shell) block map pick.
/// Kept separate so [`sync_simulation_map_pointer_gate_system`] stays under the
/// Bevy system-param budget used by `.after(...)` ordering.
pub fn apply_editor_bevy_panel_pointer_blocks_system(
    base: Option<Res<State<BaseState>>>,
    presence: Option<Res<crate::gui::PanelPresence>>,
    map_vp: Res<SimulationMapViewport>,
    mut gate: ResMut<SimulationMapPointerGate>,
) {
    let in_editor = base
        .as_ref()
        .is_some_and(|state| matches!(state.get(), BaseState::Editor));
    if !in_editor {
        return;
    }
    let bevy_editor_chrome = presence.as_ref().is_some_and(|p| {
        p.editor_dock_root
            || p.build_toolbox_root
            || p.map_editor_shell_root
            || p.road_popup_root
            || p.staged_placements_root
    });
    if !bevy_editor_chrome {
        return;
    }
    gate.chrome_blocks = true;
    gate.in_play_area = if map_vp.is_adequate_for_camera() {
        (!map_vp.valid || map_vp.contains_cursor(gate.cursor)) && !gate.chrome_blocks
    } else {
        false
    };
}

/// Floating Bevy dev panels (faction / AI explain / pressure / assembly QC) block map pick.
pub fn apply_dev_bevy_panel_pointer_blocks_system(
    presence: Option<Res<crate::gui::PanelPresence>>,
    map_vp: Res<SimulationMapViewport>,
    mut gate: ResMut<SimulationMapPointerGate>,
) {
    let blocks = presence.as_ref().is_some_and(|p| {
        p.faction_tools_root
            || p.ai_explainability_root
            || p.pressure_composer_root
            || p.assembly_snapshot_qc_root
    });
    if !blocks {
        return;
    }
    gate.chrome_blocks = true;
    gate.in_play_area = if map_vp.is_adequate_for_camera() {
        (!map_vp.valid || map_vp.contains_cursor(gate.cursor)) && !gate.chrome_blocks
    } else {
        false
    };
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

#[inline]
fn sim_build_rail_submenu_blocks_pointer(
    strip: &BuildStripState,
    tool: &ActiveBuildTool,
    picker: &crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState,
    left_stack: &CommandLeftStackState,
    cursor: Vec2,
) -> bool {
    if strip.active == ToolContext::None {
        return false;
    }
    if !picker.open
        && !tool.residential_menu_open
        && !tool.commercial_menu_open
        && !tool.industrial_menu_open
        && !tool.utilities_menu_open
        && !tool.mock_shapes_menu_open
    {
        return false;
    }
    sim_build_rail_submenu_block_rect(picker, left_stack.collapsed)
        .contains(bevy::prelude::Vec2::new(cursor.x, cursor.y))
}

#[inline]
fn cursor_over_visible_hud_widget(
    cursor: Vec2,
    dock: &HudDockRegistry,
    layout: &HudLayoutStore,
) -> bool {
    let pt = bevy::prelude::Vec2::new(cursor.x, cursor.y);
    for widget in ProductShellWidgetId::ALL {
        let slot = dock.slot(widget);
        if !slot.visible || slot.minimized {
            continue;
        }
        let frame = layout.frame(widget);
        if !frame.initialized {
            continue;
        }
        let rect = bevy::prelude::Rect::from_corners(bevy::prelude::Vec2::new(frame.pos.x, frame.pos.y), bevy::prelude::Vec2::new(frame.pos.x, frame.pos.y) + bevy::prelude::Vec2::new(frame.size.x.max(1.0), frame.size.y.max(1.0)),);
        if rect.contains(pt) {
            return true;
        }
    }
    false
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

/// Left command stack / build rail (logical px) — hide OS here while placing (§2 hide-when-build).
#[inline]
#[must_use]
pub fn simulation_cursor_over_build_rail_chrome(
    cursor: Vec2,
    left_stack: &CommandLeftStackState,
) -> bool {
    if cursor.y < SIMULATION_MAP_VIEWPORT_TOP_CHROME_PX {
        return false;
    }
    let left_block_w =
        CENTER_ROW_EDGE_PAD_PX + command_left_stack_footprint_px(left_stack.collapsed);
    cursor.x < left_block_w
}

/// Hide-region while build-active: play area **or** left build rail (not ops/minimap/tray).
#[inline]
#[must_use]
pub fn simulation_unified_cursor_hide_region(
    cursor: Vec2,
    left_stack: &CommandLeftStackState,
    in_play_area: bool,
) -> bool {
    in_play_area || simulation_cursor_over_build_rail_chrome(cursor, left_stack)
}

/// **TRIAGE-CURSOR-UNIFY-001** — OS hide only when build placement active ∧ hide-region.
/// Picks stay on [`Window::cursor_position`] / gate.cursor (aligned with game crosshair).
#[inline]
#[must_use]
pub fn simulation_unified_cursor_hide_os(
    base: BaseState,
    build_placement_active: bool,
    in_hide_region: bool,
) -> bool {
    matches!(base, BaseState::Simulation) && build_placement_active && in_hide_region
}

/// Armed tool over the play area: hide the OS cursor. Menu/chrome flags must not
/// flip this back on — that alternates the OS cursor with the crosshair.
#[inline]
#[must_use]
pub fn armed_play_area_hides_os_cursor(
    base: BaseState,
    build_placement_active: bool,
    in_play_area: bool,
    _menu_open: bool,
) -> bool {
    matches!(base, BaseState::Simulation) && build_placement_active && in_play_area
}

/// **TRIAGE-CURSOR-UNIFY-001** — build-scoped hide + rail continuity + idle show.
#[must_use]
pub fn triage_cursor_unify_001_witness_green() -> bool {
    triage_cursor_unify_001_self_check().is_ok()
}

fn triage_cursor_unify_001_self_check() -> Result<(), &'static str> {
    let strip_idle = BuildStripState {
        active: ToolContext::None,
        ..Default::default()
    };
    let strip_roads = BuildStripState {
        active: ToolContext::Roads,
        ..Default::default()
    };
    let left = CommandLeftStackState { collapsed: true };
    let play = Vec2::new(640.0, 400.0);
    let rail = Vec2::new(20.0, 200.0);
    let ops = Vec2::new(640.0, 10.0);

    if !simulation_build_placement_active(&strip_roads, None) {
        return Err("strip_active_is_placement");
    }
    if simulation_build_placement_active(&strip_idle, None) {
        return Err("idle_strip_not_placement");
    }
    // Idle sim: never hide (fixes always-on play-area hide glitch).
    if simulation_unified_cursor_hide_os(BaseState::Simulation, false, true) {
        return Err("idle_show_play");
    }
    // Build + play → hide.
    if !simulation_unified_cursor_hide_os(BaseState::Simulation, true, true) {
        return Err("hide_build_play");
    }
    // Build + left rail → hide (continuous with play — no OS pop-in).
    if !simulation_unified_cursor_hide_region(rail, &left, false) {
        return Err("rail_is_hide_region");
    }
    if !simulation_unified_cursor_hide_os(
        BaseState::Simulation,
        true,
        simulation_unified_cursor_hide_region(rail, &left, false),
    ) {
        return Err("hide_build_rail");
    }
    // Build + ops strip → show OS (not a hide region).
    if simulation_unified_cursor_hide_region(ops, &left, false) {
        return Err("ops_not_hide_region");
    }
    if simulation_unified_cursor_hide_os(BaseState::Simulation, true, false) {
        return Err("show_build_non_hide");
    }
    if simulation_unified_cursor_hide_os(BaseState::MainMenu, true, true) {
        return Err("show_menu");
    }
    // Sanity: play sample is hide-region when in_play_area.
    if !simulation_unified_cursor_hide_region(play, &left, true) {
        return Err("play_hide_region");
    }
    Ok(())
}

pub fn apply_simulation_unified_cursor_system(
    base: Res<State<BaseState>>,
    mut gate: ResMut<SimulationMapPointerGate>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
    left_stack: Res<CommandLeftStackState>,
    strip: Option<Res<BuildStripState>>,
    picker: Option<Res<crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState>>,
    tool: Option<Res<ActiveBuildTool>>,
    diagnostics: Option<Res<crate::gui::DiagnosticsUiState>>,
    keybindings: Option<Res<crate::gui::KeybindingsUiState>>,
    logistics_orders: Option<Res<crate::gui::LogisticsOrdersPanelState>>,
    logistics_targets: Option<Res<crate::gui::LogisticsTargetsPanelState>>,
) {
    // Catalog / tool menus / diagnostics / floating panels need OS cursor for accurate hits.
    // Do **not** treat all chrome_blocks as menu_open — that popped the OS cursor on the
    // build rail during place mode (dual-cursor glitch vs play-area hide).
    let picker_open = picker.as_ref().is_some_and(|p| p.open);
    let tool_menus = tool.as_ref().is_some_and(|t| {
        t.residential_menu_open
            || t.commercial_menu_open
            || t.industrial_menu_open
            || t.utilities_menu_open
            || t.mock_shapes_menu_open
    });
    let diag_open = diagnostics.as_ref().is_some_and(|d| d.visible);
    let keys_open = keybindings.as_ref().is_some_and(|k| k.visible);
    let logi_open = logistics_orders.as_ref().is_some_and(|p| p.visible)
        || logistics_targets.as_ref().is_some_and(|p| p.visible);
    let menu_open = picker_open || tool_menus || diag_open || keys_open || logi_open || gate.egui_blocks;

    let build_active = strip
        .as_ref()
        .map(|s| simulation_build_placement_active(s, tool.as_deref()))
        .unwrap_or_else(|| tool.as_ref().is_some_and(|t| !matches!(t.tool, BuildTool::None)));
    let in_hide_region = simulation_unified_cursor_hide_region(
        gate.cursor,
        left_stack.as_ref(),
        gate.in_play_area,
    );
    let play_hide = armed_play_area_hides_os_cursor(
        *base.get(),
        build_active,
        gate.in_play_area,
        menu_open,
    );
    let rail_hide = !menu_open
        && simulation_unified_cursor_hide_os(*base.get(), build_active, in_hide_region);
    let hide_os = play_hide || rail_hide;
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

/// **BUILD-VERIFY-POINTER-001** — lib witness: build rail submenu blocks map picks.
#[must_use]
pub fn build_verify_pointer_001_witness_green() -> bool {
    let strip = BuildStripState {
        active: ToolContext::Industry,
        ..Default::default()
    };
    let tool = ActiveBuildTool {
        industrial_menu_open: true,
        ..Default::default()
    };
    let picker = crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState {
        open: true,
        category: crate::gui::hud::sim_build_picker_sheet::BuildPickerCategory::Industry,
        anchor_slot: ToolContext::Industry,
    };
    let left_stack = CommandLeftStackState { collapsed: true };
    let rect = sim_build_rail_submenu_block_rect(&picker, left_stack.collapsed);
    let cursor = Vec2::new(rect.center().x, rect.center().y);
    sim_build_rail_submenu_blocks_pointer(&strip, &tool, &picker, &left_stack, cursor)
}

#[must_use]
pub fn build_verify_pointer_001_witness_json() -> serde_json::Value {
    serde_json::json!({
        "gate": "BUILD-VERIFY-POINTER-001",
        "green": build_verify_pointer_001_witness_green(),
        "pick_blocked_under_toolbox": build_verify_pointer_001_witness_green(),
        "build_toolbox_submenu_rect_wired": true,
    })
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

        let left = CommandLeftStackState { collapsed: true };
        assert!(simulation_unified_cursor_hide_os(
            BaseState::Simulation,
            true,
            true
        ));
        assert!(!simulation_unified_cursor_hide_os(
            BaseState::Simulation,
            false,
            true
        ));
        assert!(!simulation_unified_cursor_hide_os(
            BaseState::MainMenu,
            true,
            true
        ));
        assert!(simulation_unified_cursor_hide_region(
            Vec2::new(20.0, 200.0),
            &left,
            false
        ));
        assert!(!simulation_unified_cursor_hide_region(
            Vec2::new(640.0, 10.0),
            &left,
            false
        ));
    }

    #[test]
    fn cursor_hide_does_not_oscillate_over_armed_play_area() {
        let mut seen = [false; 4];
        for slot in &mut seen {
            *slot = armed_play_area_hides_os_cursor(BaseState::Simulation, true, true, true);
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
