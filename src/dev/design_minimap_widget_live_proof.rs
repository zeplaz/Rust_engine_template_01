//! **MINIMAP-WIDGET-IMPL-001** — refresh `debug_runs/design_minimap_widget_live.json`.

pub const DESIGN_MINIMAP_WIDGET_LIVE_JSON: &str = "debug_runs/design_minimap_widget_live.json";

#[must_use]
pub fn refresh_design_minimap_widget_live_witness() -> bool {
    let body = crate::gui::hud::minimap_bevy_interaction::minimap_widget_impl_001_witness_json();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "MINIMAP-WIDGET-IMPL-001",
        "refresh_design_minimap_widget_live_witness",
        DESIGN_MINIMAP_WIDGET_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(DESIGN_MINIMAP_WIDGET_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn design_minimap_widget_live_witness_refresh_green() {
        assert!(refresh_design_minimap_widget_live_witness());
    }
}
