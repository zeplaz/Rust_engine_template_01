use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use crate::terrain::{
    additively_weighted_voronoi_diagram_generation,
    centroidal_voronoi_diagram_generation,
    circular_voronoi_diagram_generation,
    manhattan_voronoi_diagram_generation,
    power_voronoi_diagram_generation,
    voronoi_diagram_generation,
};
use crate::terrain::biome::{classify_biome, BiomeBucket, BiomeWeights, BiomeTuning, TerrainClass, TerrainSurfaceMix};
use crate::terrain::ecology::estimate_ecological_suitability;
use crate::terrain::generation::terrain_noise::{
    build_fbm_perlin, build_height_noise, sample_height_field, NoiseSamplingTuning, TerrainNoiseProfile,
};
use noise::NoiseFn;

// Separate subengine plugin for world generation
pub struct WorldGeneratorSubenginePlugin;

impl Plugin for WorldGeneratorSubenginePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenParams>()
           .init_resource::<WorldGenState>()
           .add_message::<GenerateWorldEvent>()
           .add_message::<SaveWorldEvent>()
           .add_message::<LoadWorldEvent>()
           // Non-egui systems stay in Update
           .add_systems(Update, (
               handle_generate_world_event,
               handle_save_world_event,
               handle_load_world_event,
           ))
           // Egui rendering systems go in EguiPrimaryContextPass (bevy_egui 0.39)
           .add_systems(EguiPrimaryContextPass, world_gen_ui_system);
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

    // Terrain / noise
    pub noise_scale: f32,
    pub noise_octaves: u32,
    pub noise_lacunarity: f32,
    pub noise_persistence: f32,
    pub height_noise_profile: TerrainNoiseProfile,
    pub height_curve_exponent: f32,
    pub domain_warp_strength: f32,
    pub terrain_detail_mix: f32,
    pub moisture_bias: f32,
    pub temperature_bias: f32,
    #[serde(default)]
    pub noise_sampling: NoiseSamplingTuning,
    #[serde(default)]
    pub biome_tuning: BiomeTuning,

    // Feature settings
    pub river_count: u32,
    pub lake_count: u32,
    /// Legacy compatibility field; still serialized for old save files.
    pub river_threshold: f32,
    /// Legacy compatibility field; still serialized for old save files.
    pub lake_threshold: f32,
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
            noise_lacunarity: 2.0,
            noise_persistence: 0.5,
            height_noise_profile: TerrainNoiseProfile::default(),
            height_curve_exponent: 1.0,
            domain_warp_strength: 0.0,
            terrain_detail_mix: 0.0,
            moisture_bias: 0.0,
            temperature_bias: 0.0,
            noise_sampling: NoiseSamplingTuning::default(),
            biome_tuning: BiomeTuning::default(),
            river_count: 3,
            lake_count: 2,
            river_threshold: 0.45,
            lake_threshold: 0.55,
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
    /// Canonical ecology blend per tile for simulation and serialization.
    pub biome_weights_map: Vec<BiomeWeights>,
    /// Canonical terrain substrate blend per tile for simulation effects.
    pub terrain_mix_map: Vec<TerrainSurfaceMix>,
    /// Dominant terrain label derived from biome/height fields.
    pub biome_map: Vec<TerrainClass>,
    /// Derived ecological utility fields for gameplay/economy simulation hooks.
    pub flora_density_map: Vec<f32>,
    pub crop_yield_map: Vec<f32>,
    pub flower_density_map: Vec<f32>,
}
/// Legacy alias kept to preserve old references; use `TerrainClass` in new code.
#[deprecated(note = "Use terrain::biome::TerrainClass")]
pub type BiomeType = TerrainClass;

// Events
#[derive(Message)]
pub struct GenerateWorldEvent;

#[derive(Message)]
pub struct SaveWorldEvent {
    pub path: Option<String>,
}

#[derive(Message)]
pub struct LoadWorldEvent {
    pub path: String,
}

