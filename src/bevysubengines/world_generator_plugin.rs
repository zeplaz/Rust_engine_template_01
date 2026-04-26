use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::terrain::voronoi_enhanced::*;
use crate::idgen::EntityId;

// Separate subengine plugin for world generation
pub struct WorldGeneratorSubenginePlugin;

impl Plugin for WorldGeneratorSubenginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenParams>()
           .init_resource::<WorldGenState>()
           .add_event::<GenerateWorldEvent>()
           .add_event::<SaveWorldEvent>()
           .add_event::<LoadWorldEvent>()
           .add_systems(Update, (
               world_gen_ui_system, 
               handle_generate_world_event,
               handle_save_world_event,
               handle_load_world_event,
           ));
    }
}

// World generation parameters
#[derive(Resource, Clone, Serialize, Deserialize)]
pub struct WorldGenParams {
    // General settings
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    
    // Region settings
    pub num_regions: u32,
    pub region_method: RegionMethodType,
    pub region_iterations: u32,  // For centroidal relaxation
    
    // Terrain settings
    pub noise_scale: f32,
    pub noise_octaves: u32,
    pub moisture_bias: f32,
    pub temperature_bias: f32,
    
    // Feature settings
    pub river_count: u32,
    pub lake_count: u32,
    pub mountain_threshold: f32,
    pub island_mode: bool,
    pub island_falloff: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegionMethodType {
    Regular,
    Manhattan,
    Weighted,
    Centroidal,
    Circular,
    Power,
}

impl Default for WorldGenParams {
    fn default() -> Self {
        Self {
            width: 512,
            height: 512,
            seed: rand::random(),
            num_regions: 24,
            region_method: RegionMethodType::Centroidal,
            region_iterations: 3,
            noise_scale: 0.03,
            noise_octaves: 6,
            moisture_bias: 0.0,
            temperature_bias: 0.0,
            river_count: 3,
            lake_count: 2,
            mountain_threshold: 0.7,
            island_mode: true,
            island_falloff: 3.0,
        }
    }
}

// State of the world generator
#[derive(Resource, Default)]
pub struct WorldGenState {
    pub ui_visible: bool,
    pub preview_mode: PreviewMode,
    pub world_data: Option<WorldData>,
    pub last_save_path: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PreviewMode {
    #[default]
    None,
    Height,
    Moisture,
    Temperature,
    Biome,
    Regions,
}

// Simplified world data for export/import
#[derive(Clone, Serialize, Deserialize)]
pub struct WorldData {
    pub params: WorldGenParams,
    pub height_map: Vec<f32>,
    pub moisture_map: Vec<f32>,
    pub temperature_map: Vec<f32>,
    pub region_map: Vec<u32>,
    pub biome_map: Vec<BiomeType>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum BiomeType {
    DeepWater,
    ShallowWater,
    Beach,
    Desert,
    Grassland,
    Forest,
    DenseForest,
    Mountain,
    SnowCappedMountain,
    Tundra,
    Swamp,
}

// Events
#[derive(Event)]
pub struct GenerateWorldEvent;

#[derive(Event)]
pub struct SaveWorldEvent {
    pub path: Option<String>,
}

#[derive(Event)]
pub struct LoadWorldEvent {
    pub path: String,
}

// Main UI system for the world generator
fn world_gen_ui_system(
    mut egui_context: ResMut<EguiContext>,
    mut world_gen_params: ResMut<WorldGenParams>,
    mut world_gen_state: ResMut<WorldGenState>,
    mut generate_event: EventWriter<GenerateWorldEvent>,
    mut save_event: EventWriter<SaveWorldEvent>,
    mut load_event: EventWriter<LoadWorldEvent>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // Toggle UI with F8
    if keyboard_input.just_pressed(KeyCode::F8) {
        world_gen_state.ui_visible = !world_gen_state.ui_visible;
    }

    if !world_gen_state.ui_visible {
        return;
    }

    egui::Window::new("World Generator Subengine")
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
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethodType::Regular, "Regular Voronoi");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethodType::Manhattan, "Manhattan");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethodType::Weighted, "Weighted");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethodType::Centroidal, "Centroidal");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethodType::Circular, "Circular");
                    ui.radio_value(&mut world_gen_params.region_method, RegionMethodType::Power, "Power");
                    
                    if world_gen_params.region_method == RegionMethodType::Centroidal {
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
                ui.radio_value(&mut world_gen_state.preview_mode, PreviewMode::None, "None");
                ui.radio_value(&mut world_gen_state.preview_mode, PreviewMode::Height, "Height");
                ui.radio_value(&mut world_gen_state.preview_mode, PreviewMode::Moisture, "Moisture");
                ui.radio_value(&mut world_gen_state.preview_mode, PreviewMode::Temperature, "Temperature");
                ui.radio_value(&mut world_gen_state.preview_mode, PreviewMode::Biome, "Biome");
                ui.radio_value(&mut world_gen_state.preview_mode, PreviewMode::Regions, "Regions");
            });
            
            ui.add_space(20.0);
            
            ui.horizontal(|ui| {
                if ui.button("Generate World").clicked() {
                    generate_event.send(GenerateWorldEvent);
                }
                
                if ui.button("Save World Data").clicked() {
                    save_event.send(SaveWorldEvent { path: None });
                }
                
                if ui.button("Load World Data").clicked() {
                    // In a real app, you'd use a file dialog here
                    if let Some(last_path) = &world_gen_state.last_save_path {
                        load_event.send(LoadWorldEvent { path: last_path.clone() });
                    }
                }
            });
            
            // Display world data info if available
            if let Some(world_data) = &world_gen_state.world_data {
                ui.add_space(10.0);
                ui.separator();
                ui.label(format!("World Size: {}x{}", world_data.params.width, world_data.params.height));
                ui.label(format!("Regions: {}", world_data.params.num_regions));
                if let Some(path) = &world_gen_state.last_save_path {
                    ui.label(format!("Last Save: {}", path));
                }
            }
        });
}

