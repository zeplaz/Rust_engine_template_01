//! Local egui texture binding cache — map consumers use [`crate::gui::map_view::MapViewTextureCache`].

pub use crate::gui::map_view::{
    reset_map_view_texture_frame as reset_hud_egui_texture_frame,
    MapViewTextureBinding as CachedEguiTextureBinding, MapViewTextureCache as HudEguiTextureCache,
};
