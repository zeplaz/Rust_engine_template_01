//! Locked sim HUD copy — [`src/dev/sim_hud_copy_registry_v1.md`].

pub const PICKER_TITLE_ZONE: &str = "Zone";
pub const PICKER_TITLE_ROADS: &str = "Roads";
pub const PICKER_TITLE_INDUSTRY: &str = "Industry";
pub const PICKER_TITLE_UTILITIES: &str = "Utilities";
pub const PICKER_TITLE_SHAPES: &str = "Shapes";
pub const PICKER_TITLE_DEFENSE: &str = "Defense";

pub const PICKER_INDUSTRY_LEAD: &str = "Place each step separately — power adds on the grid.";
pub const PICKER_INDUSTRY_OTHER: &str = "Other industry";
pub const PICKER_GENERIC_FACTORY: &str = "Generic factory";
pub const PICKER_GENERIC_DEPOT: &str = "Generic depot";
pub const PICKER_EMPTY_CATEGORY: &str = "○ No tools in this category";
pub const PICKER_LOADING: &str = "⟳ Loading build catalog…";
pub const PICKER_ERROR_CATALOG: &str = "✗ Catalog unavailable";

pub const PICKER_DEFENSE_LEAD: &str =
    "Poured walls or staged deployables — demolish is separate.";
pub const PICKER_SECTION_WALLS: &str = "Walls";
pub const PICKER_SECTION_TRENCHES: &str = "Trenches";
pub const PICKER_SECTION_FORTIFICATION: &str = "Fortification";
pub const PICKER_SECTION_DEPLOYABLES: &str = "Deployables";
pub const PICKER_ROW_DEFENSIVE_WALL: &str = "Defensive wall";
pub const PICKER_ROW_CAPTION_DEFENSIVE_WALL: &str = "Poured concrete";
pub const PICKER_ROW_TRENCH_LINE: &str = "Trench line";
pub const PICKER_ROW_BUNKER: &str = "Bunker";
pub const PICKER_ROW_DRAGON_TEETH: &str = "Dragon's teeth";
pub const PICKER_ROW_CAPTION_DRAGON_TEETH: &str = "Heavy · staged stock required";
pub const PICKER_ROW_MINEFIELD: &str = "Minefield";
pub const PICKER_ROW_CAPTION_MINEFIELD: &str = "Light · staged stock required";
pub const PICKER_ROW_DEMOLISH: &str = "Demolish";
pub const PICKER_DEFENSE_FOOTER_HINT: &str =
    "Two clicks to place · stock gates commit · Ctrl rotate · Shift size";

pub const TRAY_DEFENSE_PREVIEW: &str = "Click map to lock";
pub const TRAY_DEFENSE_ADJUST_VALID: &str = "Place";
pub const TRAY_DEFENSE_ADJUST_INVALID: &str = "Cannot place";
pub const TRAY_DEMOLISH_ARMED: &str = "LMB: pick target · Confirm: demolish";
pub const TRAY_DEPLOYABLE_STOCK_EMPTY: &str =
    "No staged stock — manufacture and haul first";

pub const REASON_INSUFFICIENT_CONCRETE: &str = "insufficient concrete";
pub const REASON_INSUFFICIENT_STAGED: &str = "insufficient staged stock";
pub const REASON_STOCK_IN_TRANSIT: &str = "stock still in transit";

pub const HINT_DEFENSIVE_WALL: &str = "Defensive wall — poured concrete on place";
pub const HINT_DRAGON_TEETH: &str = "Dragon's teeth — needs staged heavy units";
pub const HINT_MINEFIELD: &str = "Minefield — needs staged mine units";

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
pub fn defense_preview_strip(row_label: &str) -> String {
    format!("{TRAY_DEFENSE_PREVIEW} {row_label}")
}

#[must_use]
pub fn defense_adjust_valid_strip(row_label: &str) -> String {
    format!("{TRAY_DEFENSE_ADJUST_VALID} {row_label} — click map again")
}

#[must_use]
pub fn defense_adjust_invalid_strip(reason: &str) -> String {
    format!("{TRAY_DEFENSE_ADJUST_INVALID} — {reason}")
}

/// Deployable staging readout for ghost / context strip (**DES-MIL-DEPLOYABLE-CATALOG** Q1).
#[must_use]
pub fn defense_deployable_stock_line(staged: u32, in_transit: u32, need: u32) -> String {
    if staged == 0 && in_transit == 0 {
        return TRAY_DEPLOYABLE_STOCK_EMPTY.to_string();
    }
    let mut line = format!("Staged {staged} · in transit {in_transit}");
    if staged >= need && need > 0 {
        line.push_str(" — ready");
    }
    line
}

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
