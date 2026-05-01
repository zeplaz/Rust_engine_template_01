use std::collections::HashMap;

use bevy::prelude::*;
use noise::{Fbm, NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

use super::terrain_noise::{
    build_fbm_perlin, build_height_noise, sample_height_field, HeightNoise, NoiseSamplingTuning,
};
pub use super::terrain_noise::TerrainNoiseProfile;

use super::tuning_io;
use crate::terrain::voronoi_enhanced::*;
use crate::terrain::world::GeoRegion;
use crate::terrain::biome::{classify_biome, BiomeTuning, TerrainClass};
use crate::terrain::generation::hydrology::{compute_hydrology_world, HydrologyParams, HydrologyResult};

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
    
    // Terrain / noise
    pub noise_scale: f32,
    pub noise_octaves: u32,
    /// Lacunarity between octaves (frequency multiplier per octave).
    pub noise_lacunarity: f32,
    /// Persistence between octaves (amplitude falloff per octave).
    pub noise_persistence: f32,
    /// Which fractal family drives **height** (moisture/temperature use fBm·Perlin).
    pub height_noise_profile: TerrainNoiseProfile,
    /// >1 flattens lowlands; <1 lifts lows — artistic land/ocean contrast after base noise.
    pub height_curve_exponent: f32,
    /// Domain warp: displaces sample coordinates before height noise (0 = off). Natural coasts / folds.
    pub domain_warp_strength: f32,
    /// Blend in high-frequency fBm for small-scale breakup (0–1).
    pub terrain_detail_mix: f32,
    pub moisture_bias: f32,
    pub temperature_bias: f32,

    /// Channel frequencies / warp / detail — when loading chunk fields with [`crate::terrain::generation::passes::p1_fields::fill_fields`]
    /// and `tuning_overlay` is `None`, this is the tuning used (same path as [`crate::terrain::generation::world_generator_enhanced::generate_world`]).
    /// Optionally overridden by JSON via `tuning_io` merge on startup.
    pub noise_sampling: NoiseSamplingTuning,
    /// Biome weights + class thresholds — must stay aligned with `classify_biome`.
    pub biome_tuning: BiomeTuning,
    
    // Feature settings
    pub river_count: u32,
    pub lake_count: u32,
    pub mountain_threshold: f32,
    pub island_mode: bool,
    pub island_falloff: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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
            mountain_threshold: 0.7,
            island_mode: true,
            island_falloff: 3.0,
        }
    }
}

/// Noise kernels for height / warp / detail / moisture / temperature — build once per full-world or chunk fill.
pub struct WorldNoiseKernels {
    pub height: HeightNoise,
    pub warp: Fbm<Perlin>,
    pub detail: Fbm<Perlin>,
    pub moisture: Fbm<Perlin>,
    pub temperature: Fbm<Perlin>,
}

/// Same kernel construction as legacy `generate_world` (frequencies derived from `tuning`, usually `params.noise_sampling`).
pub fn build_world_noise_kernels(params: &WorldGenParams, tuning: &NoiseSamplingTuning) -> WorldNoiseKernels {
    let lac = params.noise_lacunarity as f64;
    let per = params.noise_persistence as f64;
    let height_noise = build_height_noise(
        params.height_noise_profile,
        params.noise_scale,
        params.noise_octaves,
        params.seed,
        lac,
        per,
    );
    let warp_noise = build_fbm_perlin(
        params.noise_scale * tuning.warp_noise_scale_mul,
        tuning.warp_noise_octaves,
        params.seed.wrapping_add(tuning.warp_seed_offset),
        lac,
        per,
    );
    let detail_noise = build_fbm_perlin(
        params.noise_scale * tuning.detail_noise_scale_mul,
        tuning.detail_noise_octaves,
        params.seed.wrapping_add(tuning.detail_seed_offset),
        lac,
        per * tuning.detail_persistence_mul,
    );
    let moisture_noise = build_fbm_perlin(
        params.noise_scale * tuning.moisture_noise_scale_mul,
        params.noise_octaves,
        params.seed.wrapping_add(1),
        lac,
        per,
    );
    let temperature_noise = build_fbm_perlin(
        params.noise_scale * tuning.temperature_noise_scale_mul,
        params.noise_octaves,
        params.seed.wrapping_add(2),
        lac,
        per,
    );
    WorldNoiseKernels {
        height: height_noise,
        warp: warp_noise,
        detail: detail_noise,
        moisture: moisture_noise,
        temperature: temperature_noise,
    }
}

