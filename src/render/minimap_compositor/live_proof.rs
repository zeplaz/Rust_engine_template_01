//! Live witness: `debug_runs/minimap_compositor_live.json`.

use bevy::prelude::*;

use crate::gui::{MinimapPresentationSource, MinimapShellState};
use crate::gui::hud::ui_stress_state::UiStressState;
use crate::render::SharedOverlayFieldBuffers;

use super::pass::{
    minimap_gpu_compositor_env_enabled, MinimapCompositePath, MinimapCompositorState,
};
use super::diagnostics::{diagnostics_json_snapshot, MinimapGpuCompositorDiagnostics};
use super::render_target::MinimapRenderTargetRegistry;

const PROOF_PATH: &str = "debug_runs/minimap_compositor_live.json";

/// UI-P3-001 acceptance rollup (witness A2–A5 + default GPU path).
#[must_use]
pub fn ui_p3_001_minimap_acceptance_green(
    compositor: &MinimapCompositorState,
    registry: &MinimapRenderTargetRegistry,
    shell: &MinimapShellState,
) -> bool {
    let composite_ok = registry.committed_image != Handle::default() && compositor.stamp > 0;
    minimap_gpu_compositor_env_enabled()
        && shell.presentation_source == MinimapPresentationSource::SharedRenderTargetImage
        && composite_ok
        && !compositor.dual_minimap_present
        && compositor.composite_path == MinimapCompositePath::GpuCompute
        && compositor.extent_match_px <= 1.0
}

/// UI-P3-M3-001 — construction + ecology overlay channels on GPU minimap.
#[must_use]
pub fn ui_p3_m3_minimap_acceptance_green(compositor: &MinimapCompositorState) -> bool {
    compositor.construction_heat_enabled
        && compositor.ecology_heat_enabled
        && (compositor.construction_rows > 0 || compositor.ecology_rows > 0)
}

#[derive(Resource, Debug, Default)]
pub struct MinimapCompositorLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
    pub last_committed_stamp: u64,
}

#[must_use]
pub fn commit_minimap_compositor_live_proof(
    compositor: &MinimapCompositorState,
    registry: &MinimapRenderTargetRegistry,
    shell: &MinimapShellState,
    overlay_revision: u64,
    ui_stress_wrote_sim: bool,
    diagnostics: &MinimapGpuCompositorDiagnostics,
) -> bool {
    let body = build_minimap_compositor_proof_payload(
        compositor,
        registry,
        shell,
        overlay_revision,
        ui_stress_wrote_sim,
        diagnostics,
    );
    let payload = crate::dev::debug_run_envelope::wrap_debug_run(
        "MINIMAP_COMPOSITOR_M1",
        "minimap_compositor_live_proof",
        PROOF_PATH,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(PROOF_PATH, payload)
}

#[must_use]
pub fn build_minimap_compositor_proof_payload(
    compositor: &MinimapCompositorState,
    registry: &MinimapRenderTargetRegistry,
    shell: &MinimapShellState,
    overlay_revision: u64,
    ui_stress_wrote_sim: bool,
    diagnostics: &MinimapGpuCompositorDiagnostics,
) -> serde_json::Value {
    let presentation_source = match shell.presentation_source {
        MinimapPresentationSource::SharedCpuRaster => "SharedCpuRaster",
        MinimapPresentationSource::SharedRenderTargetImage => "SharedRenderTargetImage",
    };
    serde_json::json!({
        "composite_ok": registry.committed_image != Handle::default() && compositor.stamp > 0,
        "stamp": compositor.stamp,
        "extent": {
            "x": registry.committed_size.x,
            "y": registry.committed_size.y,
        },
        "compositor_revision": compositor.compositor_revision,
        "presentation_source": presentation_source,
        "dual_minimap_present": compositor.dual_minimap_present,
        "extent_match_px": compositor.extent_match_px,
        "overlay_revision": overlay_revision,
        "gpu_compositor_env": minimap_gpu_compositor_env_enabled(),
        "rt_bound": registry.committed_image != Handle::default(),
        "ui_stress_wrote_sim": ui_stress_wrote_sim,
        "composite_path": match compositor.composite_path {
            super::pass::MinimapCompositePath::GpuCompute => "GpuCompute",
            super::pass::MinimapCompositePath::CpuBridge => "CpuBridge",
        },
        "logistics_rows": compositor.logistics_rows,
        "construction_rows": compositor.construction_rows,
        "ecology_rows": compositor.ecology_rows,
        "fire_heat_enabled": compositor.fire_heat_enabled,
        "logistics_heat_enabled": compositor.logistics_heat_enabled,
        "construction_heat_enabled": compositor.construction_heat_enabled,
        "ecology_heat_enabled": compositor.ecology_heat_enabled,
        "gpu_budget": diagnostics_json_snapshot(diagnostics),
        "ui_p3_001_green": ui_p3_001_minimap_acceptance_green(compositor, registry, shell),
        "ui_p3_m3_green": ui_p3_m3_minimap_acceptance_green(compositor),
    })
}

pub fn write_minimap_compositor_live_proof_system(
    mut state: ResMut<MinimapCompositorLiveProofState>,
    compositor: Res<MinimapCompositorState>,
    registry: Res<MinimapRenderTargetRegistry>,
    shell: Res<MinimapShellState>,
    overlay: Option<Res<SharedOverlayFieldBuffers>>,
    stress: Option<Res<UiStressState>>,
    diagnostics: Res<MinimapGpuCompositorDiagnostics>,
) {
    if !minimap_gpu_compositor_env_enabled() {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    let composite_ok =
        registry.committed_image != Handle::default() && compositor.stamp > 0;
    let stamp_flush = composite_ok && compositor.stamp > state.last_committed_stamp;
    let due = stamp_flush || (!state.written && composite_ok);
    if !due {
        return;
    }
    state.frames_since_write = 0;
    let overlay_revision = overlay.as_ref().map(|o| o.revision).unwrap_or(0);
    let ui_stress_wrote_sim = stress.as_ref().map(|s| s.ui_stress_wrote_sim).unwrap_or(false);
    if commit_minimap_compositor_live_proof(
        &compositor,
        &registry,
        &shell,
        overlay_revision,
        ui_stress_wrote_sim,
        &diagnostics,
    ) {
        state.written = true;
        if composite_ok {
            state.last_committed_stamp = compositor.stamp;
        }
    }
}
