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

/// Operator-approved baseline (2026-07-03). Refresh via `bq_q3_golden_bootstrap_hashes` test when intentional.
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
        expected_hash: "46062d59c423463839e5f34d4f5e4009c9759bcabc6a4a9ce8b976db0e5ded11",
    },
    GoldenSeedEntry {
        archetype_id: "FactoryCluster",
        district_style: "manufacturing_row",
        seed: 36,
        expected_hash: "ee500d058422475ae2a3c172c37e1ff993cf0626737a599f2081e0a5aacc688a",
    },
    GoldenSeedEntry {
        archetype_id: "FactoryCluster",
        district_style: "manufacturing_row",
        seed: 37,
        expected_hash: "22478b847a5505a560657ef51644b8a4a64c87b6832a8f54b643eea71a90a1e1",
    },
    GoldenSeedEntry {
        archetype_id: "RailEdge",
        district_style: "rail_yard_corridor",
        seed: 42,
        expected_hash: "f6aef61394775c267cedce830774d6e15ae069b588bca89f6d56a5310cf63cf0",
    },
    GoldenSeedEntry {
        archetype_id: "RailEdge",
        district_style: "rail_yard_corridor",
        seed: 43,
        expected_hash: "ff4c2c49c3ad2715224499116b27109c73dfb823546c8a6f5ed62582d182892f",
    },
    GoldenSeedEntry {
        archetype_id: "CivicBlock",
        district_style: "main_street_civic",
        seed: 99,
        expected_hash: "27baa743f456f8435d9bf77b5ebfc5337a1761b675095b9081d6b22f18b68ee7",
    },
    GoldenSeedEntry {
        archetype_id: "CivicBlock",
        district_style: "main_street_civic",
        seed: 100,
        expected_hash: "c308a92a81a2256017ba4ce1a9bd38e0b9b83b4465800b6889eab037667bde92",
    },
    GoldenSeedEntry {
        archetype_id: "CivicBlock",
        district_style: "town_hall_row",
        seed: 101,
        expected_hash: "c9213893df4abb6bf56e115386c25d4ed9bab3b4d1733c5c73411aaf132bc5e1",
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
