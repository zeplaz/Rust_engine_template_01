use bevy::prelude::*;
use proc_A_dine01::engine::EnginePlugin;

// Simple UI state resource
#[derive(Resource, Default)]
struct UiState {
    show_world_gen: bool,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.0, 0.0, 0.0)))
        .init_resource::<UiState>()
        .add_plugins(EnginePlugin)
        .run();
}