use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use proc_a_dine01::bevysubengines::WorldGeneratorSubenginePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(WorldGeneratorSubenginePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Add a camera
    commands.spawn(Camera2dBundle::default());
    
    info!("World Generator Subengine started");
    info!("Press F8 to open the World Generator UI");
}