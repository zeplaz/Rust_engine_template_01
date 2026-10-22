//! Utilities building submenu — catalog-backed picks.
//! egui body retired; Bevy build picker owns chrome.

use crate::gui::hud::power_hud_icon_atlas::{
    PowerHudIconAtlasManifest, PowerHudIconId,
};

/// Optional power HUD atlas for utilities picker rows (COD-ART-HUD-ICON-ATLAS-001).
pub struct UtilitiesSubmenuIconUi<'a> {
    pub texture_id: (),
    pub manifest: &'a PowerHudIconAtlasManifest,
    pub idle_tint: crate::gui::UiRgba,
    pub selected_tint: crate::gui::UiRgba,
}

/// Retired egui entry — Bevy picker owns utilities picks.
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
pub fn draw_utilities_submenu() {}

#[must_use]
pub fn utilities_submenu_power_icons_wired() -> bool {
    PowerHudIconId::inventory().len() >= 13
}
