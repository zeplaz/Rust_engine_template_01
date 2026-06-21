//! **COD-POWER-NODE-HOVER-001** — power node hover card witness.

pub const POWER_NODE_HOVER_LIVE_JSON: &str = "debug_runs/power_node_hover_live.json";

#[must_use]
pub fn refresh_power_node_hover_live_witness() -> bool {
    use crate::gui::hud::power_node_hover::{
        power_node_hover_dwell_ms, power_node_hover_tier_map_attached, power_node_hover_witness_green,
        HOVER_CARD_MAX_W, HOVER_CARD_MIN_W, HOVER_HIDE_GRACE_SECS,
    };
    use crate::gui::hud::power_node_hover_egui::power_node_hover_card_wired;

    let green = power_node_hover_witness_green()
        && power_node_hover_card_wired()
        && power_node_hover_tier_map_attached()
        && power_node_hover_dwell_ms() == 150;

    let body = serde_json::json!({
        "gate": "PLAN-POWER-GRID-CONSTRUCTION-UX-001",
        "track": "B",
        "slice_id": "COD-POWER-NODE-HOVER-001",
        "design_ref": "src/dev/design_power_node_hover_v1.md",
        "hover_card_wired": power_node_hover_card_wired(),
        "popup_tier": "map_attached",
        "dwell_ms": power_node_hover_dwell_ms(),
        "hide_grace_ms": (HOVER_HIDE_GRACE_SECS * 1000.0).round() as u32,
        "card_min_w": HOVER_CARD_MIN_W,
        "card_max_w": HOVER_CARD_MAX_W,
        "transformer_title": "Distribution transformer",
        "substation_title": "Grid substation",
        "track_b_green": green,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "COD-POWER-NODE-HOVER-001",
        "refresh_power_node_hover_live_witness",
        POWER_NODE_HOVER_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(POWER_NODE_HOVER_LIVE_JSON, wrapped)
        && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_node_hover_live_witness_green() {
        assert!(refresh_power_node_hover_live_witness());
    }
}
