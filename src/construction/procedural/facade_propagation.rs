//! **BQ-H1-FACADE-001** — massing → facade propagation witnesses.

use super::building_grammar::{
    default_door_rhythm_for_massing, generate, load_building_grammar_registry,
    GrammarGenerateResult, MassingId, ResolvedFacade,
};

pub const BQ_H1_LIVE_JSON: &str = "debug_runs/bq_h1_facade_001_live.json";

#[must_use]
pub fn bq_h1_facade_massing_table_green() -> bool {
    let registry = load_building_grammar_registry();
    if !registry.load_errors.is_empty() {
        return false;
    }
    let Some(grammar) = registry.by_grammar_id("industrial_warehouse_v1") else {
        return false;
    };
    let long = grammar
        .facade
        .resolve_for_massing(&MassingId::try_new("long_hall").expect("long_hall"));
    let yard = grammar
        .facade
        .resolve_for_massing(&MassingId::try_new("yard_complex").expect("yard_complex"));
    let leg = grammar
        .facade
        .resolve_for_massing(&MassingId::try_new("l_shape").expect("l_shape"));

    long.door_rhythm == "linear_center"
        && long.placement_tags.iter().any(|t| t == "sawtooth")
        && yard.door_rhythm == "perimeter_only"
        && yard.door_slot.as_str() == "door_default"
        && leg.door_rhythm == "leg_offset"
        && leg.placement_tags.iter().any(|t| t == "leg_primary")
}

#[must_use]
pub fn bq_h1_generate_propagation_green() -> bool {
    let mut saw_long = false;
    let mut saw_double = false;
    let mut saw_yard = false;
    for seed in 0..128u64 {
        let Ok(result) = generate("IndustrialWarehouse", "industrial_west", seed) else {
            return false;
        };
        match result.massing_strategy.as_str() {
            "long_hall" => saw_long = result.door_rhythm == "linear_center",
            "double_hall" => saw_double = result.door_rhythm == "loading_bay",
            "yard_complex" => saw_yard = result.door_rhythm == "perimeter_only",
            _ => {}
        }
    }
    saw_long && saw_double && saw_yard
}

#[must_use]
pub fn bq_h1_facade_witness_green() -> bool {
    bq_h1_facade_massing_table_green() && bq_h1_generate_propagation_green()
}

#[must_use]
pub fn resolved_facade_summary(facade: &ResolvedFacade) -> serde_json::Value {
    serde_json::json!({
        "door_rhythm": facade.door_rhythm,
        "door_slot": facade.door_slot.as_str(),
        "placement_tags": facade.placement_tags,
        "massing_override_applied": facade.massing_override_applied,
    })
}

#[must_use]
pub fn build_bq_h1_facade_witness_body() -> serde_json::Value {
    let registry = load_building_grammar_registry();
    let table = registry.by_grammar_id("industrial_warehouse_v1").map(|g| {
        serde_json::json!({
            "long_hall": resolved_facade_summary(
                &g.facade.resolve_for_massing(&MassingId::try_new("long_hall").expect("long_hall")),
            ),
            "yard_complex": resolved_facade_summary(
                &g.facade.resolve_for_massing(&MassingId::try_new("yard_complex").expect("yard_complex")),
            ),
            "l_shape": resolved_facade_summary(
                &g.facade.resolve_for_massing(&MassingId::try_new("l_shape").expect("l_shape")),
            ),
        })
    });
    let sample: Option<GrammarGenerateResult> = generate("IndustrialWarehouse", "industrial_west", 43).ok();
    let green = bq_h1_facade_witness_green();
    serde_json::json!({
        "gate": "BQ-H1-FACADE-001",
        "green": green,
        "massing_table_ok": bq_h1_facade_massing_table_green(),
        "generate_propagation_ok": bq_h1_generate_propagation_green(),
        "massing_facade_table": table,
        "sample_generate": sample.map(|r| serde_json::json!({
            "massing_strategy": r.massing_strategy,
            "door_rhythm": r.door_rhythm,
            "placement_tags": r.placement_tags,
        })),
        "default_rhythms": {
            "long_hall": default_door_rhythm_for_massing(&MassingId::try_new("long_hall").unwrap()),
            "yard_complex": default_door_rhythm_for_massing(&MassingId::try_new("yard_complex").unwrap()),
            "l_shape": default_door_rhythm_for_massing(&MassingId::try_new("l_shape").unwrap()),
        },
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-H1",
    })
}

#[must_use]
pub fn refresh_bq_h1_facade_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_bq_h1_facade_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "BQ-H1-FACADE-001",
        "refresh_bq_h1_facade_witness",
        BQ_H1_LIVE_JSON,
        body,
    );
    write_debug_run_json(BQ_H1_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bq_h1_facade_witness_green_lib() {
        assert!(bq_h1_facade_witness_green());
    }

    #[test]
    fn bq_h1_refresh_witness_when_green() {
        if bq_h1_facade_witness_green() {
            assert!(refresh_bq_h1_facade_witness());
        }
    }
}
