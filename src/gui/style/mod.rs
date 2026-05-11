//! Shared UI tokens (`UiPalette`, spacing) and egui **Visuals** wiring.
//! Spec: `prompts/guides/ui_design_language_plan_v1.md`.

mod color_guard;
mod fonts;
mod palette;
mod theme;

use bevy::prelude::Color;
use bevy::prelude::Resource;

pub use color_guard::forbid_raw_colors;
pub use fonts::CmdUiMonoFont;
pub use palette::UiPalette;
pub use theme::UiThemePlugin;

use bevy_egui::egui;

/// Spacing scale for egui layout — tokenized vertical rhythm ([`v_space`], [`VertSpace`]).
#[derive(Debug, Clone, Resource)]
pub struct UiSpacing {
    pub xs: f32,
    /// Between [`VertSpace::Xs`] and [`VertSpace::Sm`] (tight block rhythm).
    pub inter: f32,
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    pub xl: f32,
}

impl Default for UiSpacing {
    fn default() -> Self {
        Self {
            xs: 4.0,
            inter: 6.0,
            sm: 8.0,
            md: 12.0,
            lg: 16.0,
            xl: 24.0,
        }
    }
}

/// Vertical gap tokens — use with [`v_space`] and [`UiSpacing`] from `Resources`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertSpace {
    Xs,
    Inter,
    Sm,
    Md,
    Lg,
    Xl,
}

impl VertSpace {
    #[inline]
    fn px(self, s: &UiSpacing) -> f32 {
        match self {
            VertSpace::Xs => s.xs,
            VertSpace::Inter => s.inter,
            VertSpace::Sm => s.sm,
            VertSpace::Md => s.md,
            VertSpace::Lg => s.lg,
            VertSpace::Xl => s.xl,
        }
    }
}

#[inline]
pub fn v_space(ui: &mut egui::Ui, spacing: &UiSpacing, step: VertSpace) {
    ui.add_space(step.px(spacing));
}

/// Primary body line (mono, theme primary) — returns [`egui::Response`] for `.on_hover_text`, etc.
#[inline]
pub fn primary_label(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) -> egui::Response {
    ui.label(
        egui::RichText::new(text.as_ref())
            .monospace()
            .color(palette.fg_primary),
    )
}

/// Small muted caption (mono) — for `.on_hover_text` chains.
#[inline]
pub fn muted_label(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) -> egui::Response {
    ui.label(
        egui::RichText::new(text.as_ref())
            .small()
            .monospace()
            .color(palette.fg_muted),
    )
}

/// Weak secondary line (mono, muted).
pub fn weak_body(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    ui.label(
        egui::RichText::new(text.as_ref())
            .weak()
            .monospace()
            .color(palette.fg_muted),
    );
}

/// Emphasized line (mono, primary).
pub fn strong_body(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    ui.label(
        egui::RichText::new(text.as_ref())
            .strong()
            .monospace()
            .color(palette.fg_primary),
    );
}

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
            .monospace()
            .color(palette.fg_primary),
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
            .monospace()
            .color(palette.warn)
            .strong(),
    );
}

pub fn error_text(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    ui.label(
        egui::RichText::new(text.as_ref())
            .monospace()
            .color(palette.danger)
            .strong(),
    );
}

pub fn success_text(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    ui.label(
        egui::RichText::new(text.as_ref())
            .monospace()
            .color(palette.accent_terminal)
            .strong(),
    );
}

pub fn muted_text(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    let _ = muted_label(ui, palette, text);
}

/// Primary body copy (normal weight) — e.g. scenario status line, neutral confirmations.
pub fn primary_text(ui: &mut egui::Ui, palette: &UiPalette, text: impl AsRef<str>) {
    let _ = primary_label(ui, palette, text);
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
    ui.label(
        egui::RichText::new(text.as_ref())
            .strong()
            .monospace()
            .color(c),
    );
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
        .stroke(egui::Stroke::new(1.0, palette.wire_magenta))
        .corner_radius(egui::CornerRadius::ZERO)
        .inner_margin(egui::Margin::same(8))
        .show(ui, add_contents)
        .inner
}
