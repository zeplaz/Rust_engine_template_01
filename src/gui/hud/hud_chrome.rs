//! Shared HUD chrome — icon buttons, side-rail frame, compact stat chips.
//!
//! Keeps egui shell panels visually consistent (SAPIP palette, soft corners).

use bevy_egui::egui::{self, CornerRadius, Frame, Margin, Response, RichText, Stroke, Ui, Vec2};

use crate::gui::style::UiPalette;

use super::panel_state::HudPanelState;

/// Soft panel corners (CRT theme, slightly friendlier than zero-radius everywhere).
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

#[must_use]
pub fn hud_panel_frame(palette: &UiPalette) -> Frame {
    Frame::new()
        .fill(palette.bg_elevated)
        .stroke(Stroke::new(1.0, palette.wire_magenta.gamma_multiply(0.85)))
        .corner_radius(CornerRadius::same(HUD_CHROME_RADIUS))
        .inner_margin(Margin::symmetric(8, 10))
}

#[must_use]
pub fn hud_side_rail_frame(palette: &UiPalette) -> Frame {
    Frame::new()
        .fill(palette.bg_elevated.gamma_multiply(0.96))
        .stroke(Stroke::new(
            1.0,
            palette.wire_magenta.gamma_multiply(0.65),
        ))
        .corner_radius(CornerRadius::same(HUD_CHROME_RADIUS))
        .inner_margin(Margin::symmetric(6, 8))
}

/// Square icon control — primary chrome affordance.
pub fn icon_button(
    ui: &mut Ui,
    palette: &UiPalette,
    icon: HudChromeIcon,
    selected: bool,
) -> Response {
    let size = Vec2::splat(ui.spacing().interact_size.y.min(28.0));
    let fill = if selected {
        palette.accent_terminal.gamma_multiply(0.35)
    } else {
        palette.bg_interactive
    };
    let stroke = if selected {
        palette.accent_terminal
    } else {
        palette.wire_magenta.gamma_multiply(0.75)
    };
    let btn = egui::Button::new(
        RichText::new(icon.glyph())
            .size(15.0)
            .monospace()
            .color(if selected {
                palette.accent_terminal
            } else {
                palette.fg_primary
            }),
    )
    .fill(fill)
    .stroke(Stroke::new(1.0, stroke))
    .corner_radius(CornerRadius::same(HUD_CHROME_RADIUS))
    .min_size(size);
    ui.add(btn).on_hover_text(icon.tooltip())
}

/// Header row: title + expand/collapse + pin.
pub fn side_panel_header(
    ui: &mut Ui,
    palette: &UiPalette,
    title: &str,
    panel_state: &mut HudPanelState,
) {
    ui.horizontal(|ui| {
        ui.set_height(28.0);
        let expand = matches!(
            *panel_state,
            HudPanelState::Collapsed | HudPanelState::Peek
        );
        let icon = if expand {
            HudChromeIcon::Expand
        } else {
            HudChromeIcon::Collapse
        };
        if icon_button(ui, palette, icon, false).clicked() {
            *panel_state = match *panel_state {
                HudPanelState::Collapsed => HudPanelState::Expanded,
                HudPanelState::Peek => HudPanelState::Collapsed,
                HudPanelState::Expanded | HudPanelState::Pinned => HudPanelState::Collapsed,
            };
        }
        if panel_state.shows_content() {
            let pinned = panel_state.is_pinned();
            if icon_button(
                ui,
                palette,
                if pinned {
                    HudChromeIcon::PinOn
                } else {
                    HudChromeIcon::PinOff
                },
                pinned,
            )
            .clicked()
            {
                panel_state.toggle_pin();
            }
        }
        ui.add_space(4.0);
        ui.label(
            RichText::new(title)
                .strong()
                .monospace()
                .color(palette.accent_terminal),
        );
    });
}

/// Collapsed rail: vertical icon stack (no text slab).
pub fn draw_collapsed_side_rail(ui: &mut Ui, palette: &UiPalette) {
    ui.vertical_centered(|ui| {
        ui.add_space(6.0);
        for icon in [
            HudChromeIcon::Sim,
            HudChromeIcon::Lod,
            HudChromeIcon::Build,
        ] {
            let _ = icon_button(ui, palette, icon, false);
            ui.add_space(4.0);
        }
        ui.label(
            RichText::new("···")
                .small()
                .weak()
                .color(palette.fg_muted),
        );
    });
}

/// One-line status chip.
pub fn stat_chip(ui: &mut Ui, palette: &UiPalette, icon: HudChromeIcon, text: impl AsRef<str>) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(icon.glyph())
                .small()
                .color(palette.fg_muted),
        );
        ui.label(
            RichText::new(text.as_ref())
                .small()
                .monospace()
                .color(palette.fg_primary),
        );
    });
}

/// Flat v2 tray tab — gold/vellum selected, muted idle (matches Bevy context tray F-08).
pub fn flat_v2_tray_tab(
    ui: &mut Ui,
    palette: &UiPalette,
    label: &str,
    selected: bool,
) -> Response {
    let fill = if selected {
        palette.bg_vellum
    } else {
        palette.bg_interactive.gamma_multiply(0.92)
    };
    let stroke = if selected {
        palette.accent_gold
    } else {
        palette.wire_magenta.gamma_multiply(0.55)
    };
    let text_color = if selected {
        palette.accent_gold
    } else {
        palette.fg_muted
    };
    ui.add(
        egui::Button::new(
            RichText::new(label)
                .small()
                .monospace()
                .color(text_color),
        )
        .fill(fill)
        .stroke(Stroke::new(
            if selected { 2.0 } else { 1.0 },
            stroke,
        ))
        .corner_radius(CornerRadius::same(HUD_CHROME_RADIUS))
        .min_size(Vec2::new(56.0, 24.0)),
    )
}

/// Subtle divider between summary and scroll body.
pub fn section_rule(ui: &mut Ui, palette: &UiPalette) {
    let h = ui.available_width();
    let y = ui.cursor().top() + 4.0;
    ui.painter().hline(
        ui.min_rect().left()..=ui.min_rect().left() + h,
        y,
        Stroke::new(1.0, palette.wire_magenta.gamma_multiply(0.35)),
    );
    ui.add_space(8.0);
}
