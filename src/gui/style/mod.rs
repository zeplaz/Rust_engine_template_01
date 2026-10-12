//! Shared UI tokens (`UiPalette`, spacing) — Bevy-only after egui dep drop.
//! Spec: `prompts/guides/ui_design_language_plan_v1.md`.

mod density;
mod color_guard;
mod fonts;
mod palette;
mod scroll;
mod theme;

use bevy::prelude::Color;
use bevy::prelude::Resource;

pub use density::{
    apply_density_to_egui_style, apply_hud_density_profile, bevy_logical_to_egui_pos,
    bevy_logical_vec_to_egui, native_ui_pixels_per_point, resolve_ui_scale,
    resolved_hud_pixels_per_point, reset_ui_scale_application_gate, spacing_for_density,
    sync_egui_context_scale_factor, HudDensityProfile, UiScaleApplicationGate,
    DEFAULT_UI_GLOBAL_SCALE, UI_GLOBAL_SCALE_STEP,
};
pub use color_guard::forbid_raw_colors;
pub use fonts::CmdUiMonoFont;
pub use palette::UiPalette;
pub use scroll::{
    apply_scroll_style, widget_scroll_both, widget_scroll_vertical, widget_scroll_vertical_capped,
    widget_scroll_vertical_fill,
};
pub use theme::UiThemePlugin;

use crate::gui::ui_types::UiRgba;

/// **PLAN-UI-THEME-MERGE-001** — default tokens wired (egui visuals retired).
#[must_use]
pub fn ui_theme_merge_contract_ok() -> bool {
    let palette = UiPalette::default();
    palette.fg_primary != palette.bg_app && palette.bevy_hud_panel_fill() != Color::NONE
}

#[must_use]
pub fn ui_w3_theme_001_green() -> bool {
    ui_theme_merge_contract_ok()
}

#[derive(Debug, Clone, Resource)]
pub struct UiSpacing {
    pub xs: f32,
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
    pub fn px(self, s: &UiSpacing) -> f32 {
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

/// Retired egui spacer — no-op.
#[inline]
pub fn v_space(_spacing: &UiSpacing, _step: VertSpace) {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CmdHeadingStyle {
    #[default]
    None,
    Gt,
    Tilde,
}

#[must_use]
pub fn gameplay_color_swatch_egui(color: Color) -> UiRgba {
    let s = color.to_srgba();
    UiRgba::from_rgba_unmultiplied(
        (s.red * 255.0) as u8,
        (s.green * 255.0) as u8,
        (s.blue * 255.0) as u8,
        (s.alpha * 255.0) as u8,
    )
}

#[inline]
pub fn neutral_image_tint() -> UiRgba {
    UiRgba::WHITE
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeverityTone {
    Routine,
    Advisory,
    Urgent,
    Emergency,
}

#[inline]
pub fn severity_tone_color(palette: &UiPalette, tone: SeverityTone) -> UiRgba {
    match tone {
        SeverityTone::Routine => palette.fg_muted,
        SeverityTone::Advisory => palette.fg_primary,
        SeverityTone::Urgent => palette.warn,
        SeverityTone::Emergency => palette.danger,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusTone {
    Success,
    Warning,
    Danger,
    Info,
    Muted,
}

// Retired egui text helpers — kept as no-ops so call sites in stubbed panels compile out.
pub fn primary_label(_palette: &UiPalette, _text: impl AsRef<str>) {}
pub fn muted_label(_palette: &UiPalette, _text: impl AsRef<str>) {}
pub fn weak_body(_palette: &UiPalette, _text: impl AsRef<str>) {}
pub fn strong_body(_palette: &UiPalette, _text: impl AsRef<str>) {}
pub fn section_heading(_palette: &UiPalette, _style: CmdHeadingStyle, _title: impl AsRef<str>) {}
pub fn path_hint(_palette: &UiPalette, _path: impl AsRef<str>) {}
pub fn warning_text(_palette: &UiPalette, _text: impl AsRef<str>) {}
pub fn error_text(_palette: &UiPalette, _text: impl AsRef<str>) {}
pub fn success_text(_palette: &UiPalette, _text: impl AsRef<str>) {}
pub fn muted_text(_palette: &UiPalette, _text: impl AsRef<str>) {}
pub fn primary_text(_palette: &UiPalette, _text: impl AsRef<str>) {}
pub fn status_badge(_palette: &UiPalette, _tone: StatusTone, _text: impl AsRef<str>) {}
pub fn framed_group<R>(_palette: &UiPalette, add_contents: impl FnOnce() -> R) -> R {
    add_contents()
}

use crate::scenario::script_host::ScenarioExecutionState;

pub fn scenario_execution_badge(_palette: &UiPalette, _state: ScenarioExecutionState) {}
