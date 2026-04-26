use bevy::prelude::*;
use proc_a_dine01::engine::EnginePlugin;

// Simple UI state resource
#[derive(Resource, Default)]
struct UiState {
    show_world_gen: bool,
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Msaa::Sample4)
        .init_resource::<UiState>()
        .add_plugin(EnginePlugin)
        .run();
}