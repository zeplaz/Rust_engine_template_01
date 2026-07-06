//! H-A2 witness helpers — spike report presence; optional `hanabi_l3` plugin stub.

use std::path::PathBuf;

/// `experiments/hanabi_validation/report_v1.md` on disk (PLAN-HANABI-H-A2-EXEC-001).
#[must_use]
pub fn hanabi_spike_report_present() -> bool {
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    root.join("experiments/hanabi_validation/report_v1.md")
        .is_file()
}

/// Default binary must not wire Hanabi unless feature + env (double gate).
#[must_use]
pub fn hanabi_l3_plugin_wired() -> bool {
    cfg!(feature = "hanabi_l3")
        && std::env::var_os("RUST_ENGINE_HANABI_L3")
            .is_some_and(|v| v != "0" && !v.eq_ignore_ascii_case("false"))
}

#[cfg(feature = "hanabi_l3")]
pub use crate::render::hanabi_embellishment::HanabiEmbellishmentPlugin;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hanabi_spike_report_on_disk() {
        assert!(hanabi_spike_report_present());
    }

    #[test]
    fn hanabi_l3_off_by_default() {
        assert!(!hanabi_l3_plugin_wired());
    }
}
