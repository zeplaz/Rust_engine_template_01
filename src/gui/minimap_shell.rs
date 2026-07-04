//! Simulation minimap **presentation** state — view over existing CPU raster / shared overlays.
//!
//! Does not own terrain or fire extraction; [`crate::render::TileWorldFallbackState`] + [`crate::render::SharedOverlayFieldBuffers`] remain authoritative.

use bevy::prelude::*;
use bevy_egui::egui;

/// Title bar height for GPU minimap widget (MINIMAP-WIDGET-IMPL-001).
pub const MINIMAP_TITLE_BAR_H_PX: f32 = 24.0;
/// Body inset from the outer panel edge (matches the rail hit-test rects).
pub const MINIMAP_EDGE_RAIL_PX: f32 = 6.0;
/// Bottom-right resize grip square side (matches the resize hit-test rect).
pub const MINIMAP_RESIZE_GRIP_PX: f32 = 14.0;

/// Logical client size — matches [`Window::width`]/[`Window::height`] and [`Window::cursor_position`].
#[inline]
#[must_use]
pub fn minimap_window_logical_size(window: &Window) -> Vec2 {
    Vec2::new(window.width().max(1.0), window.height().max(1.0))
}

/// Pointer position in the same space as [`minimap_window_logical_size`] and layout rects.
#[inline]
#[must_use]
pub fn minimap_cursor_logical(window: &Window) -> Option<Vec2> {
    window.cursor_position()
}

/// Edge rail hit targets on the minimap widget body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinimapEdge {
    Top,
    Bottom,
    Left,
    Right,
}

/// How the minimap panel is hosted (embedded HUD vs detached window vs fullscreen).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapPresentationMode {
    #[default]
    Embedded,
    Detached,
    Fullscreen,
}

/// View toggles for layers already present in the shared overlay / fallback raster path.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MinimapOverlayMask {
    /// Chunk fire heat tint from [`crate::render::SharedOverlayFieldBuffers`].
    pub fire_heat: bool,
    /// Corridor traffic heat from [`crate::render::LogisticsVisualSnapshot`] (M2).
    pub logistics_heat: bool,
    /// Corridor / site construction phase — design **M2** (coder slice **UI-P3-M3-001**, not design M3).
    pub construction_heat: bool,
    /// Ecology macro band — design **M2** (coder slice **UI-P3-M3-001**; fog/EW = **UI-P3-M4-001**).
    pub ecology_heat: bool,
    /// Fog-of-war veil — design **M3** (**UI-P3-M4-001**).
    pub fow: bool,
    /// EW / denial stress — design **M3** (**UI-P3-M4-001**).
    pub ew: bool,
    /// Unit aggregation glyphs — **UI-P3-M3-UNITS-001** (M3-03).
    pub units: bool,
    /// Replay scrub tick — **UI-P3-M3-REPLAY-001** (M3-04).
    pub replay_scrub: bool,
}

impl MinimapOverlayMask {
    /// True when any GPU heat layer upload / compositor blend is required beyond terrain.
    #[must_use]
    pub const fn needs_gpu_heat_upload(self) -> bool {
        self.fire_heat
            || self.logistics_heat
            || self.construction_heat
            || self.ecology_heat
            || self.fow
            || self.ew
            || self.units
            || self.replay_scrub
    }
}

/// Default minimap overlay toggles for **operator Simulation** (VX-P0-01).
///
/// Fire/ecology stay **off** by default so witness/ambient heat does not wash the strategic panel;
/// logistics remains on for corridor readability. Operators enable layers in the overlay tray.
#[must_use]
pub const fn simulation_minimap_overlay_defaults() -> MinimapOverlayMask {
    MinimapOverlayMask {
        fire_heat: false,
        logistics_heat: true,
        construction_heat: false,
        ecology_heat: false,
        fow: false,
        ew: false,
        units: false,
        replay_scrub: false,
    }
}

/// Full M2/M3 overlay mask for **`--test visual`** / lib witness refresh only — not operator play.
#[must_use]
pub const fn minimap_overlay_witness_harness() -> MinimapOverlayMask {
    MinimapOverlayMask {
        fire_heat: false,
        logistics_heat: true,
        construction_heat: true,
        ecology_heat: true,
        fow: true,
        ew: true,
        units: true,
        replay_scrub: true,
    }
}