/// Island edge falloff in world tile space — normalized by `params.width` / `height` like legacy tile spawn.
#[inline]
pub fn island_falloff_at_world_tile(wx: i32, wy: i32, params: &WorldGenParams) -> f32 {
    let w = params.width.max(1) as f32;
    let h = params.height.max(1) as f32;
    let normalized_x = wx as f32 / w;
    let normalized_y = wy as f32 / h;
    if params.island_mode {
        let dx = normalized_x * 2.0 - 1.0;
        let dy = normalized_y * 2.0 - 1.0;
        let distance_from_center = (dx * dx + dy * dy).sqrt();
        (1.0 - distance_from_center.powf(params.island_falloff)).max(0.0)
    } else {
        1.0
    }
}

/// Elevation (with island falloff), moisture, temperature for one world tile — shared by `generate_world` and pass 1.
pub fn sample_fields_at_world_tile(
    wx: i32,
    wy: i32,
    params: &WorldGenParams,
    kernels: &WorldNoiseKernels,
    tuning: &NoiseSamplingTuning,
) -> (f32, f32, f32) {
    let noise_x = wx as f64 * params.noise_scale as f64;
    let noise_y = wy as f64 * params.noise_scale as f64;
    let height_norm = sample_height_field(
        &kernels.height,
        &kernels.warp,
        &kernels.detail,
        noise_x,
        noise_y,
        params.height_curve_exponent,
        params.domain_warp_strength,
        params.terrain_detail_mix,
        tuning,
    );
    let height_value = height_norm * island_falloff_at_world_tile(wx, wy, params);
    let moisture_value = (kernels.moisture.get([
        noise_x * tuning.moisture_sample_freq_mul,
        noise_y * tuning.moisture_sample_freq_mul,
        0.0,
    ]) * 0.5
        + 0.5) as f32
        + params.moisture_bias;
    let temperature_value = (kernels.temperature.get([
        noise_x * tuning.temperature_sample_freq_mul,
        noise_y * tuning.temperature_sample_freq_mul,
        0.0,
    ]) * 0.5
        + 0.5) as f32
        + params.temperature_bias;
    (height_value, moisture_value, temperature_value)
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

/// Marker for hydrology feature entities spawned during enhanced world gen.
#[derive(Component)]
pub struct RiverMarker;

/// Marker for hydrology feature entities spawned during enhanced world gen.
#[derive(Component)]
pub struct LakeMarker;

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
/// Legacy alias kept for transition; canonical enum now lives in `terrain::biome`.
#[deprecated(note = "Use terrain::biome::TerrainClass")]
pub type BiomeType = TerrainClass;

// Terrain type
#[derive(Component)]
pub struct TerrainType(pub TerrainClass);

// System to generate the world
pub fn generate_world(
    mut commands: Commands,
    params: Res<WorldGenParams>,
) {
    info!("Generating world with seed: {}", params.seed);
    
    // Create a stable RNG from the seed
    let mut rng = StdRng::seed_from_u64(params.seed);
    
    let ns = &params.noise_sampling;
    let kernels = build_world_noise_kernels(&params, ns);

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
        commands.entity(world_entity).add_children(&[region_entity]);
    }

    let w = params.width as usize;
    let grid_len = w.saturating_mul(params.height as usize).max(1);
    let mut height_grid: Vec<f32> = vec![0.0; grid_len];
    let mut tile_lookup: HashMap<(u32, u32), Entity> =
        HashMap::with_capacity(grid_len.max(256));

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
            
            let (height_value, moisture_value, temperature_value) = sample_fields_at_world_tile(
                x as i32,
                y as i32,
                &params,
                &kernels,
                ns,
            );
            height_grid[y as usize * w + x as usize] = height_value;

            // Determine biome based on height, moisture, and temperature
            let biome = determine_biome(
                height_value,
                moisture_value,
                temperature_value,
                &params.biome_tuning,
            );
            
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
            commands.entity(region_entities[closest_region_index]).add_children(&[tile_entity]);
            
            // Add to GeoRegion data structure
            geo_regions[closest_region_index].add_tile(position, tile_entity);

            tile_lookup.insert((x, y), tile_entity);
        }
    }

    if params.river_count > 0 || params.lake_count > 0 {
        let hydro_params = HydrologyParams::from_biome_tuning(&params.biome_tuning);
        let hydro = compute_hydrology_world(
            params.width,
            params.height,
            &height_grid,
            &hydro_params,
            params.river_count,
            params.lake_count,
        );
        if params.river_count > 0 {
            spawn_hydrology_rivers(&mut commands, &params, &tile_lookup, &hydro);
        }
        if params.lake_count > 0 {
            spawn_hydrology_lakes(
                &mut commands,
                &params,
                &tile_lookup,
                &geo_regions,
                &region_entities,
                &hydro,
            );
        }
    }
    
    info!("World generation completed");
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
fn determine_biome(
    height: f32,
    moisture: f32,
    temperature: f32,
    tuning: &BiomeTuning,
) -> TerrainClass {
    classify_biome(height, moisture, temperature, tuning).terrain_class
}

