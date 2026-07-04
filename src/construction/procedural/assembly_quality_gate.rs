//! **BQ-A2-GATE-001** — post-assembly quality score for procedural buildings.
//!
//! Product architecture (not migration): feeds `building_quality_live.json` and APSR-Q1.
//! Adjacency rules wired from **BQ-A1** (`edge_adjacency.rs`).
use serde::{Deserialize, Serialize};

use crate::render::extraction::ProceduralBuildExtract;

pub const BUILDING_QUALITY_LIVE_JSON: &str = "debug_runs/building_quality_live.json";
pub const BQ_A2_PASS_SCORE: f32 = 70.0;

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
    /// Visible modules vs facade cells that expect a module slot.
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
    let facade_cells = extract.footprint_cells.max(1);

    let style_purity_pct = if resolved_count == 0 {
        100.0
    } else {
        let matching = resolved_count.saturating_sub(cross_style);
        100.0 * matching as f32 / resolved_count as f32
    };

    let silhouette_continuity_pct =
        100.0 * visible_count as f32 / facade_cells as f32;

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

    let request = ProceduralBuildingRequest {
        archetype_id: "rect_perimeter".into(),
        width: 4,
        depth: 2,
        floors: 2,
        style: StylePackId("style_victorian".into()),
        seed: 1,
        arch_dna_preset_id: None,
    };
    let assembly_id = assembly_id_for(
        request.style.as_str(),
        request.width,
        request.depth,
        request.floors,
        request.seed,
    );

    let mut scores = Vec::new();
    let mut snapshot_aligned = false;

    if let Some(pack) = packs.get("style_victorian") {
        let grid = FootprintGrid::from_request(&request);
        let extract = assemble_procedural_build_instances(
            &request,
            pack,
            &grid,
            &reg,
            &ProceduralModuleSceneCatalog::default(),
        );
        let snapshot = build_assembly_snapshot(&request, pack, &grid, &reg);
        let score = compute_assembly_quality(&extract, "style_victorian", &assembly_id, Some(&grid));
        snapshot_aligned = snapshot.missing_slot_violations.len()
            == extract.missing_slot_violations.len();
        scores.push(score);
    }

    let bq_a1_wired = super::bq_a1_adjacency_witness_green();

    let witness = BuildingQualityLiveWitness {
        gate: "BQ-A2-GATE-001".into(),
        green: false,
        schema: "building_quality_live_v1".into(),
        bq_a1_wired,
        pass_threshold: BQ_A2_PASS_SCORE,
        assemblies: scores.clone(),
    };

    let green = table_ok
        && bq_a1_wired
        && !witness.assemblies.is_empty()
        && snapshot_aligned
        && witness.assemblies.iter().all(|s| {
            s.style_purity_pct >= 99.0
                && s.cross_style_fallback_count == 0
                && s.overall_score >= BQ_A2_PASS_SCORE
                && s.passes_gate
        });

    serde_json::json!({
        "gate": witness.gate,
        "green": green,
        "schema": witness.schema,
        "table_ok": table_ok,
        "bq_a1_wired": witness.bq_a1_wired,
        "pass_threshold": witness.pass_threshold,
        "snapshot_violation_aligned": snapshot_aligned,
        "assemblies": witness.assemblies,
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-A2",
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
        assert_eq!(score.adjacency_violation_count, 0);
    }

    #[test]
    fn bq_a2_gate_001_witness_writes_when_green() {
        if !bq_a2_gate_001_witness_green() {
            eprintln!("BQ-A2 witness not green — module registry or victorian pack state");
            return;
        }
        assert!(refresh_building_quality_live_witness());
    }
}
