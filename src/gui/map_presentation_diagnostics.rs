//! Per-consumer map presentation diagnostics (layout / fit only).



use bevy::math::{Rect, UVec2, Vec2};

use bevy::prelude::*;

use bevy_egui::egui;



use crate::gui::map_presentation_fit::{default_fit_mode_for, MapFitMode};

use crate::gui::map_view::{MapDisplayTransform, MapFitValidation};

use crate::gui::map_view::MapViewInstanceId;



#[derive(Clone, Debug)]

pub struct MapPresentationConsumerDiagnostics {

    pub allocated_rect: Option<egui::Rect>,

    pub image_rect: Option<egui::Rect>,

    pub fit_mode: MapFitMode,

    pub aspect_texture: f32,

    pub aspect_panel: f32,

    pub viewport_extent: UVec2,

    pub uv_rect: egui::Rect,

    pub padding: f32,

    pub camera_zoom: f32,

    pub visible_world_bounds: Option<Rect>,

    pub fit_scale: f32,

    pub expected_fit_scale: f32,

    pub validation: Option<MapFitValidation>,

    pub transform: Option<MapDisplayTransform>,

}



impl Default for MapPresentationConsumerDiagnostics {

    fn default() -> Self {

        Self {

            allocated_rect: None,

            image_rect: None,

            fit_mode: MapFitMode::Contain,

            aspect_texture: 1.0,

            aspect_panel: 1.0,

            viewport_extent: UVec2::ONE,

            uv_rect: egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),

            padding: 0.0,

            camera_zoom: 1.0,

            visible_world_bounds: None,

            fit_scale: 0.0,

            expected_fit_scale: 0.0,

            validation: None,

            transform: None,

        }

    }

}



#[derive(Resource, Clone, Debug, Default)]

pub struct MapPresentationDiagnostics {

    pub world_preview: MapPresentationConsumerDiagnostics,

    pub minimap: MapPresentationConsumerDiagnostics,

}



impl MapPresentationDiagnostics {

    pub fn slot(&self, id: MapViewInstanceId) -> &MapPresentationConsumerDiagnostics {

        match id {

            MapViewInstanceId::Minimap => &self.minimap,

            _ => &self.world_preview,

        }

    }



    pub fn slot_mut(&mut self, id: MapViewInstanceId) -> &mut MapPresentationConsumerDiagnostics {

        match id {

            MapViewInstanceId::Minimap => &mut self.minimap,

            _ => &mut self.world_preview,

        }

    }



    pub fn record_fit_truth(

        &mut self,

        id: MapViewInstanceId,

        viewport_rect: egui::Rect,

        texture_size: UVec2,

        fit_mode: MapFitMode,

        padding: f32,

        actual_image_rect: egui::Rect,

        actual_uv_rect: egui::Rect,

        viewport_extent: UVec2,

        camera_center: Vec2,

        camera_zoom: f32,

        visible_world_bounds: Option<Rect>,

        contain_fit_authoritative: bool,

    ) {

        let mut transform = MapDisplayTransform::from_fit_truth(

            viewport_rect,

            texture_size,

            fit_mode,

            padding,

            actual_image_rect,

            actual_uv_rect,

            camera_center,

            camera_zoom,

            contain_fit_authoritative,

        );

        transform.expected_fit_mode = default_fit_mode_for(id);

        let slot = self.slot_mut(id);

        slot.allocated_rect = Some(viewport_rect);

        slot.image_rect = Some(actual_image_rect);

        slot.fit_mode = fit_mode;

        slot.aspect_texture = if texture_size.y > 0 {

            texture_size.x as f32 / texture_size.y as f32

        } else {

            1.0

        };

        slot.aspect_panel = if viewport_rect.height() > 0.0 {

            viewport_rect.width() / viewport_rect.height()

        } else {

            1.0

        };

        slot.viewport_extent = viewport_extent;

        slot.uv_rect = actual_uv_rect;

        slot.padding = padding;

        slot.camera_zoom = camera_zoom;

        slot.visible_world_bounds = visible_world_bounds;

        slot.fit_scale = transform.scale;

        slot.expected_fit_scale = transform.expected_scale;

        slot.validation = Some(MapFitValidation::compare(
            transform.expected_viewport_rect,
            transform.actual_viewport_rect,
            transform.expected_image_rect,
            transform.actual_image_rect,
            transform.expected_uv_rect,
            transform.uv_rect,
        ));

        slot.transform = Some(transform);

    }



    #[allow(clippy::too_many_arguments)]

    pub fn record(

        &mut self,

        id: MapViewInstanceId,

        allocated_rect: egui::Rect,

        image_rect: egui::Rect,

        fit_mode: MapFitMode,

        texture_size: Vec2,

        viewport_extent: UVec2,

        uv_rect: egui::Rect,

        padding: f32,

        camera_center: Vec2,

        camera_zoom: f32,

        visible_world_bounds: Option<Rect>,

    ) {

        self.record_fit_truth(

            id,

            allocated_rect,

            UVec2::new(texture_size.x.max(1.0) as u32, texture_size.y.max(1.0) as u32),

            fit_mode,

            padding,

            image_rect,

            uv_rect,

            viewport_extent,

            camera_center,

            camera_zoom,

            visible_world_bounds,

            false,

        );

    }

}



pub fn sync_map_fit_transform_components(
    diagnostics: Res<MapPresentationDiagnostics>,
    mut query: Query<(&crate::gui::map_view::MapFitConsumerTag, &mut MapDisplayTransform)>,
    update_attrib: Option<ResMut<crate::render::FrameUpdateAttrib>>,
    spike_guard: Option<Res<crate::engine::UxFrameSpikeGuard>>,
) {
    if spike_guard.is_some_and(|g| g.suppress_optional_diagnostics) {
        return;
    }
    let t0 = std::time::Instant::now();
    for (tag, mut transform) in &mut query {
        if let Some(snapshot) = diagnostics.slot(tag.0).transform.as_ref() {
            *transform = snapshot.clone();
        }
    }
    crate::render::record_map_fit_sync_ms(update_attrib, t0.elapsed().as_secs_f32() * 1000.0);
}


