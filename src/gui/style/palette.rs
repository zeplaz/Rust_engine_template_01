//! Token colors for egui + future Bevy UI.
//! Spec: `prompts/guides/ui_design_language_plan_v1.md`.
//! Visual reference: [SAPIP / DATA_SYS_CMD](https://orgburo.org/sapip/data-sys-cmd-modelz/index.html)
//! (wireframe CRT: black field, cyan labels, green data, magenta/red strokes — e.g.
//! `org-cmd-terminal.jpg` on that surface).

use bevy::prelude::*;
use bevy_egui::egui::{self, Color32, CornerRadius, Stroke, Visuals};

/// Authoritative dark-theme palette ([SAPIP CMD](https://orgburo.org/sapip/) wireframe language).
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
    /// 1px panel chrome (magenta family, `org-cmd-terminal` wireframes).
    pub wire_magenta: Color32,
    /// Axes / high-attention wire (red family).
    pub wire_red: Color32,
    /// Viewport / committed-highlight ring (gold, sparse use).
    pub accent_gold: Color32,
    /// Telemetry / tick data (green mono — `T+00042` ops strip).
    pub fg_data: Color32,
    /// Warm vellum panel wash (context tray selected tab).
    pub bg_vellum: Color32,
    /// Paper field wash (ops strip / archival panels).
    pub bg_paper: Color32,
}

