//! RGR-M1 chain lib witnesses — render/api.rs surface + plugin latches (M1-001..004).

use crate::dev::debug_run_envelope::{wrap_debug_run, write_debug_run_json};

pub const RGR_M1_WITNESS_JSON: &str = "debug_runs/rgr_m1_witness_live.json";

#[must_use]
pub fn rgr_m1_api_module_green() -> bool {
    std::path::Path::new("src/render/api.rs").is_file()
        && include_str!("../render/mod.rs").contains("pub mod api;")
        && include_str!("../render/mod.rs").contains("pub use api::*;")
}

#[must_use]
pub fn rgr_m1_mod_rs_pub_use_collapsed_green() -> bool {
    let mod_rs = include_str!("../render/mod.rs");
    mod_rs.matches("pub use ").count() <= 2
        && !mod_rs.contains("MinimapShellState")
}

#[must_use]
pub fn rgr_m1_gui_reexport_removed_green() -> bool {
    !include_str!("../render/mod.rs").contains("pub use crate::gui")
        && include_str!("../render/api.rs").contains("deprecated_gui_shims")
}

#[must_use]
pub fn rgr_m1_plugin_latches_green() -> bool {
    let vd = include_str!("../render/probes/visual_diagnostics.rs");
    let sw = include_str!("../render/probes/stall_watch.rs");
    let fr = include_str!("../render/probes/full_render_diagnostic.rs");
    vd.contains("visual_diag_plugin_latch")
        && vd.contains("if !visual_diag_plugin_latch()")
        && sw.contains("stall_watch_plugin_latch")
        && sw.contains("if !stall_watch_plugin_latch()")
        && fr.contains("full_render_diagnostic_plugin_latch")
        && fr.contains("if !full_render_diagnostic_plugin_latch()")
}

#[must_use]
pub fn rgr_m1_witness_green() -> bool {
    rgr_m1_api_module_green()
        && rgr_m1_mod_rs_pub_use_collapsed_green()
        && rgr_m1_gui_reexport_removed_green()
        && rgr_m1_plugin_latches_green()
}

#[must_use]
pub fn rgr_m1_witness_json() -> serde_json::Value {
    serde_json::json!({
        "schema": "rgr_m1_witness_v1",
        "green": rgr_m1_witness_green(),
        "RGR-M1-001": rgr_m1_api_module_green() && rgr_m1_mod_rs_pub_use_collapsed_green(),
        "RGR-M1-002_003": rgr_m1_gui_reexport_removed_green(),
        "RGR-M1-004": rgr_m1_plugin_latches_green(),
        "mod_rs_pub_use_count": include_str!("../render/mod.rs").matches("pub use ").count(),
        "api_rs_pub_use_count": include_str!("../render/api.rs").matches("pub use ").count(),
    })
}

#[must_use]
pub fn refresh_rgr_m1_witness() -> bool {
    let body = rgr_m1_witness_json();
    let green = body.get("green").and_then(|v| v.as_bool()).unwrap_or(false);
    let wrapped = wrap_debug_run("RGR-M1", "refresh_rgr_m1_witness", RGR_M1_WITNESS_JSON, body);
    write_debug_run_json(RGR_M1_WITNESS_JSON, wrapped) && green
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgr_m1_api_surface() {
        assert!(rgr_m1_api_module_green());
    }

    #[test]
    fn rgr_m1_mod_rs_thin() {
        assert!(rgr_m1_mod_rs_pub_use_collapsed_green());
    }

    #[test]
    fn rgr_m1_gui_shims_only() {
        assert!(rgr_m1_gui_reexport_removed_green());
    }

    #[test]
    fn rgr_m1_plugin_latches() {
        assert!(rgr_m1_plugin_latches_green());
    }

    #[test]
    fn rgr_m1_all_slices_green() {
        assert!(rgr_m1_witness_green(), "{}", rgr_m1_witness_json());
    }

    #[test]
    fn rgr_m1_witness_refresh_writes_json() {
        crate::dev::debug_run_envelope::reset_witness_refresh_gate_for_tests();
        assert!(refresh_rgr_m1_witness());
    }
}
