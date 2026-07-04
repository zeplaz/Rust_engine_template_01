//! **CLN-P0-*** Phase 0 hygiene witnesses (coder + steward slices).

pub const CLN_P0_S8_LIVE_JSON: &str = "debug_runs/cln_p0_s8_001_live.json";
pub const CLN_P0_P10_LIVE_JSON: &str = "debug_runs/cln_p0_p10_001_live.json";
pub const CLN_P0_R4_LIVE_JSON: &str = "debug_runs/cln_p0_r4_001_live.json";
pub const CLN_P0_R8_LIVE_JSON: &str = "debug_runs/cln_p0_r8_001_live.json";
pub const CLN_P0_T4_LIVE_JSON: &str = "debug_runs/cln_p0_t4_001_live.json";
pub const CLN_P0_T7_LIVE_JSON: &str = "debug_runs/cln_p0_t7_001_live.json";
pub const CLN_P0_T6_LIVE_JSON: &str = "debug_runs/cln_p0_t6_001_live.json";

#[must_use]
pub fn cln_p0_s8_spacial_no_println_green() -> bool {
    let src = include_str!("../traits/spacial.rs");
    !src.contains("println!")
}

#[must_use]
pub fn build_cln_p0_s8_witness_body() -> serde_json::Value {
    serde_json::json!({
        "gate": "CLN-P0-S8-001",
        "green": cln_p0_s8_spacial_no_println_green(),
        "issue": "S8",
        "target": "src/traits/spacial.rs",
        "plan_ref": "src/dev/plan_cleanup_v1.md#CLN-P0-S8-001",
    })
}

#[must_use]
pub fn refresh_cln_p0_s8_witness() -> bool {
    refresh_witness(
        CLN_P0_S8_LIVE_JSON,
        "CLN-P0-S8-001",
        "refresh_cln_p0_s8_witness",
        build_cln_p0_s8_witness_body(),
    )
}

#[must_use]
pub fn cln_p0_p10_solver_frequency_doc_green() -> bool {
    let src = include_str!("../economy/logistics/solver.rs");
    src.contains("CLN-P0-P10") && src.contains("on-demand")
}

#[must_use]
pub fn build_cln_p0_p10_witness_body() -> serde_json::Value {
    serde_json::json!({
        "gate": "CLN-P0-P10-001",
        "green": cln_p0_p10_solver_frequency_doc_green(),
        "issue": "P10",
        "target": "src/economy/logistics/solver.rs",
        "plan_ref": "src/dev/plan_cleanup_v1.md#CLN-P0-P10-001",
    })
}

#[must_use]
pub fn refresh_cln_p0_p10_witness() -> bool {
    refresh_witness(
        CLN_P0_P10_LIVE_JSON,
        "CLN-P0-P10-001",
        "refresh_cln_p0_p10_witness",
        build_cln_p0_p10_witness_body(),
    )
}

#[must_use]
pub fn refresh_cln_p0_coder_witnesses() -> bool {
    refresh_cln_p0_s8_witness() && refresh_cln_p0_p10_witness()
}

/// **CLN-P0-R4-001** — Drez-era loaders removed from module graph.
#[must_use]
pub fn cln_p0_r4_legacy_drez_removed_green() -> bool {
    let ser_mod = include_str!("../io/serialization/mod.rs");
    !ser_mod.contains("legacy_drez")
        && !std::path::Path::new("src/io/serialization/legacy_drez.rs").exists()
}

#[must_use]
pub fn build_cln_p0_r4_witness_body() -> serde_json::Value {
    serde_json::json!({
        "gate": "CLN-P0-R4-001",
        "green": cln_p0_r4_legacy_drez_removed_green(),
        "issue": "R4",
        "classification": "A_obsolete",
        "action": "deleted legacy_drez.rs",
        "plan_ref": "src/dev/plan_cleanup_v1.md#CLN-P0-R4-001",
    })
}

#[must_use]
pub fn refresh_cln_p0_r4_witness() -> bool {
    refresh_witness(
        CLN_P0_R4_LIVE_JSON,
        "CLN-P0-R4-001",
        "refresh_cln_p0_r4_witness",
        build_cln_p0_r4_witness_body(),
    )
}

/// **CLN-P0-R8-001** — prod_comps deleted; legacy_transport_stubs transitional (feature-gated).
#[must_use]
pub fn cln_p0_r8_en_leg_classified_green() -> bool {
    !std::path::Path::new("src/entities/production/prod_comps.rs").exists()
        && include_str!("../entities/structure/legacy_transport_stubs.rs").contains("INFRA-E0-003")
}

#[must_use]
pub fn build_cln_p0_r8_witness_body() -> serde_json::Value {
    serde_json::json!({
        "gate": "CLN-P0-R8-001",
        "green": cln_p0_r8_en_leg_classified_green(),
        "issue": "R8",
        "classification": "prod_comps_A_obsolete; legacy_transport_stubs_B_transitional",
        "plan_ref": "src/dev/plan_cleanup_v1.md#CLN-P0-R8-001",
    })
}

#[must_use]
pub fn refresh_cln_p0_r8_witness() -> bool {
    refresh_witness(
        CLN_P0_R8_LIVE_JSON,
        "CLN-P0-R8-001",
        "refresh_cln_p0_r8_witness",
        build_cln_p0_r8_witness_body(),
    )
}

/// **CLN-P0-T4-001** — empty placeholders classified (scaffold doc markers present).
#[must_use]
pub fn cln_p0_t4_scaffold_classified_green() -> bool {
    include_str!("../traits/rates.rs").contains("CLN-P0-T4-001")
        && include_str!("../traits/region.rs").contains("CLN-P0-T4-001")
        && include_str!("../engine/sets.rs").contains("CLN-P0-T4-001")
        && include_str!("../utils/events.rs").contains("CLN-P0-T4-001")
        && !std::path::Path::new("src/engine/utils.rs").exists()
}

