use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use proc_A_dine01::bevysubengines::WorldGeneratorSubenginePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(WorldGeneratorSubenginePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("World Generator Subengine started");
    info!("Press F8 to open the World Generator UI");
}
