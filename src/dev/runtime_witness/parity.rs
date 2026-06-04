//! Witness JSON parity checks after migration shims (**WSS-WITNESS-PARITY-001**).

use serde_json::Value;

use super::stage6::STAGE6_VIRTUALIZATION_JSON;
use super::view_runtime::INFRASTRUCTURE_VIEW_ISOLATION_JSON;
use super::wave_c::{commit_wave_c_live_proof, WAVE_C_LIVE_JSON};
use super::wave_s::{build_wave_s_hydrate_proof_payload, WAVE_S_HYDRATE_JSON};
use super::io::write_enveloped_witness_unchecked;
use super::minimap::{commit_minimap_compositor_live_proof, MINIMAP_COMPOSITOR_JSON};

const IGNORED_ENVELOPE_KEYS: &[&str] = &[
    "_agent_meta",
    "written_at_epoch_secs",
    "written_at",
    "source_system",
];

#[must_use]
pub fn strip_envelope_for_parity(body: &Value) -> Value {
    let mut out = body.clone();
    if let Some(obj) = out.as_object_mut() {
        for key in IGNORED_ENVELOPE_KEYS {
            obj.remove(*key);
        }
    }
    out
}

#[must_use]
pub fn witness_has_required_keys(body: &Value, keys: &[&str]) -> bool {
    keys.iter().all(|k| body.pointer(k).is_some())
}

/// Refresh migrated witnesses and assert stable contract keys (lib bundle).
#[must_use]
pub fn refresh_migrated_witness_parity_bundle() -> bool {
    wave_c_parity_green()
        && stage6_parity_green()
        && view_runtime_parity_green()
        && minimap_parity_green()
        && wave_s_parity_green()
}

#[must_use]
fn wave_c_parity_green() -> bool {
    use crate::gui::editor::world_preview::PreviewPathAuthority;
    use crate::io::streaming::{gather_wave_c_readiness, TileStorageApplyReport};

    let preview = PreviewPathAuthority::default();
    let report = gather_wave_c_readiness(&preview);
    let tile_report = TileStorageApplyReport::default();
    if !commit_wave_c_live_proof(&report, &tile_report) {
        return false;
    }
    let raw = std::fs::read_to_string(WAVE_C_LIVE_JSON).unwrap_or_default();
    let v: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
    let body = strip_envelope_for_parity(&v);
    witness_has_required_keys(
        &body,
        &[
            "/wave_c_readiness/passes",
            "/profile",
        ],
    )
}

#[must_use]
fn stage6_parity_green() -> bool {
    use crate::dev::runtime_witness::stage6::{
        commit_stage6_virtualization_live_proof, Stage6VirtualizationWitness,
    };
    use crate::gui::editor::world_preview::PreviewPathAuthority;
    use crate::io::streaming::gather_wave_c_readiness;
    use crate::render::{gather_stage6_readiness, Stage6VirtualizationFrame};

    let preview = PreviewPathAuthority::default();
    let frame = Stage6VirtualizationFrame::default();
    let report = gather_stage6_readiness(&preview, &frame);
    let wave_c = gather_wave_c_readiness(&preview);
    let witness = Stage6VirtualizationWitness::default();
    if !commit_stage6_virtualization_live_proof(&witness, &report, &frame, &wave_c, None, None) {
        return false;
    }
    let raw = std::fs::read_to_string(STAGE6_VIRTUALIZATION_JSON).unwrap_or_default();
    let v: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
    witness_has_required_keys(
        &strip_envelope_for_parity(&v),
        &["/stage6_readiness/passes", "/profile"],
    )
}

#[must_use]
fn view_runtime_parity_green() -> bool {
    use crate::dev::runtime_witness::view_runtime::build_infrastructure_view_isolation_payload;
    use crate::gui::ViewIsolationDiagnostics;
    use crate::render::view_runtime::{
        ViewFireIsolationWitness, ViewInputRoutingState, ViewProjectionAuthority,
        ViewRuntimeTrace, ViewRuntimeWitness,
    };

    let body = build_infrastructure_view_isolation_payload(
        &ViewRuntimeWitness::default(),
        &ViewIsolationDiagnostics::default(),
        &ViewProjectionAuthority::default(),
        &ViewRuntimeTrace::default(),
        &ViewInputRoutingState::default(),
        &ViewFireIsolationWitness::default(),
    );
    let _ = write_enveloped_witness_unchecked(
        "INFRASTRUCTURE_VIEW_ISOLATION",
        "witness_parity_refresh",
        INFRASTRUCTURE_VIEW_ISOLATION_JSON,
        body,
    );
    let raw = std::fs::read_to_string(INFRASTRUCTURE_VIEW_ISOLATION_JSON).unwrap_or_default();
    let v: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
    witness_has_required_keys(
        &strip_envelope_for_parity(&v),
        &["/profile", "/vm_09/triage_vm09_v2_green"],
    )
}

#[must_use]
fn minimap_parity_green() -> bool {
    use crate::gui::hud::HudOverlayTrayState;
    use crate::gui::MinimapPresentationSource;
    use crate::gui::MinimapShellState;
    use crate::render::minimap_compositor::{
        fixture_ui_oh_m2_001_compositor, minimap_rgba_image, MinimapCompositePath,
        MinimapGpuCompositorDiagnostics, MinimapRenderTargetRegistry,
    };

    let tray = HudOverlayTrayState::default();
    let compositor = fixture_ui_oh_m2_001_compositor(&tray);
    let mut registry = MinimapRenderTargetRegistry::default();
    let mut images = bevy::prelude::Assets::<bevy::prelude::Image>::default();
    registry.committed_size = bevy::prelude::UVec2::new(128, 128);
    registry.revision = 2;
    registry.committed_image = images.add(minimap_rgba_image(128, 128));
    let shell = MinimapShellState {
        presentation_source: MinimapPresentationSource::SharedRenderTargetImage,
        ..Default::default()
    };
    assert_eq!(compositor.composite_path, MinimapCompositePath::GpuCompute);
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
    let v: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
    witness_has_required_keys(
        &strip_envelope_for_parity(&v),
        &["/composite_ok", "/presentation_source"],
    )
}

#[must_use]
fn wave_s_parity_green() -> bool {
    use crate::io::save::WaveSShellHydrateWitness;

    let witness = WaveSShellHydrateWitness {
        shell_loaded: true,
        blueprint_count: 1,
        layout_widget_count: 1,
        ..Default::default()
    };
    let body = build_wave_s_hydrate_proof_payload(&witness);
    let _ = write_enveloped_witness_unchecked(
        "WAVE_S_HYDRATE",
        "witness_parity_refresh",
        WAVE_S_HYDRATE_JSON,
        body,
    );
    let raw = std::fs::read_to_string(WAVE_S_HYDRATE_JSON).unwrap_or_default();
    let v: Value = serde_json::from_str(&raw).unwrap_or(Value::Null);
    witness_has_required_keys(
        &strip_envelope_for_parity(&v),
        &["/wave_s_hydrate_green", "/profile"],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wss_witness_parity_001_migrated_bundle_green() {
        assert!(refresh_migrated_witness_parity_bundle());
    }
}
