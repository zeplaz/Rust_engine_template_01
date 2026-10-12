//! Global HUD density profile — compact RTS / tooling chrome.

use bevy::prelude::*;

pub const DEFAULT_UI_GLOBAL_SCALE: f32 = 0.86;
const UI_GLOBAL_SCALE_MIN: f32 = 0.65;
const UI_GLOBAL_SCALE_MAX: f32 = 1.25;
pub const UI_GLOBAL_SCALE_STEP: f32 = 0.05;

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

#[must_use]
pub fn native_ui_pixels_per_point() -> f32 {
    1.0
}

#[must_use]
pub fn resolved_hud_pixels_per_point(
    native_pixels_per_point: f32,
    profile: &HudDensityProfile,
) -> f32 {
    native_pixels_per_point.max(0.01) * profile.clamped_global_scale()
}

#[must_use]
pub fn resolve_ui_scale(density: &HudDensityProfile) -> f32 {
    resolved_hud_pixels_per_point(native_ui_pixels_per_point(), density)
}

/// Identity — Bevy logical coords are the UI space after egui retirement.
#[must_use]
pub fn bevy_logical_to_egui_pos(logical: Vec2) -> Vec2 {
    logical
}

#[must_use]
pub fn bevy_logical_vec_to_egui(logical: Vec2) -> Vec2 {
    logical
}

pub fn apply_hud_density_profile(_profile: &HudDensityProfile) {}

pub fn spacing_for_density(profile: &HudDensityProfile) -> (f32, f32) {
    (profile.window_padding, profile.item_spacing)
}

pub fn reset_ui_scale_application_gate(gate: &mut UiScaleApplicationGate) {
    gate.applications_this_frame = 0;
}

/// Retired egui style applicator — no-op after dep drop.
pub fn apply_density_to_egui_style(_profile: &HudDensityProfile) {}

/// Retired egui scale sync — no-op after dep drop.
pub fn sync_egui_context_scale_factor(
    profile: &HudDensityProfile,
    gate: &mut UiScaleApplicationGate,
) {
    gate.applications_this_frame = gate.applications_this_frame.saturating_add(1);
    gate.last_applied_scale = profile.clamped_global_scale();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bevy_logical_to_egui_is_identity() {
        let p = Vec2::new(12.0, 34.0);
        assert_eq!(bevy_logical_to_egui_pos(p), p);
    }
}
