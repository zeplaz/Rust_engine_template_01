//! **CODER-PRODUCT-VERIFY-001** — bundle lib witness refresh + G-PLAY rollup.

use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

pub const G_PLAY_PRODUCT_CLOSE_LIVE_JSON: &str = "debug_runs/g_play_product_close_live.json";
pub const BUILD_VERIFY_POINTER_LIVE_JSON: &str = "debug_runs/build_verify_pointer_live.json";

#[derive(Clone, Debug, Default)]
pub struct ProductVerifyRefreshResult {
    pub map_zoom: bool,
    pub pilot_catalog: bool,
    pub build_visual: bool,
    pub minimap: bool,
    pub fire_ecology: bool,
    pub vfx_fire: bool,
    pub fire_play_vis: bool,
    pub sim_effect_spine: bool,
    pub landscape_grammar: bool,
    pub pointer_gate: bool,
    pub rollup_green: bool,
}

#[must_use]
fn witness_file_green(path: &str) -> bool {
    let full = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|root| root.join(path))
        .unwrap_or_else(|| std::path::PathBuf::from(path));
    let Ok(raw) = std::fs::read_to_string(full) else {
        return false;
    };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    doc.get("green")
        .and_then(|v| v.as_bool())
        .or_else(|| doc.get("lib_contract_green").and_then(|v| v.as_bool()))
        .or_else(|| doc.get("f1_green").and_then(|v| v.as_bool()))
        .or_else(|| {
            doc.pointer("/play_truth_001/green")
                .and_then(|v| v.as_bool())
        })
        .or_else(|| {
            doc.pointer("/play_truth_001_tail/green")
                .and_then(|v| v.as_bool())
        })
        .unwrap_or(false)
}

/// WIT-RUST-003 — G-PLAY rollup requires non-lib_fixture proof grade.
#[must_use]
fn g_play_proof_grade_honest() -> bool {
    let stage5 = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|root| root.join("debug_runs/stage5_full_app_live.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("debug_runs/stage5_full_app_live.json"));
    let Ok(raw) = std::fs::read_to_string(stage5) else {
        return false;
    };
    let Ok(doc) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return false;
    };
    let grade = doc
        .pointer("/log_e01_visual_confirm_001/proof_grade")
        .or_else(|| doc.get("proof_grade"))
        .and_then(|v| v.as_str());
    matches!(grade, Some(g) if g != "lib_fixture")
}

#[must_use]
pub fn refresh_build_verify_pointer_live_witness() -> bool {
    let green = crate::gui::hud::simulation_pointer_gate::build_verify_pointer_001_witness_green();
    let body = crate::gui::hud::simulation_pointer_gate::build_verify_pointer_001_witness_json();
    if !green {
        return false;
    }
    let wrapped = wrap_debug_run(
        "BUILD-VERIFY-POINTER-001",
        "refresh_build_verify_pointer_live_witness",
        BUILD_VERIFY_POINTER_LIVE_JSON,
        body,
    );
    write_debug_run_json(BUILD_VERIFY_POINTER_LIVE_JSON, wrapped)
}

