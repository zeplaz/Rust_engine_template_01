use bevy::prelude::*;
use bevy_egui::{update_ui_size_and_scale_system, EguiContexts, EguiContextSettings, EguiPreUpdateSet};

use super::fonts::{install_egui_cmd_mono_font, load_cmd_ui_mono_font};
use super::{
    apply_hud_density_profile, apply_scroll_style, sync_egui_context_scale_factor,
    HudDensityProfile, UiPalette,
};
use super::{reset_ui_scale_application_gate, UiScaleApplicationGate};

/// Ensures egui monospace / proportional default to CMD mono once.
#[derive(Resource, Default)]
struct EguiCmdMonoFontLoaded(bool);

/// Applies [`UiPalette`] + **Fira Mono** to the primary egui context.
pub struct UiThemePlugin;

impl Plugin for UiThemePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiPalette>()
            .init_resource::<super::UiSpacing>()
            .init_resource::<HudDensityProfile>()
            .init_resource::<UiScaleApplicationGate>()
            .init_resource::<EguiCmdMonoFontLoaded>()
            .add_systems(Startup, load_cmd_ui_mono_font)
            .add_systems(
                PreUpdate,
                (
                    reset_ui_scale_application_gate_system,
                    sync_egui_density_scale_system.before(update_ui_size_and_scale_system),
                )
                    .chain(),
            )
            .add_systems(
                PreUpdate,
                apply_egui_theme_system.after(EguiPreUpdateSet::BeginPass),
            );
    }
}

fn reset_ui_scale_application_gate_system(mut gate: ResMut<UiScaleApplicationGate>) {
    reset_ui_scale_application_gate(&mut gate);
}

fn sync_egui_density_scale_system(
    density: Res<HudDensityProfile>,
    mut settings: Query<&mut EguiContextSettings>,
    mut gate: ResMut<UiScaleApplicationGate>,
) {
    for mut egui_settings in &mut settings {
        sync_egui_context_scale_factor(&density, &mut egui_settings, &mut gate);
    }
}

fn apply_egui_theme_system(
    palette: Res<UiPalette>,
    density: Res<HudDensityProfile>,
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
    apply_hud_density_profile(ctx, &density);
    ctx.style_mut(|style| apply_scroll_style(style, &palette));
}
