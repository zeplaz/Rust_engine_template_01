//! **BUILD-READ-DESIGN-002** — refresh `debug_runs/design_build_toolbox_hud_live.json`.

pub const DESIGN_BUILD_TOOLBOX_HUD_LIVE_JSON: &str = "debug_runs/design_build_toolbox_hud_live.json";

#[must_use]
pub fn refresh_design_build_toolbox_hud_live_witness() -> bool {
    let body = crate::gui::hud::contextual_tip::build_read_design_002_witness_json();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let shape_ok = crate::construction::build_read_shape_002_witness_green();
    let site_ok = crate::construction::build_read_site_v0_002_witness_green();
    let mut body = body;
    let copy_ok = body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if let Some(obj) = body.as_object_mut() {
        obj.insert("build_read_shape_002".into(), serde_json::json!(shape_ok));
        obj.insert("build_read_site_v0_002".into(), serde_json::json!(site_ok));
        obj.insert("green".into(), serde_json::json!(copy_ok && site_ok));
        obj.insert(
            "program_rollup_green".into(),
            serde_json::json!(copy_ok && shape_ok && site_ok),
        );
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "BUILD-READ-DESIGN-002",
        "refresh_design_build_toolbox_hud_live_witness",
        DESIGN_BUILD_TOOLBOX_HUD_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(DESIGN_BUILD_TOOLBOX_HUD_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn design_build_toolbox_hud_live_witness_refresh_green() {
        assert!(refresh_design_build_toolbox_hud_live_witness());
    }
}