/// Consumer path for minimap pixels — **no** alternate ECS extraction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapPresentationSource {
    /// **Effects / dev / explicit opt-in** — layered CPU raster (`fallback.minimap_image`).
    /// Not used on the default simulation HUD path when GPU compositor is on.
    #[default]
    SharedCpuRaster,
    /// **Main simulation product path** — GPU compositor RT (Bevy chrome + compositor pass).
    SharedRenderTargetImage,
}

/// Camera follow posture for minimap focus (presentation only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MinimapFollowMode {
    #[default]
    Free,
    FollowCamera,
    FollowBookmark,
}

#[derive(Clone, Debug)]
pub struct MinimapCameraBookmark {
    pub label: String,
    pub world: Vec2,
    pub zoom: f32,
}

#[must_use]
pub const fn native_minimap_window_supported() -> bool {
    cfg!(feature = "hud_native_minimap_window")
}

/// Player-facing minimap shell — egui hosts controls; texture comes from fallback raster.
#[derive(Resource, Clone, Debug)]
pub struct MinimapShellState {
    pub visible: bool,
    pub detached: bool,
    pub minimized: bool,
    pub zoom: f32,
    pub zoom_target: f32,
    pub world_center: Vec2,
    pub viewport_size: Vec2,
    pub mode: MinimapPresentationMode,
    pub presentation_source: MinimapPresentationSource,
    pub compositor_revision: u64,
    pub cached_texture_revision: u64,
    pub native_window_requested: bool,
    pub last_window_rect: Option<egui::Rect>,
    pub last_image_rect: Option<egui::Rect>,
    pub last_body_rect: Option<egui::Rect>,
    pub title_bar_rect: Option<egui::Rect>,
    pub top_rail_rect: Option<egui::Rect>,
    pub bottom_rail_rect: Option<egui::Rect>,
    pub left_rail_rect: Option<egui::Rect>,
    pub right_rail_rect: Option<egui::Rect>,
    pub resize_grip_rect: Option<egui::Rect>,
    pub panel_screen_origin: Option<Vec2>,
    pub last_fit_body_size: Vec2,
    /// Non-authoritative panel extent from the latest egui layout pass.
    pub panel_viewport_suggestion_active: bool,
    pub panel_viewport_suggestion_logical_size: Vec2,
    pub pending_camera_focus_world: Option<Vec2>,
    pub pending_camera_focus_zoom: Option<f32>,
    pub diagnostic_ui_wrote_camera: bool,
    pub diagnostic_camera_drove_ui: bool,
    /// Draw gold frame for [`crate::gui::ViewId::WorldMain`] visible bounds on the minimap image.
    pub show_tactical_viewport_frame: bool,
    /// DES-MINIMAP-VEG-LEGEND-001 — ecology topology legend strip.
    pub topology_legend_expanded: bool,
    pub topology_legend_user_toggled: bool,
}

impl Default for MinimapShellState {
    fn default() -> Self {
        Self {
            visible: true,
            detached: false,
            minimized: false,
            zoom: 0.85,
            zoom_target: 0.85,
            world_center: Vec2::ZERO,
            viewport_size: Vec2::new(260.0, 220.0),
            mode: MinimapPresentationMode::Embedded,
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            compositor_revision: 0,
            cached_texture_revision: 0,
            native_window_requested: false,
            last_window_rect: None,
            last_image_rect: None,
            last_body_rect: None,
            title_bar_rect: None,
            top_rail_rect: None,
            bottom_rail_rect: None,
            left_rail_rect: None,
            right_rail_rect: None,
            resize_grip_rect: None,
            panel_screen_origin: None,
            last_fit_body_size: Vec2::ZERO,
            panel_viewport_suggestion_active: false,
            panel_viewport_suggestion_logical_size: Vec2::new(260.0, 220.0),
            pending_camera_focus_world: None,
            pending_camera_focus_zoom: None,
            diagnostic_ui_wrote_camera: false,
            diagnostic_camera_drove_ui: false,
            show_tactical_viewport_frame: false,
            topology_legend_expanded: false,
            topology_legend_user_toggled: false,
        }
    }
}

impl MinimapShellState {
    #[inline]
    pub fn clamp_zoom(&mut self) {
        self.zoom = self.zoom.clamp(0.35, 4.0);
    }

    pub fn clamp_viewport(&mut self) {
        self.viewport_size.x = self.viewport_size.x.clamp(180.0, 720.0);
        self.viewport_size.y = self.viewport_size.y.clamp(160.0, 720.0);
    }

