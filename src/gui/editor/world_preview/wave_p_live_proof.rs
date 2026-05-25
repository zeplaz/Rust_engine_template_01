//! Live witness: `debug_runs/wave_p_live.json` (WP-B01).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::editor::world_preview::{
    gather_wave_p_readiness, wave_p_readiness_passes, PreviewLayers, PreviewPathAuthority,
};

pub const WAVE_P_LIVE_JSON: &str = "debug_runs/wave_p_live.json";

#[derive(Resource, Debug)]
pub struct WavePLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for WavePLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 120,
            written: false,
        }
    }
}

#[must_use]
pub fn build_wave_p_live_proof_payload(
    report: &super::WavePReadinessReport,
) -> serde_json::Value {
    let passes = wave_p_readiness_passes(report);
    serde_json::json!({
        "profile": "WAVE_P_PREVIEW",
        "wave_p_readiness": {
            "passes": passes,
            "report": {
                "save_manifest_schema_version": report.save_manifest_schema_version,
                "save_chunk_schema_version": report.save_chunk_schema_version,
                "composite_layer_bindings": report.composite_layer_bindings,
                "consumer_contract_ok": report.consumer_contract_ok,
                "composite_graph_sources": report.composite_graph_sources,
                "gpu_authoritative_surface": report.gpu_authoritative_surface,
                "open_backlog_items": report.open_backlog_items,
            },
        },
        "wave_p_green": passes,
    })
}

pub fn write_wave_p_live_proof_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<WavePLiveProofState>,
    authority: Res<PreviewPathAuthority>,
    graph: Option<Res<crate::gui::editor::world_preview::CompositePreviewGraphResource>>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;
    let layers = graph
        .as_deref()
        .map(|g| g.0.layers)
        .unwrap_or(PreviewLayers::BIOME);
    let report = gather_wave_p_readiness(layers, authority.as_ref());
    let body = build_wave_p_live_proof_payload(&report);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "WAVE_P_PREVIEW",
        "wave_p_live_proof",
        WAVE_P_LIVE_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(WAVE_P_LIVE_JSON, wrapped) {
        state.written = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wave_p_live_payload_shape() {
        let authority = PreviewPathAuthority::default();
        let report = gather_wave_p_readiness(PreviewLayers::BIOME, &authority);
        let body = build_wave_p_live_proof_payload(&report);
        assert_eq!(
            body.pointer("/wave_p_readiness/passes")
                .and_then(|v| v.as_bool()),
            Some(wave_p_readiness_passes(&report))
        );
    }
}
