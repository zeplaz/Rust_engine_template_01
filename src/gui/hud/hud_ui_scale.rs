//! Player UI scale hotkeys — `Ctrl` + `+` / `-` adjust [`HudDensityProfile`] global scale.

use bevy::prelude::*;

use crate::gui::style::{HudDensityProfile, UI_GLOBAL_SCALE_STEP};

pub struct HudUiScalePlugin;

impl Plugin for HudUiScalePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, hud_ui_scale_keyboard);
    }
}

fn hud_ui_scale_keyboard(keys: Res<ButtonInput<KeyCode>>, mut density: ResMut<HudDensityProfile>) {
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    if !ctrl {
        return;
    }
    if keys.just_pressed(KeyCode::Equal) || keys.just_pressed(KeyCode::NumpadAdd) {
        density.adjust_global_scale(UI_GLOBAL_SCALE_STEP);
    }
    if keys.just_pressed(KeyCode::Minus) || keys.just_pressed(KeyCode::NumpadSubtract) {
        density.adjust_global_scale(-UI_GLOBAL_SCALE_STEP);
    }
}
