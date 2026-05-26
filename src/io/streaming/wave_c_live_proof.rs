//! Live witness: `debug_runs/wave_c_live.json` (WC-D02).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::editor::world_preview::PreviewPathAuthority;

use super::tile_storage_apply::TileStorageApplyReport;
use super::tile_storage_contract::TILE_STORAGE_DIFF_CONTRACT_BQ;
use super::wave_c_prerequisites::WAVE_C_DEPTH_001_CLOSED_ITEM;
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

/// **WC-DEPTH-001** — Wave C backlog row closed + tile apply contract (BQ-101) wired.
#[must_use]
pub fn wc_depth_001_green(
    wave_c: &super::WaveCReadinessReport,
    tile_report: &TileStorageApplyReport,
) -> bool {
    wave_c_readiness_passes(wave_c)
        && wave_c.open_backlog_items == 0
        && TILE_STORAGE_DIFF_CONTRACT_BQ == WAVE_C_DEPTH_001_CLOSED_ITEM
        && matches!(
            tile_report.last_apply_timing,
            super::tile_storage_contract::TileStorageApplyTiming::AfterDomainReconstruct
                | super::tile_storage_contract::TileStorageApplyTiming::Bq101Deferred
        )
}

#[must_use]
pub fn build_wave_c_live_proof_payload(
    wave_c: &super::WaveCReadinessReport,
    tile_report: &TileStorageApplyReport,
) -> serde_json::Value {
    let depth_green = wc_depth_001_green(wave_c, tile_report);
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
    wave_c: &super::WaveCReadinessReport,
    tile_report: &TileStorageApplyReport,
) -> bool {
    #[cfg(test)]
    let _guard = wave_c_proof_file_lock();
    let body = build_wave_c_live_proof_payload(wave_c, tile_report);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "WAVE_C_STREAMING",
        "wave_c_live_proof",
        WAVE_C_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(WAVE_C_LIVE_JSON, wrapped)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::io::streaming::tile_storage_contract::TileStorageApplyTiming;

    #[test]
    fn wc_depth_001_green_when_backlog_empty_and_bq_101_contract() {
        let wave_c = gather_wave_c_readiness(&PreviewPathAuthority::default());
        let tile = TileStorageApplyReport {
            last_apply_timing: TileStorageApplyTiming::AfterDomainReconstruct,
            applied_chunks: 1,
            pending_smooth_tiles: 0,
        };
        assert!(wc_depth_001_green(&wave_c, &tile));
    }

    /// **WC-DEPTH-001** — refresh witness with BQ-101 closure rollup.
    #[test]
    fn wc_depth_001_writes_wave_c_live_json() {
        let wave_c = gather_wave_c_readiness(&PreviewPathAuthority::default());
        let tile = TileStorageApplyReport {
            last_apply_timing: TileStorageApplyTiming::AfterDomainReconstruct,
            applied_chunks: 2,
            pending_smooth_tiles: 4,
        };
        assert!(commit_wave_c_live_proof(&wave_c, &tile));
        let text = std::fs::read_to_string(WAVE_C_LIVE_JSON).expect("wave_c witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["wc_depth_001_green"], serde_json::json!(true));
        assert_eq!(
            body["wc_depth_001"]["closed_backlog_item"],
            serde_json::json!("BQ-101")
        );
        assert_eq!(body["wave_c_green"], serde_json::json!(true));
        assert_eq!(
            body["tile_storage_apply"]["contract_bq"],
            serde_json::json!("BQ-101")
        );
    }
}
