//! Map presentation spine: shared backend, per-consumer presentation state.
//!
//! @orchestrator-status STABLE
//! @orchestrator-owner viewport_cleanup_agent
//! Witness: `debug_runs/stage5_full_app_live.json` (FULL_APP, map_presentation_stability green).

mod backend;
mod consumers;
mod debug;
mod presentation;
mod projection;
mod resolved;
mod texture_cache;
mod view_state;
mod widgets;

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::gui::ViewRepresentationSystemSet;
use presentation::{
    advance_map_view_presentation_interactions, sync_map_view_interaction_targets,
};

pub use backend::{
    minimap_cpu_raster_handle, minimap_effects_cpu_raster_active,
    minimap_main_display_uses_gpu_compositor, resolve_minimap_effects_cpu_raster_source,
    resolve_minimap_texture_source, resolve_world_preview_texture_source, MapTextureSource,
};
pub use consumers::{minimap, world_preview};
pub use debug::{
    validate_map_fit_system, MapFitConsumerTag, MapFitValidation, MapFitValidationLog,
};
pub use presentation::{
    clear_active_map_view_input_before_map_widgets, commit_map_view_interaction_system,
    commit_map_view_viewport_suggestions, paint_map_display_debug_outlines,
    paint_map_view_placeholder, sync_map_view_ready_states,
    sync_shell_layout_drag_gate, update_minimap_view, update_world_preview_view, ActiveMapViewInput,
    MapDisplayTransform, MapShellPointerGate, MapViewInstanceId, MapViewPresentationInteractions,
    MapViewPresentationState, MapViewPresentationStates, MapViewReadyStates,
    MapViewViewportSuggestions, MapViewInteractionByView, MinimapInteractionBuffer, SmoothedMapInteraction,
    ViewHandle, WorldPreviewInteractionBuffer,
};
pub use view_state::{MapViewInstances, MapViewState};
pub use widgets::{map_toolbar, map_toolbar_minimap_zoom, map_toolbar_preview_zoom, MapToolbarConfig};
pub use projection::{sync_resolved_map_view_frames, ResolvedMapViewFrames};
pub use resolved::ResolvedMapViewFrame;
pub use texture_cache::{
    reset_map_view_texture_frame, MapViewTextureBinding, MapViewTextureCache,
};

pub struct MapViewPlugin;

impl Plugin for MapViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapViewPresentationStates>()
            .init_resource::<MapViewInstances>()
            .init_resource::<ResolvedMapViewFrames>()
            .init_resource::<MapViewTextureCache>()
            .init_resource::<MapViewPresentationInteractions>()
            .init_resource::<MapViewReadyStates>()
            .init_resource::<MapViewViewportSuggestions>()
            .init_resource::<MapShellPointerGate>()
            .init_resource::<MapViewInteractionByView>()
            .init_resource::<ActiveMapViewInput>()
            .init_resource::<crate::gui::MapPresentationDiagnostics>()
            .init_resource::<MapFitValidationLog>()
            .add_systems(Startup, spawn_map_fit_validate_entities)
            .add_systems(
                Update,
                sync_resolved_map_view_frames
                    .in_set(ViewRepresentationSystemSet::ResolveViewport)
                    .after(crate::render::ViewportPipelineSet::Resolve),
            )
            .add_systems(
                Update,
                (
                    sync_map_view_interaction_targets,
                    advance_map_view_presentation_interactions,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                sync_map_view_ready_states
                    .after(sync_resolved_map_view_frames)
                    .in_set(ViewRepresentationSystemSet::ResolveViewport),
            )
            .add_systems(
                PostUpdate,
                (
                    update_world_preview_view,
                    update_minimap_view,
                    crate::render::view_runtime::commit_deferred_map_view_poses_to_authority,
                    commit_map_view_interaction_system,
                    commit_map_view_viewport_suggestions,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                reset_map_view_texture_frame.in_set(ViewRepresentationSystemSet::UiCollect),
            )
            .add_systems(
                EguiPrimaryContextPass,
                clear_active_map_view_input_before_map_widgets
                    .before(crate::gui::hud::hud_root_tick::hud_product_shell_egui_root)
                    .before(crate::gui::editor::world_preview::display_world_preview),
            )
            .add_systems(
                EguiPrimaryContextPass,
                presentation::sync_shell_layout_drag_gate,
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    crate::gui::sync_map_fit_transform_components
                        .after(crate::gui::hud::hud_root_tick::hud_product_shell_egui_root)
                        .after(crate::gui::editor::world_preview::display_world_preview),
                    validate_map_fit_system.after(crate::gui::sync_map_fit_transform_components),
                )
                    .chain(),
            );
    }
}

fn spawn_map_fit_validate_entities(mut commands: Commands) {
    commands.spawn((
        MapFitConsumerTag(MapViewInstanceId::WorldPreview),
        MapDisplayTransform::default(),
    ));
    commands.spawn((
        MapFitConsumerTag(MapViewInstanceId::Minimap),
        MapDisplayTransform::default(),
    ));
}

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
