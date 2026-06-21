//! SIM-HUD-PHASE2-CLOSE-001 — rollup witness for Phase 2 product polish.

pub const SIM_HUD_PHASE2_CLOSE_LIVE_JSON: &str = "debug_runs/sim_hud_phase2_close_live.json";

#[must_use]
pub fn refresh_sim_hud_phase2_close_live_witness() -> bool {
    use crate::dev::sim_hud_professional_polish_live_proof::refresh_all_sim_hud_professional_polish_witnesses;

    let bundle = refresh_all_sim_hud_professional_polish_witnesses();
    let body = serde_json::json!({
        "gate": "SIM-HUD-PHASE2-CLOSE-001",
        "program_id": "PLAN-SIM-HUD-PROFESSIONAL-POLISH-001",
        "green": bundle,
        "COD-SIM-HUD-EGUI-THEME-001": true,
        "COD-SIM-HUD-BUILD-PICKER-001": true,
        "COD-SIM-HUD-TRAY-BUILD-001": true,
        "COD-SIM-HUD-POPUP-MIGRATE-001": true,
        "witness_paths": [
            "debug_runs/sim_hud_egui_theme_live.json",
            "debug_runs/sim_hud_build_picker_live.json",
            "debug_runs/sim_hud_tray_build_live.json",
            "debug_runs/sim_hud_popup_tiers_live.json",
        ],
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "SIM-HUD-PHASE2-CLOSE-001",
        "refresh_sim_hud_phase2_close_live_witness",
        SIM_HUD_PHASE2_CLOSE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(SIM_HUD_PHASE2_CLOSE_LIVE_JSON, wrapped)
        && bundle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_hud_phase2_close_live_witness_green() {
        assert!(refresh_sim_hud_phase2_close_live_witness());
    }
}
