use bevy::prelude::*;
use noise::{NoiseFn, Perlin, Simplex, Fbm};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::collections::HashMap;

use crate::terrain::voronoi_enhanced::*;
use crate::terrain::world::{World, GeoRegion};

// World generation parameters structure
#[derive(Resource, Clone)]
pub struct WorldGenParams {
    // General settings
    pub width: u32,
    pub height: u32,
    pub seed: u64,
    
    // Region settings
    pub num_regions: u32,
    pub region_method: RegionMethod,
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RegionMethod {
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
            region_method: RegionMethod::Centroidal,
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

// Component to tag entities as part of the world
#[derive(Component)]
pub struct WorldMarker;

// Component to tag entities as part of a region
#[derive(Component)]
pub struct RegionMarker;

// Component to tag entities as tiles
#[derive(Component)]
pub struct TileMarker;

// Height component
#[derive(Component)]
pub struct Height(pub f32);

// Moisture component (for biome determination)
#[derive(Component)]
pub struct Moisture(pub f32);

// Temperature component (for biome determination)
#[derive(Component)]
pub struct Temperature(pub f32);

// Biome type
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug, Hash)]
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

// Terrain type
#[derive(Component)]
pub struct TerrainType(pub BiomeType);

// System to generate the world
pub fn generate_world(
    mut commands: Commands,
    params: Res<WorldGenParams>,
) {
    info!("Generating world with seed: {}", params.seed);
    
    // Create a stable RNG from the seed
    let mut rng = StdRng::seed_from_u64(params.seed);
    
    // Create noise functions
    let height_noise = create_noise_function(params.noise_scale, params.noise_octaves, params.seed);
    let moisture_noise = create_noise_function(params.noise_scale * 1.5, params.noise_octaves, params.seed + 1);
    let temperature_noise = create_noise_function(params.noise_scale * 0.8, params.noise_octaves, params.seed + 2);
    
    // Generate regions using the specified method
    let regions = generate_regions(&params, &mut rng);
    
    // Create the world entity
    let world_entity = commands.spawn((
        WorldMarker,
        Name::new("World"),
    )).id();
    
    // Create and populate the regions
    let mut geo_regions = Vec::new();
    let mut region_entities = Vec::new();
    
    for (region_index, region_points) in regions.iter().enumerate() {
        // Calculate region center
        let mut center = Vec2::ZERO;
        for point in region_points {
            center += point.position;
        }
        center /= region_points.len() as f32;
        
        // Create a GeoRegion
        let mut geo_region = GeoRegion::new();
        geo_region.center = center;
        
        // Create the region entity
        let region_entity = commands.spawn((
            RegionMarker,
            Transform::from_translation(Vec3::new(center.x, 0.0, center.y)),
            Name::new(format!("Region {}", region_index)),
        )).id();
        
        region_entities.push(region_entity);
        geo_regions.push(geo_region);
        
        // Make the region a child of the world
        commands.entity(world_entity).push_children(&[region_entity]);
    }
    
    // Process all points to generate terrain
    for y in 0..params.height {
        for x in 0..params.width {
            let position = Vec2::new(x as f32, y as f32);
            
            // Determine which region this point belongs to
            let mut closest_region_index = 0;
            let mut min_distance = f32::MAX;
            
            for (region_index, region_points) in regions.iter().enumerate() {
                // Find the closest point in the region
                for point in region_points {
                    let distance = position.distance(point.position);
                    if distance < min_distance {
                        min_distance = distance;
                        closest_region_index = region_index;
                    }
                }
            }
            
            // Calculate terrain properties
            let normalized_x = x as f32 / params.width as f32;
            let normalized_y = y as f32 / params.height as f32;
            
            // Apply island falloff if enabled
            let island_falloff_factor = if params.island_mode {
                // Create falloff from edges (0 at edges, 1 in center)
                let dx = normalized_x * 2.0 - 1.0;
                let dy = normalized_y * 2.0 - 1.0;
                let distance_from_center = (dx * dx + dy * dy).sqrt();
                (1.0 - distance_from_center.powf(params.island_falloff)).max(0.0)
            } else {
                1.0
            };
            
            // Generate height value
            let noise_x = x as f64 * params.noise_scale as f64;
            let noise_y = y as f64 * params.noise_scale as f64;
            let height_value = (height_noise.get([noise_x, noise_y, 0.0]) * 0.5 + 0.5) as f32 * island_falloff_factor;
            
            // Generate moisture and temperature
            let moisture_value = (moisture_noise.get([noise_x * 1.5, noise_y * 1.5, 0.0]) * 0.5 + 0.5) as f32 + params.moisture_bias;
            let temperature_value = (temperature_noise.get([noise_x * 0.8, noise_y * 0.8, 0.0]) * 0.5 + 0.5) as f32 + params.temperature_bias;
            
            // Determine biome based on height, moisture, and temperature
            let biome = determine_biome(height_value, moisture_value, temperature_value);
            
            // Create tile entity
            let tile_entity = commands.spawn((
                TileMarker,
                Transform::from_translation(Vec3::new(x as f32, height_value * 20.0, y as f32)),
                Height(height_value),
                Moisture(moisture_value),
                Temperature(temperature_value),
                TerrainType(biome),
                Name::new(format!("Tile ({}, {})", x, y)),
            )).id();
            
            // Add to region
            commands.entity(region_entities[closest_region_index]).push_children(&[tile_entity]);
            
            // Add to GeoRegion data structure
            geo_regions[closest_region_index].add_tile(position, tile_entity);
        }
    }
    
    // Generate rivers if specified
    if params.river_count > 0 {
        generate_rivers(&mut commands, &params, &geo_regions, &region_entities, &mut rng);
    }
    
    // Generate lakes if specified
    if params.lake_count > 0 {
        generate_lakes(&mut commands, &params, &geo_regions, &region_entities, &mut rng);
    }
    
    info!("World generation completed");
}

