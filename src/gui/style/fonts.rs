//! Shared **Fira Mono** for Bevy UI.

use bevy::prelude::*;

/// Handle to `assets/fonts/FiraMono-Medium.ttf` for `TextFont` on HUD / menus.
#[derive(Resource, Clone, Debug)]
pub struct CmdUiMonoFont(pub Handle<Font>);

pub fn load_cmd_ui_mono_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(CmdUiMonoFont(
        asset_server.load::<Font>("fonts/FiraMono-Medium.ttf"),
    ));
}
