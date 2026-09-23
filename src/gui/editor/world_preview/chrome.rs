//! World-gen / preview chrome gates, D01–D07 layout witnesses, and open/dismiss helpers.

use bevy::prelude::*;
use bevy_egui::egui;

use crate::engine::states::{BaseState, WorldGenFlowState};
use crate::engine::{AppState, WorldGenChromeLatch};

use super::preview_lifecycle::WorldPreviewLifecycle;

/// D-01: single egui workspace hosts map chrome; generator is a slide sheet (not a second float).
pub const WORLD_PREVIEW_UNIFIED_WORKSPACE: bool = true;

/// Toggles for the World Preview egui workspace window.
#[derive(Resource)]
pub struct WorldPreviewUiState {
    pub window_open: bool,
    pub last_window_rect: Option<egui::Rect>,
    pub last_viewport_rect: Option<egui::Rect>,
    /// D-07: corner overview painted on map viewport last frame.
    pub d07_corner_inset_active: bool,
    pub d07_inset_side_px: f32,
}

impl Default for WorldPreviewUiState {
    fn default() -> Self {
        Self {
            window_open: false,
            last_window_rect: None,
            last_viewport_rect: None,
            d07_corner_inset_active: false,
            d07_inset_side_px: 0.0,
        }
    }
}

/// True when procedural world-gen UI should stay available (preview + generator panels).
#[must_use]
pub fn world_gen_flow_expects_chrome(flow: WorldGenFlowState) -> bool {
    matches!(
        flow,
        WorldGenFlowState::NewWorldSetup
            | WorldGenFlowState::PreviewReady
            | WorldGenFlowState::FullReady
    )
}

/// True while the operator is in the world-generator UX lane (not in-game / paused gameplay).
#[must_use]
pub fn world_gen_ux_app_active(app: Res<State<AppState>>) -> bool {
    matches!(*app.get(), AppState::WorldGen)
}

/// Editor / world-gen flows only — never while in active gameplay simulation.
#[must_use]
pub fn world_gen_editor_chrome_allowed(
    base: Res<State<BaseState>>,
    app: Res<State<AppState>>,
) -> bool {
    world_gen_ux_app_active(app) && !matches!(*base.get(), BaseState::Simulation)
}

/// True when D-01 unified workspace shell is active (map + generator slide sheet, one float).
#[must_use]
pub fn world_preview_unified_workspace(preview_ui: &WorldPreviewUiState) -> bool {
    WORLD_PREVIEW_UNIFIED_WORKSPACE && preview_ui.window_open
}

/// UI-WP-LAYOUT-002 / D-04: map dim when generator slide sheet is open (~40% opacity).
pub const D04_MAP_DIM_ALPHA: u8 = 102;

/// D-04 sheet width as fraction of workspace client width (35–40% spec).
pub const D04_SHEET_WIDTH_FRAC: f32 = 0.375;

pub const D04_SHEET_WIDTH_MIN: f32 = 400.0;

pub const D04_SHEET_WIDTH_MAX: f32 = 720.0;

/// D-02 — map viewport should occupy ≥ this fraction of workspace client area (sheet closed).
pub const D02_MAP_MIN_AREA_FRAC: f32 = 0.65;

pub const D02_HD_BASELINE_W: f32 = 1280.0;

pub const D02_HD_BASELINE_H: f32 = 720.0;

/// Approximate chrome heights for dominance math (toolbar + status).
pub const D02_TOOLBAR_H_PX: f32 = 40.0;

pub const D02_STATUS_H_PX: f32 = 24.0;

pub const D02_SIDEBAR_MIN_W: f32 = 160.0;

/// Max left sidebar width so map keeps ≥ **65%** area at HD baseline (sheet closed).
#[must_use]
pub fn d02_sidebar_max_width_px(workspace_w: f32) -> f32 {
    ((workspace_w * (1.0 - D02_MAP_MIN_AREA_FRAC)) - 32.0)
        .clamp(D02_SIDEBAR_MIN_W, 220.0)
}

