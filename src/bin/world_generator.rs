//! Desktop world-gen entry: **same tooling stack as `EnginePlugin`** (F8 UI + preview + material registries).
//! Legacy `WorldGeneratorSubenginePlugin` remains in `bevysubengines` for save-format experiments; do not
//! reintroduce it here — see `prompts/guides/world_assets_tools_rulebook_v1.md`.

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use proc_A_dine01::systems::terrain::MaterialUnificationPlugin;
use proc_A_dine01::terrain::generation::WorldGenToolsPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EguiPlugin::default())
        .add_plugins(MaterialUnificationPlugin)
        .add_plugins(WorldGenToolsPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    info!("World Generator (canonical stack). Press F8 for World Generator + preview (registry-backed biome / tag overlay).");
}
