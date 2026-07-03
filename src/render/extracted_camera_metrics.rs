//! Single main-world camera metrics for VFX emitters and GPU passes.
//!
//! Pulled from [`MainWorldCamera`] components (not [`ViewManager`] / authority chains).
//! Copied to the render world via [`ExtractResource`] each extract tick.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};

use crate::gui::{
    map_zoom_alpha_with_limits, map_zoom_limits_for_world, sync_main_world_camera_viewport_and_projection,
    MapCameraDesired, MapCameraSystemSet, MainWorldCamera, SimulationMapFillRect,
};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Shader/emitter-facing camera metrics — one writer: [`sync_extracted_camera_metrics`].
#[derive(Resource, Clone, Copy, Debug, ExtractResource)]
pub struct ExtractedCameraMetrics {
    pub translation: Vec2,
    pub zoom_level: f32,
    pub zoom_alpha: f32,
    /// RTT / tactical fill logical size (viewport for zoom-limit math).
    pub view_pixels: Vec2,
}

impl Default for ExtractedCameraMetrics {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            zoom_level: 1.0,
            zoom_alpha: 0.5,
            view_pixels: Vec2::new(1280.0, 720.0),
        }
    }
}

impl ExtractedCameraMetrics {
    #[inline]
    #[must_use]
    pub fn camera_zoom(&self) -> f32 {
        self.zoom_level
    }

    /// Test / harness helper.
    #[must_use]
    pub fn for_tests(zoom_level: f32, zoom_alpha: f32) -> Self {
        Self {
            translation: Vec2::ZERO,
            zoom_level,
            zoom_alpha,
            view_pixels: Vec2::new(1280.0, 720.0),
        }
    }
}

/// Deprecated alias — use [`ExtractedCameraMetrics`].
pub type FireParticleCameraScale = ExtractedCameraMetrics;

/// Schedule anchor for [`sync_extracted_camera_metrics`] — use for ordering, not the fn type.
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ExtractedCameraMetricsSet {
    Sync,
}

/// Sync from the live [`MainWorldCamera`] entity before particle emit / weather VFX.
pub fn sync_extracted_camera_metrics(
    q: Query<(&Transform, &MapCameraDesired), With<MainWorldCamera>>,
    fill: Option<Res<SimulationMapFillRect>>,
    params: Option<Res<WorldGenParams>>,
    mut metrics: ResMut<ExtractedCameraMetrics>,
) {
    let Ok((tf, desired)) = q.single() else {
        return;
    };
    let view_px = fill
        .as_deref()
        .filter(|f| f.is_adequate_for_camera())
        .map(|f| f.logical_size())
        .unwrap_or(Vec2::new(1280.0, 720.0));
    let zoom = desired.scale.x.max(0.06);
    let (lo, hi) = params
        .as_deref()
        .map(|p| map_zoom_limits_for_world(p.width as f32, p.height as f32, view_px))
        .unwrap_or((0.08, 4.0));
    *metrics = ExtractedCameraMetrics {
        translation: tf.translation.truncate(),
        zoom_level: zoom,
        zoom_alpha: map_zoom_alpha_with_limits(zoom, lo, hi),
        view_pixels: view_px,
    };
}

/// Deprecated alias — [`sync_extracted_camera_metrics`].
pub use sync_extracted_camera_metrics as sync_fire_particle_camera_scale;

pub struct ExtractedCameraMetricsPlugin;

impl Plugin for ExtractedCameraMetricsPlugin {
    fn build(&self, app: &mut App) {
        let run = crate::gui::in_simulation_or_editor_map;
        app.init_resource::<ExtractedCameraMetrics>()
            .add_plugins(ExtractResourcePlugin::<ExtractedCameraMetrics>::default())
            .configure_sets(Update, ExtractedCameraMetricsSet::Sync.run_if(run))
            .configure_sets(PostUpdate, ExtractedCameraMetricsSet::Sync.run_if(run))
            .add_systems(
                Update,
                sync_extracted_camera_metrics
                    .in_set(ExtractedCameraMetricsSet::Sync)
                    .after(MapCameraSystemSet::Smooth),
            )
            .add_systems(
                PostUpdate,
                sync_extracted_camera_metrics
                    .in_set(ExtractedCameraMetricsSet::Sync)
                    .after(sync_main_world_camera_viewport_and_projection),
            );
    }
}
