//! **HUD-A-003** — ban `egui::Window` on the simulation place path (lint / lib witness).

use std::path::PathBuf;

use serde_json::{json, Value};

use crate::dev::runtime_witness::write_enveloped_witness_unchecked;

pub const HUD_A_SIM_WINDOW_BAN_JSON: &str = "debug_runs/hud_a_sim_window_ban_live.json";

/// Sim place-path modules that must use `egui::Area` / Bevy sheets — not floating `Window`.
const SIM_PLACE_PATH_MODULES: &[&str] = &[
    "src/gui/hud/sim_build_picker_sheet.rs",
    "src/gui/hud/sim_road_tool_sheet.rs",
    "src/gui/hud/sim_power_tool_sheet.rs",
    "src/gui/hud/context_tray_build_egui.rs",
    "src/gui/hud/context_tray_power_repair_egui.rs",
    "src/gui/hud/plant_focus_card.rs",
    "src/gui/hud/power_node_hover_egui.rs",
];

fn repo_root() -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn module_has_window(rel: &str) -> bool {
    let path = repo_root().join(rel);
    let Ok(content) = std::fs::read_to_string(&path) else {
        return true; // missing = fail closed
    };
    content.contains("egui::Window::new") || content.contains("std_floating(egui::Window")
}

/// Road floating popup must stay editor-only (`ROAD_POPUP_FLOATING_IN_SIM = false`).
#[must_use]
pub fn road_popup_floating_banned_in_sim() -> bool {
    let sheet = std::fs::read_to_string(repo_root().join("src/gui/hud/sim_road_tool_sheet.rs"))
        .unwrap_or_default();
    sheet.contains("ROAD_POPUP_FLOATING_IN_SIM: bool = false")
        && sheet.contains("egui::Area::new")
}

/// Placement debug Window must not schedule in Simulation.
#[must_use]
pub fn placement_debug_editor_only_scheduled() -> bool {
    let mod_rs =
        std::fs::read_to_string(repo_root().join("src/construction/mod.rs")).unwrap_or_default();
    let dbg = std::fs::read_to_string(repo_root().join("src/construction/placement_debug.rs"))
        .unwrap_or_default();
    if !dbg.contains("PLACEMENT_DEBUG_WINDOW_IN_SIM: bool = false") {
        return false;
    }
    let Some(idx) = mod_rs.find("draw_construction_placement_debug_overlay") else {
        return false;
    };
    let window = &mod_rs[idx..idx.saturating_add(160).min(mod_rs.len())];
    window.contains("product_egui_shell_active")
}

#[must_use]
pub fn hud_a_003_sim_place_path_window_ban_green() -> bool {
    let offenders: Vec<&str> = SIM_PLACE_PATH_MODULES
        .iter()
        .copied()
        .filter(|rel| module_has_window(rel))
        .collect();
    offenders.is_empty() && road_popup_floating_banned_in_sim() && placement_debug_editor_only_scheduled()
}

#[must_use]
pub fn build_hud_a_003_witness_body() -> Value {
    let green = hud_a_003_sim_place_path_window_ban_green();
    let offenders: Vec<String> = SIM_PLACE_PATH_MODULES
        .iter()
        .copied()
        .filter(|rel| module_has_window(rel))
        .map(str::to_string)
        .collect();
    json!({
        "schema": "hud_a_sim_window_ban_witness_v1",
        "slice_id": "HUD-A-003",
        "status": if green { "green" } else { "pending" },
        "green": green,
        "sim_place_path_modules": SIM_PLACE_PATH_MODULES,
        "window_offenders": offenders,
        "road_popup_floating_in_sim": false,
        "placement_debug_editor_only": placement_debug_editor_only_scheduled(),
    })
}

#[must_use]
pub fn refresh_hud_a_003_live_witness() -> bool {
    write_enveloped_witness_unchecked(
        "HUD-A-003-SIM-WINDOW-BAN",
        "coder_hud_a_window_ban",
        HUD_A_SIM_WINDOW_BAN_JSON,
        build_hud_a_003_witness_body(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hud_a_003_ban_green() {
        assert!(
            hud_a_003_sim_place_path_window_ban_green(),
            "sim place path must not use egui::Window; placement debug editor-only"
        );
        assert!(refresh_hud_a_003_live_witness());
    }
}
