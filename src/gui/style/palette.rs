//! Token colors for egui + future Bevy UI. See `prompts/guides/ui_design_language_plan_v1.md`.

use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, CornerRadius, Stroke, Visuals};

/// Authoritative dark-theme palette (Orgburo “CMD” + readable Rust-ecosystem contrast).
#[derive(Resource, Debug, Clone)]
pub struct UiPalette {
    pub bg_app: Color32,
    pub bg_elevated: Color32,
    pub bg_deep: Color32,
    pub bg_interactive: Color32,
    pub fg_primary: Color32,
    pub fg_muted: Color32,
    pub accent_terminal: Color32,
    pub accent_action: Color32,
    pub accent_hot: Color32,
    pub fg_on_accent: Color32,
    pub selection_bg: Color32,
    pub warn: Color32,
    pub danger: Color32,
}

impl Default for UiPalette {
    fn default() -> Self {
        Self {
            bg_app: Color32::from_rgb(0x12, 0x12, 0x12),
            bg_elevated: Color32::from_rgb(0x1c, 0x1e, 0x22),
            bg_deep: Color32::from_rgb(0x0a, 0x0a, 0x0c),
            bg_interactive: Color32::from_rgb(0x28, 0x2c, 0x34),
            fg_primary: Color32::from_rgb(0xec, 0xee, 0xf6),
            fg_muted: Color32::from_rgb(0x98, 0xa2, 0xb0),
            accent_terminal: Color32::from_rgb(0x5d, 0xca, 0x31),
            accent_action: Color32::from_rgb(0xf5, 0x7c, 0x00),
            accent_hot: Color32::from_rgb(0xc6, 0x46, 0x00),
            fg_on_accent: Color32::BLACK,
            selection_bg: Color32::from_rgba_unmultiplied(0x5d, 0xca, 0x31, 0x35),
            warn: Color32::from_rgb(0xe9, 0xc4, 0x6a),
            danger: Color32::from_rgb(0xf8, 0x71, 0x71),
        }
    }
}

impl UiPalette {
    /// Full egui `Visuals` for `Context::set_visuals`.
    #[must_use]
    pub fn to_egui_visuals(&self) -> Visuals {
        let mut v = Visuals::dark();
        v.dark_mode = true;
        v.panel_fill = self.bg_app;
        v.window_fill = self.bg_elevated;
        v.extreme_bg_color = self.bg_deep;
        v.faint_bg_color = self.bg_deep;
        v.code_bg_color = Color32::from_rgb(0x14, 0x18, 0x14);

        v.override_text_color = Some(self.fg_primary);

        let w = &mut v.widgets;
        w.noninteractive.bg_fill = self.bg_elevated;
        w.noninteractive.weak_bg_fill = self.bg_deep;
        w.noninteractive.fg_stroke = Stroke::new(1.0, self.fg_muted);
        w.noninteractive.bg_stroke = Stroke::NONE;

        w.inactive.bg_fill = self.bg_interactive;
        w.inactive.weak_bg_fill = self.bg_app;
        w.inactive.fg_stroke = Stroke::new(1.0, self.fg_primary);
        w.inactive.bg_stroke = Stroke::new(1.0, Color32::from_gray(45));

        w.hovered.bg_fill = self.accent_hot;
        w.hovered.weak_bg_fill = self.accent_hot.gamma_multiply(0.85);
        w.hovered.fg_stroke = Stroke::new(1.0, self.fg_on_accent);
        w.hovered.bg_stroke = Stroke::new(1.0, self.accent_hot);

        w.active.bg_fill = self.accent_action;
        w.active.weak_bg_fill = self.accent_action.gamma_multiply(0.9);
        w.active.fg_stroke = Stroke::new(1.0, self.fg_on_accent);
        w.active.bg_stroke = Stroke::new(1.0, self.accent_action);

        w.open.fg_stroke = Stroke::new(1.0, self.accent_terminal);

        v.selection.bg_fill = self.selection_bg;
        v.selection.stroke = Stroke::new(1.0, self.accent_terminal);

        v.hyperlink_color = self.accent_terminal;
        v.warn_fg_color = self.warn;
        v.error_fg_color = self.danger;

        let r = CornerRadius::same(4);
        v.window_corner_radius = r;
        v.menu_corner_radius = r;
        v.popup_shadow = egui::epaint::Shadow {
            offset: [4, 8],
            blur: 12,
            spread: 0,
            color: Color32::from_black_alpha(180),
        };

        v
    }

    /// Bevy UI backdrop aligned with [`Self::bg_app`] (main menu / shell parity — optional use).
    #[must_use]
    pub fn bevy_backdrop(&self) -> Color {
        self.color32_to_bevy(self.bg_app)
    }

    /// Bevy UI primary text / chip color.
    #[must_use]
    pub fn bevy_primary_text(&self) -> Color {
        self.color32_to_bevy(self.fg_primary)
    }

    /// Bevy UI button surface (inactive), aligned with egui `bg_interactive`.
    #[must_use]
    pub fn bevy_button_idle(&self) -> Color {
        self.color32_to_bevy(self.bg_interactive)
    }

    #[must_use]
    pub fn bevy_text_muted(&self) -> Color {
        self.color32_to_bevy(self.fg_muted)
    }

    /// Between primary and muted (path lines, captions).
    #[must_use]
    pub fn bevy_secondary_text(&self) -> Color {
        let p = self.fg_primary;
        let m = self.fg_muted;
        Color::srgba_u8(
            ((p.r() as u16 + m.r() as u16) / 2) as u8,
            ((p.g() as u16 + m.g() as u16) / 2) as u8,
            ((p.b() as u16 + m.b() as u16) / 2) as u8,
            255,
        )
    }

    fn color32_to_bevy(&self, c: Color32) -> Color {
        Color::srgba_u8(c.r(), c.g(), c.b(), 255)
    }
}
