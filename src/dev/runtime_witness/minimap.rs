//! Minimap compositor witness — `debug_runs/minimap_compositor_live.json` (DEV-CONTAIN-SLICE-1).

use bevy::prelude::*;

use crate::gui::MinimapShellState;
use crate::gui::hud::ui_stress_state::UiStressState;
use crate::render::minimap_compositor::{
    build_minimap_compositor_proof_payload_with_tray, minimap_gpu_compositor_env_enabled,
    MinimapCompositorState, MinimapGpuCompositorDiagnostics, MinimapRenderTargetRegistry,
};
use crate::render::SharedOverlayFieldBuffers;

use super::io::write_enveloped_witness_unchecked;

pub const MINIMAP_COMPOSITOR_JSON: &str = "debug_runs/minimap_compositor_live.json";

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
    tray: Option<&crate::gui::hud::HudOverlayTrayState>,
) -> bool {
    let body = build_minimap_compositor_proof_payload_with_tray(
        compositor,
        registry,
        shell,
        overlay_revision,
        ui_stress_wrote_sim,
        diagnostics,
        tray,
    );
    write_enveloped_witness_unchecked(
        "MINIMAP_COMPOSITOR_M1",
        "minimap_compositor_live_proof",
        MINIMAP_COMPOSITOR_JSON,
        body,
    )
}

pub fn write_minimap_compositor_live_proof_system(
    mut state: ResMut<MinimapCompositorLiveProofState>,
    compositor: Res<MinimapCompositorState>,
    registry: Res<MinimapRenderTargetRegistry>,
    shell: Res<MinimapShellState>,
    tray: Option<Res<crate::gui::hud::HudOverlayTrayState>>,
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
        tray.as_deref(),
    ) {
        state.written = true;
        if composite_ok {
            state.last_committed_stamp = compositor.stamp;
        }
    }
}

/// Lib refresh — **UI-W3-M3-001** / M2+M3 rollups on disk.
#[must_use]
pub fn refresh_ui_w3_m3_001_live_witness() -> bool {
    use crate::gui::MinimapPresentationSource;
    use crate::render::minimap_compositor::{
        fixture_ui_w3_m3_001_compositor, ui_p3_001_minimap_acceptance_green, ui_w3_m2_001_green,
        ui_w3_m3_001_green, minimap_rgba_image, MinimapGpuCompositorDiagnostics,
    };

    let tray = crate::render::minimap_compositor::witness_harness_tray();
    let compositor = fixture_ui_w3_m3_001_compositor(&tray);
    assert!(ui_w3_m3_001_green(&compositor), "UI-W3-M3-001 construction + ecology");
    assert!(ui_w3_m2_001_green(&compositor), "UI-W3-M2-001 logistics + construction");
    let mut registry = MinimapRenderTargetRegistry::default();
    let mut images = Assets::<Image>::default();
    registry.committed_size = UVec2::new(128, 128);
    registry.revision = 2;
    registry.committed_image = images.add(minimap_rgba_image(128, 128));
    let shell = MinimapShellState {
        presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
        ..Default::default()
    };
    assert!(
        ui_p3_001_minimap_acceptance_green(&compositor, &registry, &shell),
        "UI-P3-001 minimap acceptance"
    );
    commit_minimap_compositor_live_proof(
        &compositor,
        &registry,
        &shell,
        7,
        false,
        &MinimapGpuCompositorDiagnostics::default(),
        Some(&tray),
    )
}

/// Lib refresh — minimap + Stage 7 behavioral witnesses operational rollup.
#[must_use]
pub fn refresh_ui_w3_m3_001_stage7_operational_witness() -> bool {
    use crate::dev::stage7_behavioral_witness::{
        refresh_s7b_m3_001_live_witness, stage7_behavioral_live_s7b_m3_green,
    };

    assert!(refresh_ui_w3_m3_001_live_witness(), "minimap M2+M3+UI-P3-001");
    assert!(refresh_s7b_m3_001_live_witness(), "stage7 S7B-M3 overlay");
    assert!(stage7_behavioral_live_s7b_m3_green(), "s7b_m3_green on disk");
    true
}

