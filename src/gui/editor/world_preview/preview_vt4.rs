//! **VT-4** world-preview probe capture (editor preview surface).

use bevy::prelude::*;

use crate::gui::editor::world_preview::preview_render_contract::{
    preview_authoritative_surface, PreviewCameraState,
};
use crate::gui::editor::world_preview::{WorldPreviewGpuRuntime, WorldPreviewUiState};
use crate::render::{hash_shared_overlay_heat, SharedOverlayFieldBuffers, WorldPreviewVt4Probe};

pub fn capture_world_preview_vt4_probe(
    preview_ui: Res<WorldPreviewUiState>,
    shared: Res<SharedOverlayFieldBuffers>,
    gpu_rt: Res<WorldPreviewGpuRuntime>,
    preview_cam: Res<PreviewCameraState>,
    mut probe: ResMut<WorldPreviewVt4Probe>,
) {
    update_world_preview_vt4_probe(
        preview_ui.as_ref(),
        shared.as_ref(),
        gpu_rt.as_ref(),
        preview_cam.as_ref(),
        probe.as_mut(),
    );
}

pub fn update_world_preview_vt4_probe(
    preview_ui: &WorldPreviewUiState,
    shared: &SharedOverlayFieldBuffers,
    gpu_rt: &WorldPreviewGpuRuntime,
    preview_cam: &PreviewCameraState,
    probe: &mut WorldPreviewVt4Probe,
) {
    if !preview_ui.window_open {
        probe.consumer_active = false;
        return;
    }
    probe.consumer_active = true;
    probe.stamp = shared.stamp;
    probe.overlay_heat_hash = hash_shared_overlay_heat(&shared.chunk_fire_heat);
    probe.overlay_revision = shared.revision;
    let _ = preview_authoritative_surface(gpu_rt, preview_cam);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::sim_control::SimStepStamp;

    #[test]
    fn probe_copies_shared_overlay_hash_when_preview_open() {
        let mut shared = SharedOverlayFieldBuffers::default();
        shared.stamp = SimStepStamp::new(3, 9);
        shared.revision = 2;
        shared.chunk_fire_heat.insert(IVec2::ZERO, 0.5);

        let mut probe = WorldPreviewVt4Probe::default();
        update_world_preview_vt4_probe(
            &WorldPreviewUiState {
                window_open: true,
                ..Default::default()
            },
            &shared,
            &WorldPreviewGpuRuntime::default(),
            &PreviewCameraState::default(),
            &mut probe,
        );
        assert_eq!(probe.stamp, shared.stamp);
        assert_eq!(probe.overlay_revision, 2);
        assert_eq!(
            probe.overlay_heat_hash,
            hash_shared_overlay_heat(&shared.chunk_fire_heat)
        );
        assert!(probe.consumer_active);
    }

    #[test]
    fn probe_marks_consumer_inactive_when_preview_closed() {
        let mut shared = SharedOverlayFieldBuffers::default();
        shared.stamp = SimStepStamp::new(3, 9);
        shared.revision = 2;
        shared.chunk_fire_heat.insert(IVec2::ZERO, 0.5);

        let mut probe = WorldPreviewVt4Probe {
            stamp: shared.stamp,
            overlay_heat_hash: 1,
            overlay_revision: 2,
            consumer_active: true,
        };
        update_world_preview_vt4_probe(
            &WorldPreviewUiState {
                window_open: false,
                ..Default::default()
            },
            &shared,
            &WorldPreviewGpuRuntime::default(),
            &PreviewCameraState::default(),
            &mut probe,
        );
        assert!(!probe.consumer_active);
    }
}
