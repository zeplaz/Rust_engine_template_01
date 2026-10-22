//! Shared map toolbar for preview + minimap consumers.



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
