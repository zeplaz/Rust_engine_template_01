//! **BQ-Q3-GOLDEN-001** — committed golden-seed regression set (~12 seeds × archetype × district).
//!
//! Any grammar/kit change must diff against these hashes; update only with operator approval.

use super::assembly_snapshot::{
    assembly_snapshot_stable_hash, build_assembly_snapshot_from_grammar,
};
use super::{load_procedural_module_registry, load_style_pack_registry};
use crate::construction::PILOT_GRAMMAR_ARCHETYPE_WAREHOUSE;

pub const BQ_Q3_LIVE_JSON: &str = "debug_runs/bq_q3_golden_001_live.json";

#[derive(Clone, Copy, Debug)]
pub struct GoldenSeedEntry {
    pub archetype_id: &'static str,
    pub district_style: &'static str,
    pub seed: u64,
    pub expected_hash: &'static str,
}

/// Operator-approved baseline (2026-07-03, BQ-K3 refresh 2026-07-04). Refresh via `bq_q3_golden_bootstrap_hashes` when intentional.
pub const GOLDEN_SEED_SET_V1: &[GoldenSeedEntry] = &[
    GoldenSeedEntry {
        archetype_id: PILOT_GRAMMAR_ARCHETYPE_WAREHOUSE,
        district_style: "industrial_west",
        seed: 43,
        expected_hash: "892d651edb4be9bbab05a1bf6c0d4b7194498c01029b624c1297ee86209f0ff6",
    },
    GoldenSeedEntry {
        archetype_id: PILOT_GRAMMAR_ARCHETYPE_WAREHOUSE,
        district_style: "industrial_west",
        seed: 44,
        expected_hash: "0677e981e918998babd0a6cc9686505ae9c648b5f10ef72a23153d3f47e5877a",
    },
    GoldenSeedEntry {
        archetype_id: PILOT_GRAMMAR_ARCHETYPE_WAREHOUSE,
        district_style: "industrial_west",
        seed: 45,
        expected_hash: "b1e9339907cdf887f4a87f5dfcf606b3ccd0ee19cfbf6806a2f29f769d17f6a1",
    },
    GoldenSeedEntry {
        archetype_id: PILOT_GRAMMAR_ARCHETYPE_WAREHOUSE,
        district_style: "industrial_west",
        seed: 46,
        expected_hash: "993d591dd12890f3cdc7c296f03afc6919a0024432617c32cbad2d9257c8a9c8",
    },
    GoldenSeedEntry {
        archetype_id: "FactoryCluster",
        district_style: "manufacturing_row",
        seed: 35,
        expected_hash: "2434101afd07d8a3453f6977f07c29d6e333396e045c1b0a9316d7d1e42cb458",
    },
    GoldenSeedEntry {
        archetype_id: "FactoryCluster",
        district_style: "manufacturing_row",
        seed: 36,
        expected_hash: "4233c3388bbe5777f98f35418f7e5990bacafc6e3120851a562f851e3eb9a2c4",
    },
    GoldenSeedEntry {
        archetype_id: "FactoryCluster",
        district_style: "manufacturing_row",
        seed: 37,
        expected_hash: "6974652df9ff19a34aae3f7da263ace1165870165be3b9114c08d20f495221b1",
    },
    GoldenSeedEntry {
        archetype_id: "RailEdge",
        district_style: "rail_yard_corridor",
        seed: 42,
        expected_hash: "28b7a7299edc313b4a48459ab32ec47eb9834780744cb1c282ea317145a0c7ac",
    },
    GoldenSeedEntry {
        archetype_id: "RailEdge",
        district_style: "rail_yard_corridor",
        seed: 43,
        expected_hash: "997eb0c4ab6b843bc5716a0787f3bc7f588d2db648b952a3aa7f4df2a92f368f",
    },
    GoldenSeedEntry {
        archetype_id: "CivicBlock",
        district_style: "main_street_civic",
        seed: 99,
        expected_hash: "0063f860ab9f9688cb220e3cbf52dc09e3076e554c6e301da114a09e4105dc98",
    },
    GoldenSeedEntry {
        archetype_id: "CivicBlock",
        district_style: "main_street_civic",
        seed: 100,
        expected_hash: "3eb738e0fc9d962b995e11518354786f657258b5238ba452663b61b0948931ae",
    },
    GoldenSeedEntry {
        archetype_id: "CivicBlock",
        district_style: "town_hall_row",
        seed: 101,
        expected_hash: "1dc335830d87eb82d79e1b13298c0bb5e2bed6b5ab7a4388e7896de1582fc4d2",
    },
];

#[must_use]
pub fn golden_seed_hash_for(entry: &GoldenSeedEntry) -> Result<String, String> {
    let modules = load_procedural_module_registry();
    let packs = load_style_pack_registry();
    if !modules.load_errors.is_empty() || !packs.load_errors.is_empty() {
        return Err(format!(
            "registry load errors: modules={} packs={}",
            modules.load_errors.len(),
            packs.load_errors.len()
        ));
    }
    let snapshot = build_assembly_snapshot_from_grammar(
        entry.archetype_id,
        entry.district_style,
        entry.seed,
        &modules,
        &packs,
    )?;
    Ok(assembly_snapshot_stable_hash(&snapshot))
}

#[must_use]
pub fn bq_q3_golden_regression_green() -> bool {
    GOLDEN_SEED_SET_V1.iter().all(|entry| {
        golden_seed_hash_for(entry)
            .ok()
            .is_some_and(|h| h == entry.expected_hash)
    })
}

#[must_use]
pub fn build_bq_q3_golden_witness_body() -> serde_json::Value {
    let mut rows = Vec::new();
    let mut pass = 0usize;
    for entry in GOLDEN_SEED_SET_V1 {
        let actual = golden_seed_hash_for(entry).ok();
        let ok = actual.as_deref() == Some(entry.expected_hash);
        if ok {
            pass += 1;
        }
        rows.push(serde_json::json!({
            "archetype_id": entry.archetype_id,
            "district_style": entry.district_style,
            "seed": entry.seed,
            "expected_hash": entry.expected_hash,
            "actual_hash": actual,
            "match": ok,
        }));
    }
    let total = GOLDEN_SEED_SET_V1.len();
    let green = pass == total;
    serde_json::json!({
        "gate": "BQ-Q3-GOLDEN-001",
        "green": green,
        "pass_count": pass,
        "total": total,
        "seeds": rows,
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-Q3",
    })
}

#[must_use]
pub fn refresh_bq_q3_golden_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_bq_q3_golden_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "BQ-Q3-GOLDEN-001",
        "refresh_bq_q3_golden_witness",
        BQ_Q3_LIVE_JSON,
        body,
    );
    write_debug_run_json(BQ_Q3_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bq_q3_golden_bootstrap_hashes() {
        for entry in GOLDEN_SEED_SET_V1 {
            let hash = golden_seed_hash_for(entry).expect("snapshot");
            eprintln!(
                "GoldenSeedEntry {{ archetype_id: \"{}\", district_style: \"{}\", seed: {}, expected_hash: \"{}\" }},",
                entry.archetype_id, entry.district_style, entry.seed, hash
            );
        }
    }

    #[test]
    fn bq_q3_golden_regression_matches_committed_hashes() {
        assert!(
            bq_q3_golden_regression_green(),
            "golden seed hash drift — run bq_q3_golden_bootstrap_hashes and update GOLDEN_SEED_SET_V1 with operator approval"
        );
    }

    #[test]
    fn bq_q3_refresh_witness_when_green() {
        if bq_q3_golden_regression_green() {
            assert!(refresh_bq_q3_golden_witness());
        }
    }
}
