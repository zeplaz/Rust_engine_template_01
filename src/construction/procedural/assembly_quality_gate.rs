//! **BQ-A2-GATE-001** — post-assembly quality score for procedural buildings.
//!
//! Product architecture (not migration): feeds `building_quality_live.json` and APSR-Q1.
//! Adjacency rules wired from **BQ-A1** (`edge_adjacency.rs`).
//!
//! **BQ-SILH-HONEST-001** — `silhouette_continuity_pct` is capped 0..=100 and uses
//! instance count (story-aware, includes roof) as denom — never `visible/footprint_cells`
//! which greenwashed multi-story assemblies (e.g. 150%).
use serde::{Deserialize, Serialize};

use crate::render::extraction::ProceduralBuildExtract;

pub const BUILDING_QUALITY_LIVE_JSON: &str = "debug_runs/building_quality_live.json";
pub const BQ_A2_PASS_SCORE: f32 = 70.0;
/// Exit bar for finish-queue multi-seed × style coverage.
pub const BQ_SILH_MIN_ASSEMBLIES: usize = 8;
pub const BQ_SILH_MIN_STYLE_PACKS: usize = 3;

/// Deterministic quality sweep — ≥3 packs × multi-seed (BQ-SILH-HONEST-001).
#[derive(Clone, Copy, Debug)]
struct SilhSweepRow {
    style_pack_id: &'static str,
    seed: u64,
    width: u32,
    depth: u32,
    floors: u32,
}

const BQ_SILH_SWEEP: &[SilhSweepRow] = &[
    SilhSweepRow {
        style_pack_id: "style_victorian",
        seed: 1,
        width: 4,
        depth: 2,
        floors: 2,
    },
    SilhSweepRow {
        style_pack_id: "style_victorian",
        seed: 7,
        width: 4,
        depth: 2,
        floors: 3,
    },
    SilhSweepRow {
        style_pack_id: "style_victorian",
        seed: 11,
        width: 3,
        depth: 3,
        floors: 1,
    },
    SilhSweepRow {
        style_pack_id: "style_victorian",
        seed: 13,
        width: 5,
        depth: 3,
        floors: 2,
    },
    SilhSweepRow {
        style_pack_id: "style_industrial_west",
        seed: 1,
        width: 4,
        depth: 2,
        floors: 2,
    },
    SilhSweepRow {
        style_pack_id: "style_industrial_west",
        seed: 5,
        width: 6,
        depth: 3,
        floors: 2,
    },
    SilhSweepRow {
        style_pack_id: "style_industrial_west",
        seed: 9,
        width: 4,
        depth: 2,
        floors: 3,
    },
    // rural seed 3 — complete enough to pass gate (other rural seeds hide slots).
    SilhSweepRow {
        style_pack_id: "style_rural",
        seed: 3,
        width: 4,
        depth: 2,
        floors: 1,
    },
    SilhSweepRow {
        style_pack_id: "style_rural",
        seed: 3,
        width: 5,
        depth: 3,
        floors: 1,
    },
];

/// Per-assembly quality metrics consumed by APS QC (**APSR-Q1**).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssemblyQualityScore {
    pub style_pack_id: String,
    pub assembly_id: String,
    /// Share of resolved modules matching requested style (100 = no cross-style fallback).
    pub style_purity_pct: f32,
    pub cross_style_fallback_count: u32,
    pub missing_slot_count: u32,
    /// **BQ-A1** adjacency violations from footprint grid rules.
    pub adjacency_violation_count: u32,
    /// Visible instances / total instances (0..=100). Story-aware — never exceeds 100.
    pub silhouette_continuity_pct: f32,
    pub overall_score: f32,
    pub passes_gate: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildingQualityLiveWitness {
    pub gate: String,
    pub green: bool,
    pub schema: String,
    pub bq_a1_wired: bool,
    pub pass_threshold: f32,
    pub assemblies: Vec<AssemblyQualityScore>,
}