// Handle generate world event
fn handle_generate_world_event(
    mut events: EventReader<GenerateWorldEvent>,
    world_gen_params: Res<WorldGenParams>,
    mut world_gen_state: ResMut<WorldGenState>,
) {
    for _ in events.read() {
        info!("Generating world with seed: {}", world_gen_params.seed);
        
        // Generate world data
        let world_data = generate_world_data(world_gen_params.clone());
        
        // Store in state
        world_gen_state.world_data = Some(world_data);
        
        info!("World generation completed");
    }
}

// Handle save world event
fn handle_save_world_event(
    mut events: EventReader<SaveWorldEvent>,
    world_gen_state: Res<WorldGenState>,
    mut world_state: ResMut<WorldGenState>,
) {
    for event in events.read() {
        if let Some(world_data) = &world_gen_state.world_data {
            let path = if let Some(path) = &event.path {
                path.clone()
            } else {
                // Default path
                format!("world_{}x{}_seed_{}.json", 
                    world_data.params.width, 
                    world_data.params.height,
                    world_data.params.seed)
            };
            
            save_world_data(world_data, &path);
            world_state.last_save_path = Some(path.clone());
            info!("World data saved to: {}", path);
        } else {
            warn!("No world data to save");
        }
    }
}

// Handle load world event
fn handle_load_world_event(
    mut events: EventReader<LoadWorldEvent>,
    mut world_gen_params: ResMut<WorldGenParams>,
    mut world_gen_state: ResMut<WorldGenState>,
) {
    for event in events.read() {
        match load_world_data(&event.path) {
            Ok(world_data) => {
                // Update parameters from loaded data
                *world_gen_params = world_data.params.clone();
                world_gen_state.world_data = Some(world_data);
                world_gen_state.last_save_path = Some(event.path.clone());
                info!("World data loaded from: {}", event.path);
            }
            Err(err) => {
                error!("Failed to load world data: {}", err);
            }
        }
    }
}

