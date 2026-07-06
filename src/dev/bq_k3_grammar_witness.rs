//! **BQ-K3-GRAMMAR-001** — grammar enrichment RON merge witness (massing · facade · APS age bands).

use crate::construction::procedural::{
    generate_building_grammar as generate, load_building_grammar_registry, BuildingGrammar,
};

pub const BQ_K3_LIVE_JSON: &str = "debug_runs/bq_k3_grammar_001_live.json";

const K3_GRAMMARS: &[(&str, &[&str])] = &[
    ("civic_block_v1", &["t_block", "u_courtyard"]),
    ("factory_cluster_v1", &["stepped_row", "t_loading"]),
    ("rail_edge_v1", &["t_rail_spur", "stepped_dock"]),
];

fn facade_override_tags(grammar: &BuildingGrammar, massing_id: &str) -> Option<(String, Vec<String>)> {
    grammar
        .facade
        .by_massing
        .iter()
        .find(|entry| entry.massing_id.as_str() == massing_id)
        .map(|entry| (entry.door_rhythm.clone(), entry.placement_tags.clone()))
}

#[must_use]
pub fn bq_k3_registry_load_green() -> bool {
    let registry = load_building_grammar_registry();
    registry.load_errors.is_empty()
}

#[must_use]
pub fn bq_k3_massing_strategies_green() -> bool {
    let registry = load_building_grammar_registry();
    K3_GRAMMARS.iter().all(|(grammar_id, new_ids)| {
        let Some(grammar) = registry.by_grammar_id(grammar_id) else {
            return false;
        };
        new_ids.iter().all(|id| {
            grammar
                .massing
                .strategies
                .iter()
                .any(|s| s.id.as_str() == *id)
        })
    })
}

#[must_use]
pub fn bq_k3_facade_by_massing_green() -> bool {
    let registry = load_building_grammar_registry();
    let civic = registry.by_grammar_id("civic_block_v1");
    let factory = registry.by_grammar_id("factory_cluster_v1");
    let rail = registry.by_grammar_id("rail_edge_v1");

    let civic_ok = civic.is_some_and(|g| {
        let Some((t_rhythm, t_tags)) = facade_override_tags(g, "t_block") else {
            return false;
        };
        let Some((u_rhythm, u_tags)) = facade_override_tags(g, "u_courtyard") else {
            return false;
        };
        t_rhythm == "stem_center"
            && t_tags.iter().any(|tag| tag == "t_stem")
            && u_rhythm == "court_perimeter"
            && u_tags.iter().any(|tag| tag == "courtyard_perimeter")
    });

    let factory_ok = factory.is_some_and(|g| {
        facade_override_tags(g, "stepped_row").is_some_and(|(rhythm, tags)| {
            rhythm == "bay_alternating" && tags.iter().any(|tag| tag == "stepped_facade")
        })
    });

    let rail_ok = rail.is_some_and(|g| {
        facade_override_tags(g, "t_rail_spur").is_some_and(|(rhythm, tags)| {
            rhythm == "spur_end" && tags.iter().any(|tag| tag == "rail_spur")
        })
    });

    civic_ok && factory_ok && rail_ok
}

#[must_use]
pub fn bq_k3_age_aps_tags_green() -> bool {
    let registry = load_building_grammar_registry();
    K3_GRAMMARS.iter().all(|(grammar_id, _)| {
        let Some(grammar) = registry.by_grammar_id(grammar_id) else {
            return false;
        };
        grammar.age.bands.iter().all(|band| {
            !band.aps_mandate_tags.is_empty() && !band.condition_tags.is_empty()
        })
    })
}

#[must_use]
pub fn bq_k3_footprint_modes_green() -> bool {
    let registry = load_building_grammar_registry();
    let civic = registry.by_grammar_id("civic_block_v1");
    civic.is_some_and(|g| {
        g.massing.strategies.iter().any(|s| s.footprint_mode.as_str() == "t_shape")
            && g
                .massing
                .strategies
                .iter()
                .any(|s| s.footprint_mode.as_str() == "u_shape")
    })
}

#[must_use]
pub fn bq_k3_generate_propagation_green() -> bool {
    let mut saw_civic_t = false;
    let mut saw_factory_stepped = false;
    let mut saw_rail_spur = false;
    for seed in 0..256u64 {
        if let Ok(r) = generate("CivicBlock", "main_street_civic", seed) {
            if r.massing_strategy == "t_block" {
                saw_civic_t = r.door_rhythm == "stem_center" && r.footprint_mode == "t_shape";
            }
        }
        if let Ok(r) = generate("FactoryCluster", "manufacturing_row", seed) {
            if r.massing_strategy == "stepped_row" {
                saw_factory_stepped = r.door_rhythm == "bay_alternating";
            }
        }
        if let Ok(r) = generate("RailEdge", "rail_yard_corridor", seed) {
            if r.massing_strategy == "t_rail_spur" {
                saw_rail_spur =
                    r.door_rhythm == "spur_end" && r.footprint_mode == "t_shape";
            }
        }
    }
    saw_civic_t && saw_factory_stepped && saw_rail_spur
}

#[must_use]
pub fn bq_k3_grammar_witness_green() -> bool {
    bq_k3_registry_load_green()
        && bq_k3_massing_strategies_green()
        && bq_k3_facade_by_massing_green()
        && bq_k3_age_aps_tags_green()
        && bq_k3_footprint_modes_green()
        && bq_k3_generate_propagation_green()
}

#[must_use]
pub fn build_bq_k3_grammar_witness_body() -> serde_json::Value {
    let registry = load_building_grammar_registry();
    let massing_counts: serde_json::Map<String, serde_json::Value> = K3_GRAMMARS
        .iter()
        .filter_map(|(id, _)| {
            registry.by_grammar_id(id).map(|g| {
                (
                    (*id).to_string(),
                    serde_json::json!(g.massing.strategies.len()),
                )
            })
        })
        .collect();

    serde_json::json!({
        "gate": "BQ-K3-GRAMMAR-001",
        "green": bq_k3_grammar_witness_green(),
        "registry_load_ok": bq_k3_registry_load_green(),
        "massing_strategies_ok": bq_k3_massing_strategies_green(),
        "facade_by_massing_ok": bq_k3_facade_by_massing_green(),
        "age_aps_tags_ok": bq_k3_age_aps_tags_green(),
        "footprint_modes_ok": bq_k3_footprint_modes_green(),
        "generate_propagation_ok": bq_k3_generate_propagation_green(),
        "massing_strategy_counts": massing_counts,
        "load_errors": registry.load_errors,
        "charter": "src/dev/design_bq_k3_grammar_enrichment_v1.md",
        "manifest": "tools/mcp/schemas/examples/bq_k3_grammar_enrichment_v1.json",
    })
}

#[must_use]
pub fn refresh_bq_k3_grammar_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_bq_k3_grammar_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "BQ-K3-GRAMMAR-001",
        "refresh_bq_k3_grammar_witness",
        BQ_K3_LIVE_JSON,
        body,
    );
    write_debug_run_json(BQ_K3_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bq_k3_registry_loads_without_errors() {
        assert!(
            bq_k3_registry_load_green(),
            "grammar load errors: {:?}",
            load_building_grammar_registry().load_errors
        );
    }

    #[test]
    fn bq_k3_grammar_witness_green_lib() {
        assert!(bq_k3_grammar_witness_green());
    }

    #[test]
    fn bq_k3_refresh_witness_when_green() {
        if bq_k3_grammar_witness_green() {
            assert!(refresh_bq_k3_grammar_witness());
        }
    }
}
