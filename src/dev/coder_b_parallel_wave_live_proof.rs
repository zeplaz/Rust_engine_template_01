//! CDR-B parallel wave B — witness refresh bundle (APS E5 consumer parity).

const REPRESENTATION_PARITY_SIGNOFF_MD: &str = "src/dev/representation_parity_signoff_v1.md";

#[must_use]
pub fn representation_parity_signoff_confirmed() -> bool {
    std::fs::read_to_string(REPRESENTATION_PARITY_SIGNOFF_MD)
        .map(|doc| {
            doc.contains("CDR-B-REPRESENTATION-PARITY-001") && doc.contains("CONFIRMED")
        })
        .unwrap_or(false)
}

#[must_use]
pub fn refresh_coder_b_parallel_wave_witnesses() -> bool {
    let build_consumer =
        crate::dev::aps_dna_consumer_live_proof::refresh_aps_dna_consumer_rust_live_witness();
    let build_visual =
        crate::dev::build_read_visual_001_live_proof::refresh_build_read_visual_001_live_witness();
    let map_stamp =
        crate::dev::landscape_map_stamp_contract_live_proof::refresh_landscape_map_stamp_contract_live_witness(
        );
    let wit_hon = crate::dev::wit_hon_phase6_reconcile_live_proof::refresh_wit_hon_phase6_reconcile_live_witness();
    let grammar =
        crate::construction::procedural::refresh_pg_quality_001_grammar_diversity_witness();
    let landscape_resolver =
        crate::construction::procedural::landscape_tile_resolver_witness_green();
    let growth_hud =
        crate::gui::construction_growth_inspector::growth_hud_ecology_hint_wired_witness_green();
    let ind_play = crate::dev::ind_play_witness_live_proof::refresh_ind_play_witness_live();
    let veg_parity =
        crate::dev::veg_resolver_parity_live_proof::refresh_veg_resolver_parity_live_witness();
    let representation_parity = representation_parity_signoff_confirmed();
    let infra_overlay_polish =
        crate::dev::utility_network_live_proof::refresh_utility_network_live_witness();

    let green = build_consumer
        && build_visual
        && map_stamp
        && wit_hon
        && grammar
        && landscape_resolver
        && growth_hud
        && ind_play
        && veg_parity
        && representation_parity
        && infra_overlay_polish;

    let body = serde_json::json!({
        "gate": "CDR-B-PARALLEL-WAVE-001",
        "green": green,
        "slices": {
            "CDR-B-BUILD-CONSUMER-MCP-001": build_consumer,
            "CDR-B-BUILD-VISUAL-RUN-002": build_visual,
            "CDR-B-MAP-STAMP-CONTRACT-001": map_stamp,
            "CDR-B-WIT-HON-PHASE6-001": wit_hon,
            "CDR-B-CONSTRUCTION-GRAMMAR-DEPTH-001": grammar,
            "CDR-B-TILE-RESOLVER-VEG-001": landscape_resolver,
            "CDR-B-GROWTH-HUD-VEG-001": growth_hud,
            "CDR-B-IND-PLAY-WITNESS-001": ind_play,
            "CDR-B-VEG-RESOLVER-PARITY-001": veg_parity,
            "CDR-B-REPRESENTATION-PARITY-001": representation_parity,
            "CDR-B-INFRA-OVERLAY-POLISH-001": infra_overlay_polish,
        },
        "witness_paths": {
            "build_consumer": "debug_runs/aps_dna_consumer_rust_live.json",
            "veg_parity": "debug_runs/art_pipeline/veg_resolver_parity_live.json",
            "representation_signoff": REPRESENTATION_PARITY_SIGNOFF_MD,
            "utility_network": "debug_runs/utility_network_live.json",
        },
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CDR-B-PARALLEL-WAVE-001",
        "refresh_coder_b_parallel_wave_witnesses",
        "debug_runs/coder_b_parallel_wave_live.json",
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(
        "debug_runs/coder_b_parallel_wave_live.json",
        wrapped,
    ) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coder_b_parallel_wave_witness_bundle_green() {
        assert!(refresh_coder_b_parallel_wave_witnesses());
        let raw =
            std::fs::read_to_string("debug_runs/coder_b_parallel_wave_live.json").expect("bundle");
        let bundle: serde_json::Value = serde_json::from_str(&raw).expect("parse bundle");
        assert_eq!(bundle.get("green").and_then(|v| v.as_bool()), Some(true));
        let slices = bundle.get("slices").and_then(|v| v.as_object()).expect("slices");
        for id in [
            "CDR-B-BUILD-CONSUMER-MCP-001",
            "CDR-B-VEG-RESOLVER-PARITY-001",
            "CDR-B-TILE-RESOLVER-VEG-001",
            "CDR-B-MAP-STAMP-CONTRACT-001",
            "CDR-B-WIT-HON-PHASE6-001",
            "CDR-B-GROWTH-HUD-VEG-001",
            "CDR-B-CONSTRUCTION-GRAMMAR-DEPTH-001",
            "CDR-B-BUILD-VISUAL-RUN-002",
            "CDR-B-IND-PLAY-WITNESS-001",
            "CDR-B-REPRESENTATION-PARITY-001",
            "CDR-B-INFRA-OVERLAY-POLISH-001",
        ] {
            assert_eq!(
                slices.get(id).and_then(|v| v.as_bool()),
                Some(true),
                "{id}"
            );
        }
        let parity_raw = std::fs::read_to_string("debug_runs/art_pipeline/veg_resolver_parity_live.json")
            .expect("parity witness");
        let parity: serde_json::Value = serde_json::from_str(&parity_raw).expect("parse parity");
        assert_eq!(
            parity.get("byte_parity").and_then(|v| v.as_bool()),
            Some(true),
            "CDR-B-VEG-RESOLVER-PARITY-001"
        );
        let visual_raw =
            std::fs::read_to_string("debug_runs/build_read_visual_001_live.json").expect("visual");
        let visual: serde_json::Value = serde_json::from_str(&visual_raw).expect("parse visual");
        assert_eq!(
            visual.get("runtime_sim_verified").and_then(|v| v.as_bool()),
            Some(true),
            "CDR-B-BUILD-VISUAL-RUN-002"
        );
        assert!(representation_parity_signoff_confirmed());
    }
}