#[must_use]
pub fn d02_map_area_fraction(
    workspace_w: f32,
    workspace_h: f32,
    sidebar_w: f32,
    sheet_w: f32,
) -> f32 {
    let map_w = (workspace_w - sidebar_w - sheet_w).max(0.0);
    let map_h = (workspace_h - D02_TOOLBAR_H_PX - D02_STATUS_H_PX).max(0.0);
    let total = (workspace_w * workspace_h).max(1.0);
    (map_w * map_h) / total
}

#[must_use]
pub fn d02_layout_witness(
    workspace_w: f32,
    workspace_h: f32,
    sidebar_w: f32,
    sheet_open: bool,
    sheet_w: f32,
) -> bool {
    let sheet = if sheet_open { sheet_w } else { 0.0 };
    d02_map_area_fraction(workspace_w, workspace_h, sidebar_w, sheet) >= D02_MAP_MIN_AREA_FRAC
}

#[must_use]
pub fn d02_layout_witness_hd_baseline_sheet_closed() -> bool {
    d02_layout_witness(D02_HD_BASELINE_W, D02_HD_BASELINE_H, 180.0, false, 0.0)
}

#[must_use]
pub fn d04_sheet_width_px(workspace_width: f32) -> f32 {
    (workspace_width * D04_SHEET_WIDTH_FRAC).clamp(D04_SHEET_WIDTH_MIN, D04_SHEET_WIDTH_MAX)
}

#[must_use]
pub fn d04_layout_witness(
    unified_workspace: bool,
    generator_sheet_open: bool,
    sheet_width_px: f32,
) -> bool {
    unified_workspace
        && sheet_width_px >= D04_SHEET_WIDTH_MIN
        && sheet_width_px <= D04_SHEET_WIDTH_MAX
        && (!generator_sheet_open
            || (sheet_width_px >= D04_SHEET_WIDTH_MIN && D04_MAP_DIM_ALPHA == 102))
}

/// D-07 A: corner overview inset 120–160px (not sidebar thumb).
pub const D07_INSET_SIDE_MIN: f32 = 120.0;

pub const D07_INSET_SIDE_MAX: f32 = 160.0;

pub const D07_INSET_SIDE_DEFAULT: f32 = 140.0;

#[must_use]
pub fn d07_inset_side_px() -> f32 {
    D07_INSET_SIDE_DEFAULT
}

#[must_use]
pub fn d07_layout_witness(
    corner_inset_on_map: bool,
    inset_side_px: f32,
    sidebar_minimap_removed: bool,
) -> bool {
    sidebar_minimap_removed
        && corner_inset_on_map
        && inset_side_px >= D07_INSET_SIDE_MIN
        && inset_side_px <= D07_INSET_SIDE_MAX
}

/// Open the unified world-gen workspace (F8 / NewWorldSetup latch).
pub fn open_world_gen_workspace(
    world_gen_ui: &mut crate::gui::editor::world_gen_ui::WorldGenUiState,
    preview_ui: &mut WorldPreviewUiState,
) {
    world_gen_ui.visible = true;
    preview_ui.window_open = true;
    if WORLD_PREVIEW_UNIFIED_WORKSPACE {
        world_gen_ui.generator_sheet_open = true;
    }
}

/// Open generator + preview once when entering a new world-build flow (not every frame).
pub fn open_world_gen_chrome_on_new_world_setup(
    base: Res<State<BaseState>>,
    app: Res<State<AppState>>,
    latch: Res<WorldGenChromeLatch>,
    mut world_gen_ui: ResMut<crate::gui::editor::world_gen_ui::WorldGenUiState>,
    mut preview_ui: ResMut<WorldPreviewUiState>,
) {
    if !world_gen_editor_chrome_allowed(base, app) || latch.full_ready_dismissed {
        return;
    }
    let opened = !world_gen_ui.visible || !preview_ui.window_open;
    open_world_gen_workspace(&mut world_gen_ui, &mut preview_ui);
    if opened {
        crate::engine::worldgen_chrome_debug::log_chrome_open(
            "on_enter_new_world_setup",
            world_gen_ui.visible,
            preview_ui.window_open,
        );
    }
}

