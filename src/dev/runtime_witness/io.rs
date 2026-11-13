//! Shared envelope + JSON write helpers for runtime witness lanes.
//!
//! Retired: `*_live.json` I/O is off. Review path is `sim_run_dump` + `tools/sim_review`.

use serde_json::Value;

/// Retired — always false. Use `RUST_ENGINE_SIM_DUMP=1` (debug) + `score_dump.py`.
#[must_use]
pub fn write_enveloped_witness(
    _profile: &str,
    _source_system: &str,
    _relative_path: &str,
    _body: Value,
) -> bool {
    false
}

/// Retired — always false.
#[must_use]
pub fn write_enveloped_witness_unchecked(
    _profile: &str,
    _source_system: &str,
    _relative_path: &str,
    _body: Value,
) -> bool {
    false
}