fn region_entity_for_tile(
    geo_regions: &[GeoRegion],
    region_entities: &[Entity],
    x: u32,
    y: u32,
) -> Option<Entity> {
    let key = (x as usize, y as usize);
    let idx = geo_regions.iter().position(|r| r.tiles.contains_key(&key))?;
    Some(region_entities[idx])
}

fn apply_shallow_water_visual(
    commands: &mut Commands,
    tile: Entity,
    gx: f32,
    gy: f32,
    water_height: f32,
) {
    commands.entity(tile).insert((
        TerrainType(TerrainClass::ShallowWater),
        Height(water_height),
        Moisture(0.95),
        Transform::from_xyz(gx, water_height * 20.0, gy),
    ));
}

fn spawn_hydrology_rivers(
    commands: &mut Commands,
    params: &WorldGenParams,
    tile_lookup: &HashMap<(u32, u32), Entity>,
    hydro: &HydrologyResult,
) {
    let water_line = params.biome_tuning.shallow_water_height_max;
    let river_depth = (water_line * 0.92).clamp(0.02, 0.98);
    for (river_index, path) in hydro.rivers.iter().enumerate() {
        let river_entity = commands
            .spawn((RiverMarker, Name::new(format!("River {}", river_index))))
            .id();
        for &(tx, ty) in path {
            let Some(&tile_e) = tile_lookup.get(&(tx, ty)) else {
                continue;
            };
            apply_shallow_water_visual(
                commands,
                tile_e,
                tx as f32,
                ty as f32,
                river_depth,
            );
            commands.entity(tile_e).insert(ChildOf(river_entity));
        }
    }
}

