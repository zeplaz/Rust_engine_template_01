//! ECON-OG-SAVE-001 live witness — `debug_runs/econ_og_save_live.json`.

use serde_json::{json, Value};

use crate::dev::debug_run_envelope;
use crate::io::save::{
    build_settlement_overlay_refs, econ_og_save_001_witness_green,
    settlement_books_manifest_roundtrip_witness_green,
    settlement_books_save_roundtrip_witness_green, SETTLEMENT_BOOKS_REL_PATH,
    SETTLEMENT_OVERLAY_NAME,
};

pub const ECON_OG_SAVE_LIVE_JSON: &str = "debug_runs/econ_og_save_live.json";

#[must_use]
pub fn build_econ_og_save_001_proof_payload() -> Value {
    let ron_roundtrip = settlement_books_save_roundtrip_witness_green();
    let manifest_roundtrip = settlement_books_manifest_roundtrip_witness_green();
    let green = econ_og_save_001_witness_green();

    json!({
        "gate": "ECON-OG-SAVE-001",
        "profile": "ECON_OG_SAVE_001",
        "econ_og_save_001": {
            "green": green,
            "ron_roundtrip_ok": ron_roundtrip,
            "manifest_overlay_roundtrip_ok": manifest_roundtrip,
            "overlay_name": SETTLEMENT_OVERLAY_NAME,
            "artifact_rel_path": SETTLEMENT_BOOKS_REL_PATH,
            "manifest_overlay_refs": build_settlement_overlay_refs(),
            "save_pipeline_wired": true,
            "runtime_hydrate_system": "try_hydrate_settlement_books_on_bundle_dir",
        },
    })
}

#[must_use]
pub fn commit_econ_og_save_live_proof() -> bool {
    let body = build_econ_og_save_001_proof_payload();
    let green = body["econ_og_save_001"]["green"].as_bool().unwrap_or(false);
    if !green {
        return false;
    }
    let wrapped = debug_run_envelope::wrap_debug_run(
        "ECON_OG_SAVE_001",
        "commit_econ_og_save_live_proof",
        ECON_OG_SAVE_LIVE_JSON,
        body["econ_og_save_001"].clone(),
    );
    debug_run_envelope::write_debug_run_json(ECON_OG_SAVE_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn econ_og_save_live_witness() {
        assert!(commit_econ_og_save_live_proof());
    }
}
