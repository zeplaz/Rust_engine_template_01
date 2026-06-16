//! Rect-based pointer gate for simulation Bevy chrome over the full-window map hole.
//!
//! The map viewport fill spans the window; left stack, ops strips, minimap, and context tray
//! overlay it without shrinking the measured hole. Map picks and camera wheel must respect these
//! regions so ghost placement and hover targets align with visible chrome.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::window::{CursorOptions, PrimaryWindow};
use bevy_egui::{egui, EguiContexts};

use crate::engine::states::BaseState;

use crate::gui::hud::layout_store::HudLayoutStore;
use crate::gui::hud::panel_state::HudPanelState;
use crate::gui::hud::shell_framework::{HudDockRegistry, ProductShellWidgetId};
use crate::gui::hud::simulation_shell_phase2::{
    command_left_stack_footprint_px, sim_build_rail_submenu_block_rect, ContextTrayState,
    CONTEXT_TRAY_BODY_H_PX, CONTEXT_TRAY_PEEK_BODY_H_PX, CONTEXT_TRAY_TAB_H_PX,
};
use crate::construction::{ActiveBuildTool, BuildStripState, ToolContext};
use crate::gui::{
    CommandLeftStackState, MinimapShellState, SimulationMapViewport,
    SIMULATION_MAP_VIEWPORT_TOP_CHROME_PX,
};
use crate::gui::in_game_hud::CENTER_ROW_EDGE_PAD_PX;

/// Per-frame simulation map pointer routing (chrome vs play area).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct SimulationMapPointerGate {
    pub cursor: Vec2,
    pub window_logical: Vec2,
    pub chrome_blocks: bool,
    pub in_play_area: bool,
    /// Set after egui pass — floating HUD panels / menus over the map hole.
    pub egui_blocks: bool,
    /// Last frame OS cursor visibility (debug / witness).
    pub os_cursor_visible: bool,
}

pub fn sync_simulation_map_pointer_gate_system(
    window: Query<&Window, With<PrimaryWindow>>,
    map_vp: Res<SimulationMapViewport>,
    left_stack: Res<CommandLeftStackState>,
    minimap: Res<MinimapShellState>,
    context_tray: Res<ContextTrayState>,
    mut gate: ResMut<SimulationMapPointerGate>,
) {
    let Ok(w) = window.single() else {
        *gate = SimulationMapPointerGate::default();
        return;
    };
    let cursor = w.cursor_position().unwrap_or(Vec2::ZERO);
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
    gate.in_play_area = if map_vp.is_adequate_for_camera() {
        (!map_vp.valid || map_vp.contains_cursor(cursor)) && !gate.chrome_blocks
    } else {
        false
    };
    gate.egui_blocks = false;
    gate.os_cursor_visible = true;
}

/// After egui HUD: block map picks / hide unified cursor when floating panels capture the pointer.
pub fn finalize_simulation_map_pointer_gate_egui_system(
    mut gate: ResMut<SimulationMapPointerGate>,
    dock: Res<HudDockRegistry>,
    layout: Res<HudLayoutStore>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    mut contexts: EguiContexts,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let egui_blocks = ctx.wants_pointer_input()
        || cursor_over_visible_hud_widget(gate.cursor, &dock, &layout)
        || sim_build_rail_submenu_blocks_pointer(strip.as_ref(), tool.as_ref(), gate.cursor);
    gate.egui_blocks = egui_blocks;
    if egui_blocks {
        gate.chrome_blocks = true;
        gate.in_play_area = false;
    }
}

#[inline]
fn sim_build_rail_submenu_blocks_pointer(
    strip: &BuildStripState,
    tool: &ActiveBuildTool,
    cursor: Vec2,
) -> bool {
    if strip.active == ToolContext::None {
        return false;
    }
    if !tool.residential_menu_open
        && !tool.commercial_menu_open
        && !tool.industrial_menu_open
        && !tool.utilities_menu_open
        && !tool.mock_shapes_menu_open
    {
        return false;
    }
    sim_build_rail_submenu_block_rect().contains(egui::pos2(cursor.x, cursor.y))
}

#[inline]
fn cursor_over_visible_hud_widget(
    cursor: Vec2,
    dock: &HudDockRegistry,
    layout: &HudLayoutStore,
) -> bool {
    let pt = egui::pos2(cursor.x, cursor.y);
    for widget in ProductShellWidgetId::ALL {
        let slot = dock.slot(widget);
        if !slot.visible || slot.minimized {
            continue;
        }
        let frame = layout.frame(widget);
        if !frame.initialized {
            continue;
        }
        let rect = egui::Rect::from_min_size(
            egui::pos2(frame.pos.x, frame.pos.y),
            egui::vec2(frame.size.x.max(1.0), frame.size.y.max(1.0)),
        );
        if rect.contains(pt) {
            return true;
        }
    }
    false
}

/// **TRIAGE-CURSOR-UNIFY-001** — hide OS cursor over sim play area; picks use gate cursor coords.
#[inline]
#[must_use]
pub fn simulation_unified_cursor_hide_os(base: BaseState, in_play_area: bool) -> bool {
    matches!(base, BaseState::Simulation) && in_play_area
}

/// **TRIAGE-CURSOR-UNIFY-001** — OS cursor hidden over sim play area only.
#[must_use]
pub fn triage_cursor_unify_001_witness_green() -> bool {
    triage_cursor_unify_001_self_check().is_ok()
}

fn triage_cursor_unify_001_self_check() -> Result<(), &'static str> {
    if !simulation_unified_cursor_hide_os(BaseState::Simulation, true) {
        return Err("hide_sim_play");
    }
    if simulation_unified_cursor_hide_os(BaseState::Simulation, false) {
        return Err("show_sim_chrome");
    }
    if simulation_unified_cursor_hide_os(BaseState::MainMenu, true) {
        return Err("show_menu");
    }
    Ok(())
}

pub fn apply_simulation_unified_cursor_system(
    base: Res<State<BaseState>>,
    mut gate: ResMut<SimulationMapPointerGate>,
    mut cursors: Query<&mut CursorOptions, With<PrimaryWindow>>,
) {
    let hide_os = simulation_unified_cursor_hide_os(*base.get(), gate.in_play_area);
    gate.os_cursor_visible = !hide_os;
    for mut cursor in &mut cursors {
        cursor.visible = !hide_os;
    }
}

#[inline]
fn rect_contains(rect: egui::Rect, cursor: Vec2) -> bool {
    rect.contains(egui::pos2(cursor.x, cursor.y))
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
    let rect = sim_build_rail_submenu_block_rect();
    let cursor = Vec2::new(rect.center().x, rect.center().y);
    sim_build_rail_submenu_blocks_pointer(&strip, &tool, cursor)
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

        assert!(simulation_unified_cursor_hide_os(BaseState::Simulation, true));
        assert!(!simulation_unified_cursor_hide_os(BaseState::Simulation, false));
        assert!(!simulation_unified_cursor_hide_os(BaseState::MainMenu, true));
    }
}
