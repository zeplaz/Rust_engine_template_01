//! Shared HUD chrome — Bevy-only after egui retirement.

use crate::gui::style::UiPalette;

use super::panel_state::HudPanelState;

pub const HUD_CHROME_RADIUS: u8 = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HudChromeIcon {
    Expand,
    Collapse,
    PinOn,
    PinOff,
    Sim,
    Build,
    Theater,
    Stack,
    Lod,
    Keys,
}

impl HudChromeIcon {
    #[must_use]
    pub const fn glyph(self) -> &'static str {
        match self {
            Self::Expand => "›",
            Self::Collapse => "‹",
            Self::PinOn => "◆",
            Self::PinOff => "◇",
            Self::Sim => "⏱",
            Self::Build => "⛭",
            Self::Theater => "⚑",
            Self::Stack => "☰",
            Self::Lod => "◎",
            Self::Keys => "⌨",
        }
    }

    #[must_use]
    pub const fn tooltip(self) -> &'static str {
        match self {
            Self::Expand => "Expand panel",
            Self::Collapse => "Collapse panel",
            Self::PinOn => "Pinned — click to unpin",
            Self::PinOff => "Pin panel open",
            Self::Sim => "Simulation",
            Self::Build => "Construction",
            Self::Theater => "Theater summary",
            Self::Stack => "Context stack",
            Self::Lod => "Level of detail",
            Self::Keys => "Keyboard shortcuts",
        }
    }
}

pub fn hud_panel_frame(_palette: &UiPalette) {}
pub fn hud_side_rail_frame(_palette: &UiPalette) {}
pub fn section_rule(_palette: &UiPalette) {}
pub fn side_panel_header(_palette: &UiPalette, _title: &str) {}
pub fn stat_chip(_palette: &UiPalette, _label: &str, _value: &str) {}
pub fn draw_collapsed_side_rail(_palette: &UiPalette, _state: &HudPanelState) {}
pub fn flat_v2_tray_tab(_palette: &UiPalette, _label: &str, _selected: bool) -> bool {
    false
}
pub fn icon_button(_palette: &UiPalette, _icon: HudChromeIcon) -> bool {
    false
}
pub fn panel_state_cycle_button(_palette: &UiPalette, _state: &mut HudPanelState) {}
