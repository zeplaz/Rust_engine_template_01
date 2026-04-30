use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::terrain::biome::{BiomeWeights, TerrainSurfaceMix};
use crate::terrain::ecology::{CropType, FlowerType, FloraType};

/// Serializable, data-only generation profile for tile-scale ecology.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerrainPoint {
    pub position: Vec3,
    pub gradient: Vec3,
    pub biome_weights: BiomeWeights,
    pub surface_mix: TerrainSurfaceMix,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VegetationCandidates {
    pub flora: Vec<FloraType>,
    pub crops: Vec<CropType>,
    pub flowers: Vec<FlowerType>,
}

// Legacy migration note: this file previously held invalid prototype structs/enums;
// those names are replaced by `TerrainPoint` and `VegetationCandidates` for canonical
// serializable generation inputs.