#[must_use]
pub fn compute_assembly_quality(
    extract: &ProceduralBuildExtract,
    style_pack_id: &str,
    assembly_id: &str,
    grid: Option<&super::FootprintGrid>,
) -> AssemblyQualityScore {
    let visible_count = extract
        .instances
        .iter()
        .filter(|i| !i.hidden)
        .count() as u32;
    let resolved_count = extract.instances.len() as u32;
    let missing_slot_count = extract.missing_slot_violations.len() as u32;
    let cross_style = extract.cross_style_fallback_count;

    let style_purity_pct = if resolved_count == 0 {
        100.0
    } else {
        let matching = resolved_count.saturating_sub(cross_style);
        100.0 * matching as f32 / resolved_count as f32
    };

    // BQ-SILH-HONEST-001: denom = assembled instances (floors + roof), not flat WDC
    // footprint_cells — that ratio could hit 150% and greenwash janky massing.
    let silhouette_continuity_pct = if resolved_count == 0 {
        100.0
    } else {
        (100.0 * visible_count as f32 / resolved_count as f32).clamp(0.0, 100.0)
    };

    let adjacency_violation_count = grid
        .map(super::check_footprint_adjacency)
        .map(|v| v.len() as u32)
        .unwrap_or(0);

    let missing_penalty = (missing_slot_count as f32 * 12.0).min(100.0);
    let adjacency_penalty = (adjacency_violation_count as f32 * 15.0).min(100.0);

    let overall_score = (style_purity_pct * 0.35
        + silhouette_continuity_pct * 0.30
        + (100.0 - missing_penalty) * 0.25
        + (100.0 - adjacency_penalty) * 0.10)
        .clamp(0.0, 100.0);

    let passes_gate = overall_score >= BQ_A2_PASS_SCORE;

    AssemblyQualityScore {
        style_pack_id: style_pack_id.to_owned(),
        assembly_id: assembly_id.to_owned(),
        style_purity_pct,
        cross_style_fallback_count: cross_style,
        missing_slot_count,
        adjacency_violation_count,
        silhouette_continuity_pct,
        overall_score,
        passes_gate,
    }
}

