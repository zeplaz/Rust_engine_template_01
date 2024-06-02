use crate::engine::EnginePlugin;
use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Msaa::Sample4)
        .init_resource::<UiState>()
        .add_plugin(EnginePlugin)
        .run();
}
