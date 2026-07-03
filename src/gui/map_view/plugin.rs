//! Map view plugin — schedule wiring only.

use bevy::prelude::*;
use bevy_egui::EguiPrimaryContextPass;

use crate::gui::ViewRepresentationSystemSet;

use super::debug::{validate_map_fit_system, MapFitConsumerTag, MapFitValidationLog};
use super::presentation::{
    clear_active_map_view_input_before_map_widgets, commit_map_view_interaction_system,
    commit_map_view_viewport_suggestions, sync_map_view_interaction_targets,
    advance_map_view_presentation_interactions, update_minimap_view, update_world_preview_view,
    ActiveMapViewInput, MapDisplayTransform, MapShellPointerGate, MapViewInstanceId,
    MapViewInteractionByView, MapViewPresentationInteractions, MapViewPresentationStates,
    MapViewReadyStates, MapViewViewportSuggestions,
};
use super::projection::sync_resolved_map_view_frames;
use super::texture_cache::{reset_map_view_texture_frame, MapViewTextureCache};
use super::view_state::MapViewInstances;

pub struct MapViewPlugin;

impl Plugin for MapViewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapViewPresentationStates>()
            .init_resource::<MapViewInstances>()
            .init_resource::<super::projection::ResolvedMapViewFrames>()
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
                super::presentation::sync_map_view_ready_states
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
                super::presentation::sync_shell_layout_drag_gate,
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
        MapFitConsumerTag(MapViewInstanceId::Minimap),
        MapDisplayTransform::default(),
    ));
    commands.spawn((
        MapFitConsumerTag(MapViewInstanceId::WorldPreview),
        MapDisplayTransform::default(),
    ));
}
