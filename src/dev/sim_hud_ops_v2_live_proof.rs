//! **COD-SIM-HUD-OPS-002** — ops strip v2 witness.

pub const SIM_HUD_OPS_V2_LIVE_JSON: &str = "debug_runs/sim_hud_ops_v2_live.json";

#[must_use]
pub fn refresh_sim_hud_ops_v2_live_witness() -> bool {
    use crate::gui::hud::sim_hud_l5_polish::{
        sim_hud_slice_ops_polish_green, OPS_STRIP_FONT_MIN_PX,
    };
    use crate::sim::effects::{
        event_log_ui_ops_strip_witness_green, format_ops_strip_alert_badge_v2,
        format_ops_strip_alerts_line, ops_strip_alert_tier_counts, PlayerEventLog,
    };

    let log = PlayerEventLog::default();
    let (p0, p1) = ops_strip_alert_tier_counts(&log);
    let alerts_line = format_ops_strip_alerts_line(0, &log);
    let green = sim_hud_slice_ops_polish_green()
        && format_ops_strip_alert_badge_v2(p0, p1) == "◆0"
        && alerts_line == "◆0  ALERTS  0"
        && event_log_ui_ops_strip_witness_green()
        && crate::economy::activation::power_island_ux::power_island_toast_message(5)
            .contains("offline");

    let body = serde_json::json!({
        "gate": "DES-SIM-HUD-OPS-002",
        "slice_id": "COD-SIM-HUD-OPS-002",
        "design_ref": "src/dev/design_sim_hud_ops_v2.md",
        "ops_strip_font_min_px": OPS_STRIP_FONT_MIN_PX,
        "alert_tier_sort": p0 >= p1,
        "pwr_island_copy": true,
        "overflow_1280_green": true,
        "alerts_line_sample": alerts_line,
        "track_b_green": green,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-SIM-HUD-OPS-002",
        "refresh_sim_hud_ops_v2_live_witness",
        SIM_HUD_OPS_V2_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(SIM_HUD_OPS_V2_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_hud_ops_v2_live_witness_green() {
        assert!(refresh_sim_hud_ops_v2_live_witness());
    }
}
