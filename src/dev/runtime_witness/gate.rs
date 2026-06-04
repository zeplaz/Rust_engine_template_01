//! Global enable switch for runtime witness file I/O (DEV-ARTIFACT-CONTAINMENT-001).

use crate::dev::debug_run_envelope::{env_flag, env_opt};

/// Environment variable: set `1` or `true` to allow witness JSON writes in release builds.
pub const ENV_RUNTIME_WITNESS_WRITES: &str = "RUNTIME_WITNESS_WRITES";

/// Environment variable: set `1` to force skip writes even in debug/test (parity / release smoke).
pub const ENV_RUNTIME_WITNESS_WRITES_FORCE_OFF: &str = "RUNTIME_WITNESS_WRITES_FORCE_OFF";

/// Whether operational witness files may be written this process.
///
/// - **Tests:** always enabled (lib witness refresh + parity).
/// - **Release:** only when [`ENV_RUNTIME_WITNESS_WRITES`] is set.
/// - **Debug:** enabled unless [`ENV_RUNTIME_WITNESS_WRITES_FORCE_OFF`] is set.
#[must_use]
pub fn witness_writes_enabled() -> bool {
    if cfg!(test) {
        return !env_flag(ENV_RUNTIME_WITNESS_WRITES_FORCE_OFF);
    }
    if env_flag(ENV_RUNTIME_WITNESS_WRITES_FORCE_OFF) {
        return false;
    }
    if env_flag(ENV_RUNTIME_WITNESS_WRITES) {
        return true;
    }
    cfg!(debug_assertions)
}

/// Human-readable gate state for witness payloads / diagnostics.
#[must_use]
pub fn witness_gate_snapshot() -> serde_json::Value {
    serde_json::json!({
        "writes_enabled": witness_writes_enabled(),
        "env_runtime_witness_writes": env_opt(ENV_RUNTIME_WITNESS_WRITES),
        "env_force_off": env_flag(ENV_RUNTIME_WITNESS_WRITES_FORCE_OFF),
        "profile_debug": cfg!(debug_assertions),
        "cfg_test": cfg!(test),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_writes_enabled_in_unit_tests() {
        assert!(witness_writes_enabled());
    }
}
