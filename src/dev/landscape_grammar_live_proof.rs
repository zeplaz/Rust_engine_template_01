//! Landscape grammar live proof — FULL_APP witness refresh (VEG-WITNESS-LIVE-PROOF-001).

use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};
use crate::systems::ecology::{
    evaluate_landscape_program, load_landscape_grammar_catalog, refresh_lg2_witness,
    refresh_lg4_preview_witness, LandscapeGrammarLg2Witness, LG1_PILOT_CHUNK,
    LG1_PILOT_PRESET_ID, LANDSCAPE_GRAMMAR_LG2_LIVE_JSON,
    LANDSCAPE_GRAMMAR_LG4_PREVIEW_LIVE_JSON,
};
use crate::systems::ecology::{ChunkEcology, VegetationField};
use crate::systems::weather::ChunkWeather;

pub const LANDSCAPE_GRAMMAR_LIVE_PROOF_JSON: &str =
    "debug_runs/landscape_grammar_live_proof.json";

#[must_use]
pub fn landscape_grammar_live_proof_green(witness: &LandscapeGrammarLg2Witness) -> bool {
    witness.fire_disturbances >= 1 && witness.construction_disturbances >= 1
}

#[must_use]
pub fn commit_landscape_grammar_live_proof(witness: &LandscapeGrammarLg2Witness) -> bool {
    let catalog = load_landscape_grammar_catalog();
    let Some(preset) = catalog.presets.get(LG1_PILOT_PRESET_ID) else {
        return false;
    };
    let ecology = ChunkEcology::default();
    let veg = VegetationField::default();
    let weather = ChunkWeather::default();
    let eval = evaluate_landscape_program(preset, LG1_PILOT_CHUNK, &ecology, &veg, &weather);
    let _ = refresh_lg2_witness(&eval, witness);
    let _ = refresh_lg4_preview_witness(&eval);
    let green = landscape_grammar_live_proof_green(witness);
    let body = serde_json::json!({
        "gate": "VEG-WITNESS-LIVE-PROOF-001",
        "green": green,
        "fire_disturbances": witness.fire_disturbances,
        "construction_disturbances": witness.construction_disturbances,
        "lg2_path": LANDSCAPE_GRAMMAR_LG2_LIVE_JSON,
        "lg4_path": LANDSCAPE_GRAMMAR_LG4_PREVIEW_LIVE_JSON,
    });
    let wrapped = wrap_debug_run(
        "VEG-WITNESS-LIVE-PROOF-001",
        "commit_landscape_grammar_live_proof",
        LANDSCAPE_GRAMMAR_LIVE_PROOF_JSON,
        body,
    );
    write_debug_run_json(LANDSCAPE_GRAMMAR_LIVE_PROOF_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn repo_asset_path(rel: &str) -> PathBuf {
        std::env::var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .map(|root| root.join(rel))
            .unwrap_or_else(|| PathBuf::from(rel))
    }

    #[test]
    fn live_proof_writes_json_when_disturbances_present() {
        let witness = LandscapeGrammarLg2Witness {
            fire_disturbances: 1,
            construction_disturbances: 1,
            ..Default::default()
        };
        assert!(commit_landscape_grammar_live_proof(&witness));
        let raw =
            std::fs::read_to_string(repo_asset_path(LANDSCAPE_GRAMMAR_LIVE_PROOF_JSON)).unwrap();
        let doc: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(doc.get("green").and_then(|v| v.as_bool()), Some(true));
    }

    #[test]
    fn pilot_preset_loads_from_disk() {
        use crate::systems::ecology::{
            load_landscape_preset_from_path, LANDSCAPE_PRESETS_DIR, LG1_PILOT_PRESET_ID,
        };
        let path = repo_asset_path(&format!("{LANDSCAPE_PRESETS_DIR}/{LG1_PILOT_PRESET_ID}.json"));
        assert!(load_landscape_preset_from_path(&path).is_ok());
    }
}
