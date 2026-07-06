//! **BQ-Q3-GOLDEN-001** — committed golden-seed regression set (~12 seeds × archetype × district).
//!
//! Any grammar/kit change must diff against these hashes; update only with operator approval.

use super::assembly_snapshot::{assembly_snapshot_stable_hash, build_assembly_snapshot_from_grammar};
use super::{load_procedural_module_registry, load_style_pack_registry};

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
        archetype_id: "IndustrialWarehouse",
        district_style: "industrial_west",
        seed: 43,
        expected_hash: "6d8031cfea5ad1c3ec81f891386f6375b422cd668a2958cb49cdfb0a849f6b09",
    },
    GoldenSeedEntry {
        archetype_id: "IndustrialWarehouse",
        district_style: "industrial_west",
        seed: 44,
        expected_hash: "e70133127bd7ad0609ac042953acae6dbc4c499e8a138fa341b18cb28ed2f30e",
    },
    GoldenSeedEntry {
        archetype_id: "IndustrialWarehouse",
        district_style: "industrial_west",
        seed: 45,
        expected_hash: "fa0f147f89014aba668ec53c6b672e05e129a8a52df4bdb954de7aa2ec21c708",
    },
    GoldenSeedEntry {
        archetype_id: "IndustrialWarehouse",
        district_style: "industrial_west",
        seed: 46,
        expected_hash: "cfe4516da50c45d5ad2900920116f490243e70c01c7b21d1eb1c5814aa40b2f6",
    },
    GoldenSeedEntry {
        archetype_id: "FactoryCluster",
        district_style: "manufacturing_row",
        seed: 35,
        expected_hash: "b44a3d2b12c73271e293d0b27ab30246a3b4f31c8dc0a3f22014dc0c39f56d30",
    },
    GoldenSeedEntry {
        archetype_id: "FactoryCluster",
        district_style: "manufacturing_row",
        seed: 36,
        expected_hash: "363a3a6978549e14195052ae2b6e780fe3eee34723fbb850bf84cd8559bfbe08",
    },
    GoldenSeedEntry {
        archetype_id: "FactoryCluster",
        district_style: "manufacturing_row",
        seed: 37,
        expected_hash: "b9bab56562f4560f2520117634a7cfa457d76c7da51e494eba7b71f93349bd3c",
    },
    GoldenSeedEntry {
        archetype_id: "RailEdge",
        district_style: "rail_yard_corridor",
        seed: 42,
        expected_hash: "5ec5e25e8fa682c80411c9bf1d742d5ffa672427fd23b9458ed89b5ffc359a15",
    },
    GoldenSeedEntry {
        archetype_id: "RailEdge",
        district_style: "rail_yard_corridor",
        seed: 43,
        expected_hash: "4036fc1c02d000535698d07ea8f72e9f8738e08f773c4375dc9073bfd0143a04",
    },
    GoldenSeedEntry {
        archetype_id: "CivicBlock",
        district_style: "main_street_civic",
        seed: 99,
        expected_hash: "439c5667ac50f0ad0213fde4c9cf697bc3883c508bed95f8c98d769fee898ae9",
    },
    GoldenSeedEntry {
        archetype_id: "CivicBlock",
        district_style: "main_street_civic",
        seed: 100,
        expected_hash: "3aa9ff1f3258d11795c02e2ec41460a449f295392ff203dbf9a37705c785d2cd",
    },
    GoldenSeedEntry {
        archetype_id: "CivicBlock",
        district_style: "town_hall_row",
        seed: 101,
        expected_hash: "92e49be09a47d7fe6c510136928971363bd81ebff73fc23e71eac6c5dde696c8",
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
