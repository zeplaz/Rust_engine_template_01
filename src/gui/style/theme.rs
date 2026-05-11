use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPreUpdateSet};

use super::fonts::{install_egui_cmd_mono_font, load_cmd_ui_mono_font};
use super::UiPalette;

/// Ensures egui monospace / proportional default to CMD mono once.
#[derive(Resource, Default)]
struct EguiCmdMonoFontLoaded(bool);

/// Applies [`UiPalette`] + **Fira Mono** to the primary egui context.
pub struct UiThemePlugin;

impl Plugin for UiThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiPalette>()
            .init_resource::<super::UiSpacing>()
            .init_resource::<EguiCmdMonoFontLoaded>()
            .add_systems(Startup, load_cmd_ui_mono_font)
            .add_systems(
                PreUpdate,
                apply_egui_theme_system.after(EguiPreUpdateSet::BeginPass),
            );
    }
}

fn apply_egui_theme_system(
    palette: Res<UiPalette>,
    mut contexts: EguiContexts,
    mut loaded: ResMut<EguiCmdMonoFontLoaded>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    if !loaded.0 {
        install_egui_cmd_mono_font(ctx);
        loaded.0 = true;
    }
    ctx.set_visuals(palette.to_egui_visuals());
}
