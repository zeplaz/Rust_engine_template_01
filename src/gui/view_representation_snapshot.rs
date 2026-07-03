//! Frame-stable view snapshot consumed by GPU preview, minimap, and Stage 5 harness.

use bevy::math::{Rect, UVec2, Vec2};
use bevy::prelude::*;

use crate::gui::editor::world_preview::{PreviewCameraState, WorldPreviewRenderTargetRegistry};
use crate::gui::map_camera::{MapCameraDesiredRes};
use crate::gui::{ViewId, ViewManager};
use crate::render::{
    AppStage5ReadinessReport, GpuParticleInstance, ResolvedViewports, Stage5ReadinessProfile,
    WorldFireParticleFrame,
};

/// World-space bounds for particle routing consumers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldBounds {
    pub min: Vec2,
    pub max: Vec2,
}

impl WorldBounds {
    #[must_use]
    pub fn from_particle_instances(instances: &[GpuParticleInstance]) -> Self {
        if instances.is_empty() {
            return Self::default();
        }
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for inst in instances {
            let world = inst.world_xyz_heat;
            min = min.min(Vec2::new(world.x, world.y));
            max = max.max(Vec2::new(world.x, world.y));
        }
        Self { min, max }
    }
}

/// Camera routing snapshot for render + particle consumers.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SnapshotCameraState {
    pub translation: Vec2,
    pub zoom: f32,
}

/// Committed per-frame view contract for downstream render surfaces.
#[derive(Clone, Resource, Debug, Default)]
pub struct ViewRepresentationSnapshot {
    pub frame_id: u64,
    pub viewport: Rect,
    pub camera: SnapshotCameraState,
    pub gpu_target_size: UVec2,
    pub minimap_rect: Rect,
    pub render_target: Handle<Image>,
    pub particle_bounds: WorldBounds,
    pub committed: bool,
}

pub fn build_view_representation_snapshot(
    resolved: Res<ResolvedViewports>,
    registry: Res<WorldPreviewRenderTargetRegistry>,
    preview_cam: Res<PreviewCameraState>,
    view_manager: Res<ViewManager>,
    map_desired: Res<MapCameraDesiredRes>,
    particles: Option<Res<WorldFireParticleFrame>>,
    mut snapshot: ResMut<ViewRepresentationSnapshot>,
) {
    snapshot.frame_id = snapshot.frame_id.wrapping_add(1);

    let logical = if resolved.world_preview.valid {
        resolved.world_preview.logical_size
    } else {
        Vec2::ONE
    };
    snapshot.viewport = Rect::from_corners(Vec2::ZERO, logical.max(Vec2::ONE));

    snapshot.gpu_target_size = if resolved.world_preview.valid {
        resolved.world_preview.physical_extent
    } else {
        UVec2::ONE
    };

    if resolved.world_preview.valid {
        if let Some(v) = view_manager.view(ViewId::WorldPreview) {
            snapshot.camera = SnapshotCameraState {
                translation: v.camera.translation,
                zoom: v.camera.zoom.max(1e-4),
            };
        } else {
            snapshot.camera = SnapshotCameraState {
                translation: Vec2::new(preview_cam.center.x, preview_cam.center.y),
                zoom: preview_cam.zoom.max(1e-4),
            };
        }
    } else if let Some(v) = view_manager.view(ViewId::WorldMain) {
        snapshot.camera = SnapshotCameraState {
            translation: v.camera.translation,
            zoom: v.camera.zoom.max(1e-4),
        };
    } else if map_desired.scale.x.is_finite() && map_desired.scale.x > 0.0 {
        snapshot.camera = SnapshotCameraState {
            translation: Vec2::new(map_desired.translation.x, map_desired.translation.y),
            zoom: map_desired.scale.x.max(1e-4),
        };
    } else {
        snapshot.camera = SnapshotCameraState {
            translation: Vec2::new(preview_cam.center.x, preview_cam.center.y),
            zoom: preview_cam.zoom.max(1e-4),
        };
    }

    let minimap_size = if resolved.minimap_panel.valid {
        resolved.minimap_panel.logical_size
    } else {
        Vec2::ONE
    };
    snapshot.minimap_rect = Rect::from_corners(Vec2::ZERO, minimap_size.max(Vec2::ONE));

    snapshot.render_target = registry.committed_image.clone();
    snapshot.particle_bounds = particles
        .as_deref()
        .map(|frame| WorldBounds::from_particle_instances(&frame.instances))
        .unwrap_or_default();

    snapshot.committed = resolved.world_preview.valid
        || resolved.minimap_panel.valid
        || resolved.primary_window.valid;
}

pub fn validate_view_representation_snapshot(
    profile: Res<Stage5ReadinessProfile>,
    snapshot: Res<ViewRepresentationSnapshot>,
    mut report: ResMut<AppStage5ReadinessReport>,
) {
    if *profile != Stage5ReadinessProfile::FULL_APP || !snapshot.committed {
        return;
    }
    if snapshot.gpu_target_size.x == 0 || snapshot.gpu_target_size.y == 0 {
        report
            .violations
            .push("ViewRepresentationSnapshot gpu_target_size is zero".into());
    }
    if snapshot.viewport.width() <= 0.0 || snapshot.viewport.height() <= 0.0 {
        report
            .violations
            .push("ViewRepresentationSnapshot viewport is empty".into());
    }
}
