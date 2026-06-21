//! **FIRE-F2-READINESS-ALIGN-001** — fire_inst proxy vs sim heat stability witness.

pub const FIRE_F2_READINESS_LIVE_JSON: &str = "debug_runs/fire_f2_readiness_align_live.json";

#[must_use]
pub fn refresh_fire_f2_readiness_align_witness() -> bool {
    use crate::dev::fire_ecology_lib_harness::run_fire_ecology_lib_harness;
    use crate::systems::fire::witness_collectors::build_fire_f2_readiness_align_block;

    let witness = run_fire_ecology_lib_harness();
    let block = build_fire_f2_readiness_align_block(&witness);
    let green = block.get("green").and_then(|v| v.as_bool()) == Some(true);

    let body = serde_json::json!({
        "gate": "FIRE-F2-READINESS-ALIGN-001",
        "fire_f2_readiness_align_001": block,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "FIRE-F2-READINESS-ALIGN-001",
        "refresh_fire_f2_readiness_align_witness",
        FIRE_F2_READINESS_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(FIRE_F2_READINESS_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_f2_readiness_align_witness_green() {
        assert!(refresh_fire_f2_readiness_align_witness());
    }
}
