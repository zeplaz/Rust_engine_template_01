//! Live witness: `debug_runs/wave_c_live.json` (WC-D02).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::editor::world_preview::PreviewPathAuthority;

use super::tile_storage_apply::TileStorageApplyReport;
use super::wave_c_readiness::{gather_wave_c_readiness, wave_c_readiness_passes};

pub const WAVE_C_LIVE_JSON: &str = "debug_runs/wave_c_live.json";

#[derive(Resource, Debug)]
pub struct WaveCLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for WaveCLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 120,
            written: false,
        }
    }
}

#[must_use]
pub fn build_wave_c_live_proof_payload(
    wave_c: &super::WaveCReadinessReport,
    tile_report: &TileStorageApplyReport,
) -> serde_json::Value {
    serde_json::json!({
        "profile": "WAVE_C_STREAMING",
        "wave_c_readiness": {
            "passes": wave_c_readiness_passes(wave_c),
            "open_backlog_items": wave_c.open_backlog_items,
        },
        "tile_storage_apply": {
            "applied_chunks": tile_report.applied_chunks,
            "pending_smooth_tiles": tile_report.pending_smooth_tiles,
            "last_timing": format!("{:?}", tile_report.last_apply_timing),
        },
        "wave_c_green": wave_c_readiness_passes(wave_c),
    })
}

pub fn write_wave_c_live_proof_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<WaveCLiveProofState>,
    preview: Res<PreviewPathAuthority>,
    tile_report: Res<TileStorageApplyReport>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;
    let wave_c = gather_wave_c_readiness(preview.as_ref());
    let body = build_wave_c_live_proof_payload(&wave_c, tile_report.as_ref());
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "WAVE_C_STREAMING",
        "wave_c_live_proof",
        WAVE_C_LIVE_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(WAVE_C_LIVE_JSON, wrapped) {
        state.written = true;
    }
}