    pub fn tick_smooth_zoom(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let k = 1.0 - (-dt * 16.0).exp();
        self.zoom += (self.zoom_target - self.zoom) * k;
        self.clamp_zoom();
    }

    pub fn focus_world_tile(&mut self, world: Vec2) {
        self.world_center = world;
    }

    /// Seed Bevy minimap chrome before the first GPU layout pass (sim has no editor product shell).
    ///
    /// Must populate **all** hit-test rects (title bar, body, rails, grip) on frame 0 — the old path
    /// only set `last_window_rect`, leaving `title_bar_rect` / `last_body_rect` empty until a later
    /// chrome sync, so drag/resize/wheel missed input on the first seconds of Simulation.
    pub fn bootstrap_simulation_layout_rect(&mut self, window_width: f32, window_height: f32) {
        if self.panel_screen_origin.is_none() {
            let rect = simulation_minimap_bootstrap_rect(
                window_width,
                window_height,
                self.viewport_size,
            );
            self.panel_screen_origin = Some(Vec2::new(rect.min.x, rect.min.y));
        }
        self.sync_layout_rects_from_panel_origin();
        self.sync_panel_viewport_suggestion_from_layout();
    }

    /// GPU compositor resize reads [`panel_viewport_suggestion_*`] — sim must set without egui layout.
    pub fn sync_panel_viewport_suggestion_from_layout(&mut self) {
        if !self.visible || self.minimized {
            self.panel_viewport_suggestion_active = false;
            return;
        }
        self.panel_viewport_suggestion_active = true;
        // Authoritative content size — never derive from chrome body rects (sub-pixel jitter
        // caused +1px RT resize churn and compositor rebind tearing).
        let logical = Vec2::new(
            self.viewport_size.x.round().max(180.0),
            self.viewport_size.y.round().max(160.0),
        );
        let prev = self.panel_viewport_suggestion_logical_size;
        if (prev - logical).length_squared() <= 1.0 {
            return;
        }
        self.panel_viewport_suggestion_logical_size = logical;
        crate::render::trace_minimap_size_writer(
            "shell.sync_from_layout",
            self.panel_viewport_suggestion_logical_size.x,
            self.panel_viewport_suggestion_logical_size.y,
        );
    }

    pub fn ensure_panel_screen_origin(&mut self, window_w: f32, window_h: f32) {
        if self.panel_screen_origin.is_some() {
            return;
        }
        let rect = self.last_window_rect.unwrap_or_else(|| {
            simulation_minimap_bootstrap_rect(window_w, window_h, self.viewport_size)
        });
        self.panel_screen_origin = Some(Vec2::new(rect.min.x, rect.min.y));
    }

    fn apply_window_rect_layout(&mut self, window: egui::Rect) {
        self.last_window_rect = Some(window);
        let title = egui::Rect::from_min_size(
            window.min,
            egui::vec2(window.width(), MINIMAP_TITLE_BAR_H_PX),
        );
        self.title_bar_rect = Some(title);
        let body_min = egui::pos2(
            window.min.x + MINIMAP_EDGE_RAIL_PX,
            window.min.y + MINIMAP_TITLE_BAR_H_PX + MINIMAP_EDGE_RAIL_PX,
        );
        let body_max = egui::pos2(
            window.max.x - MINIMAP_EDGE_RAIL_PX,
            window.max.y - MINIMAP_EDGE_RAIL_PX - MINIMAP_RESIZE_GRIP_PX,
        );
        let body = egui::Rect::from_min_max(body_min, body_max);
        self.last_body_rect = Some(body);
        self.last_image_rect = Some(body);
        // PERF-INSTR-VFX-001: this body rect (derived from the outer `window` rect) becomes the
        // suggestion via `sync_panel_viewport_suggestion_from_layout`. Trace it to see if the outer
        // window rect (`last_window_rect`) is the +2px/frame ratchet source feeding the body.
        crate::render::trace_minimap_size_writer(
            "shell.apply_window_rect_body",
            body.width(),
            body.height(),
        );
        self.top_rail_rect = Some(egui::Rect::from_min_max(
            egui::pos2(window.min.x, title.max.y),
            egui::pos2(window.max.x, body_min.y),
        ));
        self.bottom_rail_rect = Some(egui::Rect::from_min_max(
            egui::pos2(window.min.x, body_max.y),
            egui::pos2(window.max.x, window.max.y - MINIMAP_RESIZE_GRIP_PX),
        ));
        self.left_rail_rect = Some(egui::Rect::from_min_max(
            egui::pos2(window.min.x, body_min.y),
            egui::pos2(body_min.x, body_max.y),
        ));
        self.right_rail_rect = Some(egui::Rect::from_min_max(
            egui::pos2(body_max.x, body_min.y),
            egui::pos2(window.max.x, body_max.y),
        ));
        self.resize_grip_rect = Some(egui::Rect::from_min_size(
            egui::pos2(
                window.max.x - MINIMAP_RESIZE_GRIP_PX,
                window.max.y - MINIMAP_RESIZE_GRIP_PX,
            ),
            egui::vec2(MINIMAP_RESIZE_GRIP_PX, MINIMAP_RESIZE_GRIP_PX),
        ));
    }