// Generate world data from parameters
fn generate_world_data(params: WorldGenParams) -> WorldData {
    let width = params.width;
    let height = params.height;
    let total_size = (width * height) as usize;
    
    // Initialize maps
    let mut height_map = vec![0.0; total_size];
    let mut moisture_map = vec![0.0; total_size];
    let mut temperature_map = vec![0.0; total_size];
    let mut region_map = vec![0; total_size];
    let mut biome_map = vec![BiomeType::Grassland; total_size];
    
    // Generate regions
    let regions = match params.region_method {
        RegionMethodType::Regular => voronoi_diagram_generation(
            params.num_regions, width, height
        ),
        RegionMethodType::Manhattan => manhattan_voronoi_diagram_generation(
            params.num_regions, width, height
        ),
        RegionMethodType::Centroidal => centroidal_voronoi_diagram_generation(
            params.num_regions, width, height, params.region_iterations
        ),
        RegionMethodType::Weighted => {
            // Generate random weights
            let mut rng = rand::thread_rng();
            let weights = (0..params.num_regions)
                .map(|_| rng.gen_range(0.0..10.0))
                .collect::<Vec<f32>>();
                
            additively_weighted_voronoi_diagram_generation(
                params.num_regions, width, height,
                Some(weights)
            )
        },
        RegionMethodType::Power => {
            // Generate random weights
            let mut rng = rand::thread_rng();
            let weights = (0..params.num_regions)
                .map(|_| rng.gen_range(0.0..5.0))
                .collect::<Vec<f32>>();
                
            power_voronoi_diagram_generation(
                params.num_regions, width, height,
                Some(weights)
            )
        },
        RegionMethodType::Circular => circular_voronoi_diagram_generation(
            params.num_regions, width, height, 2.0
        ),
    };
    
    // Create noise functions
    let noise = noise::Fbm::new();
    let moisture_noise = noise::Fbm::new();
    let temperature_noise = noise::Fbm::new();
    
    // Fill maps
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            
            // Height
            let nx = x as f64 * params.noise_scale as f64;
            let ny = y as f64 * params.noise_scale as f64;
            let mut height_val = (noise.get([nx, ny, 0.0]) * 0.5 + 0.5) as f32;
            
            // Island falloff
            if params.island_mode {
                let normalized_x = x as f32 / width as f32;
                let normalized_y = y as f32 / height as f32;
                let dx = normalized_x * 2.0 - 1.0;
                let dy = normalized_y * 2.0 - 1.0;
                let distance_from_center = (dx * dx + dy * dy).sqrt();
                let falloff = (1.0 - distance_from_center.powf(params.island_falloff)).max(0.0);
                height_val *= falloff;
            }
            
            height_map[idx] = height_val;
            
            // Moisture
            let moisture_val = (moisture_noise.get([nx * 1.5, ny * 1.5, 0.0]) * 0.5 + 0.5) as f32 + params.moisture_bias;
            moisture_map[idx] = moisture_val;
            
            // Temperature
            let temp_val = (temperature_noise.get([nx * 0.8, ny * 0.8, 0.0]) * 0.5 + 0.5) as f32 + params.temperature_bias;
            temperature_map[idx] = temp_val;
            
            // Determine region
            let position = bevy::math::Vec2::new(x as f32, y as f32);
            
            // Find which region this point belongs to
            let mut closest_region_idx = 0;
            let mut min_distance = f32::MAX;
            
            for (region_idx, region_points) in regions.iter().enumerate() {
                for point in region_points {
                    let distance = position.distance(point.position);
                    if distance < min_distance {
                        min_distance = distance;
                        closest_region_idx = region_idx;
                    }
                }
            }
            
            region_map[idx] = closest_region_idx as u32;
            
            // Determine biome based on height, moisture, and temperature
            biome_map[idx] = determine_biome(height_val, moisture_val, temp_val);
        }
    }
    
    // Create rivers (simplified)
    for _ in 0..params.river_count {
        // In a real implementation, you would create realistic river paths here
    }
    
    // Create lakes (simplified)
    for _ in 0..params.lake_count {
        // In a real implementation, you would create realistic lakes here
    }
    
    WorldData {
        params,
        height_map,
        moisture_map,
        temperature_map,
        region_map,
        biome_map,
    }
}

