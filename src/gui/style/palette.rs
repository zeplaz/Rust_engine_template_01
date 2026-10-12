//! Token colors for Bevy UI (post egui drop).
//! Spec: `prompts/guides/ui_design_language_plan_v1.md`.

use bevy::prelude::*;

use crate::gui::ui_types::UiRgba;

/// Authoritative dark-theme palette ([SAPIP CMD](https://orgburo.org/sapip/) wireframe language).
#[derive(Resource, Debug, Clone)]
pub struct UiPalette {
    pub bg_app: UiRgba,
    pub bg_elevated: UiRgba,
    pub bg_deep: UiRgba,
    pub bg_interactive: UiRgba,
    pub fg_primary: UiRgba,
    pub fg_muted: UiRgba,
    pub accent_terminal: UiRgba,
    pub accent_action: UiRgba,
    pub accent_hot: UiRgba,
    pub fg_on_accent: UiRgba,
    pub selection_bg: UiRgba,
    pub warn: UiRgba,
    pub danger: UiRgba,
    pub wire_magenta: UiRgba,
    pub wire_red: UiRgba,
    pub accent_gold: UiRgba,
    pub fg_data: UiRgba,
    pub bg_vellum: UiRgba,
    pub bg_paper: UiRgba,
}

impl Default for UiPalette {
    fn default() -> Self {
        Self {
            bg_app: UiRgba::BLACK,
            bg_elevated: UiRgba::from_rgb(0x06, 0x08, 0x08),
            bg_deep: UiRgba::BLACK,
            bg_interactive: UiRgba::from_rgb(0x0c, 0x12, 0x12),
            fg_primary: UiRgba::from_rgb(0x5e, 0xe0, 0xdc),
            fg_muted: UiRgba::from_rgb(0x4a, 0x78, 0x78),
            accent_terminal: UiRgba::from_rgb(0x5d, 0xca, 0x31),
            accent_action: UiRgba::from_rgb(0xf0, 0xa8, 0x28),
            accent_hot: UiRgba::from_rgb(0xdc, 0x38, 0xb8),
            fg_on_accent: UiRgba::BLACK,
            selection_bg: UiRgba::from_rgba_unmultiplied(0x5d, 0xca, 0x31, 0x40),
            warn: UiRgba::from_rgb(0xe9, 0xc4, 0x6a),
            danger: UiRgba::from_rgb(0xff, 0x44, 0x44),
            wire_magenta: UiRgba::from_rgb(0xd9, 0x46, 0xef),
            wire_red: UiRgba::from_rgb(0xff, 0x3d, 0x3d),
            accent_gold: UiRgba::from_rgb(0xe8, 0xc0, 0x3a),
            fg_data: UiRgba::from_rgb(0x5d, 0xca, 0x31),
            bg_vellum: UiRgba::from_rgb(0x18, 0x16, 0x12),
            bg_paper: UiRgba::from_rgb(0x0a, 0x0c, 0x0a),
        }
    }
}

impl UiPalette {
    #[must_use]
    pub fn bevy_bg_elevated(&self) -> Color {
        self.ui_to_bevy(self.bg_elevated)
    }

    #[must_use]
    pub fn bevy_bg_deep(&self) -> Color {
        self.ui_to_bevy(self.bg_deep)
    }

    #[must_use]
    pub fn bevy_accent_terminal(&self) -> Color {
        self.ui_to_bevy(self.accent_terminal)
    }

    #[must_use]
    pub fn bevy_accent_action(&self) -> Color {
        self.ui_to_bevy(self.accent_action)
    }

    #[must_use]
    pub fn bevy_wire_magenta(&self) -> Color {
        self.ui_to_bevy(self.wire_magenta)
    }

    #[must_use]
    pub fn bevy_accent_hot(&self) -> Color {
        self.ui_to_bevy(self.accent_hot)
    }

    #[must_use]
    pub fn bevy_border_subtle(&self) -> Color {
        let m = self.wire_magenta;
        Color::srgba(
            m.r as f32 / 255.0,
            m.g as f32 / 255.0,
            m.b as f32 / 255.0,
            0.55,
        )
    }

    #[must_use]
    pub fn bevy_hud_panel_fill(&self) -> Color {
        let c = self.bg_elevated;
        Color::srgba(
            c.r as f32 / 255.0,
            c.g as f32 / 255.0,
            c.b as f32 / 255.0,
            0.92,
        )
    }

    #[must_use]
    pub fn bevy_backdrop(&self) -> Color {
        self.ui_to_bevy(self.bg_app)
    }

    #[must_use]
    pub fn bevy_primary_text(&self) -> Color {
        self.ui_to_bevy(self.fg_primary)
    }

    #[must_use]
    pub fn bevy_button_idle(&self) -> Color {
        self.ui_to_bevy(self.bg_interactive)
    }

    #[must_use]
    pub fn bevy_text_muted(&self) -> Color {
        self.ui_to_bevy(self.fg_muted)
    }

    #[must_use]
    pub fn bevy_secondary_text(&self) -> Color {
        let p = self.fg_primary;
        let m = self.fg_muted;
        Color::srgba_u8(
            ((p.r as u16 + m.r as u16) / 2) as u8,
            ((p.g as u16 + m.g as u16) / 2) as u8,
            ((p.b as u16 + m.b as u16) / 2) as u8,
            255,
        )
    }

    pub fn bevy_paper_fill(&self) -> Color {
        let c = self.bg_paper;
        Color::srgba(
            c.r as f32 / 255.0,
            c.g as f32 / 255.0,
            c.b as f32 / 255.0,
            0.94,
        )
    }

    #[must_use]
    pub fn bevy_sim_map_field_clear(&self) -> Color {
        self.bevy_sim_map_field_clear_for_daylight(1.0)
    }

    #[must_use]
    pub fn bevy_sim_map_field_clear_day(&self) -> Color {
        Color::srgb(0.48, 0.54, 0.64)
    }

    #[must_use]
    pub fn bevy_sim_map_field_clear_night(&self) -> Color {
        Color::srgb(0.06, 0.09, 0.14)
    }

    #[must_use]
    pub fn bevy_sim_map_field_clear_for_daylight(&self, daylight: f32) -> Color {
        let d = daylight.clamp(0.0, 1.0);
        let day = self.bevy_sim_map_field_clear_day().to_srgba();
        let night = self.bevy_sim_map_field_clear_night().to_srgba();
        Color::srgb(
            day.red * d + night.red * (1.0 - d),
            day.green * d + night.green * (1.0 - d),
            day.blue * d + night.blue * (1.0 - d),
        )
    }

    #[must_use]
    pub fn bevy_fg_data(&self) -> Color {
        self.ui_to_bevy(self.fg_data)
    }

    #[must_use]
    pub fn bevy_bg_vellum(&self) -> Color {
        let c = self.bg_vellum;
        Color::srgba(
            c.r as f32 / 255.0,
            c.g as f32 / 255.0,
            c.b as f32 / 255.0,
            0.96,
        )
    }

    #[must_use]
    pub fn bevy_accent_gold(&self) -> Color {
        self.ui_to_bevy(self.accent_gold)
    }

    fn ui_to_bevy(&self, c: UiRgba) -> Color {
        Color::srgba_u8(c.r, c.g, c.b, 255)
    }
}
