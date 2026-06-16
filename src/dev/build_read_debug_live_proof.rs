//! **BUILD-VERIFY-DEBUG-001** — lib witness for placement debug triage fields.

pub const BUILD_VERIFY_DEBUG_LIVE_JSON: &str = "debug_runs/build_verify_debug_live.json";

#[must_use]
pub fn refresh_build_verify_debug_live_witness() -> bool {
    let green = crate::construction::build_read_debug_001_witness_green();
    if !green {
        return false;
    }
    let body = crate::construction::build_verify_debug_001_witness_json();
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "BUILD-VERIFY-DEBUG-001",
        "refresh_build_verify_debug_live_witness",
        BUILD_VERIFY_DEBUG_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(BUILD_VERIFY_DEBUG_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_verify_debug_live_witness_refresh_green() {
        assert!(refresh_build_verify_debug_live_witness());
    }
}
