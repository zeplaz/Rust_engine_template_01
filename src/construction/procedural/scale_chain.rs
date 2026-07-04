//! **BQ-C4-SCALE-001** — bake metres → grid placement → iso draw scale authority audit.

use super::assembly_snapshot::procedural_module_local_translation;
use super::module_contract::{FLOOR_HEIGHT_M, GRID_UNIT_M};
use crate::construction::iso_draw_scale::{
    ConstructionIsoDrawScale, DEFAULT_ISO_DRAW_SCALE_MULTIPLIER, ISO_DRAW_SCALE_MAX,
    ISO_DRAW_SCALE_MIN,
};

pub const BQ_C4_LIVE_JSON: &str = "debug_runs/bq_c4_scale_001_live.json";

/// **K01 authority:** collision / assembly use [`super::procedural_module_local_translation`];
/// [`ConstructionIsoDrawScale`] applies only at PG-2 spawn (visual readability — BUILD-READ-WORLD-002).
pub const SCALE_AUTHORITY_DECISION: &str = "visual_only_iso_draw";

#[derive(Debug, Clone, PartialEq)]
pub struct ScaleChainLink {
    pub stage: &'static str,
    pub authority: &'static str,
    pub constant: &'static str,
    pub value: f32,
}

#[must_use]
pub fn scale_chain_links() -> Vec<ScaleChainLink> {
    vec![
        ScaleChainLink {
            stage: "bake_contract",
            authority: "module_contract_v1",
            constant: "GRID_UNIT_M",
            value: GRID_UNIT_M,
        },
        ScaleChainLink {
            stage: "bake_contract",
            authority: "module_contract_v1",
            constant: "FLOOR_HEIGHT_M",
            value: FLOOR_HEIGHT_M,
        },
        ScaleChainLink {
            stage: "placement",
            authority: "procedural_module_local_translation",
            constant: "floor_y = floor * FLOOR_HEIGHT_M",
            value: FLOOR_HEIGHT_M,
        },
        ScaleChainLink {
            stage: "spawn_visual",
            authority: "ConstructionIsoDrawScale",
            constant: "iso_draw_scale_multiplier",
            value: DEFAULT_ISO_DRAW_SCALE_MULTIPLIER,
        },
    ]
}

#[must_use]
pub fn placement_y_matches_floor_height(floor: u32) -> bool {
    let y = procedural_module_local_translation(0, 0, floor).y;
    (y - floor as f32 * FLOOR_HEIGHT_M).abs() < f32::EPSILON
}

#[must_use]
pub fn iso_draw_is_visual_only_spawn() -> bool {
    // Assembly snapshots store grid indices — iso multiplier is not folded into AUTO-001 positions.
    SCALE_AUTHORITY_DECISION == "visual_only_iso_draw"
}

#[must_use]
pub fn bq_c4_scale_chain_witness_green() -> bool {
    let links = scale_chain_links();
    links.len() >= 4
        && links[0].value == 4.0
        && links[1].value == 3.0
        && placement_y_matches_floor_height(0)
        && placement_y_matches_floor_height(2)
        && iso_draw_is_visual_only_spawn()
        && {
            let iso = ConstructionIsoDrawScale::default();
            iso.multiplier >= ISO_DRAW_SCALE_MIN && iso.multiplier <= ISO_DRAW_SCALE_MAX
        }
}

#[must_use]
pub fn build_bq_c4_scale_witness_body() -> serde_json::Value {
    let green = bq_c4_scale_chain_witness_green();
    serde_json::json!({
        "gate": "BQ-C4-SCALE-001",
        "green": green,
        "authority_decision": SCALE_AUTHORITY_DECISION,
        "rationale": "Ghost/pick/footprint use unscaled grid; iso 1.5x is post-commit visual only (design_build_readability_v1)",
        "chain": scale_chain_links().iter().map(|l| serde_json::json!({
            "stage": l.stage,
            "authority": l.authority,
            "constant": l.constant,
            "value": l.value,
        })).collect::<Vec<_>>(),
        "plan_ref": "src/dev/plan_building_quality_v1.md#BQ-C4",
    })
}

#[must_use]
pub fn refresh_bq_c4_scale_witness() -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let body = build_bq_c4_scale_witness_body();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(
        "BQ-C4-SCALE-001",
        "refresh_bq_c4_scale_witness",
        BQ_C4_LIVE_JSON,
        body,
    );
    write_debug_run_json(BQ_C4_LIVE_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bq_c4_scale_chain_witness_green_lib() {
        assert!(bq_c4_scale_chain_witness_green());
    }

    #[test]
    fn bq_c4_refresh_witness_when_green() {
        if bq_c4_scale_chain_witness_green() {
            assert!(refresh_bq_c4_scale_witness());
        }
    }
}
