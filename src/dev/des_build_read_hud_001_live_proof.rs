//! **DES-BUILD-READ-HUD-001** — grammar read HUD live witness.

pub const DES_BUILD_READ_HUD_001_LIVE_JSON: &str = "debug_runs/des_build_read_hud_001_live.json";

#[must_use]
pub fn build_des_build_read_hud_001_body() -> serde_json::Value {
    use crate::gui::hud::grammar_read_hud::des_build_read_hud_001_witness_green;

    let lib_green = des_build_read_hud_001_witness_green();
    serde_json::json!({
        "gate": "DES-BUILD-READ-HUD-001",
        "slice_id": "DES-BUILD-READ-HUD-001",
        "green": lib_green,
        "grammar_labels_loaded": lib_green,
        "context_strip_wired": lib_green,
        "placement_debug_style_chip": lib_green,
        "runtime_sim_verified": false,
        "design": "src/dev/design_build_grammar_read_hud_v1.md",
        "code": [
            "src/gui/hud/grammar_read_hud.rs",
            "src/gui/hud/contextual_tip.rs",
            "src/construction/placement_debug.rs",
        ],
    })
}

#[must_use]
pub fn refresh_des_build_read_hud_001_live_witness() -> bool {
    let body = build_des_build_read_hud_001_body();
    if !body.get("green").and_then(|v| v.as_bool()).unwrap_or(false) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "DES-BUILD-READ-HUD-001",
        "refresh_des_build_read_hud_001_live_witness",
        DES_BUILD_READ_HUD_001_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(DES_BUILD_READ_HUD_001_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn des_build_read_hud_001_live_witness_refresh_green() {
        assert!(refresh_des_build_read_hud_001_live_witness());
    }
}
