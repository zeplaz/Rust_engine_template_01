//! Wave C streaming witness — `debug_runs/wave_c_live.json` (WC-D02).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::editor::world_preview::PreviewPathAuthority;
use crate::io::streaming::{
    gather_wave_c_readiness, wave_c_readiness_passes, TileStorageApplyReport,
    TileStorageApplyTiming, WaveCReadinessReport, WAVE_C_DEPTH_001_CLOSED_ITEM,
    TILE_STORAGE_DIFF_CONTRACT_BQ,
};

use super::common::{arm_live_proof_cadence, LiveProofCadence};
use super::io::{write_enveloped_witness, write_enveloped_witness_unchecked};

pub const WAVE_C_LIVE_JSON: &str = "debug_runs/wave_c_live.json";

const PROFILE: &str = "WAVE_C_STREAMING";
const SOURCE: &str = "wave_c_live_proof";

/// Slice B compat alias — same cadence as [`LiveProofCadence`].
pub type WaveCLiveProofState = LiveProofCadence;

/// **WC-DEPTH-001** — Wave C backlog row closed + tile apply contract (BQ-101) wired.
#[must_use]
pub fn wc_depth_001_green(
    wave_c: &WaveCReadinessReport,
    tile_report: &TileStorageApplyReport,
) -> bool {
    wave_c_readiness_passes(wave_c)
        && wave_c.open_backlog_items == 0
        && TILE_STORAGE_DIFF_CONTRACT_BQ == WAVE_C_DEPTH_001_CLOSED_ITEM
        && matches!(
            tile_report.last_apply_timing,
            TileStorageApplyTiming::AfterDomainReconstruct
                | TileStorageApplyTiming::Bq101Deferred
        )
}

#[must_use]
pub fn build_wave_c_live_proof_payload(
    wave_c: &WaveCReadinessReport,
    tile_report: &TileStorageApplyReport,
) -> serde_json::Value {
    let depth_green = wc_depth_001_green(wave_c, tile_report);
    serde_json::json!({
        "profile": PROFILE,
        "wave_c_readiness": {
            "passes": wave_c_readiness_passes(wave_c),
            "open_backlog_items": wave_c.open_backlog_items,
        },
        "tile_storage_apply": {
            "applied_chunks": tile_report.applied_chunks,
            "pending_smooth_tiles": tile_report.pending_smooth_tiles,
            "last_timing": format!("{:?}", tile_report.last_apply_timing),
            "contract_bq": TILE_STORAGE_DIFF_CONTRACT_BQ,
        },
        "wc_depth_001": {
            "green": depth_green,
            "closed_backlog_item": WAVE_C_DEPTH_001_CLOSED_ITEM,
            "open_backlog_items": wave_c.open_backlog_items,
        },
        "wc_depth_001_green": depth_green,
        "wave_c_green": wave_c_readiness_passes(wave_c) && depth_green,
    })
}

#[cfg(test)]
static WAVE_C_PROOF_FILE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
fn wave_c_proof_file_lock() -> std::sync::MutexGuard<'static, ()> {
    WAVE_C_PROOF_FILE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

/// **WC-DEPTH-001** — lib refresh of `wave_c_live.json`.
#[must_use]
pub fn commit_wave_c_live_proof(
    wave_c: &WaveCReadinessReport,
    tile_report: &TileStorageApplyReport,
) -> bool {
    #[cfg(test)]
    let _guard = wave_c_proof_file_lock();
    let body = build_wave_c_live_proof_payload(wave_c, tile_report);
    write_enveloped_witness_unchecked(PROFILE, SOURCE, WAVE_C_LIVE_JSON, body)
}

/// MIG-A7 — cadence tick in [`LiveProofCadencePlugin`]; pair with [`write_wave_c_live_proof_system`].
pub fn arm_wave_c_live_proof_cadence(mut state: ResMut<WaveCLiveProofState>) {
    arm_live_proof_cadence(&mut state);
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
    let wave_c = gather_wave_c_readiness(preview.as_ref());
    let body = build_wave_c_live_proof_payload(&wave_c, tile_report.as_ref());
    if write_enveloped_witness(PROFILE, SOURCE, WAVE_C_LIVE_JSON, body) {
        state.written = true;
    }
}
