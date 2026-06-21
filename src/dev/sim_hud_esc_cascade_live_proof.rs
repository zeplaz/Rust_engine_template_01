//! **COD-SIM-HUD-ESC-CASCADE-001** — Esc cascade witness (picker → trays → pause).

pub const SIM_HUD_ESC_CASCADE_LIVE_JSON: &str = "debug_runs/sim_hud_esc_cascade_live.json";

#[must_use]
pub fn refresh_sim_hud_esc_cascade_live_witness() -> bool {
    use crate::gui::hud::sim_hud_esc_cascade::{
        sim_hud_esc_cascade_witness_green, truncate_build_read_line,
    };

    let green = sim_hud_esc_cascade_witness_green();
    let truncated = truncate_build_read_line(
        "BUILD  ·  Blocked ✗ · reason that is way too long for one strip line",
    );
    let body = serde_json::json!({
        "gate": "COD-SIM-HUD-ESC-CASCADE-001",
        "slice_id": "COD-SIM-HUD-ESC-CASCADE-001",
        "design_ref": "src/dev/design_sim_hud_esc_cascade_v1.md",
        "cascade_wired": green,
        "picker_before_pause": green,
        "tray_before_pause": green,
        "build_read_truncated_sample": truncated,
        "green": green,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-SIM-HUD-ESC-CASCADE-001",
        "refresh_sim_hud_esc_cascade_live_witness",
        SIM_HUD_ESC_CASCADE_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(SIM_HUD_ESC_CASCADE_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_hud_esc_cascade_live_witness_green() {
        assert!(refresh_sim_hud_esc_cascade_live_witness());
    }
}
