//! **MINIMAP-WIDGET-IMPL-001** — refresh `debug_runs/design_minimap_widget_live.json`.
//!
//! Lib green retired (CLN-WIT-001): egui drag/tap cannot be proven without a runtime session.

pub const DESIGN_MINIMAP_WIDGET_LIVE_JSON: &str = "debug_runs/design_minimap_widget_live.json";

#[must_use]
pub fn refresh_design_minimap_widget_live_witness() -> bool {
    let body = crate::gui::hud::minimap_bevy_interaction::minimap_widget_impl_001_witness_json();
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "MINIMAP-WIDGET-IMPL-001",
        "refresh_design_minimap_widget_live_witness",
        DESIGN_MINIMAP_WIDGET_LIVE_JSON,
        body.clone(),
    );
    let written =
        crate::dev::debug_run_envelope::write_debug_run_json(DESIGN_MINIMAP_WIDGET_LIVE_JSON, wrapped);
    // Honest: write retired witness, but do not report lib-green.
    written
        && body
            .get("green")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn design_minimap_widget_live_witness_refresh_writes_retired() {
        // Refresh writes JSON but returns false (lib green retired).
        assert!(!refresh_design_minimap_widget_live_witness());
        let raw = std::fs::read_to_string(DESIGN_MINIMAP_WIDGET_LIVE_JSON).expect("witness");
        let doc: serde_json::Value = serde_json::from_str(&raw).expect("parse");
        assert_eq!(doc.get("cheat_retired").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(
            doc.get("proof_grade").and_then(|v| v.as_str()),
            Some("lib_fixture_retired")
        );
        assert_eq!(doc.get("green").and_then(|v| v.as_bool()), Some(false));
    }
}