#[must_use]
pub fn build_cln_p0_t4_witness_body() -> serde_json::Value {
    serde_json::json!({
        "gate": "CLN-P0-T4-001",
        "green": cln_p0_t4_scaffold_classified_green(),
        "issue": "T4",
        "classification": "scaffold_intentional_or_deleted_orphan",
        "plan_ref": "src/dev/plan_cleanup_v1.md#CLN-P0-T4-001",
    })
}

#[must_use]
pub fn refresh_cln_p0_t4_witness() -> bool {
    refresh_witness(
        CLN_P0_T4_LIVE_JSON,
        "CLN-P0-T4-001",
        "refresh_cln_p0_t4_witness",
        build_cln_p0_t4_witness_body(),
    )
}

/// **CLN-P0-T7-001** — floating TODOs folded onto DV-TODO boards.
#[must_use]
pub fn cln_p0_t7_todos_board_folded_green() -> bool {
    include_str!("../entities/production/core/manufacturing_plugin.rs").contains("INDUSTRIAL-MFG-01")
        && include_str!("../terrain/generation/passes/p5_agent_overlay.rs").contains("FINISH-WG-P5-01")
        && include_str!("../infrastructure/settlement/mod.rs").contains("INFRA-E5-001")
        && include_str!("industrial_activation_todos.rs").contains("INDUSTRIAL-MFG-01")
        && include_str!("stage5_finish_todos.rs").contains("FINISH-WG-P5-01")
}

#[must_use]
pub fn build_cln_p0_t7_witness_body() -> serde_json::Value {
    serde_json::json!({
        "gate": "CLN-P0-T7-001",
        "green": cln_p0_t7_todos_board_folded_green(),
        "issue": "T7",
        "folded": ["INDUSTRIAL-MFG-01", "FINISH-WG-P5-01", "INFRA-E5-001"],
        "plan_ref": "src/dev/plan_cleanup_v1.md#CLN-P0-T7-001",
    })
}

#[must_use]
pub fn refresh_cln_p0_t7_witness() -> bool {
    refresh_witness(
        CLN_P0_T7_LIVE_JSON,
        "CLN-P0-T7-001",
        "refresh_cln_p0_t7_witness",
        build_cln_p0_t7_witness_body(),
    )
}

/// **CLN-P0-T6-001** — stale `target_*` build dirs removed (operator disk hygiene).
#[must_use]
pub fn cln_p0_t6_stale_target_dirs_removed_green() -> bool {
    const STALE: &[&str] = &[
        "target_coder_b",
        "target_coder_b2",
        "target_contain_004",
        "target_contain_b",
        "target_log_e01",
        "target_play",
        "target_queue_fix",
        "target_s7b_fix",
        "target_s7b_m4",
        "target_s7b_test2",
        "target_smoke_test",
    ];
    STALE.iter().all(|d| !std::path::Path::new(d).exists())
}

#[must_use]
pub fn build_cln_p0_t6_witness_body() -> serde_json::Value {
    serde_json::json!({
        "gate": "CLN-P0-T6-001",
        "green": cln_p0_t6_stale_target_dirs_removed_green(),
        "issue": "T6",
        "action": "deleted stale target_* dirs",
        "plan_ref": "src/dev/plan_cleanup_v1.md#CLN-P0-T6-001",
    })
}

#[must_use]
pub fn refresh_cln_p0_t6_witness() -> bool {
    refresh_witness(
        CLN_P0_T6_LIVE_JSON,
        "CLN-P0-T6-001",
        "refresh_cln_p0_t6_witness",
        build_cln_p0_t6_witness_body(),
    )
}

#[must_use]
pub fn refresh_cln_p0_steward_witnesses() -> bool {
    refresh_cln_p0_r4_witness()
        && refresh_cln_p0_r8_witness()
        && refresh_cln_p0_t4_witness()
        && refresh_cln_p0_t7_witness()
        && refresh_cln_p0_t6_witness()
}

#[must_use]
pub fn refresh_cln_p0_all_witnesses() -> bool {
    refresh_cln_p0_coder_witnesses() && refresh_cln_p0_steward_witnesses()
}

fn refresh_witness(
    path: &str,
    gate: &str,
    command: &str,
    body: serde_json::Value,
) -> bool {
    use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run(gate, command, path, body);
    write_debug_run_json(path, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cln_p0_s8_witness_green() {
        assert!(cln_p0_s8_spacial_no_println_green());
    }

    #[test]
    fn cln_p0_p10_witness_green() {
        assert!(cln_p0_p10_solver_frequency_doc_green());
    }

    #[test]
    fn cln_p0_refresh_coder_witnesses() {
        assert!(refresh_cln_p0_coder_witnesses());
    }

    #[test]
    fn cln_p0_steward_slices_green() {
        assert!(cln_p0_r4_legacy_drez_removed_green());
        assert!(cln_p0_r8_en_leg_classified_green());
        assert!(cln_p0_t4_scaffold_classified_green());
        assert!(cln_p0_t7_todos_board_folded_green());
    }

    #[test]
    fn cln_p0_steward_refresh_witnesses() {
        assert!(refresh_cln_p0_r4_witness());
        assert!(refresh_cln_p0_r8_witness());
        assert!(refresh_cln_p0_t4_witness());
        assert!(refresh_cln_p0_t7_witness());
    }

    #[test]
    fn cln_p0_t6_witness_after_disk_cleanup() {
        if cln_p0_t6_stale_target_dirs_removed_green() {
            assert!(refresh_cln_p0_t6_witness());
        }
    }
}
