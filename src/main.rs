use bevy::prelude::*;
use proc_A_dine01::engine::EnginePlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .add_plugins(EnginePlugin)
        .run();
}