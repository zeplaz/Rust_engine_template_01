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
    /// Visible instances / total instances (0..=100). Story-aware — never exceeds 100.
    pub silhouette_continuity_pct: f32,
    pub overall_score: f32,
    pub passes_gate: bool,
}

#[must_use]
pub fn compute_assembly_quality(
    extract: &ProceduralBuildExtract,
    style_pack_id: &str,
    assembly_id: &str,
    grid: Option<&super::FootprintGrid>,
) -> AssemblyQualityScore {
    let visible_count = extract.instances.iter().filter(|i| !i.hidden).count() as u32;
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
        let pack = packs.get("style_victorian").expect("style_victorian pack");
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
        let pack = packs.get("style_victorian").expect("style_victorian pack");
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
        let dishonest = 100.0 * extract.instances.iter().filter(|i| !i.hidden).count() as f32
            / extract.footprint_cells.max(1) as f32;
        assert!(
            dishonest > 100.0,
            "fixture must still exhibit dishonest WDC ratio (>100); got {dishonest}"
        );
    }



}