// Helper function to create a noise function
fn create_noise_function(scale: f32, octaves: u32, seed: u64) -> Fbm {
    let mut fbm = Fbm::new();
    fbm.seed = seed as u32;
    fbm.octaves = octaves as usize;
    fbm.frequency = scale as f64;
    fbm
}

// Generate regions using the specified method
fn generate_regions(params: &WorldGenParams, rng: &mut StdRng) -> Vec<Vec<VoronoiSite>> {
    match params.region_method {
        RegionMethod::Regular => voronoi_diagram_generation(
            params.num_regions, 
            params.width, 
            params.height
        ),
        RegionMethod::Manhattan => manhattan_voronoi_diagram_generation(
            params.num_regions, 
            params.width, 
            params.height
        ),
        RegionMethod::Centroidal => centroidal_voronoi_diagram_generation(
            params.num_regions, 
            params.width, 
            params.height, 
            params.region_iterations
        ),
        RegionMethod::Weighted => {
            // Generate random weights
            let weights = (0..params.num_regions)
                .map(|_| rng.gen_range(0.0..10.0))
                .collect::<Vec<f32>>();
                
            additively_weighted_voronoi_diagram_generation(
                params.num_regions, 
                params.width, 
                params.height,
                Some(weights)
            )
        },
        RegionMethod::Power => {
            // Generate random weights
            let weights = (0..params.num_regions)
                .map(|_| rng.gen_range(0.0..5.0))
                .collect::<Vec<f32>>();
                
            power_voronoi_diagram_generation(
                params.num_regions, 
                params.width, 
                params.height,
                Some(weights)
            )
        },
        RegionMethod::Circular => circular_voronoi_diagram_generation(
            params.num_regions, 
            params.width, 
            params.height,
            2.0
        ),
    }
}

// Determine biome type based on height, moisture, and temperature
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

