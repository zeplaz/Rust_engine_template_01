//! World preview egui sampling through the shared map-view backend.

use bevy_egui::{egui, EguiContexts};

use crate::gui::map_view::presentation::{MapViewInstanceId, MapViewReadyStates};
use crate::gui::map_view::projection::ResolvedMapViewFrames;
use crate::gui::map_view::texture_cache::MapViewTextureCache;

pub fn resolve_world_preview_egui_texture(
    contexts: &mut EguiContexts,
    frames: &ResolvedMapViewFrames,
    cache: &mut MapViewTextureCache,
    ready: &mut MapViewReadyStates,
    interaction_frozen: bool,
) -> Option<egui::TextureId> {
    if !ready.world_preview.ready_to_bind() {
        return None;
    }
    let frame = frames.get(MapViewInstanceId::WorldPreview);
    if frame.texture_source.handle() == &bevy::prelude::Handle::default() {
        cache.binding_mut(MapViewInstanceId::WorldPreview).clear();
        return None;
    }
    let tex_id = cache
        .binding_mut(MapViewInstanceId::WorldPreview)
        .resolve(contexts, frame, interaction_frozen)?;
    ready.mark_frame_committed(MapViewInstanceId::WorldPreview);
    Some(tex_id)
}
