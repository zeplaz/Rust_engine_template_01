// Terrain systems
mod locational;
mod registry_serde_path;
mod tiles;
mod tools;
mod voronoi;
mod voronoi_enhanced;
mod world;
pub mod bevy_terrain;
pub mod biome;
pub mod ecology;
pub mod family;
pub mod material;
pub mod generation;
pub mod mobility;
pub mod fire;
pub mod dynamic_overlay;
pub mod editor;
pub mod world_map_scale;
pub mod world_scale_contract;

// Public exports
pub use locational::*;
pub use bevy_terrain::*;
pub use biome::*;
pub use family::{
    classify_biome, default_terrain_families, hash_terrain_family_registry, BiomeClassification,
    TerrainFamilyDef, TerrainFamilyId, TerrainFamilyRegistry, TerrainFamilyRegistryLoader,
};
pub use ecology::*;
pub use tiles::*;
pub use voronoi::*;
pub use world::*;
pub use world_map_scale::{
    derive_land_features, LandFeatureRhythm, TerrainFieldStorage, TileExtentPreset,
    WorldMapScale,
};
pub use world_scale_contract::{
    BUILDING_LARGE_FOOTPRINT_TILES, BUILDING_TYPICAL_FOOTPRINT_TILES,
    BUILDINGS_PER_FRAME_TARGET, CHUNK_TILES_DISPLAY, CHUNK_TILES_SIM,
    OPERATIONAL_PX_PER_TILE, OPERATIONAL_ZOOM_ALPHA, SITE_TYPICAL_ENVELOPE_TILES,
    TACTICAL_PX_PER_TILE, TACTICAL_PROOF_ZOOM_ALPHA, TILE_METERS_SYMBOLIC,
    TILE_MIN_SCREEN_PX, TILE_SIM_UNIT, WORLD_DEFAULT_TILES_AXIS,
    WORLD_HARNESS_TILES_AXIS,
};
pub use dynamic_overlay::{
    apply_chunk_weather_to_dynamic_overlay, decay_dynamic_terrain_overlay,
    overlay_mud_at, stub_accumulate_overlay_from_chunk_fields, ChunkCellKey, DynamicTerrainOverlay,
};
pub use fire::{
    ammo_dump_profile, fuel_depot_profile, fuel_material_def, lithium_battery_warehouse,
    FuelLayer, FuelMaterialDef, FuelMaterialKind, ScenarioHazardLayer, StructureFireProfile,
    VegetationFuelLayer, layer_fuel_mass,
};