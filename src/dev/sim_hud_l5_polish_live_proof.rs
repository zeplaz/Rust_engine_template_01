//! CODER-B-HUD-L5-POLISH-001 — simulation HUD lane 5 readability witness.

use serde_json::{json, Value};

use crate::gui::hud::{
    sim_hud_info_panel_tokens_ok, sim_hud_l5_polish_rollup_green, sim_hud_slice_ops_polish_green,
    INFO_PANEL_BODY_FONT_MIN_PX, OPS_STRIP_BODY_FONT_PT, OPS_STRIP_FONT_MIN_PX,
};
use crate::gui::simulation_minimap_overlay_defaults;

pub const SIM_HUD_L5_POLISH_LIVE_JSON: &str = "debug_runs/sim_hud_l5_polish_live.json";

#[must_use]
pub fn build_sim_hud_l5_polish_payload() -> Value {
    let mask = simulation_minimap_overlay_defaults();
    let lib_green = sim_hud_l5_polish_rollup_green();
    json!({
        "gate": "CODER-B-HUD-L5-POLISH-001",
        "slice_id": "CODER-B-HUD-L5-POLISH-001",
        "program_id": "SIM-HUD-PRODUCT-001",
        "lane": "BEVY-HUD-5",
        "green": lib_green,
        "polish": {
            "ops_strip_font_min_px": OPS_STRIP_FONT_MIN_PX,
            "ops_strip_body_font_pt": OPS_STRIP_BODY_FONT_PT,
            "info_panel_body_font_min_px": INFO_PANEL_BODY_FONT_MIN_PX,
            "ops_slice_polish_green": sim_hud_slice_ops_polish_green(),
            "info_panel_tokens_ok": sim_hud_info_panel_tokens_ok(),
            "minimap_fire_heat_off_by_default": !mask.fire_heat,
            "infra_legend_rows": crate::render::infrastructure_overlay_legend_rows().len(),
            "growth_ecology_hint_wired":
                crate::gui::construction_growth_inspector::growth_hud_ecology_hint_wired_witness_green(),
        },
        "parent_close_witness": "debug_runs/sim_hud_product_close_001_live.json",
        "docs": [
            "docs/archive/2026-06-src-dev/plans/bevy_hud_lanes_agent_orders_v1.md",
            "docs/archive/2026-06-src-dev/plans/design_sim_hud_ops_v1.md",
        ],
        "code": [
            "src/gui/hud/sim_hud_l5_polish.rs",
            "src/gui/hud/info_tabs.rs",
            "src/gui/in_game_hud.rs",
        ],
    })
}

#[must_use]
pub fn refresh_sim_hud_l5_polish_live_witness() -> bool {
    let body = build_sim_hud_l5_polish_payload();
    if body.get("green").and_then(|v| v.as_bool()) != Some(true) {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "CODER-B-HUD-L5-POLISH-001",
        "refresh_sim_hud_l5_polish_live_witness",
        SIM_HUD_L5_POLISH_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(SIM_HUD_L5_POLISH_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_hud_l5_polish_live_witness_green() {
        assert!(refresh_sim_hud_l5_polish_live_witness());
        let raw = std::fs::read_to_string(SIM_HUD_L5_POLISH_LIVE_JSON).expect("witness");
        let doc: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(doc.get("green").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            doc.pointer("/polish/ops_strip_font_min_px").and_then(|v| v.as_f64()),
            Some(11.0)
        );
    }
}
