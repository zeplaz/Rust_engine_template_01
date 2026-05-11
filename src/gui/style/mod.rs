//! Shared UI tokens (`UiPalette`, spacing) and egui **Visuals** wiring.
//! Spec: `prompts/guides/ui_design_language_plan_v1.md`.

mod color_guard;
mod palette;
mod theme;

use bevy::prelude::Color;
use bevy::prelude::Resource;

pub use color_guard::forbid_raw_colors;
pub use palette::UiPalette;
pub use theme::UiThemePlugin;

use bevy_egui::egui;

/// Optional CMD-style prefix for tooling section headers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CmdHeadingStyle {
    #[default]
    None,
    /// Leading `> ` (e.g. notez / command cue).
    Gt,
    /// Leading `~ ` (e.g. data / model cue).
    Tilde,
}

/// Section title: strong + optional CMD prefix; uses palette (no embedded hex).
pub fn section_heading(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    style: CmdHeadingStyle,
    title: impl AsRef<str>,
) {
    let title = title.as_ref();
    let prefix = match style {
        CmdHeadingStyle::None => "",
        CmdHeadingStyle::Gt => "> ",
        CmdHeadingStyle::Tilde => "~ ",
    };
    ui.label(
        egui::RichText::new(format!("{prefix}{title}"))
            .strong()
            .color(palette.accent_terminal),
    );
}

/// Monospace muted path (save/load hints, runbook paths).
pub fn path_hint(ui: &mut egui::Ui, palette: &UiPalette, path: impl AsRef<str>) {
    ui.label(
        egui::RichText::new(path.as_ref())
            .monospace()
            .small()
            .color(palette.fg_muted),
    );
}

/// Convert a Bevy [`Color`] (e.g. agent identity) to egui for small swatches — **not** a theme token.
pub fn gameplay_color_swatch_egui(color: Color) -> egui::Color32 {
    let s = color.to_srgba();
    egui::Color32::from_rgba_unmultiplied(
        (s.red * 255.0) as u8,
        (s.green * 255.0) as u8,
        (s.blue * 255.0) as u8,
        (s.alpha * 255.0) as u8,
    )
}

// --- Semantic text (P1+) ---

pub fn warning_text(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    ui.label(
        egui::RichText::new(text.as_ref())
            .color(palette.warn)
            .strong(),
    );
}

pub fn error_text(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    ui.label(
        egui::RichText::new(text.as_ref())
            .color(palette.danger)
            .strong(),
    );
}

pub fn success_text(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    ui.label(
        egui::RichText::new(text.as_ref())
            .color(palette.accent_terminal)
            .strong(),
    );
}

pub fn muted_text(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    ui.label(
        egui::RichText::new(text.as_ref())
            .small()
            .color(palette.fg_muted),
    );
}

/// Primary body copy (normal weight) — e.g. scenario status line, neutral confirmations.
pub fn primary_text(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    ui.label(egui::RichText::new(text.as_ref()).color(palette.fg_primary));
}

/// Tint for egui `painter.image` when the texture should display at full brightness (no modulation).
#[inline]
pub fn neutral_image_tint() -> egui::Color32 {
    egui::Color32::WHITE
}

/// Tonal palette for compact status lines (scenario, diagnostics, logistics).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusTone {
    Success,
    Warning,
    Danger,
    Info,
    Muted,
}

pub fn status_badge(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    tone: StatusTone,
    text: impl AsRef<str>,
) {
    let c = match tone {
        StatusTone::Success => palette.accent_terminal,
        StatusTone::Warning => palette.warn,
        StatusTone::Danger => palette.danger,
        StatusTone::Info => palette.fg_primary,
        StatusTone::Muted => palette.fg_muted,
    };
    ui.label(egui::RichText::new(text.as_ref()).strong().color(c));
}

use crate::scenario::script_host::ScenarioExecutionState;

/// Scenario script host state → consistent colors (orchestration / scripting UI).
pub fn scenario_execution_badge(ui: &mut egui::Ui, palette: &UiPalette, state: ScenarioExecutionState) {
    let (tone, label) = match state {
        ScenarioExecutionState::Idle => (StatusTone::Muted, "Idle"),
        ScenarioExecutionState::Running => (StatusTone::Success, "Running"),
        ScenarioExecutionState::Completed => (StatusTone::Info, "Completed"),
        ScenarioExecutionState::Failed => (StatusTone::Danger, "Failed"),
    };
    status_badge(ui, palette, tone, label);
}

/// Inset panel with token background (diagnostics / objectives / queues).
pub fn framed_group<R>(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(palette.bg_elevated)
        .inner_margin(egui::Margin::same(8))
        .show(ui, add_contents)
        .inner
}

/// Spacing scale for future layout helpers (P1+); reserved on `Resource` later.
#[derive(Debug, Clone, Resource)]
pub struct UiSpacing {
    pub xs: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
}

impl Default for UiSpacing {
    fn default() -> Self {
        Self {
            xs: 4.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
        }
    }
}
