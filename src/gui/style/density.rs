//! Global HUD density profile — compact RTS / tooling chrome.

use bevy::prelude::*;
use bevy_egui::egui;

/// Default egui widget scale multiplier (applied via [`egui::Context::set_pixels_per_point`]).
pub const DEFAULT_UI_GLOBAL_SCALE: f32 = 0.86;
const UI_GLOBAL_SCALE_MIN: f32 = 0.65;
const UI_GLOBAL_SCALE_MAX: f32 = 1.25;
pub const UI_GLOBAL_SCALE_STEP: f32 = 0.05;

/// Tracks how often egui UI scale is written each frame (debug guard).
#[derive(Resource, Clone, Debug, Default)]
pub struct UiScaleApplicationGate {
    pub applications_this_frame: u8,
    pub last_applied_scale: f32,
}

#[derive(Resource, Clone, Debug)]
pub struct HudDensityProfile {
    pub global_scale: f32,
    pub window_padding: f32,
    pub item_spacing: f32,
    pub icon_size: f32,
    pub compact_mode: bool,
}

impl Default for HudDensityProfile {
    fn default() -> Self {
        Self {
            global_scale: DEFAULT_UI_GLOBAL_SCALE,
            window_padding: 6.0,
            item_spacing: 4.0,
            icon_size: 16.0,
            compact_mode: true,
        }
    }
}

impl HudDensityProfile {
    pub fn clamped_global_scale(&self) -> f32 {
        self.global_scale.clamp(UI_GLOBAL_SCALE_MIN, UI_GLOBAL_SCALE_MAX)
    }

    pub fn adjust_global_scale(&mut self, delta: f32) {
        self.global_scale = (self.global_scale + delta).clamp(UI_GLOBAL_SCALE_MIN, UI_GLOBAL_SCALE_MAX);
    }
}

/// OS/viewport DPI only — never the already density-adjusted `pixels_per_point`.
#[must_use]
pub fn native_ui_pixels_per_point(ctx: &egui::Context) -> f32 {
    ctx.native_pixels_per_point().unwrap_or(1.0).max(0.01)
}

#[must_use]
pub fn resolved_hud_pixels_per_point(
    native_pixels_per_point: f32,
    profile: &HudDensityProfile,
) -> f32 {
    native_pixels_per_point.max(0.01) * profile.clamped_global_scale()
}

#[must_use]
pub fn resolve_ui_scale(ctx: &egui::Context, density: &HudDensityProfile) -> f32 {
    resolved_hud_pixels_per_point(native_ui_pixels_per_point(ctx), density)
}

/// Single writer for egui widget scale — absolute multiplier, not compounded each frame.
pub fn sync_egui_context_scale_factor(
    profile: &HudDensityProfile,
    ctx: &egui::Context,
    gate: &mut UiScaleApplicationGate,
) {
    let factor = profile.clamped_global_scale();
    gate.applications_this_frame = gate.applications_this_frame.saturating_add(1);
    if gate.applications_this_frame > 1 {
        warn!(
            "UI scale applied {} times this frame (previous {:.3}, now {:.3})",
            gate.applications_this_frame,
            gate.last_applied_scale,
            factor,
        );
    }
    let native = native_ui_pixels_per_point(ctx);
    let pixels_per_point = resolved_hud_pixels_per_point(native, profile);
    if (ctx.pixels_per_point() - pixels_per_point).abs() > 1e-4 {
        ctx.set_pixels_per_point(pixels_per_point);
    }
    gate.last_applied_scale = factor;
}

pub fn apply_density_to_egui_style(style: &mut egui::Style, profile: &HudDensityProfile) {
    style.spacing.item_spacing = egui::vec2(profile.item_spacing, profile.item_spacing);
    style.spacing.window_margin = egui::Margin::same(profile.window_padding.round() as i8);
    style.spacing.button_padding = egui::vec2(profile.item_spacing + 1.0, profile.item_spacing);
    style.spacing.indent = profile.item_spacing * 2.0;
    style.spacing.interact_size = egui::vec2(profile.icon_size * 2.0, profile.icon_size + 2.0);
    style.spacing.scroll.bar_width = (profile.item_spacing + 4.0).max(6.0);
    style.spacing.scroll.bar_inner_margin = 2.0;
    style.spacing.scroll.bar_outer_margin = 2.0;
    style.spacing.menu_margin = egui::Margin::same(profile.window_padding.round() as i8);
    if profile.compact_mode {
        style.text_styles.insert(egui::TextStyle::Body, egui::FontId::proportional(13.0));
        style.text_styles.insert(egui::TextStyle::Button, egui::FontId::proportional(13.0));
        style.text_styles.insert(egui::TextStyle::Small, egui::FontId::proportional(11.0));
        style.text_styles.insert(egui::TextStyle::Heading, egui::FontId::proportional(15.0));
    }
}

#[must_use]
pub fn spacing_for_density(profile: &HudDensityProfile) -> egui::Vec2 {
    egui::vec2(profile.item_spacing, profile.item_spacing)
}

pub fn apply_hud_density_profile(ctx: &egui::Context, profile: &HudDensityProfile) {
    ctx.global_style_mut(|style| apply_density_to_egui_style(style, profile));
}

pub fn reset_ui_scale_application_gate(gate: &mut UiScaleApplicationGate) {
    gate.applications_this_frame = 0;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolved_hud_scale_is_absolute_from_native_dpi() {
        let profile = HudDensityProfile::default();
        let scale = resolved_hud_pixels_per_point(1.0, &profile);
        assert!((scale - DEFAULT_UI_GLOBAL_SCALE).abs() < 1e-6);
        assert!(
            (resolved_hud_pixels_per_point(scale, &profile) - scale * DEFAULT_UI_GLOBAL_SCALE).abs()
                < 1e-6,
            "must not treat prior pixels_per_point as native DPI"
        );
    }

    #[test]
    fn resolved_hud_scale_is_stable_for_constant_native_dpi() {
        let profile = HudDensityProfile::default();
        let first = resolved_hud_pixels_per_point(1.5, &profile);
        let second = resolved_hud_pixels_per_point(1.5, &profile);
        assert_eq!(first, second);
    }
}
