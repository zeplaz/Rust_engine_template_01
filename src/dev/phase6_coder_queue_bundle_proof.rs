//! Phase 6 coder queue — witness refresh bundle for waves 2–6 tail rows.

#[must_use]
pub fn refresh_phase6_tail_witnesses() -> bool {
    let veg_snapshot =
        crate::dev::vegetation_snapshot_roundtrip_live_proof::refresh_vegetation_snapshot_roundtrip_live_witness();
    let minimap =
        crate::dev::minimap_topology_legend_live_proof::refresh_minimap_topology_legend_live_witness();
    let infra_e0 =
        crate::dev::infra_e0_profile_catalog_live_proof::refresh_infra_e0_profile_catalog_live_witness();
    let infra_overlay =
        crate::dev::infra_utility_overlay_live_proof::refresh_infra_utility_overlay_live_witness();
    let grammar_diversity = crate::construction::procedural::refresh_pg_quality_001_grammar_diversity_witness();
    let fire_corridor =
        crate::systems::ecology::fire_corridor_population_fuel_witness_green();
    let transport_r8 = crate::strategic::corridor_r8_roundtrip_witness_green();
    let pt_5_002 = crate::construction::procedural::procedural_tiles_runtime_witness_green();
    let pg2_tail = crate::construction::procedural::procedural_pg2_tail_001_witness_green();
    let og_4 = crate::strategic::construction_organic_growth_001_witness_green();
    let og_ux = crate::gui::construction_growth_inspector::growth_inspector_wired_witness_green();
    let replay = crate::dev::replay_editor_parity::refresh_replay_editor_parity_live_witness();
    let infra_vm = crate::infrastructure::infra_e0_003_legacy_transport_stubs_gated_witness_green();
    let build_program = build_grammar_program_witness_green();
    let build_site_zone = build_grammar_site_zone_witness_green();
    let build_beta = build_grammar_beta_world_witness_green();
    let con_r4 =
        crate::construction::construction_r4_corridor_legend_wired_witness_green();
    let aps_dna_consumer =
        crate::dev::aps_dna_consumer_live_proof::refresh_aps_dna_consumer_rust_live_witness();
    let nav_agent =
        crate::dev::nav_agent_routing_live_proof::refresh_nav_agent_routing_live_witness();
    let s7b_m1 = s7b_m1_witness_green();

    let green = veg_snapshot
        && minimap
        && infra_e0
        && infra_overlay
        && grammar_diversity
        && fire_corridor
        && transport_r8
        && pt_5_002
        && pg2_tail
        && og_4
        && og_ux
        && replay
        && infra_vm
        && build_program
        && build_site_zone
        && build_beta
        && con_r4
        && aps_dna_consumer
        && nav_agent
        && s7b_m1;

    let body = serde_json::json!({
        "gate": "PHASE6-TAIL-BUNDLE-001",
        "green": green,
        "rows": {
            "VEG-SNAPSHOT-PLAY-001": veg_snapshot,
            "VEG-MINIMAP-OVERLAY-002": minimap,
            "VEG-FIRE-CORRIDOR-FULLAPP-001": fire_corridor,
            "INFRA-E0-003": infra_e0,
            "INFRA-UTILITY-OVERLAY-001": infra_overlay,
            "INFRA-TRANSPORT-R8-001": transport_r8,
            "INFRA-VM-DEEP-001": infra_vm,
            "BUILD-GRAMMAR-WITNESS-002": grammar_diversity,
            "BUILD-GRAMMAR-PROGRAM-001": build_program,
            "BUILD-GRAMMAR-SITE-ZONE-001": build_site_zone,
            "BUILD-GRAMMAR-β-WORLD-001": build_beta,
            "BUILD-READ-CONSUMER-MCP-001": aps_dna_consumer,
            "INFRA-E6-002-NAV-AGENT": nav_agent,
            "PROC-PG-2-TAIL-001": pg2_tail,
            "PROC-OG-4-001": og_4,
            "PROC-OG-UX-WIRE-001": og_ux,
            "CON-R4-TAIL-001": con_r4,
            "PT-5-002": pt_5_002,
            "REPLAY-RING-001": replay,
            "S7B-M1-001": s7b_m1,
        },
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "PHASE6-TAIL-BUNDLE-001",
        "refresh_phase6_tail_witnesses",
        "debug_runs/phase6_coder_queue_bundle_live.json",
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(
        "debug_runs/phase6_coder_queue_bundle_live.json",
        wrapped,
    ) && green
}

#[must_use]
fn build_grammar_program_witness_green() -> bool {
    crate::construction::procedural::program_graph_stub_for_preset("logistics_rail_warehouse_v0")
        .is_some_and(|g| !g.site_zones.is_empty())
}

#[must_use]
fn build_grammar_site_zone_witness_green() -> bool {
    use crate::construction::pilot_catalog::site_zone_grid_for_arch_dna_preset;
    site_zone_grid_for_arch_dna_preset("logistics_rail_warehouse_v0")
        .is_some_and(|g| crate::construction::site_zone_grid::site_zone_occupancy_witness_green(&g))
}

#[must_use]
fn build_grammar_beta_world_witness_green() -> bool {
    let Ok(preset) = crate::construction::procedural::load_logistics_rail_warehouse_v0_preset()
    else {
        return false;
    };
    let biased =
        crate::construction::procedural::beta_with_world_transport_bias(preset.pressure_field, 4);
    biased.beta_yard > preset.pressure_field.beta_yard
}

#[must_use]
fn s7b_m1_witness_green() -> bool {
    use crate::dev::stage7_behavioral_witness::{
        behavioral_contract_ok, commit_stage7_behavioral_witness, s7b_m1_green,
    };
    use crate::dev::stage7_play_witness::refresh_s7p_steward_001_live_witness;
    use crate::strategic::{
        seed_stage7_behavioral_witness_for_lib_proof, Stage7BehavioralHud,
        Stage7BehavioralWitnessState, Stage7BeliefState, StrategicCommandQueue,
    };

    let s7p_ok = refresh_s7p_steward_001_live_witness();
    if !s7b_m1_green(behavioral_contract_ok(), s7p_ok) {
        return false;
    }
    let mut queue = StrategicCommandQueue::default();
    let mut behavioral = Stage7BehavioralWitnessState::default();
    let mut beliefs = Stage7BeliefState::default();
    seed_stage7_behavioral_witness_for_lib_proof(&mut queue, &mut behavioral, &mut beliefs);
    let hud = Stage7BehavioralHud {
        pending_orders: queue.pending_count(),
        orders_pending_ui_hook: true,
        orders_pending_label: format!("Orders pending: {}", queue.pending_count()),
    };
    commit_stage7_behavioral_witness(&queue, &behavioral, &hud)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase6_tail_witness_bundle_green() {
        assert!(refresh_phase6_tail_witnesses());
    }
}
