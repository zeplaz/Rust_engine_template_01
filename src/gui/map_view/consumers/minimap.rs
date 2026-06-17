//! Minimap egui sampling through the shared map-view backend.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::gui::hud::shell_framework::{shell_widget_runs_egui, HudWidgetId};
use crate::gui::hud::HudDockRegistry;
use crate::gui::map_view::presentation::{MapViewInstanceId, MapViewReadyStates};
use crate::gui::map_view::projection::ResolvedMapViewFrames;
use crate::gui::map_view::texture_cache::MapViewTextureCache;
use crate::gui::{MinimapShellState};
use crate::render::{SimMinimapUiState, TileWorldFallbackState};

pub fn resolve_minimap_egui_texture(
    contexts: &mut EguiContexts,
    shell: &mut MinimapShellState,
    legacy: &mut SimMinimapUiState,
    dock: &mut HudDockRegistry,
    fallback: &TileWorldFallbackState,
    frames: &ResolvedMapViewFrames,
    cache: &mut MapViewTextureCache,
    ready: &mut MapViewReadyStates,
    interaction_frozen: bool,
) -> Option<egui::TextureId> {
    legacy.open = shell.visible;
    // Main sim HUD uses Bevy GPU chrome — egui CPU minimap only for explicit effects opt-in.
    if crate::gui::map_view::minimap_main_display_uses_gpu_compositor(shell) {
        cache.binding_mut(MapViewInstanceId::Minimap).clear();
        return None;
    }
    if !shell.visible || fallback.sprite_entity.is_none() {
        cache.binding_mut(MapViewInstanceId::Minimap).clear();
        return None;
    }

    if shell.minimized || !shell_widget_runs_egui(dock, HudWidgetId::Minimap, shell.visible) {
        dock.slot_mut(HudWidgetId::Minimap).minimized = shell.minimized;
        cache.binding_mut(MapViewInstanceId::Minimap).clear();
        return None;
    }

    if !ready.minimap.ready_to_bind() {
        return None;
    }

    let frame = frames.get(MapViewInstanceId::Minimap);
    if frame.texture_source.handle() == &Handle::default() {
        cache.binding_mut(MapViewInstanceId::Minimap).clear();
        return None;
    }

    let tex_id = cache
        .binding_mut(MapViewInstanceId::Minimap)
        .resolve(contexts, frame, interaction_frozen)?;
    ready.mark_frame_committed(MapViewInstanceId::Minimap);
    shell.cached_texture_revision = frame.projection_revision;
    Some(tex_id)
}
