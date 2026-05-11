use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPreUpdateSet};

use super::UiPalette;

/// Applies [`UiPalette`] to the primary egui context each frame (cheap; enables future hot-reload).
pub struct UiThemePlugin;

impl Plugin for UiThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiPalette>()
            .init_resource::<super::UiSpacing>()
            .add_systems(
                PreUpdate,
                apply_egui_theme_system.after(EguiPreUpdateSet::BeginPass),
            );
    }
}

fn apply_egui_theme_system(palette: Res<UiPalette>, mut contexts: EguiContexts) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ctx.set_visuals(palette.to_egui_visuals());
}
