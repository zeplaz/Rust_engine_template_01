//! APS-BEVY-QC-HUD-001 live witness — `debug_runs/aps_bevy_qc_hud_001_live.json`.

use serde_json::{json, Value};

use crate::dev::debug_run_envelope;
use crate::gui::{
    aps_bevy_qc_hud_001_witness_green, aps_bevy_qc_hud_v2_witness_green, load_qc_snapshot,
    APS_BEVY_QC_HUD_DEFAULT_SNAPSHOT,
};
use crate::preview::repo_root_from_manifest;

pub const APS_BEVY_QC_HUD_001_LIVE_JSON: &str = "debug_runs/aps_bevy_qc_hud_001_live.json";
pub const APS_BEVY_QC_HUD_V2_LIVE_JSON: &str = "debug_runs/aps_bevy_qc_hud_001_v2_live.json";

#[must_use]
pub fn build_aps_bevy_qc_hud_001_proof_payload() -> Value {
    let repo_root = repo_root_from_manifest();
    let path = repo_root.join(APS_BEVY_QC_HUD_DEFAULT_SNAPSHOT);
    let green = aps_bevy_qc_hud_001_witness_green();
    let (placement_count, table_rows, assembly_id) = if green {
        let Ok((_, summary)) = load_qc_snapshot(&path) else {
            return json!({ "aps_bevy_qc_hud_001": { "green": false, "error": "load failed" } });
        };
        (
            summary.placement_count,
            summary.rows.len(),
            summary.assembly_id,
        )
    } else {
        (0, 0, String::new())
    };

    json!({
        "gate": "APS-BEVY-QC-HUD-001",
        "profile": "APS_BEVY_QC_HUD_001",
        "aps_bevy_qc_hud_001": {
            "green": green,
            "panel_module": "src/gui/assembly_snapshot_qc_ui.rs",
            "toggle": "Ctrl+Shift+Q",
            "diagnostics_entry": "F3 → Assembly snapshot QC",
            "example_snapshot": APS_BEVY_QC_HUD_DEFAULT_SNAPSHOT,
            "assembly_id": assembly_id,
            "placement_count": placement_count,
            "table_row_count": table_rows,
            "row_count_matches_json": placement_count == table_rows && green,
        },
        "aps_bevy_qc_hud_v2": {
            "green": aps_bevy_qc_hud_v2_witness_green(),
            "p0_readonly_strip": true,
            "footprint_highlight_on_preview": true,
        },
    })
}

#[must_use]
pub fn commit_aps_bevy_qc_hud_v2_live_proof() -> bool {
    let body = build_aps_bevy_qc_hud_001_proof_payload();
    let green = body["aps_bevy_qc_hud_v2"]["green"].as_bool().unwrap_or(false);
    if !green {
        return false;
    }
    let wrapped = debug_run_envelope::wrap_debug_run(
        "APS_BEVY_QC_HUD_V2",
        "commit_aps_bevy_qc_hud_v2_live_proof",
        APS_BEVY_QC_HUD_V2_LIVE_JSON,
        body["aps_bevy_qc_hud_v2"].clone(),
    );
    debug_run_envelope::write_debug_run_json(APS_BEVY_QC_HUD_V2_LIVE_JSON, wrapped) && green
}

#[must_use]
pub fn commit_aps_bevy_qc_hud_001_live_proof() -> bool {
    let body = build_aps_bevy_qc_hud_001_proof_payload();
    let green = body["aps_bevy_qc_hud_001"]["green"].as_bool().unwrap_or(false);
    if !green {
        return false;
    }
    let wrapped = debug_run_envelope::wrap_debug_run(
        "APS_BEVY_QC_HUD_001",
        "commit_aps_bevy_qc_hud_001_live_proof",
        APS_BEVY_QC_HUD_001_LIVE_JSON,
        body,
    );
    debug_run_envelope::write_debug_run_json(APS_BEVY_QC_HUD_001_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aps_bevy_qc_hud_001_live_witness() {
        assert!(commit_aps_bevy_qc_hud_001_live_proof());
        let text = std::fs::read_to_string(APS_BEVY_QC_HUD_001_LIVE_JSON).expect("proof json");
        let v: Value = serde_json::from_str(&text).expect("parse");
        assert!(v["aps_bevy_qc_hud_001"]["green"].as_bool().unwrap_or(false));
        assert!(v["aps_bevy_qc_hud_001"]["row_count_matches_json"]
            .as_bool()
            .unwrap_or(false));
    }

    #[test]
    fn aps_bevy_qc_hud_v2_live_witness() {
        assert!(commit_aps_bevy_qc_hud_v2_live_proof());
        let text = std::fs::read_to_string(APS_BEVY_QC_HUD_V2_LIVE_JSON).expect("proof json");
        let v: Value = serde_json::from_str(&text).expect("parse");
        assert!(v["green"].as_bool().unwrap_or(false));
        assert!(v["p0_readonly_strip"].as_bool().unwrap_or(false));
        assert!(v["footprint_highlight_on_preview"].as_bool().unwrap_or(false));
    }
}
