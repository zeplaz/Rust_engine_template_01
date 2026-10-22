//! HUD / shell developer overlay — read-only diagnostics.

use bevy::prelude::*;

use crate::gui::style::{
    HudDensityProfile, UI_GLOBAL_SCALE_STEP,
};

#[derive(Resource, Clone, Debug)]
pub struct HudDevOverlayState {
    pub visible: bool,
    pub show_ui_timing: bool,
    pub show_viewport_mismatch: bool,
    pub show_texture_refresh: bool,
    pub show_map_transform: bool,
}

impl Default for HudDevOverlayState {
    fn default() -> Self {
        Self {
            visible: false,
            show_ui_timing: true,
            show_viewport_mismatch: true,
            show_texture_refresh: true,
            show_map_transform: true,
        }
    }
}

pub struct HudDevOverlayPlugin;

impl Plugin for HudDevOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudDevOverlayState>()
            .add_systems(Update, (hud_dev_overlay_keyboard_toggle, hud_ui_scale_keyboard));
    }
}

fn hud_dev_overlay_keyboard_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<HudDevOverlayState>,
) {
    if keys.just_pressed(KeyCode::F4) {
        state.visible = !state.visible;
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

