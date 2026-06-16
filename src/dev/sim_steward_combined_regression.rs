//! **SIM-STEWARD-COMBINED-001** — stage5 + fire_ecology + landscape_grammar regression spine.

#[cfg(test)]
mod tests {
    #[test]
    fn sim_steward_combined_regression_spine_green() {
        assert!(
            crate::dev::landscape_grammar_sim_harness::refresh_landscape_grammar_harness_witnesses(),
            "landscape_grammar harness witnesses"
        );
        assert!(
            crate::dev::fire_ecology_lib_harness::refresh_fire_ecology_lib_harness_witness(),
            "fire_ecology lib harness"
        );
        assert!(
            crate::dev::phase6_coder_queue_bundle_proof::refresh_phase6_tail_witnesses(),
            "phase6 tail witness bundle"
        );
        let stage5_path = std::path::Path::new("debug_runs/stage5_full_app_live.json");
        if stage5_path.exists() {
            let text = std::fs::read_to_string(stage5_path).expect("read stage5 witness");
            let json: serde_json::Value = serde_json::from_str(&text).expect("parse");
            let source = json
                .pointer("/ecology_rows_source")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            assert_eq!(
                source, "live_landscape_program_on_chunk",
                "stage5 ecology_rows_source"
            );
            let fire_corridor = json
                .pointer("/fire_corridor_witness/green")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            assert!(fire_corridor, "fire corridor population fuel wired");
        }
        let lg4_path =
            std::path::Path::new("debug_runs/landscape_grammar_lg4_preview_live.json");
        if lg4_path.exists() {
            let text = std::fs::read_to_string(lg4_path).expect("read lg4 witness");
            let json: serde_json::Value = serde_json::from_str(&text).expect("parse");
            let visible = json
                .pointer("/topology_kind_count_visible")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            assert!(
                visible >= 3,
                "lg4 topology_kind_count_visible >= 3, got {visible}"
            );
        }
        let lg3_path = std::path::Path::new("debug_runs/landscape_grammar_lg3_live.json");
        if lg3_path.exists() {
            let text = std::fs::read_to_string(lg3_path).expect("read lg3 witness");
            let json: serde_json::Value = serde_json::from_str(&text).expect("parse");
            assert_eq!(
                json.pointer("/industrial_preset_anchored")
                    .and_then(|v| v.as_bool()),
                Some(true),
                "lg3 industrial preset anchored"
            );
            assert_eq!(
                json.pointer("/military_preset_anchored")
                    .and_then(|v| v.as_bool()),
                Some(true),
                "lg3 military preset anchored"
            );
        }
    }
}
