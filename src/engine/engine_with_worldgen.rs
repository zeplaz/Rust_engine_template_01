use crate::engine::states::EnigneState;
use crate::terrain::generation::WorldGenToolsPlugin;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;

pub struct EnginePlugin;

impl Plugin for EnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins)
           .add_plugin(EguiPlugin)
           .add_plugin(WorldGenToolsPlugin);
        
        info!("Engine initialized with world generation tools. Press F8 to open the World Generator.");
    }
}