// Main UI system — runs in EguiPrimaryContextPass, returns Result (bevy_egui 0.39).
fn world_gen_ui_system(
    mut contexts: EguiContexts,
    mut world_gen_params: ResMut<WorldGenParams>,
    mut world_gen_state: ResMut<WorldGenState>,
    mut generate_event: MessageWriter<GenerateWorldEvent>,
    mut save_event: MessageWriter<SaveWorldEvent>,
    mut load_event: MessageWriter<LoadWorldEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    bindings: Option<Res<crate::gui::InputBindings>>,
) -> Result {
    let toggle_key = bindings
        .as_deref()
        .map(|b| b.toggle_world_generator)
        .unwrap_or_else(|| crate::gui::InputBindings::default().toggle_world_generator);
    if keyboard_input.just_pressed(toggle_key) {
        world_gen_state.ui_visible = !world_gen_state.ui_visible;
    }

    if !world_gen_state.ui_visible {
        return Ok(());
    }

    egui::Window::new("World Generator Subengine")
        .resizable(true)
        .collapsible(true)
        .show(contexts.ctx_mut()?, |ui| {
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
                    ui.label("Height noise profile:");
                    ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::FbmPerlin, "fBm · Perlin (baseline)");
                    ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::RidgedMulti, "Ridged (ranges)");
                    ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::Billow, "Billow (soft masses)");
                    ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::HybridMulti, "Hybrid multifractal");
                    ui.radio_value(&mut world_gen_params.height_noise_profile, TerrainNoiseProfile::FbmOpenSimplex, "fBm · OpenSimplex");

                    ui.add(egui::Slider::new(&mut world_gen_params.noise_scale, 0.01..=0.1).text("Noise Scale"));
                    ui.add(egui::Slider::new(&mut world_gen_params.noise_octaves, 1..=8).text("Noise Octaves"));
                    ui.add(egui::Slider::new(&mut world_gen_params.noise_lacunarity, 1.2..=3.5).text("Lacunarity"));
                    ui.add(egui::Slider::new(&mut world_gen_params.noise_persistence, 0.2..=0.85).text("Persistence"));

                    egui::CollapsingHeader::new("Post shaping (after base fractal)")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.add(egui::Slider::new(&mut world_gen_params.height_curve_exponent, 0.35..=2.5).text("Height curve (1=linear)"));
                            ui.add(egui::Slider::new(&mut world_gen_params.domain_warp_strength, 0.0..=1.5).text("Domain warp"));
                            ui.add(egui::Slider::new(&mut world_gen_params.terrain_detail_mix, 0.0..=1.0).text("Detail mix"));
                        });

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
                    generate_event.write(GenerateWorldEvent);
                }

                if ui.button("Save World Data").clicked() {
                    save_event.write(SaveWorldEvent { path: None });
                }

                if ui.button("Load World Data").clicked() {
                    // In a real app, you'd use a file dialog here
                    if let Some(last_path) = &world_gen_state.last_save_path {
                        load_event.write(LoadWorldEvent { path: last_path.clone() });
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
    Ok(())
}

// Handle generate world event
fn handle_generate_world_event(
    mut events: MessageReader<GenerateWorldEvent>,
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
    mut events: MessageReader<SaveWorldEvent>,
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
    mut events: MessageReader<LoadWorldEvent>,
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
    let mut biome_weights_map = vec![BiomeWeights::default(); total_size];
    let mut terrain_mix_map = vec![TerrainSurfaceMix::default(); total_size];
    let mut biome_map = vec![TerrainClass::Grassland; total_size];
    let mut flora_density_map = vec![0.0; total_size];
    let mut crop_yield_map = vec![0.0; total_size];
    let mut flower_density_map = vec![0.0; total_size];

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

    let lac = params.noise_lacunarity as f64;
    let per = params.noise_persistence as f64;

    let ns = &params.noise_sampling;

    let height_noise = build_height_noise(
        params.height_noise_profile,
        params.noise_scale,
        params.noise_octaves,
        params.seed,
        lac,
        per,
    );
    let warp_noise = build_fbm_perlin(
        params.noise_scale * ns.warp_noise_scale_mul,
        ns.warp_noise_octaves,
        params.seed.wrapping_add(ns.warp_seed_offset),
        lac,
        per,
    );
    let detail_noise = build_fbm_perlin(
        params.noise_scale * ns.detail_noise_scale_mul,
        ns.detail_noise_octaves,
        params.seed.wrapping_add(ns.detail_seed_offset),
        lac,
        per * ns.detail_persistence_mul,
    );
    let moisture_noise = build_fbm_perlin(
        params.noise_scale * ns.moisture_noise_scale_mul,
        params.noise_octaves,
        params.seed.wrapping_add(1),
        lac,
        per,
    );
    let temperature_noise = build_fbm_perlin(
        params.noise_scale * ns.temperature_noise_scale_mul,
        params.noise_octaves,
        params.seed.wrapping_add(2),
        lac,
        per,
    );

    // Fill maps
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;

            // Height
            let nx = x as f64 * params.noise_scale as f64;
            let ny = y as f64 * params.noise_scale as f64;

            let mut height_val = sample_height_field(
                &height_noise,
                &warp_noise,
                &detail_noise,
                nx,
                ny,
                params.height_curve_exponent,
                params.domain_warp_strength,
                params.terrain_detail_mix,
                ns,
            );

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
            let moisture_val = (moisture_noise.get([
                nx * ns.moisture_sample_freq_mul,
                ny * ns.moisture_sample_freq_mul,
                0.0,
            ]) * 0.5 + 0.5) as f32 + params.moisture_bias;
            moisture_map[idx] = moisture_val;

            // Temperature
            let temp_val = (temperature_noise.get([
                nx * ns.temperature_sample_freq_mul,
                ny * ns.temperature_sample_freq_mul,
                0.0,
            ]) * 0.5 + 0.5) as f32 + params.temperature_bias;
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

            // Canonical data boundary: deterministic, serializable classification.
            let classification =
                classify_biome(height_val, moisture_val, temp_val, &params.biome_tuning);
            biome_map[idx] = classification.terrain_class;
            biome_weights_map[idx] = classification.biome_weights;
            let terrain_mix = TerrainSurfaceMix::default();
            terrain_mix_map[idx] = terrain_mix;
            let suitability = estimate_ecological_suitability(
                classification.biome_weights,
                terrain_mix,
                moisture_val,
                temp_val,
            );
            flora_density_map[idx] = suitability.flora_density;
            crop_yield_map[idx] = suitability.crop_yield_factor;
            flower_density_map[idx] = suitability.flower_density;
        }
    }

    // Rivers/lakes are not produced on this legacy JSON export path. Designer-facing hydrology
    // lives in `terrain::generation::passes::p4_hydrology` and ECS `world_generator_enhanced`
    // (`compute_hydrology_world`) per `prompts/guides/world_assets_tools_rulebook_v1.md` §1.

    WorldData {
        params,
        height_map,
        moisture_map,
        temperature_map,
        region_map,
        biome_weights_map,
        terrain_mix_map,
        biome_map,
        flora_density_map,
        crop_yield_map,
        flower_density_map,
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
    /// Derived render/gameplay bucket view, not canonical storage.
    pub biome: BiomeBucket,
    pub region_id: u32,
}

fn convert_biome_to_game_biome(biome: TerrainClass) -> BiomeBucket {
    biome.into()
}