#[must_use]
pub fn refresh_product_verify_live_witnesses() -> ProductVerifyRefreshResult {
    let mut r = ProductVerifyRefreshResult::default();

    r.map_zoom = crate::dev::map_zoom_coherence_live_proof::refresh_map_zoom_coherence_live_witness();
    r.pilot_catalog =
        crate::dev::pilot_catalog_parity_live_proof::refresh_pilot_catalog_parity_live_witness();
    r.build_visual =
        crate::dev::build_read_visual_001_live_proof::refresh_build_read_visual_001_live_witness();
    r.minimap =
        crate::dev::design_minimap_widget_live_proof::refresh_design_minimap_widget_live_witness();
    r.fire_ecology = crate::dev::fire_ecology_lib_harness::refresh_fire_ecology_lib_harness_witness();
    r.vfx_fire =
        crate::dev::vfx_fire_test_highlight_live_proof::refresh_vfx_fire_test_highlight_live_witness();
    r.fire_play_vis = crate::dev::design_fire_play_visibility_live_proof::refresh_design_fire_play_visibility_live_witness();
    r.sim_effect_spine =
        crate::dev::sim_effect_spine_live_proof::refresh_sim_effect_spine_live_witness();
    r.landscape_grammar =
        crate::dev::landscape_grammar_sim_harness::refresh_landscape_grammar_harness_witnesses();
    r.pointer_gate = refresh_build_verify_pointer_live_witness();
    let _ = crate::dev::build_read_debug_live_proof::refresh_build_verify_debug_live_witness();
    let _ = crate::dev::construction_placement_live_proof::refresh_construction_placement_live_witness();
    let _ = crate::engine::play_scenario::refresh_play_scenario_001_live_witness();

    let play_lib = witness_file_green("debug_runs/play_scenario_live.json");
    let play_operator = std::fs::read_to_string(
        std::env::var_os("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .map(|r| r.join("debug_runs/play_scenario_live.json"))
            .unwrap_or_else(|| std::path::PathBuf::from("debug_runs/play_scenario_live.json")),
    )
    .ok()
    .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
    .and_then(|doc| doc.get("operator_session_green").and_then(|v| v.as_bool()))
    .unwrap_or(false);
    let stage5_fire = {
        let full = std::env::var_os("CARGO_MANIFEST_DIR")
            .map(std::path::PathBuf::from)
            .map(|root| root.join("debug_runs/stage5_full_app_live.json"))
            .unwrap_or_else(|| std::path::PathBuf::from("debug_runs/stage5_full_app_live.json"));
        std::fs::read_to_string(full)
            .ok()
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .and_then(|doc| {
                doc.pointer("/f2_extract_witness/fire_instance_buffer_rows")
                    .and_then(|v| v.as_u64())
                    .map(|n| n >= 1)
            })
            .unwrap_or(false)
    };

    let proof_grade_honest = g_play_proof_grade_honest();
    let coder_rollup_green = r.map_zoom
        && r.pilot_catalog
        && r.build_visual
        && r.minimap
        && r.fire_ecology
        && r.vfx_fire
        && r.fire_play_vis
        && r.landscape_grammar
        && r.pointer_gate
        && play_lib
        && stage5_fire;
    r.rollup_green = proof_grade_honest && coder_rollup_green && play_operator;

    let proof_grade = read_json_proof_grade();
    let body = serde_json::json!({
        "gate": "PRODUCT-VERIFY-GPLAY-001",
        "green": r.rollup_green,
        "proof_grade": proof_grade,
        "proof_grade_honest": proof_grade_honest,
        "g_play_coder_rollup_green": coder_rollup_green,
        "g_play_operator_pending": !play_operator,
        "play_lib_contract": play_lib,
        "play_operator_session": play_operator,
        "play_truth": play_lib,
        "stage5_fire_instances": stage5_fire,
        "lanes": {
            "map_zoom": r.map_zoom,
            "pilot_catalog": r.pilot_catalog,
            "build_visual": r.build_visual,
            "minimap": r.minimap,
            "fire_ecology": r.fire_ecology,
            "vfx_fire": r.vfx_fire,
            "fire_play_vis": r.fire_play_vis,
            "sim_effect_spine": r.sim_effect_spine,
            "landscape_grammar": r.landscape_grammar,
            "pointer_gate": r.pointer_gate,
        },
        "demo_fire_sparks_visible_at_operational_zoom": r.fire_play_vis,
    });
    let wrapped = wrap_debug_run(
        "PRODUCT-VERIFY-GPLAY-001",
        "refresh_product_verify_live_witnesses",
        G_PLAY_PRODUCT_CLOSE_LIVE_JSON,
        body,
    );
    let _ = write_debug_run_json(G_PLAY_PRODUCT_CLOSE_LIVE_JSON, wrapped);
    r
}

fn read_json_proof_grade() -> Option<String> {
    let stage5 = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(std::path::PathBuf::from)
        .map(|root| root.join("debug_runs/stage5_full_app_live.json"))
        .unwrap_or_else(|| std::path::PathBuf::from("debug_runs/stage5_full_app_live.json"));
    let raw = std::fs::read_to_string(stage5).ok()?;
    let doc = serde_json::from_str::<serde_json::Value>(&raw).ok()?;
    doc.pointer("/log_e01_visual_confirm_001/proof_grade")
        .or_else(|| doc.get("proof_grade"))
        .and_then(|v| v.as_str())
        .map(str::to_owned)
}

#[must_use]
pub fn product_verify_rollup_green() -> bool {
    refresh_product_verify_live_witnesses().rollup_green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_verify_bundle_refreshes_lane_witnesses() {
        let r = refresh_product_verify_live_witnesses();
        assert!(r.map_zoom, "map_zoom");
        assert!(r.pilot_catalog, "pilot_catalog");
        assert!(r.build_visual, "build_visual");
        assert!(r.minimap, "minimap");
        assert!(r.fire_ecology, "fire_ecology");
        assert!(r.vfx_fire, "vfx_fire");
        assert!(r.fire_play_vis, "fire_play_vis");
        assert!(r.landscape_grammar, "landscape_grammar");
        assert!(r.pointer_gate, "pointer_gate");
        if g_play_proof_grade_honest() {
            assert!(r.rollup_green || {
                let raw = std::fs::read_to_string(G_PLAY_PRODUCT_CLOSE_LIVE_JSON).expect("g_play");
                let doc: serde_json::Value = serde_json::from_str(&raw).expect("parse");
                doc.get("g_play_coder_rollup_green")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
            }, "rollup or honest coder rollup");
        } else {
            assert!(
                !r.rollup_green,
                "WIT-RUST-003: lib_fixture must not close G-PLAY rollup"
            );
        }
    }
}