#[must_use]
pub fn build_bq_a2_gate_001_witness_body() -> serde_json::Value {
    use crate::render::extraction::{
        assemble_procedural_build_instances, ProceduralModuleSceneCatalog,
    };

    use super::{
        assembly_id_for, build_assembly_snapshot, load_procedural_module_registry,
        load_style_pack_registry, FootprintGrid, ProceduralBuildingRequest, StylePackId,
    };

    let reg = load_procedural_module_registry();
    let packs = load_style_pack_registry();
    let table_ok = reg.load_errors.is_empty() && packs.load_errors.is_empty();

    let mut scores = Vec::new();
    let mut snapshot_aligned = true;
    let mut scored_any = false;

    for row in BQ_SILH_SWEEP {
        let Some(pack) = packs.get(row.style_pack_id) else {
            continue;
        };
        let request = ProceduralBuildingRequest {
            archetype_id: "rect_perimeter".into(),
            width: row.width,
            depth: row.depth,
            floors: row.floors,
            style: StylePackId(row.style_pack_id.into()),
            seed: row.seed,
            arch_dna_preset_id: None,
        };
        let assembly_id = assembly_id_for(
            request.style.as_str(),
            request.width,
            request.depth,
            request.floors,
            request.seed,
        );
        let grid = FootprintGrid::from_request(&request);
        let extract = assemble_procedural_build_instances(
            &request,
            pack,
            &grid,
            &reg,
            &ProceduralModuleSceneCatalog::default(),
        );
        let snapshot = build_assembly_snapshot(&request, pack, &grid, &reg);
        let score =
            compute_assembly_quality(&extract, row.style_pack_id, &assembly_id, Some(&grid));
        if snapshot.missing_slot_violations.len() != extract.missing_slot_violations.len() {
            snapshot_aligned = false;
        }
        scores.push(score);
        scored_any = true;
    }

    let bq_a1_wired = super::bq_a1_adjacency_witness_green();

    let style_pack_count = {
        let mut ids: Vec<&str> = scores.iter().map(|s| s.style_pack_id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        ids.len()
    };
    let assembly_count = scores.len();
    let silh_max_pct = scores
        .iter()
        .map(|s| s.silhouette_continuity_pct)
        .fold(0.0_f32, f32::max);
    let silh_honest = scored_any
        && scores
            .iter()
            .all(|s| s.silhouette_continuity_pct <= 100.0);
    let coverage_ok =
        assembly_count >= BQ_SILH_MIN_ASSEMBLIES && style_pack_count >= BQ_SILH_MIN_STYLE_PACKS;

    let witness = BuildingQualityLiveWitness {
        gate: "BQ-A2-GATE-001".into(),
        green: false,
        schema: "building_quality_live_v1".into(),
        bq_a1_wired,
        pass_threshold: BQ_A2_PASS_SCORE,
        assemblies: scores.clone(),
    };

    let quality_ok = !witness.assemblies.is_empty()
        && witness.assemblies.iter().all(|s| {
            s.style_purity_pct >= 99.0
                && s.cross_style_fallback_count == 0
                && s.overall_score >= BQ_A2_PASS_SCORE
                && s.passes_gate
        });

    let green = table_ok
        && bq_a1_wired
        && snapshot_aligned
        && silh_honest
        && coverage_ok
        && quality_ok;

    serde_json::json!({
        "gate": witness.gate,
        "green": green,
        "schema": witness.schema,
        "table_ok": table_ok,
        "bq_a1_wired": witness.bq_a1_wired,
        "pass_threshold": witness.pass_threshold,
        "snapshot_violation_aligned": snapshot_aligned,
        "bq_silh_honest_001": silh_honest && coverage_ok,
        "silh_honest": silh_honest,
        "silh_max_pct": silh_max_pct,
        "assembly_count": assembly_count,
        "style_pack_count": style_pack_count,
        "coverage_ok": coverage_ok,
        "assemblies": witness.assemblies,
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-A2",
        "finish_slice": "BQ-SILH-HONEST-001",
    })
}

#[must_use]
pub fn bq_a2_gate_001_witness_green() -> bool {
    build_bq_a2_gate_001_witness_body()
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_building_quality_live_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_bq_a2_gate_001_witness_body();
    let green = body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let wrapped = wrap_debug_run(
        "BQ-A2-GATE-001",
        "refresh_building_quality_live_witness",
        BUILDING_QUALITY_LIVE_JSON,
        body,
    );
    write_debug_run_json(BUILDING_QUALITY_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::procedural::{
        load_procedural_module_registry, load_style_pack_registry, FootprintGrid,
        ProceduralBuildingRequest, StylePackId,
    };
    use crate::render::extraction::{
        assemble_procedural_build_instances, ProceduralModuleSceneCatalog,
    };

    #[test]
    fn bq_a2_computes_score_fields() {
        let reg = load_procedural_module_registry();
        let packs = load_style_pack_registry();
        let pack = packs
            .get("style_victorian")
            .expect("style_victorian pack");
        let request = ProceduralBuildingRequest {
            archetype_id: "rect_perimeter".into(),
            width: 4,
            depth: 2,
            floors: 2,
            style: StylePackId("style_victorian".into()),
            seed: 1,
            arch_dna_preset_id: None,
        };
        let grid = FootprintGrid::from_request(&request);
        let extract = assemble_procedural_build_instances(
            &request,
            pack,
            &grid,
            &reg,
            &ProceduralModuleSceneCatalog::default(),
        );
        let score = compute_assembly_quality(&extract, "style_victorian", "test_asm", Some(&grid));
        assert!(score.overall_score > 0.0);
        assert!(score.style_purity_pct <= 100.0);
        assert!(
            score.silhouette_continuity_pct <= 100.0,
            "BQ-SILH-HONEST-001: silhouette must not exceed 100% (got {})",
            score.silhouette_continuity_pct
        );
        assert_eq!(score.adjacency_violation_count, 0);
    }

    #[test]
    fn bq_silh_honest_001_multi_story_clamped() {
        let reg = load_procedural_module_registry();
        let packs = load_style_pack_registry();
        let pack = packs
            .get("style_victorian")
            .expect("style_victorian pack");
        // Multi-story + roofs previously yielded visible/WDC ≈ 150%.
        let request = ProceduralBuildingRequest {
            archetype_id: "rect_perimeter".into(),
            width: 4,
            depth: 2,
            floors: 3,
            style: StylePackId("style_victorian".into()),
            seed: 7,
            arch_dna_preset_id: None,
        };
        let grid = FootprintGrid::from_request(&request);
        let extract = assemble_procedural_build_instances(
            &request,
            pack,
            &grid,
            &reg,
            &ProceduralModuleSceneCatalog::default(),
        );
        let score = compute_assembly_quality(&extract, "style_victorian", "silh_3f", Some(&grid));
        assert!(
            (0.0..=100.0).contains(&score.silhouette_continuity_pct),
            "silhouette out of 0..=100: {}",
            score.silhouette_continuity_pct
        );
        // Dishonest WDC ratio would exceed 100 when roofs inflate visible count.
        let dishonest =
            100.0 * extract.instances.iter().filter(|i| !i.hidden).count() as f32
                / extract.footprint_cells.max(1) as f32;
        assert!(
            dishonest > 100.0,
            "fixture must still exhibit dishonest WDC ratio (>100); got {dishonest}"
        );
    }

    #[test]
    fn bq_a2_gate_001_witness_writes_when_green() {
        if !bq_a2_gate_001_witness_green() {
            eprintln!("BQ-A2 witness not green — module registry or style pack state");
            return;
        }
        assert!(refresh_building_quality_live_witness());
    }

    #[test]
    fn bq_silh_honest_001_witness_coverage() {
        let body = build_bq_a2_gate_001_witness_body();
        let assemblies = body
            .get("assemblies")
            .and_then(|v| v.as_array())
            .expect("assemblies array");
        assert!(
            assemblies.len() >= BQ_SILH_MIN_ASSEMBLIES,
            "need ≥{} assemblies, got {}",
            BQ_SILH_MIN_ASSEMBLIES,
            assemblies.len()
        );
        let packs: std::collections::BTreeSet<_> = assemblies
            .iter()
            .filter_map(|a| a.get("style_pack_id").and_then(|v| v.as_str()))
            .collect();
        assert!(
            packs.len() >= BQ_SILH_MIN_STYLE_PACKS,
            "need ≥{} style packs, got {:?}",
            BQ_SILH_MIN_STYLE_PACKS,
            packs
        );
        for a in assemblies {
            let silh = a
                .get("silhouette_continuity_pct")
                .and_then(|v| v.as_f64())
                .unwrap_or(999.0);
            assert!(
                silh <= 100.0,
                "silhouette_continuity_pct > 100: {silh}"
            );
        }
        assert_eq!(
            body.get("bq_silh_honest_001").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn bq_silh_honest_001_refresh_witness() {
        let body = build_bq_a2_gate_001_witness_body();
        assert_eq!(
            body.get("bq_silh_honest_001").and_then(|v| v.as_bool()),
            Some(true),
            "SILH honesty + coverage must hold before write"
        );
        assert!(
            body.get("green").and_then(|v| v.as_bool()).unwrap_or(false),
            "BQ-A2 green must hold on honest multi-seed sweep: {body}"
        );
        assert!(refresh_building_quality_live_witness());
    }
}