/// Lib refresh — **UI-OH-M2-001** / **UI-W3-M2-001** rollup.
#[must_use]
pub fn refresh_ui_oh_m2_001_live_witness() -> bool {
    use crate::gui::MinimapPresentationSource;
    use crate::render::minimap_compositor::{
        fixture_ui_oh_m2_001_compositor, minimap_rgba_image, ui_oh_m2_001_green,
        MinimapGpuCompositorDiagnostics,
    };

    let tray = crate::render::minimap_compositor::witness_harness_tray();
    let compositor = fixture_ui_oh_m2_001_compositor(&tray);
    assert!(ui_oh_m2_001_green(&compositor), "UI-OH-M2-001 predicate");
    let mut registry = MinimapRenderTargetRegistry::default();
    let mut images = Assets::<Image>::default();
    registry.committed_size = UVec2::new(128, 128);
    registry.revision = 2;
    registry.committed_image = images.add(minimap_rgba_image(128, 128));
    let shell = MinimapShellState {
        presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
        ..Default::default()
    };
    commit_minimap_compositor_live_proof(
        &compositor,
        &registry,
        &shell,
        7,
        false,
        &MinimapGpuCompositorDiagnostics::default(),
        Some(&tray),
    )
}

/// Alias for [`refresh_ui_oh_m2_001_live_witness`].
#[must_use]
pub fn refresh_ui_w3_m2_001_live_witness() -> bool {
    refresh_ui_oh_m2_001_live_witness()
}

/// **PERF-VIS-P1B-GPU-DEFAULT-001** — lib refresh with GPU default path.
#[must_use]
pub fn refresh_perf_vis_p1b_gpu_default_live_witness() -> bool {
    use crate::gui::MinimapPresentationSource;
    use crate::render::minimap_compositor::{
        fixture_ui_oh_m2_001_compositor, minimap_rgba_image, perf_vis_p1b_gpu_default_001_green,
        MinimapGpuCompositorDiagnostics,
    };

    let tray = crate::render::minimap_compositor::witness_harness_tray();
    let compositor = fixture_ui_oh_m2_001_compositor(&tray);
    let mut registry = MinimapRenderTargetRegistry::default();
    let mut images = Assets::<Image>::default();
    registry.committed_size = UVec2::new(128, 128);
    registry.revision = 2;
    registry.committed_image = images.add(minimap_rgba_image(128, 128));
    let shell = MinimapShellState {
        presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
        ..Default::default()
    };
    assert!(
        perf_vis_p1b_gpu_default_001_green(&shell, &registry, &compositor),
        "PERF-VIS-P1B fixture predicate"
    );
    if !commit_minimap_compositor_live_proof(
        &compositor,
        &registry,
        &shell,
        1,
        false,
        &MinimapGpuCompositorDiagnostics::default(),
        Some(&tray),
    ) {
        return false;
    }
    let raw = std::fs::read_to_string(MINIMAP_COMPOSITOR_JSON).unwrap_or_default();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);
    v["presentation_source"].as_str() == Some("SharedRenderTargetImage")
        && v["perf_vis_p1b_gpu_default_001"]["green"]
            .as_bool()
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::MinimapPresentationSource;
    use crate::render::minimap_compositor::{
        fixture_ui_oh_m2_001_compositor, minimap_rgba_image, MinimapCompositePath,
    };

    #[test]
    fn minimap_runtime_witness_commit_roundtrip() {
        let tray = crate::render::minimap_compositor::witness_harness_tray();
        let compositor = fixture_ui_oh_m2_001_compositor(&tray);
        let mut registry = MinimapRenderTargetRegistry::default();
        let mut images = Assets::<Image>::default();
        registry.committed_size = UVec2::new(128, 128);
        registry.revision = 2;
        registry.committed_image = images.add(minimap_rgba_image(128, 128));
        let shell = MinimapShellState {
            presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
            ..Default::default()
        };
        assert_eq!(compositor.composite_path, MinimapCompositePath::GpuCompute);
        assert!(commit_minimap_compositor_live_proof(
            &compositor,
            &registry,
            &shell,
            1,
            false,
            &MinimapGpuCompositorDiagnostics::default(),
            Some(&tray),
        ));
    }

    /// **UI-W3-M3-001** — Stage 7 operational minimap + overlay reader witness refresh.
    #[test]
    fn ui_w3_m3_001_stage7_operational_witness_refresh() {
        use std::fs;
        use std::path::PathBuf;

        assert!(super::refresh_ui_w3_m3_001_stage7_operational_witness());
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(MINIMAP_COMPOSITOR_JSON);
        let v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&path).expect("read")).expect("parse");
        assert_eq!(v["ui_w3_m3_001"]["green"], serde_json::json!(true));
        assert_eq!(v["ui_p3_001_green"], serde_json::json!(true));
        assert_eq!(v["ui_p3_m3_green"], serde_json::json!(true));
        let s7 = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("debug_runs/stage7_behavioral_live.json");
        let s7v: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&s7).expect("read s7")).expect("parse");
        assert_eq!(s7v["s7b_m3_green"], serde_json::json!(true));
    }
}
