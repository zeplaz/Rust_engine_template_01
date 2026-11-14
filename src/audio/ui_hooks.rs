//! HUD / panel button presses → [`AudioOneShotQueue`] (presentation only).

use bevy::prelude::*;
use bevy::ui_widgets::Button;

use super::mix_frame::{AudioOneShotKind, AudioOneShotQueue};

/// Any Bevy UI `Button` press enqueues a UI click (play UI only — no sim mut).
pub fn enqueue_ui_button_clicks(
    mut audio: ResMut<AudioOneShotQueue>,
    q: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
) {
    for interaction in &q {
        if *interaction == Interaction::Pressed {
            audio.push_ui(AudioOneShotKind::UiClick);
        }
    }
}
