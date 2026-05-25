//! Top strip: shared map controls + preview zoom / render mode.

use bevy_egui::egui;

use crate::gui::editor::world_preview::preview_render_contract::PreviewCameraState;
use crate::gui::map_view::{map_toolbar, map_toolbar_preview_zoom, MapToolbarConfig, MapViewState};
use crate::gui::style::UiPalette;

pub fn world_preview_toolbar(
    ui: &mut egui::Ui,
    view: &mut MapViewState,
    preview_cam: &mut PreviewCameraState,
    tex_w: u32,
    tex_h: u32,
    palette: &UiPalette,
) {
    map_toolbar(
        ui,
        view,
        palette,
        "world_preview",
        MapToolbarConfig {
            show_follow: true,
            show_bookmarks: true,
            show_generation_tools: false,
            show_render_mode: false,
            show_zoom_reset: false,
        },
    );
    map_toolbar_preview_zoom(
        ui,
        view,
        preview_cam,
        tex_w,
        tex_h,
        palette,
        MapToolbarConfig {
            show_follow: false,
            show_bookmarks: false,
            show_generation_tools: false,
            show_render_mode: true,
            show_zoom_reset: true,
        },
    );
}
