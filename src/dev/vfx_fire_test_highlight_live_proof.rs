//! **VFX-FIRE-HIGHLIGHT-001** — refresh `debug_runs/vfx_fire_test_highlight_live.json`.

pub const VFX_FIRE_TEST_HIGHLIGHT_LIVE_JSON: &str =
    "debug_runs/vfx_fire_test_highlight_live.json";

#[must_use]
pub fn refresh_vfx_fire_test_highlight_live_witness() -> bool {
    let body = crate::gui::vfx_fire_test_highlight_001_witness_json();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "VFX-FIRE-HIGHLIGHT-001",
        "refresh_vfx_fire_test_highlight_live_witness",
        VFX_FIRE_TEST_HIGHLIGHT_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(VFX_FIRE_TEST_HIGHLIGHT_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vfx_fire_test_highlight_live_witness_refresh_green() {
        assert!(refresh_vfx_fire_test_highlight_live_witness());
    }
}
