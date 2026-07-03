//! Map presentation spine: module root — re-exports and tests only.

mod backend;
mod consumers;
mod debug;
mod plugin;
mod presentation;
mod projection;
mod resolved;
mod texture_cache;
mod view_state;
mod widgets;

pub use backend::{
    minimap_bevy_display_texel_extent, minimap_main_display_uses_gpu_compositor,
    resolve_minimap_bevy_display_handle, resolve_minimap_texture_source,
    resolve_world_preview_texture_source, MapTextureSource,
};
pub use consumers::{minimap, world_preview};
pub use debug::{MapFitConsumerTag, MapFitValidation, MapFitValidationLog};
pub use plugin::MapViewPlugin;
pub use presentation::{
    clear_active_map_view_input_before_map_widgets, paint_map_display_debug_outlines,
    paint_map_view_placeholder, sync_shell_layout_drag_gate, ActiveMapViewInput,
    MapDisplayTransform, MapShellPointerGate, MapViewInstanceId, MapViewPresentationInteractions,
    MapViewPresentationState, MapViewPresentationStates, MapViewReadyStates,
    MapViewInteractionByView, MinimapInteractionBuffer, SmoothedMapInteraction,
    ViewHandle, WorldPreviewInteractionBuffer,
};
pub use view_state::{MapViewInstances, MapViewState};
pub use widgets::{map_toolbar, map_toolbar_minimap_zoom, map_toolbar_preview_zoom, MapToolbarConfig};
pub use projection::ResolvedMapViewFrames;
pub use resolved::ResolvedMapViewFrame;
pub use texture_cache::{
    reset_map_view_texture_frame, MapViewTextureBinding, MapViewTextureCache,
};

#[cfg(test)]
mod tests {
    use super::projection::ResolvedMapViewFrames;
    use super::presentation::MapViewInstanceId;
    use super::view_state::MapViewInstances;
    use crate::gui::editor::world_preview::layers::PreviewLayers;

    #[test]
    fn resolved_frames_do_not_alias_world_preview_to_simulation_map() {
        let mut frames = ResolvedMapViewFrames::default();
        frames.world_preview.viewport_extent = bevy::math::UVec2::new(800, 600);
        frames.world_preview.projection_revision = 42;
        assert_eq!(frames.get(MapViewInstanceId::WorldPreview).viewport_extent.x, 800);
        assert_eq!(frames.get(MapViewInstanceId::SimulationMap).viewport_extent.x, 0);
        assert_ne!(
            frames.get(MapViewInstanceId::WorldPreview).projection_revision,
            frames.get(MapViewInstanceId::TacticalMap).projection_revision
        );
    }

    #[test]
    fn presentation_states_are_independent_per_consumer() {
        let mut instances = MapViewInstances::default();
        instances
            .world_preview
            .layers
            .replace_base(PreviewLayers::HEIGHT);
        instances.world_preview.bump_revision();
        assert!(!instances
            .minimap
            .layers
            .contains(PreviewLayers::HEIGHT));
        assert_ne!(instances.world_preview.revision, instances.minimap.revision);
    }
}