// Determine biome based on height, moisture, and temperature
fn determine_biome(height: f32, moisture: f32, temperature: f32) -> BiomeType {
    // Deep water
    if height < 0.2 {
        return BiomeType::DeepWater;
    }
    
    // Shallow water
    if height < 0.35 {
        return BiomeType::ShallowWater;
    }
    
    // Beach
    if height < 0.38 {
        return BiomeType::Beach;
    }
    
    // Mountain and snow
    if height > 0.75 {
        if temperature < 0.2 {
            return BiomeType::SnowCappedMountain;
        } else {
            return BiomeType::Mountain;
        }
    }
    
    // Handle other biomes based on temperature and moisture
    if temperature < 0.3 {
        // Cold biomes
        return BiomeType::Tundra;
    } else if temperature > 0.7 {
        // Hot biomes
        if moisture < 0.3 {
            return BiomeType::Desert;
        } else if moisture > 0.8 {
            return BiomeType::Swamp;
        } else {
            return BiomeType::Grassland;
        }
    } else {
        // Moderate temperature biomes
        if moisture < 0.4 {
            return BiomeType::Grassland;
        } else if moisture < 0.7 {
            return BiomeType::Forest;
        } else {
            return BiomeType::DenseForest;
        }
    }
}

// Save world data to file
fn save_world_data(world_data: &WorldData, path: &str) {
    let json = serde_json::to_string_pretty(world_data).expect("Failed to serialize world data");
    let mut file = File::create(path).expect("Failed to create file");
    file.write_all(json.as_bytes()).expect("Failed to write file");
}

// Load world data from file
fn load_world_data(path: &str) -> Result<WorldData, String> {
    let file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
    let world_data: WorldData = serde_json::from_reader(file)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    Ok(world_data)
}

// Export world data to binary format for game engine consumption
pub fn export_world_data_binary(world_data: &WorldData, path: &str) -> Result<(), String> {
    // This is where you would implement the binary export format
    // For now, we'll just write a placeholder
    let mut file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
    
    // Write header info
    let header = format!(
        "WORLD_DATA_V1\n{}\n{}\n{}\n", 
        world_data.params.width,
        world_data.params.height,
        world_data.params.seed
    );
    file.write_all(header.as_bytes()).map_err(|e| format!("Failed to write header: {}", e))?;
    
    // Write height map
    for height in &world_data.height_map {
        let bytes = height.to_le_bytes();
        file.write_all(&bytes).map_err(|e| format!("Failed to write height map: {}", e))?;
    }
    
    // Write other maps similarly...
    
    Ok(())
}

// Convert world data to a format compatible with the main game engine
pub fn convert_to_game_engine_format(world_data: &WorldData) -> GameWorldData {
    // Here we would convert the world data to the format expected by the main game engine
    GameWorldData {
        width: world_data.params.width,
        height: world_data.params.height,
        tile_data: world_data.height_map.iter().zip(&world_data.biome_map)
            .enumerate()
            .map(|(idx, (&height, &biome))| {
                GameTileData {
                    position: Vec2::new(
                        (idx % world_data.params.width as usize) as f32,
                        (idx / world_data.params.width as usize) as f32
                    ),
                    height,
                    biome: convert_biome_to_game_biome(biome),
                    region_id: world_data.region_map[idx],
                }
            })
            .collect(),
    }
}

// Game engine formats
pub struct GameWorldData {
    pub width: u32,
    pub height: u32,
    pub tile_data: Vec<GameTileData>,
}

pub struct GameTileData {
    pub position: Vec2,
    pub height: f32,
    pub biome: GameBiomeType,
    pub region_id: u32,
}

pub enum GameBiomeType {
    Water,
    Beach,
    Plains,
    Forest,
    Mountain,
    Snow,
    Desert,
    Swamp,
}

fn convert_biome_to_game_biome(biome: BiomeType) -> GameBiomeType {
    match biome {
        BiomeType::DeepWater | BiomeType::ShallowWater => GameBiomeType::Water,
        BiomeType::Beach => GameBiomeType::Beach,
        BiomeType::Desert => GameBiomeType::Desert,
        BiomeType::Grassland => GameBiomeType::Plains,
        BiomeType::Forest | BiomeType::DenseForest => GameBiomeType::Forest,
        BiomeType::Mountain => GameBiomeType::Mountain,
        BiomeType::SnowCappedMountain | BiomeType::Tundra => GameBiomeType::Snow,
        BiomeType::Swamp => GameBiomeType::Swamp,
    }
}