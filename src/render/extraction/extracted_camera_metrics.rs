//! Single main-world camera metrics for VFX emitters and GPU passes.
//!
//! Pose is read from [`ViewProjectionAuthority`]'s committed `WorldMain` surface
//! (RGR-V2-001 — completes the VM migration for the fire/water raster + extract path);
//! it no longer queries the live [`MainWorldCamera`] `Transform`/`Camera`/`GlobalTransform`.
//! Copied to the render world via [`ExtractResource`] each extract tick.

use bevy::math::Vec2;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};

use crate::gui::{
    map_zoom_alpha_with_limits, map_zoom_limits_for_world, orthographic_fixed_world_span,
    sync_main_world_camera_viewport_and_projection, MapCameraDesiredRes, MapCameraSystemSet,
    MAIN_WORLD_CAMERA_Z, TacticalMapFillRect,
};
use crate::render::view_runtime::{ViewProjectionAuthority, ViewSurfaceId};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Shader/emitter-facing camera metrics — one writer: [`sync_extracted_camera_metrics`].
#[derive(Resource, Clone, Copy, Debug, ExtractResource)]
pub struct ExtractedCameraMetrics {
    pub translation: Vec2,
    pub zoom_level: f32,
    pub zoom_alpha: f32,
    /// RTT / tactical fill logical size (viewport for zoom-limit math).
    pub view_pixels: Vec2,
    /// Clip-space matrix for tactical map RTT — single writer: [`sync_extracted_camera_metrics`].
    pub view_proj: Mat4,
}

impl Default for ExtractedCameraMetrics {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            zoom_level: 1.0,
            zoom_alpha: 0.5,
            view_pixels: Vec2::new(1280.0, 720.0),
            view_proj: Mat4::IDENTITY,
        }
    }
}

impl ExtractedCameraMetrics {
    /// Analytical orthographic view-proj for lib tests (matches RTT Camera2d when rotation is identity).
    #[must_use]
    pub fn compute_tactical_view_proj(
        translation: Vec2,
        view_pixels: Vec2,
        zoom: f32,
        world_w: f32,
        world_h: f32,
    ) -> Mat4 {
        let (fw, fh) = orthographic_fixed_world_span(
            view_pixels,
            zoom.max(1e-4),
            world_w.max(1.0),
            world_h.max(1.0),
        );
        let view = Mat4::from_translation(-Vec3::new(
            translation.x,
            translation.y,
            MAIN_WORLD_CAMERA_Z,
        ));
        let proj = Mat4::orthographic_rh(-fw * 0.5, fw * 0.5, -fh * 0.5, fh * 0.5, 0.0, 2000.0);
        proj * view
    }

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
            view_proj: Self::compute_tactical_view_proj(
                Vec2::ZERO,
                Vec2::new(1280.0, 720.0),
                zoom_level,
                320.0,
                320.0,
            ),
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

/// Sync from the committed [`ViewProjectionAuthority`] `WorldMain` surface pose before
/// particle emit / weather VFX (RGR-V2-001 — no raw [`MainWorldCamera`] query).
///
/// `translation`/`zoom` come from the authority pose (single writer:
/// [`crate::gui::sync_map_camera_pose_to_view_authority`]); when the surface has not
/// committed yet (pre-bootstrap / headless) this falls back to [`MapCameraDesiredRes`],
/// the same mirror [`crate::gui::derive_map_camera_desired_from_view_authority`] derives
/// from the authority one frame later. `view_proj` is always the analytical projection —
/// it is built from the same `fill`/`desired`/`params` triple that
/// `apply_main_world_camera_ortho_core` uses to drive the live camera's `Transform` /
/// `OrthographicProjection`, so the two agree by construction.
pub fn sync_extracted_camera_metrics(
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    fill: Option<Res<TacticalMapFillRect>>,
    params: Option<Res<WorldGenParams>>,
    mut metrics: ResMut<ExtractedCameraMetrics>,
) {
    let view_px = fill
        .as_deref()
        .filter(|f| f.is_adequate_for_camera())
        .map(|f| f.logical_size())
        .unwrap_or(Vec2::new(1280.0, 720.0));
    let authority_pose = authority
        .as_deref()
        .and_then(|a| a.surface(ViewSurfaceId::WorldMain))
        .map(|s| s.camera);
    let (translation, zoom) = authority_pose
        .map(|c| (c.translation, c.zoom.max(0.06)))
        .unwrap_or_else(|| (desired.0.translation.truncate(), desired.0.scale.x.max(0.06)));
    let (lo, hi) = params
        .as_deref()
        .map(|p| map_zoom_limits_for_world(p.width as f32, p.height as f32, view_px))
        .unwrap_or((0.08, 4.0));
    let world_w = params.as_deref().map(|p| p.width as f32).unwrap_or(320.0);
    let world_h = params.as_deref().map(|p| p.height as f32).unwrap_or(320.0);
    let view_proj =
        ExtractedCameraMetrics::compute_tactical_view_proj(translation, view_px, zoom, world_w, world_h);
    *metrics = ExtractedCameraMetrics {
        translation,
        zoom_level: zoom,
        zoom_alpha: map_zoom_alpha_with_limits(zoom, lo, hi),
        view_pixels: view_px,
        view_proj,
    };
}

/// Populate draw globals from [`ExtractedCameraMetrics`] (RTT-B5 — raster sync uses this, not camera query).
#[must_use]
pub fn particle_view_globals_from_metrics(
    metrics: &ExtractedCameraMetrics,
    time_secs: f32,
    vertex_count: u32,
) -> crate::render::gpu_instanced_quad::ParticleViewGlobals {
    crate::render::gpu_instanced_quad::ParticleViewGlobals {
        view_proj: metrics.view_proj,
        vertex_count,
        time_secs,
        zoom_alpha: metrics.zoom_alpha,
        _pad: 0.0,
    }
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