fn spawn_hydrology_lakes(
    commands: &mut Commands,
    params: &WorldGenParams,
    tile_lookup: &HashMap<(u32, u32), Entity>,
    geo_regions: &[GeoRegion],
    region_entities: &[Entity],
    hydro: &HydrologyResult,
) {
    let tuning = &params.biome_tuning;
    let lake_depth = ((tuning.deep_water_height_max + tuning.shallow_water_height_max) * 0.5)
        .clamp(0.02, 0.98);

    for (lake_index, region) in hydro.lakes.iter().enumerate() {
        if region.cells.is_empty() {
            continue;
        }
        let (cx, cy) = region.cells[region.cells.len() / 2];
        let Some(region_parent) = region_entity_for_tile(geo_regions, region_entities, cx, cy)
        else {
            continue;
        };
        let lake_entity = commands
            .spawn((
                LakeMarker,
                Name::new(format!("Lake {}", lake_index)),
                ChildOf(region_parent),
            ))
            .id();
        for &(lx, ly) in &region.cells {
            let Some(&tile_e) = tile_lookup.get(&(lx, ly)) else {
                continue;
            };
            apply_shallow_water_visual(
                commands,
                tile_e,
                lx as f32,
                ly as f32,
                lake_depth,
            );
            commands.entity(tile_e).insert(ChildOf(lake_entity));
        }
    }
}

// Message to trigger world generation (buffered; processed before `generate_world` in chain).
#[derive(Message)]
pub struct GenerateWorldEvent(pub WorldGenParams);

/// Default path for optional JSON overlay (`noise_sampling` + `biome_tuning`).
pub const WORLD_GEN_TUNING_JSON_PATH: &str = "assets/config/world_gen_tuning.json";

// Plugin to register world generation systems
pub struct WorldGeneratorPlugin;

impl Plugin for WorldGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldGenParams>()
           .add_message::<GenerateWorldEvent>()
           .add_systems(Startup, merge_world_gen_tuning_from_json)
           .add_systems(
               Update,
               (apply_generate_world_request, generate_world).chain(),
           );
    }
}

fn merge_world_gen_tuning_from_json(mut params: ResMut<WorldGenParams>) {
    if let Ok(Some(overlay)) = tuning_io::load_overlay(WORLD_GEN_TUNING_JSON_PATH) {
        if let Some(n) = overlay.noise_sampling {
            params.noise_sampling = n;
        }
        if let Some(b) = overlay.biome_tuning {
            params.biome_tuning = b;
        }
    }
}

fn apply_generate_world_request(
    mut events: MessageReader<GenerateWorldEvent>,
    mut params: ResMut<WorldGenParams>,
) {
    for GenerateWorldEvent(new_params) in events.read() {
        *params = new_params.clone();
    }
}

#[cfg(test)]
mod hydrology_spawning_tests {
    use crate::terrain::generation::hydrology::{
        compute_hydrology_rect, compute_hydrology_world, HydrologyParams,
    };

    #[test]
    fn generate_rivers_uses_compute_hydrology() {
        let w = 16u32;
        let h = 16u32;
        let mut dem = vec![0.45f32; (w * h) as usize];
        for x in 0..w {
            dem[x as usize] = 0.95;
        }
        let p = HydrologyParams::default();
        let r = compute_hydrology_world(w, h, &dem, &p, 2, 1);
        assert!(r.rivers.len() <= 2);
        assert!(r.lakes.len() <= 1);
    }

    #[test]
    fn enhanced_generator_consumes_p4_hydrology_tags() {
        let w = 12u32;
        let h = 12u32;
        let mut dem = Vec::new();
        for y in 0..h {
            for x in 0..w {
                let fx = x as f32 / (w.saturating_sub(1).max(1) as f32);
                let fy = y as f32 / (h.saturating_sub(1).max(1) as f32);
                dem.push(fx * 0.6 + fy * 0.4);
            }
        }
        let p = HydrologyParams::default();
        let rect = compute_hydrology_rect(w, h, &dem, &p, 3);
        let world = compute_hydrology_world(w, h, &dem, &p, 3, 0);
        assert_eq!(world.accumulation, rect.accumulation);
        assert_eq!(world.river_mask, rect.river_mask);
        assert_eq!(world.lake_mask, rect.lake_mask);
        assert!(world.lakes.is_empty());
    }
}