//! Simulation minimap **presentation** state — view over existing CPU raster / shared overlays.
//!
//! Does not own terrain or fire extraction; [`crate::render::TileWorldFallbackState`] + [`crate::render::SharedOverlayFieldBuffers`] remain authoritative.

use bevy::prelude::*;
use bevy_egui::egui;

/// Title bar height for GPU minimap widget (MINIMAP-WIDGET-IMPL-001).
pub const MINIMAP_TITLE_BAR_H_PX: f32 = 24.0;
const MINIMAP_EDGE_RAIL_PX: f32 = 6.0;
const MINIMAP_RESIZE_GRIP_PX: f32 = 14.0;

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

/// Default minimap overlay toggles for **operator Simulation** (VX-P0-01).
///
/// Fire heat stays **off** so ambient `chunk_fire_heat` does not paint a full-map pink wash at
/// strategic zoom; operators enable **Fire heat** in the overlay tray or diagnostics when needed.
/// M2 logistics / construction / ecology remain on for play-readability.
#[must_use]
pub const fn simulation_minimap_overlay_defaults() -> MinimapOverlayMask {
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
            show_tactical_viewport_frame: true,
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

    /// Seed Bevy minimap chrome before the first egui layout pass (sim has no editor product shell).
    pub fn bootstrap_simulation_layout_rect(&mut self, window_width: f32, window_height: f32) {
        if self.last_window_rect.is_none() && self.last_image_rect.is_none() {
            self.last_window_rect = Some(simulation_minimap_bootstrap_rect(
                window_width,
                window_height,
                self.viewport_size,
            ));
        }
        self.sync_panel_viewport_suggestion_from_layout();
    }

    /// GPU compositor resize reads [`panel_viewport_suggestion_*`] — sim must set without egui layout.
    pub fn sync_panel_viewport_suggestion_from_layout(&mut self) {
        let rect = self
            .last_image_rect
            .or(self.last_body_rect)
            .or(self.last_window_rect)
            .unwrap_or_else(|| simulation_minimap_bootstrap_rect(1280.0, 720.0, self.viewport_size));
        if !self.visible || self.minimized {
            self.panel_viewport_suggestion_active = false;
            return;
        }
        self.panel_viewport_suggestion_active = true;
        self.panel_viewport_suggestion_logical_size =
            Vec2::new(rect.width().max(180.0), rect.height().max(160.0));
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

    /// Sync pointer hit-test rects from the outer Bevy chrome box (logical px).
    ///
    /// Does **not** overwrite [`panel_screen_origin`] — drag uses origin + [`sync_layout_rects_from_panel_origin`].
    pub fn apply_chrome_outer_rect(&mut self, min_x: f32, min_y: f32, outer_w: f32, outer_h: f32) {
        let window = egui::Rect::from_min_size(
            egui::pos2(min_x, min_y),
            egui::vec2(outer_w.max(1.0), outer_h.max(1.0)),
        );
        self.apply_window_rect_layout(window);
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
    fn bootstrap_activates_panel_viewport_suggestion() {
        let mut state = MinimapShellState::default();
        state.bootstrap_simulation_layout_rect(1280.0, 720.0);
        assert!(state.panel_viewport_suggestion_active);
        assert!(state.panel_viewport_suggestion_logical_size.x >= 180.0);
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
}
