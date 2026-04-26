use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

use crate::terrain::generation::world_generator_enhanced::{
    WorldGenParams, RegionMethod, GenerateWorldEvent
};

pub struct WorldGenUiState {
    pub visible: bool,
    pub preview_mode: PreviewMode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PreviewMode {
    None,
    Height,
    Moisture,
    Temperature,
    Biome,
    Regions,
}

impl Default for WorldGenUiState {
    fn default() -> Self {
        Self {
            visible: false,
            preview_mode: PreviewMode::None,
        }
    }
}

// Toggle event for the world gen UI
#[derive(Event)]
pub struct ToggleWorldGenUiEvent;

// UI System for world generation
pub fn world_gen_ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut world_gen_params: ResMut<WorldGenParams>,
    mut world_gen_ui_state: ResMut<WorldGenUiState>,
    mut generate_event: EventWriter<GenerateWorldEvent>,
) {
    if !world_gen_ui_state.visible {
        return;
    }

    egui::Window::new("World Generator")
        .resizable(true)
        .collapsible(true)
        .show(egui_context.ctx_mut(), |ui| {
            ui.heading("World Generator");
            ui.add_space(10.0);
            
            egui::CollapsingHeader::new("General Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut world_gen_params.width, 128..=2048).text("Width"));
                    ui.add(egui::Slider::new(&mut world_gen_params.height, 128..=2048).text("Height"));
                    
                    ui.horizontal(|ui| {
                        if ui.button("Random Seed").clicked() {
                            world_gen_params.seed = rand::random();
                        }
                        ui.label(format!("Seed: {}", world_gen_params.seed));
                    });
                    
                    ui.add_space(5.0);
                });
            
            egui::CollapsingHeader::new("Region Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut world_gen_params.num_regions, 4..=64).text("Number of Regions"));
                    
                    ui.label("Region Method:");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Regular, "Regular Voronoi");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Manhattan, "Manhattan");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Weighted, "Weighted");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Centroidal, "Centroidal");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Circular, "Circular");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethod::Power, "Power");
                    
                    if world_gen_params.region_method == RegionMethod::Centroidal {
                        ui.add(egui::Slider::new(&mut world_gen_params.region_iterations, 1..=10).text("Relaxation Iterations"));
                    }
                    
                    ui.add_space(5.0);
                });
            
            egui::CollapsingHeader::new("Terrain Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut world_gen_params.noise_scale, 0.01..=0.1).text("Noise Scale"));
                    ui.add(egui::Slider::new(&mut world_gen_params.noise_octaves, 1..=8).text("Noise Octaves"));
                    ui.add(egui::Slider::new(&mut world_gen_params.moisture_bias, -0.5..=0.5).text("Moisture Bias"));
                    ui.add(egui::Slider::new(&mut world_gen_params.temperature_bias, -0.5..=0.5).text("Temperature Bias"));
                    
                    ui.add_space(5.0);
                });
            
            egui::CollapsingHeader::new("Feature Settings")
                .default_open(true)
                .show(ui, |ui| {
                    ui.add(egui::Slider::new(&mut world_gen_params.river_count, 0..=10).text("Rivers"));
                    ui.add(egui::Slider::new(&mut world_gen_params.lake_count, 0..=5).text("Lakes"));
                    ui.add(egui::Slider::new(&mut world_gen_params.mountain_threshold, 0.5..=0.9).text("Mountain Threshold"));
                    ui.checkbox(&mut world_gen_params.island_mode, "Island Mode");
                    
                    if world_gen_params.island_mode {
                        ui.add(egui::Slider::new(&mut world_gen_params.island_falloff, 1.0..=5.0).text("Island Falloff"));
                    }
                    
                    ui.add_space(5.0);
                });
            
            ui.add_space(10.0);
            
            ui.heading("Preview Mode");
            ui.horizontal(|ui| {
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::None, "None");
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Height, "Height");
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Moisture, "Moisture");
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Temperature, "Temperature");
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Biome, "Biome");
                ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Regions, "Regions");
            });
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("Generate World").clicked() {
                    generate_event.send(GenerateWorldEvent(world_gen_params.clone()));
                }
                
                if ui.button("Close").clicked() {
                    world_gen_ui_state.visible = false;
                }
            });
        });
}

// System to toggle the world gen UI
pub fn toggle_world_gen_ui_system(
    mut events: EventReader<ToggleWorldGenUiEvent>,
    mut world_gen_ui_state: ResMut<WorldGenUiState>,
) {
    for _ in events.read() {
        world_gen_ui_state.visible = !world_gen_ui_state.visible;
    }
}

// Plugin to register all world generation UI components
pub struct WorldGenUiPlugin;

impl Plugin for WorldGenUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenUiState>()
           .add_event::<ToggleWorldGenUiEvent>()
           .add_systems(Update, (world_gen_ui_system, toggle_world_gen_ui_system));
    }
}