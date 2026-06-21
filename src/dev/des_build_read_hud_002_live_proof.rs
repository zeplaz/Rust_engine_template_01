//! **DES-BUILD-READ-HUD-002** — compact build strip live witness.

pub const DES_BUILD_READ_HUD_002_LIVE_JSON: &str = "debug_runs/des_build_read_hud_002_live.json";

#[must_use]
pub fn refresh_des_build_read_hud_002_live_witness() -> bool {
    use crate::gui::hud::grammar_read_hud::des_build_read_hud_002_witness_green;

    let green = des_build_read_hud_002_witness_green();
    let body = serde_json::json!({
        "gate": "DES-BUILD-READ-HUD-002",
        "slice_id": "DES-BUILD-READ-HUD-002",
        "design_ref": "src/dev/design_build_read_hud_v2.md",
        "green": green,
        "valid_blocked_glyphs": green,
        "corridor_phase_wired": green,
        "context_strip_v2": green,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "DES-BUILD-READ-HUD-002",
        "refresh_des_build_read_hud_002_live_witness",
        DES_BUILD_READ_HUD_002_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(DES_BUILD_READ_HUD_002_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn des_build_read_hud_002_live_witness_green() {
        assert!(refresh_des_build_read_hud_002_live_witness());
    }
}
