//! Shared **Fira Mono** for Bevy UI + matching egui face (`ui_design_language_plan_v1.md`).

use bevy::prelude::*;

/// Bytes for [`install_egui_cmd_mono_font`]; same file as Bevy loads from `assets/fonts/`.
const CMD_UI_MONO_TTF: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/FiraMono-Medium.ttf"
));

/// Handle to `assets/fonts/FiraMono-Medium.ttf` for `TextFont` on HUD / menus.
#[derive(Resource, Clone, Debug)]
pub struct CmdUiMonoFont(pub Handle<Font>);

pub fn load_cmd_ui_mono_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CmdUiMonoFont(
        asset_server.load::<Font>("fonts/FiraMono-Medium.ttf"),
    ));
}

/// Install as first choice for proportional + monospace egui families (CRT / CMD tooling).
pub fn install_egui_cmd_mono_font(ctx: &bevy_egui::egui::Context) {
    use bevy_egui::egui::{FontData, FontDefinitions, FontFamily};

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "cmd_mono".to_owned(),
        FontData::from_static(CMD_UI_MONO_TTF).into(),
    );
    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        if let Some(v) = fonts.families.get_mut(&family) {
            v.insert(0, "cmd_mono".to_owned());
        }
    }
    ctx.set_fonts(fonts);
}
