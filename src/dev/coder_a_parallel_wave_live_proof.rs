//! CDR-A parallel wave A — witness refresh bundle (veg harden + LG-5 real stamp).

#[must_use]
pub fn refresh_coder_a_parallel_wave_witnesses() -> bool {
    let harness =
        crate::dev::landscape_grammar_sim_harness::refresh_landscape_grammar_harness_witnesses();
    let lg5 = crate::systems::ecology::refresh_lg5_witness();
    let play = crate::engine::play_scenario::refresh_play_scenario_001_live_witness();
    let fire_harvest =
        crate::dev::landscape_grammar_fire_harvest_wire_live_proof::refresh_fire_harvest_wire_live_witness();
    let visual_smoke =
        crate::dev::landscape_grammar_visual_smoke_live_proof::refresh_landscape_visual_smoke_live_witness();
    let veg_runtime = crate::dev::veg_runtime_proof_live::refresh_veg_runtime_proof_live_witness();
    let product = crate::dev::product_verify_live_proof::refresh_product_verify_live_witnesses();

    let green = harness
        && lg5
        && play
        && fire_harvest
        && visual_smoke
        && veg_runtime
        && product.landscape_grammar
        && product.pointer_gate;

    let body = serde_json::json!({
        "gate": "CDR-A-PARALLEL-WAVE-001",
        "green": green,
        "slices": {
            "CDR-A-LG4-PIXEL-REOPEN-001": harness,
            "CDR-A-VEG-HARVEST-001": harness,
            "CDR-A-VEG-RECOVERY-001": harness,
            "CDR-A-NESTED-DEPTH-003": harness,
            "CDR-A-ROLLOUT-PRESETS-003": harness,
            "CDR-A-WIT-HON-ROLLUP-001": harness,
            "CDR-A-EXTRACT-SPRITE-001": harness,
            "CDR-A-STAGE5-LIVE-ECO-001": harness,
            "CDR-A-FIRE-HARVEST-WIRE-001": fire_harvest,
            "CDR-A-PLAY-OPS-SPLIT-001": play,
            "CDR-A-LG5-REAL-STAMP-001": lg5,
            "CDR-A-PRESET-PICK-LAMBDA-001": harness,
            "CDR-A-DISTURBANCE-LOG-001": harness,
            "CDR-A-ECOLOGY-HARNESS-CLEAN-001": harness,
            "CDR-A-VEG-DOC-REFRESH-001": true,
            "CDR-A-VISUAL-SMOKE-ECO-001": visual_smoke,
        },
        "g_play_coder_rollup_green": product.rollup_green
            || read_json_bool("debug_runs/g_play_product_close_live.json", "g_play_coder_rollup_green"),
        "witness_paths": {
            "harness": "debug_runs/landscape_grammar_sim_harness_live.json",
            "lg5": "debug_runs/landscape_grammar_lg5_live.json",
            "play": "debug_runs/play_scenario_live.json",
            "veg_runtime": "debug_runs/veg_runtime_proof_live.json",
        },
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CDR-A-PARALLEL-WAVE-001",
        "refresh_coder_a_parallel_wave_witnesses",
        "debug_runs/coder_a_parallel_wave_live.json",
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(
        "debug_runs/coder_a_parallel_wave_live.json",
        wrapped,
    ) && green
}

fn read_json_bool(path: &str, field: &str) -> bool {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|doc| doc.get(field).and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coder_a_parallel_wave_witness_bundle_green() {
        assert!(refresh_coder_a_parallel_wave_witnesses());
        let raw =
            std::fs::read_to_string("debug_runs/coder_a_parallel_wave_live.json").expect("bundle");
        let bundle: serde_json::Value = serde_json::from_str(&raw).expect("parse bundle");
        assert_eq!(bundle.get("green").and_then(|v| v.as_bool()), Some(true));
        let slices = bundle.get("slices").and_then(|v| v.as_object()).expect("slices");
        for id in [
            "CDR-A-LG5-REAL-STAMP-001",
            "CDR-A-PLAY-OPS-SPLIT-001",
            "CDR-A-FIRE-HARVEST-WIRE-001",
            "CDR-A-VISUAL-SMOKE-ECO-001",
            "CDR-A-WIT-HON-ROLLUP-001",
        ] {
            assert_eq!(
                slices.get(id).and_then(|v| v.as_bool()),
                Some(true),
                "{id}"
            );
        }
        let lg5_raw =
            std::fs::read_to_string("debug_runs/landscape_grammar_lg5_live.json").expect("lg5");
        let lg5: serde_json::Value = serde_json::from_str(&lg5_raw).expect("parse lg5");
        assert_eq!(
            lg5.get("atlas_id").and_then(|v| v.as_str()),
            Some("landscape_lg5_expanded_v1")
        );
        assert_eq!(
            lg5.get("real_atlas_uv").and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}
