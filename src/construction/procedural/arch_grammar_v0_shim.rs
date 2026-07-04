//! **BQ-H3-V0-RETIRE-001** — freeze `arch_build_grammar_v0` behind validation shim.
//!
//! v0 DNA reweighting remains opt-in via `arch_dna_preset_id`; unknown massing override ids
//! in presets are rejected before they can bias T1 grammar picks.

use super::arch_build_grammar_v0::{ArchGrammarV0Preset, MassingWeightOverride};
use super::building_grammar::{load_building_grammar_registry, BuildingGrammar};

pub const BQ_H3_LIVE_JSON: &str = "debug_runs/bq_h3_v0_retire_001_live.json";

/// Massing ids that v0 bias tables know about (`arch_build_grammar_v0::massing_bias`).
pub const V0_KNOWN_MASSING_IDS: &[&str] = &[
    "long_hall",
    "double_hall",
    "l_shape",
    "yard_complex",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V0PresetValidation {
    pub preset_id: String,
    pub grammar_id: String,
    pub ok: bool,
    pub unknown_massing_overrides: Vec<String>,
    pub unknown_grammar_massings: Vec<String>,
}

#[must_use]
pub fn validate_v0_preset_against_grammar(
    preset: &ArchGrammarV0Preset,
    grammar: &BuildingGrammar,
) -> V0PresetValidation {
    let grammar_massings: Vec<String> = grammar
        .massing
        .strategies
        .iter()
        .map(|s| s.id.as_str().to_owned())
        .collect();
    let unknown_overrides: Vec<String> = preset
        .massing_weight_override
        .iter()
        .filter(|o| !grammar_massings.iter().any(|id| id == &o.id))
        .map(|o| o.id.clone())
        .collect();
    let unknown_v0_bias: Vec<String> = grammar_massings
        .iter()
        .filter(|id| !V0_KNOWN_MASSING_IDS.contains(&id.as_str()))
        .cloned()
        .collect();
    let ok = unknown_overrides.is_empty();
    V0PresetValidation {
        preset_id: preset.preset_id.clone(),
        grammar_id: preset.grammar_id.clone(),
        ok,
        unknown_massing_overrides: unknown_overrides,
        unknown_grammar_massings: unknown_v0_bias,
    }
}

#[must_use]
pub fn validate_massing_override_row(row: &MassingWeightOverride, grammar: &BuildingGrammar) -> bool {
    grammar
        .massing
        .strategies
        .iter()
        .any(|s| s.id.as_str() == row.id)
}

#[must_use]
pub fn bq_h3_v0_shim_witness_green() -> bool {
    let registry = load_building_grammar_registry();
    if !registry.load_errors.is_empty() {
        return false;
    }
    let preset_ok = super::arch_build_grammar_v0::load_logistics_rail_warehouse_v0_preset()
        .ok()
        .and_then(|preset| {
            registry
                .grammars
                .values()
                .find(|g| g.grammar_id == preset.grammar_id)
                .map(|grammar| validate_v0_preset_against_grammar(&preset, grammar))
        })
        .is_some_and(|v| v.ok);
    let reject_unknown = !validate_massing_override_row(
        &MassingWeightOverride {
            id: "not_a_real_massing".into(),
            weight: 10,
        },
        registry
            .grammars
            .values()
            .next()
            .expect("grammar"),
    );
    preset_ok && reject_unknown
}

#[must_use]
pub fn build_bq_h3_v0_shim_witness_body() -> serde_json::Value {
    let registry = load_building_grammar_registry();
    let preset_validation = super::arch_build_grammar_v0::load_logistics_rail_warehouse_v0_preset()
        .ok()
        .and_then(|preset| {
            registry
                .grammars
                .values()
                .find(|g| g.grammar_id == preset.grammar_id)
                .map(|grammar| {
                    let v = validate_v0_preset_against_grammar(&preset, grammar);
                    serde_json::json!({
                        "preset_id": v.preset_id,
                        "grammar_id": v.grammar_id,
                        "ok": v.ok,
                        "unknown_massing_overrides": v.unknown_massing_overrides,
                        "unknown_grammar_massings": v.unknown_grammar_massings,
                    })
                })
        });
    let green = bq_h3_v0_shim_witness_green();
    serde_json::json!({
        "gate": "BQ-H3-V0-RETIRE-001",
        "green": green,
        "decision": "frozen_shim",
        "note": "v0 DNA reweighting opt-in only; preset massing_weight_override must match T1 grammar strategies",
        "v0_known_massing_ids": V0_KNOWN_MASSING_IDS,
        "logistics_rail_warehouse_v0": preset_validation,
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-H3",
    })
}

#[must_use]
pub fn refresh_bq_h3_v0_shim_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_bq_h3_v0_shim_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "BQ-H3-V0-RETIRE-001",
        "refresh_bq_h3_v0_shim_witness",
        BQ_H3_LIVE_JSON,
        body,
    );
    write_debug_run_json(BQ_H3_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bq_h3_v0_shim_witness_green_lib() {
        assert!(bq_h3_v0_shim_witness_green());
    }

    #[test]
    fn bq_h3_refresh_witness_when_green() {
        if bq_h3_v0_shim_witness_green() {
            assert!(refresh_bq_h3_v0_shim_witness());
        }
    }
}
