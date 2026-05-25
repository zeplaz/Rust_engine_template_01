//! Presentation-only map interaction smoothing.

use bevy::math::Vec2;
use bevy::prelude::*;

use crate::gui::map_view::MapViewInstances;

#[derive(Clone, Debug, Default)]
pub struct SmoothedMapInteraction {
    pub current_zoom: f32,
    pub target_zoom: f32,
    pub current_pan: Vec2,
    pub target_pan: Vec2,
}

impl SmoothedMapInteraction {
    pub fn set_targets(&mut self, zoom: f32, pan: Vec2) {
        self.target_zoom = zoom;
        self.target_pan = pan;
    }

    pub fn snap_to_targets(&mut self) {
        self.current_zoom = self.target_zoom;
        self.current_pan = self.target_pan;
    }

    pub fn step(&mut self, dt: f32) {
        if dt <= 0.0 {
            return;
        }
        let k = 1.0 - (-dt * 16.0).exp();
        self.current_zoom += (self.target_zoom - self.current_zoom) * k;
        self.current_pan += (self.target_pan - self.current_pan) * k;
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct MapViewPresentationInteractions;

pub fn sync_map_view_interaction_targets(mut views: ResMut<MapViewInstances>) {
    let minimap_zoom_target = views.minimap.zoom_target;
    let minimap_center = views.minimap.camera_center;
    views
        .minimap
        .interaction
        .set_targets(minimap_zoom_target, minimap_center);
    let preview_zoom = views.world_preview.zoom;
    let preview_center = views.world_preview.camera_center;
    views
        .world_preview
        .interaction
        .set_targets(preview_zoom, preview_center);
}

pub fn advance_map_view_presentation_interactions(
    time: Res<Time>,
    mut views: ResMut<MapViewInstances>,
) {
    let dt = time.delta_secs();
    views.world_preview.interaction.step(dt);
    views.minimap.interaction.step(dt);
}