// Generate rivers
fn generate_rivers(
    commands: &mut Commands,
    params: &WorldGenParams,
    geo_regions: &[GeoRegion],
    region_entities: &[Entity],
    rng: &mut StdRng,
) {
    // Implementation for river generation
    // This is a simplified approach that traces paths from high to low elevation
    
    for river_index in 0..params.river_count {
        // Choose a random starting point in a high elevation area
        let mut start_x = rng.gen_range(0..params.width);
        let mut start_y = rng.gen_range(0..params.height);
        
        // Find the region for this point
        let mut current_position = Vec2::new(start_x as f32, start_y as f32);
        
        // Create a river entity
        let river_entity = commands.spawn((
            Name::new(format!("River {}", river_index)),
        )).id();
        
        // Generate river path
        let max_steps = 1000; // Prevent infinite loops
        let mut river_tiles = Vec::new();
        
        for _ in 0..max_steps {
            // Add current position to river
            river_tiles.push(current_position);
            
            // Find lowest neighbor
            let neighbors = [
                Vec2::new(current_position.x - 1.0, current_position.y),
                Vec2::new(current_position.x + 1.0, current_position.y),
                Vec2::new(current_position.x, current_position.y - 1.0),
                Vec2::new(current_position.x, current_position.y + 1.0),
                Vec2::new(current_position.x - 1.0, current_position.y - 1.0),
                Vec2::new(current_position.x + 1.0, current_position.y - 1.0),
                Vec2::new(current_position.x - 1.0, current_position.y + 1.0),
                Vec2::new(current_position.x + 1.0, current_position.y + 1.0),
            ];
            
            // We would need to find these tiles and check their heights
            // This is simplified - in a real implementation, you'd look up the actual entities
            
            // For now, we'll just move in a random direction that's within bounds
            let direction = rng.gen_range(0..8);
            let next_position = neighbors[direction];
            
            // Check if in bounds
            if next_position.x < 0.0 || next_position.x >= params.width as f32 || 
               next_position.y < 0.0 || next_position.y >= params.height as f32 {
                break;
            }
            
            // Check if we've reached water
            // We would need to check the biome of the tile at this position
            
            // Move to next position
            current_position = next_position;
        }
        
        // For each river tile, change its biome to water and attach to river entity
        for position in river_tiles {
            // In a real implementation, you would:
            // 1. Find the tile entity at this position
            // 2. Change its TerrainType to water
            // 3. Possibly make it a child of the river entity
        }
    }
}

// Generate lakes
fn generate_lakes(
    commands: &mut Commands,
    params: &WorldGenParams,
    geo_regions: &[GeoRegion],
    region_entities: &[Entity],
    rng: &mut StdRng,
) {
    // Implementation for lake generation
    // This is a simplified placeholder
    
    for lake_index in 0..params.lake_count {
        // Choose a random center point
        let center_x = rng.gen_range(0..params.width);
        let center_y = rng.gen_range(0..params.height);
        
        // Create a lake entity
        let lake_entity = commands.spawn((
            Name::new(format!("Lake {}", lake_index)),
        )).id();
        
        // Determine lake size (radius)
        let radius = rng.gen_range(5..20);
        
        // Fill the circular area with water tiles
        for y in center_y.saturating_sub(radius)..=(center_y + radius).min(params.height - 1) {
            for x in center_x.saturating_sub(radius)..=(center_x + radius).min(params.width - 1) {
                let distance = ((x as i32 - center_x as i32).pow(2) + 
                               (y as i32 - center_y as i32).pow(2)) as f32;
                
                if distance <= (radius * radius) as f32 {
                    // In a real implementation, you would:
                    // 1. Find the tile entity at this position
                    // 2. Change its TerrainType to water
                    // 3. Possibly make it a child of the lake entity
                }
            }
        }
    }
}

// Event to trigger world generation
#[derive(Event)]
pub struct GenerateWorldEvent(pub WorldGenParams);

// Plugin to register world generation systems
pub struct WorldGeneratorPlugin;

impl Plugin for WorldGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenParams>()
           .add_event::<GenerateWorldEvent>()
           .add_systems(Update, handle_generate_world_event);
    }
}

// System to handle world generation events
fn handle_generate_world_event(
    mut events: EventReader<GenerateWorldEvent>,
    mut commands: Commands,
) {
    for event in events.read() {
        // Clear existing world entities
        commands.insert_resource(event.0.clone());
        generate_world(commands, event.0.clone().into());
        break; // Only process one event per frame
    }
}