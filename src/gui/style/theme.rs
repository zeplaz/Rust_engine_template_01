use bevy::prelude::*;

use super::fonts::load_cmd_ui_mono_font;
use super::{HudDensityProfile, UiPalette, UiScaleApplicationGate, UiSpacing};

/// Bevy-only UI theme tokens (egui visuals path retired — PLAN-EGUI-FULL-DEP-DROP-v1).
pub struct UiThemePlugin;

impl Plugin for UiThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiPalette>()
            .init_resource::<UiSpacing>()
            .init_resource::<HudDensityProfile>()
            .init_resource::<UiScaleApplicationGate>()
            .add_systems(Startup, load_cmd_ui_mono_font);
    }
}