    pub fn sync_layout_rects_from_panel_origin(&mut self) {
        let Some(origin) = self.panel_screen_origin else {
            return;
        };
        let size = self.viewport_size;
        let window = egui::Rect::from_min_size(
            egui::pos2(origin.x, origin.y),
            egui::vec2(size.x.max(120.0), size.y.max(120.0)),
        );
        self.apply_window_rect_layout(window);
    }

    pub fn enforce_square_viewport(&mut self) {
        let s = self.viewport_size.x.max(self.viewport_size.y);
        self.viewport_size = Vec2::splat(s.clamp(120.0, 480.0));
        self.clamp_viewport();
    }
}

/// Default floating minimap placement for simulation (matches editor shell top-right bias).
#[must_use]
pub fn simulation_minimap_bootstrap_rect(
    window_width: f32,
    window_height: f32,
    viewport: Vec2,
) -> egui::Rect {
    let margin = 12.0;
    let top = 64.0;
    let w = viewport.x.min(window_width * 0.42);
    let h = viewport.y.min(window_height * 0.5);
    let x = (window_width - w - margin).max(margin);
    let y = (top + margin).min(window_height - h - margin);
    egui::Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h))
}

/// Map normalized click UV (0..1) on the minimap image to world tile coordinates.
#[must_use]
pub fn minimap_uv_to_world_tile(uv: Vec2, world_width: f32, world_height: f32) -> Vec2 {
    let u = uv.x.clamp(0.0, 1.0);
    let v = uv.y.clamp(0.0, 1.0);
    Vec2::new(u * world_width, v * world_height)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_harness_overlays_richer_than_simulation_defaults() {
        let sim = simulation_minimap_overlay_defaults();
        let harness = minimap_overlay_witness_harness();
        assert_eq!(sim.fire_heat, harness.fire_heat);
        assert!(harness.construction_heat);
        assert!(harness.ecology_heat);
        assert!(harness.fow);
        assert!(harness.ew);
        assert!(harness.units);
        assert!(harness.replay_scrub);
        assert!(!sim.construction_heat);
        assert!(!sim.ecology_heat);
    }

    #[test]
    fn simulation_defaults_skip_fire_heat_upload() {
        let defaults = simulation_minimap_overlay_defaults();
        assert!(!defaults.fire_heat);
        assert!(defaults.needs_gpu_heat_upload());
        let terrain_only = MinimapOverlayMask {
            fire_heat: false,
            logistics_heat: false,
            ..Default::default()
        };
        assert!(!terrain_only.needs_gpu_heat_upload());
    }

    #[test]
    fn bootstrap_activates_panel_viewport_suggestion() {
        let mut state = MinimapShellState::default();
        state.bootstrap_simulation_layout_rect(1280.0, 720.0);
        assert!(state.panel_viewport_suggestion_active);
        assert!(state.panel_viewport_suggestion_logical_size.x >= 180.0);
        assert!(state.panel_screen_origin.is_some());
        assert!(state.title_bar_rect.is_some());
        assert!(state.last_body_rect.is_some());
    }

    #[test]
    fn minimap_uv_corners_map_to_world_extent() {
        assert_eq!(
            minimap_uv_to_world_tile(Vec2::ZERO, 100.0, 50.0),
            Vec2::new(0.0, 0.0)
        );
        assert_eq!(
            minimap_uv_to_world_tile(Vec2::ONE, 100.0, 50.0),
            Vec2::new(100.0, 50.0)
        );
    }

    #[test]
    fn minimap_zoom_clamps() {
        let mut state = MinimapShellState {
            zoom: 10.0,
            ..Default::default()
        };
        state.clamp_zoom();
        assert_eq!(state.zoom, 4.0);
    }

    #[test]
    fn minimap_smooth_zoom_moves_toward_target() {
        let mut state = MinimapShellState {
            zoom: 1.0,
            zoom_target: 2.0,
            ..Default::default()
        };
        state.tick_smooth_zoom(0.1);
        assert!(state.zoom > 1.0 && state.zoom < 2.0);
    }

    #[test]
    fn bootstrap_rect_top_right_bias() {
        let r = simulation_minimap_bootstrap_rect(1280.0, 720.0, Vec2::new(260.0, 220.0));
        assert!(r.max.x <= 1280.0);
        assert!(r.min.y >= 64.0);
        assert!(r.width() > 100.0);
    }

    /// MINIMAP-WIDGET-IMPL-001 chrome geometry: a content rect yields a title bar pinned to the top,
    /// a body inset from the edges, and a resize grip in the bottom-right corner. These rects are
    /// what `sync_minimap_chrome_layout_system` renders the title-bar / grip children from.
    #[test]
    fn chrome_layout_rects_match_window_regions() {
        let mut state = MinimapShellState::default();
        state.viewport_size = Vec2::new(260.0, 220.0);
        state.panel_screen_origin = Some(Vec2::new(100.0, 80.0));
        state.sync_layout_rects_from_panel_origin();

        let window = state.last_window_rect.expect("content rect");
        let title = state.title_bar_rect.expect("title rect");
        let body = state.last_body_rect.expect("body rect");
        let grip = state.resize_grip_rect.expect("grip rect");

        // Title bar pinned to the top edge of the content, full width, fixed bar height.
        assert!((title.min.x - window.min.x).abs() < 0.01);
        assert!((title.min.y - window.min.y).abs() < 0.01);
        assert!((title.width() - window.width()).abs() < 0.01);
        assert!((title.height() - MINIMAP_TITLE_BAR_H_PX).abs() < 0.01);

        // Body sits below the title bar, inset by the edge rails — strictly inside the window.
        assert!(body.min.y >= title.max.y - 0.01);
        assert!(body.min.x > window.min.x);
        assert!(body.max.x < window.max.x);
        assert!(body.width() > 0.0 && body.height() > 0.0);

        // Resize grip is the bottom-right square of the content box.
        assert!((grip.max.x - window.max.x).abs() < 0.01);
        assert!((grip.max.y - window.max.y).abs() < 0.01);
        assert!((grip.width() - MINIMAP_RESIZE_GRIP_PX).abs() < 0.01);
    }

    /// MINIMAP-SIZE-AUTHORITY-001 regression guard: repeated chrome layout passes on an UNCHANGED
    /// window must produce a byte-stable content rect (no +Npx/frame ratchet). Pre-fix, feeding the
    /// outer (content + stroke pad) box back as the next content rect grew the panel every frame.
    #[test]
    fn static_window_layout_does_not_ratchet() {
        let mut state = MinimapShellState::default();
        state.viewport_size = Vec2::new(300.0, 240.0);
        state.panel_screen_origin = Some(Vec2::new(200.0, 120.0));

        state.sync_layout_rects_from_panel_origin();
        let first = state.last_window_rect.expect("content rect");
        let first_body = state.last_body_rect.expect("body rect");

        // Re-run the layout many times as the schedule would each frame for a static window.
        for _ in 0..120 {
            state.sync_layout_rects_from_panel_origin();
        }
        let last = state.last_window_rect.expect("content rect");
        let last_body = state.last_body_rect.expect("body rect");

        assert_eq!(first.width(), last.width(), "content width must not ratchet");
        assert_eq!(first.height(), last.height(), "content height must not ratchet");
        assert_eq!(first_body.width(), last_body.width(), "body width must not ratchet");
        assert_eq!(first_body.height(), last_body.height(), "body height must not ratchet");

        // The suggestion that feeds `resolve_minimap_panel_viewport` is likewise stable.
        state.sync_panel_viewport_suggestion_from_layout();
        let suggestion_a = state.panel_viewport_suggestion_logical_size;
        for _ in 0..16 {
            state.sync_layout_rects_from_panel_origin();
            state.sync_panel_viewport_suggestion_from_layout();
        }
        let suggestion_b = state.panel_viewport_suggestion_logical_size;
        assert_eq!(suggestion_a, suggestion_b, "panel size suggestion must settle");
    }
}