impl Default for UiPalette {
    fn default() -> Self {
        Self {
            // Pure black field; lifted panels stay near-black for scan contrast.
            bg_app: Color32::BLACK,
            bg_elevated: Color32::from_rgb(0x06, 0x08, 0x08),
            bg_deep: Color32::BLACK,
            bg_interactive: Color32::from_rgb(0x0c, 0x12, 0x12),
            // Cyan/teal = primary labels (not warm white).
            fg_primary: Color32::from_rgb(0x5e, 0xe0, 0xdc),
            fg_muted: Color32::from_rgb(0x4a, 0x78, 0x78),
            // Vibrant green = telemetry / OK rails / selection accent.
            accent_terminal: Color32::from_rgb(0x5d, 0xca, 0x31),
            accent_action: Color32::from_rgb(0xf0, 0xa8, 0x28),
            accent_hot: Color32::from_rgb(0xdc, 0x38, 0xb8),
            fg_on_accent: Color32::BLACK,
            selection_bg: Color32::from_rgba_unmultiplied(0x5d, 0xca, 0x31, 0x40),
            warn: Color32::from_rgb(0xe9, 0xc4, 0x6a),
            danger: Color32::from_rgb(0xff, 0x44, 0x44),
            wire_magenta: Color32::from_rgb(0xd9, 0x46, 0xef),
            wire_red: Color32::from_rgb(0xff, 0x3d, 0x3d),
            accent_gold: Color32::from_rgb(0xe8, 0xc0, 0x3a),
            fg_data: Color32::from_rgb(0x5d, 0xca, 0x31),
            bg_vellum: Color32::from_rgb(0x18, 0x16, 0x12),
            bg_paper: Color32::from_rgb(0x0a, 0x0c, 0x0a),
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
        v.window_fill = self.bg_app;
        v.extreme_bg_color = self.bg_deep;
        v.faint_bg_color = self.bg_elevated;
        v.code_bg_color = Color32::from_rgb(0x06, 0x10, 0x0a);

        v.override_text_color = Some(self.fg_primary);
        v.weak_text_color = Some(self.fg_muted);
        v.weak_text_alpha = 0.7;

        let soft = CornerRadius::same(4);
        v.window_corner_radius = soft;
        v.menu_corner_radius = soft;
        v.window_stroke = Stroke::new(1.0, self.wire_magenta);
        v.window_shadow = egui::epaint::Shadow::NONE;
        v.popup_shadow = egui::epaint::Shadow::NONE;

        let w = &mut v.widgets;
        w.noninteractive.corner_radius = soft;
        w.noninteractive.bg_fill = self.bg_elevated;
        w.noninteractive.weak_bg_fill = self.bg_deep;
        w.noninteractive.fg_stroke = Stroke::new(1.0, self.fg_muted);
        let sep = self.wire_magenta;
        w.noninteractive.bg_stroke = Stroke::new(
            1.0,
            Color32::from_rgba_unmultiplied(sep.r(), sep.g(), sep.b(), 70),
        );

        w.inactive.corner_radius = soft;
        w.inactive.bg_fill = self.bg_interactive;
        w.inactive.weak_bg_fill = self.bg_app;
        w.inactive.fg_stroke = Stroke::new(1.0, self.fg_primary);
        w.inactive.bg_stroke = Stroke::new(1.0, self.wire_magenta);

        w.hovered.corner_radius = soft;
        w.hovered.bg_fill = self.bg_interactive;
        w.hovered.weak_bg_fill = self.bg_deep;
        w.hovered.fg_stroke = Stroke::new(1.0, self.fg_primary);
        w.hovered.bg_stroke = Stroke::new(1.0, self.accent_hot);

        w.active.corner_radius = soft;
        w.active.bg_fill = self.accent_action;
        w.active.weak_bg_fill = self.accent_action.gamma_multiply(0.92);
        w.active.fg_stroke = Stroke::new(1.0, self.fg_on_accent);
        w.active.bg_stroke = Stroke::new(1.0, self.accent_gold);

        w.open.corner_radius = soft;
        w.open.fg_stroke = Stroke::new(1.0, self.accent_terminal);

        v.selection.bg_fill = self.selection_bg;
        v.selection.stroke = Stroke::new(1.0, self.accent_terminal);

        v.hyperlink_color = self.accent_terminal;
        v.warn_fg_color = self.warn;
        v.error_fg_color = self.danger;

        v
    }

    /// Bevy UI panel surface (cards / HUD chrome), aligned with [`Self::bg_elevated`].
    #[must_use]
    pub fn bevy_bg_elevated(&self) -> Color {
        self.color32_to_bevy(self.bg_elevated)
    }

    /// Deepest backdrop (splash bleed, egui extreme bg).
    #[must_use]
    pub fn bevy_bg_deep(&self) -> Color {
        self.color32_to_bevy(self.bg_deep)
    }

    #[must_use]
    pub fn bevy_accent_terminal(&self) -> Color {
        self.color32_to_bevy(self.accent_terminal)
    }

    #[must_use]
    pub fn bevy_accent_action(&self) -> Color {
        self.color32_to_bevy(self.accent_action)
    }

    /// Panel wire on Bevy nodes (parity with egui `window_stroke`).
    #[must_use]
    pub fn bevy_wire_magenta(&self) -> Color {
        self.color32_to_bevy(self.wire_magenta)
    }

    #[must_use]
    pub fn bevy_accent_hot(&self) -> Color {
        self.color32_to_bevy(self.accent_hot)
    }

    /// Subtle stroke for HUD / panels — magenta wire at reduced alpha.
    #[must_use]
    pub fn bevy_border_subtle(&self) -> Color {
        let m = self.wire_magenta;
        Color::srgba(
            m.r() as f32 / 255.0,
            m.g() as f32 / 255.0,
            m.b() as f32 / 255.0,
            0.55,
        )
    }

    /// Semitransparent HUD stack over the map (readability without a solid slab).
    #[must_use]
    pub fn bevy_hud_panel_fill(&self) -> Color {
        let c = self.bg_elevated;
        Color::srgba(
            c.r() as f32 / 255.0,
            c.g() as f32 / 255.0,
            c.b() as f32 / 255.0,
            0.92,
        )
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

    /// Paper field wash (ops strip / archival panels).
    pub fn bevy_paper_fill(&self) -> Color {
        let c = self.bg_paper;
        Color::srgba(
            c.r() as f32 / 255.0,
            c.g() as f32 / 255.0,
            c.b() as f32 / 255.0,
            0.94,
        )
    }

    /// Tactical map RTT void — cool gray-blue (matches world-preview GPU clear).
    #[must_use]
    pub fn bevy_sim_map_field_clear(&self) -> Color {
        Color::srgb(0.06, 0.09, 0.14)
    }

    /// Telemetry mono (`T+00042` tick line).
    #[must_use]
    pub fn bevy_fg_data(&self) -> Color {
        self.color32_to_bevy(self.fg_data)
    }

    /// Selected context-tray tab wash.
    #[must_use]
    pub fn bevy_bg_vellum(&self) -> Color {
        let c = self.bg_vellum;
        Color::srgba(
            c.r() as f32 / 255.0,
            c.g() as f32 / 255.0,
            c.b() as f32 / 255.0,
            0.96,
        )
    }

    #[must_use]
    pub fn bevy_accent_gold(&self) -> Color {
        self.color32_to_bevy(self.accent_gold)
    }

    fn color32_to_bevy(&self, c: Color32) -> Color {
        Color::srgba_u8(c.r(), c.g(), c.b(), 255)
    }
}
