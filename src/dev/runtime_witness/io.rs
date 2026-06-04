//! Shared envelope + JSON write helpers for runtime witness lanes.

use serde_json::Value;

use super::gate;

/// Wrap with [`crate::dev::debug_run_envelope::wrap_debug_run`] and write when the gate allows.
#[must_use]
pub fn write_enveloped_witness(
    profile: &str,
    source_system: &str,
    relative_path: &str,
    body: Value,
) -> bool {
    if !gate::witness_writes_enabled() {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        profile,
        source_system,
        relative_path,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(relative_path, wrapped)
}

/// Same as [`write_enveloped_witness`] but ignores the runtime gate (lib tests / forced refresh).
#[must_use]
pub fn write_enveloped_witness_unchecked(
    profile: &str,
    source_system: &str,
    relative_path: &str,
    body: Value,
) -> bool {
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        profile,
        source_system,
        relative_path,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(relative_path, wrapped)
}