/// Close generator + preview chrome and park lifecycle (load-in / FullReady dismiss).
pub fn dismiss_world_gen_preview_chrome(
    world_gen_ui: &mut crate::gui::editor::world_gen_ui::WorldGenUiState,
    preview_ui: &mut WorldPreviewUiState,
    lifecycle: &mut WorldPreviewLifecycle,
    latch: &mut crate::engine::WorldGenChromeLatch,
    reason: &'static str,
) {
    latch.mark_full_ready_dismissed();
    world_gen_ui.visible = false;
    world_gen_ui.generator_sheet_open = false;
    preview_ui.window_open = false;
    lifecycle.park_uninitialized();
    crate::engine::worldgen_chrome_debug::log_chrome_dismiss(
        reason,
        latch.full_ready_dismissed,
        world_gen_ui.visible,
        preview_ui.window_open,
    );
}

/// World-gen / preview egui may draw only in [`AppState::WorldGen`] with an open panel/window.
#[must_use]
pub fn world_gen_chrome_may_render(
    app: Res<State<AppState>>,
    preview_ui: &WorldPreviewUiState,
    world_gen: &crate::gui::editor::world_gen_ui::WorldGenUiState,
) -> bool {
    if WORLD_PREVIEW_UNIFIED_WORKSPACE {
        world_gen_ux_app_active(app) && preview_ui.window_open
    } else {
        world_gen_ux_app_active(app) && (preview_ui.window_open || world_gen.visible)
    }
}

/// World preview GPU/CPU work runs only while the operator can see preview or generator chrome.
#[must_use]
pub fn world_preview_chrome_active(
    app: Res<State<AppState>>,
    preview_ui: Res<WorldPreviewUiState>,
    world_gen: Res<crate::gui::editor::world_gen_ui::WorldGenUiState>,
) -> bool {
    world_gen_chrome_may_render(app, preview_ui.as_ref(), world_gen.as_ref())
}

/// FINISH-UX-06/07: preview raster/lifecycle only when UX world-gen is active, chrome visible, no spike guard.
#[must_use]
pub fn world_preview_pipeline_enabled(
    app: Res<State<AppState>>,
    worldgen: Res<State<crate::engine::WorldGenState>>,
    guard: Res<crate::engine::UxFrameSpikeGuard>,
    preview_ui: Res<WorldPreviewUiState>,
    world_gen: Res<crate::gui::editor::world_gen_ui::WorldGenUiState>,
) -> bool {
    crate::engine::worldgen_preview_systems_enabled(worldgen.get())
        && !guard.suppress_preview_this_frame
        && world_gen_chrome_may_render(app, preview_ui.as_ref(), world_gen.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unified_workspace_open_syncs_flags() {
        let mut world_gen = crate::gui::editor::world_gen_ui::WorldGenUiState::default();
        let mut preview = WorldPreviewUiState::default();
        open_world_gen_workspace(&mut world_gen, &mut preview);
        assert!(world_gen.visible);
        assert!(preview.window_open);
        assert!(world_gen.generator_sheet_open);
    }

    #[test]
    fn unified_workspace_chrome_may_render_uses_preview_only() {
        let preview = WorldPreviewUiState {
            window_open: true,
            ..Default::default()
        };
        assert!(world_preview_unified_workspace(&preview));
    }

    #[test]
    fn d02_map_dominance_hd_baseline_sheet_closed() {
        assert!(d02_layout_witness_hd_baseline_sheet_closed());
        assert!(d02_sidebar_max_width_px(D02_HD_BASELINE_W) <= 220.0);
        let w = crate::gui::editor::world_preview::wave_p_witness::build_d02_layout_witness(
            D02_HD_BASELINE_W,
            D02_HD_BASELINE_H,
            180.0,
            false,
            0.0,
        );
        assert!(w["ui_wp_layout_d02_opt_green"].as_bool().unwrap_or(false));
    }
}
