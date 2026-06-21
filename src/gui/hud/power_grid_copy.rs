//! Locked power grid HUD copy — [`src/dev/design_power_grid_copy_v1.md`] §hover.

pub const POWER_HOVER_TRANSFORMER_TITLE: &str = "Distribution transformer";
pub const POWER_HOVER_SUBSTATION_TITLE: &str = "Grid substation";

pub const POWER_HOVER_STATUS_ONLINE: &str = "● Online";
pub const POWER_HOVER_STATUS_OFFLINE: &str = "○ Offline";
pub const POWER_HOVER_STATUS_DAMAGED: &str = "◆ Damaged";
pub const POWER_HOVER_STATUS_DESTROYED: &str = "× Destroyed";
pub const POWER_HOVER_STATUS_OVERLOAD: &str = "⟳ Overload";

pub const POWER_HOVER_LOAD: &str = "Load";
pub const POWER_HOVER_CAPACITY: &str = "Capacity";
pub const POWER_HOVER_FEEDS: &str = "Feeds";
pub const POWER_HOVER_LINKS: &str = "Links";
pub const POWER_HOVER_VOLTAGE_MIXED: &str = "Mixed voltage";

pub const POWER_HOVER_YARD_BUS: &str = "bus + breakers";

#[must_use]
pub fn power_hover_feeds_fmt(n: u32) -> String {
    format!("{n} consumers")
}

#[must_use]
pub fn power_hover_links_fmt(lines: u32, upstream: u32) -> String {
    format!("{lines} lines · {upstream} upstream")
}

#[must_use]
pub fn power_hover_capacity_fmt(used: f32, max: f32) -> String {
    format!("{used:.1} / {max:.1} MVA")
}

#[must_use]
pub fn power_hover_yard_fmt(width: u32, depth: u32) -> String {
    format!("{width}×{depth} · {POWER_HOVER_YARD_BUS}")
}
