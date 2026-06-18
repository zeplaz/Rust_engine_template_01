//! Locked sim HUD copy — [`src/dev/sim_hud_copy_registry_v1.md`].

pub const PICKER_TITLE_ZONE: &str = "Zone";
pub const PICKER_TITLE_ROADS: &str = "Roads";
pub const PICKER_TITLE_INDUSTRY: &str = "Industry";
pub const PICKER_TITLE_UTILITIES: &str = "Utilities";
pub const PICKER_TITLE_SHAPES: &str = "Shapes";

pub const PICKER_INDUSTRY_LEAD: &str = "Place each step separately — power adds on the grid.";
pub const PICKER_INDUSTRY_OTHER: &str = "Other industry";
pub const PICKER_GENERIC_FACTORY: &str = "Generic factory";
pub const PICKER_GENERIC_DEPOT: &str = "Generic depot";
pub const PICKER_EMPTY_CATEGORY: &str = "○ No tools in this category";
pub const PICKER_LOADING: &str = "⟳ Loading build catalog…";
pub const PICKER_ERROR_CATALOG: &str = "✗ Catalog unavailable";

pub const POWER_LIGHT: &str = "⚡ light";
pub const POWER_MEDIUM: &str = "⚡ medium";
pub const POWER_HEAVY: &str = "⚡ heavy";
pub const POWER_GRID: &str = "⊞ grid";

pub const TRAY_BUILD_TAB: &str = "Build";
pub const TRAY_LEGEND_TITLE: &str = "Site stub";
pub const TRAY_LEGEND_FOOTPRINT: &str = "Green — building footprint";
pub const TRAY_LEGEND_YARD: &str = "Dashed — yard / rail / park";
pub const TRAY_STAGING_TITLE: &str = "Staged placement";
pub const TRAY_STAGING_EMPTY: &str = "○ No staged placements";
pub const TRAY_QUEUE_TITLE: &str = "Pending queue";
pub const TRAY_QUEUE_EMPTY: &str = "○ Queue empty";
pub const TRAY_PEEK_MODIFIERS: &str = "Ctrl rotate · Shift scale";

pub const ROAD_SHEET_HINT_INPUT: &str = "LMB add · RMB undo · Shift+LMB commit";
pub const ROAD_SHEET_BUILD: &str = "Build";
pub const ROAD_SHEET_CANCEL: &str = "Cancel";
pub const ROAD_SHEET_UPGRADE: &str = "Upgrade nearest segment";

#[must_use]
pub fn human_chain_label(chain_id: &str) -> String {
    match chain_id {
        "concrete_portland" => "Concrete (Portland)".to_string(),
        "concrete_geopolymer" => "Concrete (Geopolymer)".to_string(),
        "aluminum_primary" => "Aluminum primary".to_string(),
        other => other.replace('_', " "),
    }
}

#[must_use]
pub fn power_tier_compact(power: f32) -> &'static str {
    if power >= 80.0 {
        POWER_HEAVY
    } else if power >= 20.0 {
        POWER_MEDIUM
    } else {
        POWER_LIGHT
    }
}

#[must_use]
pub fn tray_queue_summary(n: usize, first_label: &str) -> String {
    if n == 0 {
        TRAY_QUEUE_EMPTY.to_string()
    } else {
        format!("{n} pending · {first_label}")
    }
}
