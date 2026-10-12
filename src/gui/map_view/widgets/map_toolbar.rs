//! Shared map toolbar for preview + minimap consumers.


use crate::gui::editor::world_preview::layers::PreviewLayers;
use crate::gui::editor::world_preview::{PreviewCameraState, PreviewRenderMode};
use crate::gui::map_view::MapViewState as EditorViewport;
use crate::gui::map_view::MapViewState;
use crate::gui::map_presentation_controls::map_overlay_controls_ui;
use crate::gui::style::{muted_label, primary_label, section_heading, CmdHeadingStyle, UiPalette};
use crate::gui::{MinimapCameraBookmark, MinimapShellState};

#[derive(Clone, Copy, Debug, Default)]
pub struct MapToolbarConfig {
    pub show_follow: bool,
    pub show_bookmarks: bool,
    pub show_generation_tools: bool,
    pub show_render_mode: bool,
    pub show_zoom_reset: bool,
}

pub fn map_toolbar() {}

#[allow(dead_code)] // Called only from the retired world-preview egui toolbar.
pub fn map_toolbar_preview_zoom() {}

pub fn map_toolbar_minimap_zoom() {}
