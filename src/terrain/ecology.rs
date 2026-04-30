use serde::{Deserialize, Serialize};

use crate::terrain::biome::{BiomeWeights, TerrainSurfaceMix};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FloraType {
    BryophyteMoss,
    Vines,
    Shrubs,
    BroadleafTrees,
    ConiferousTrees,
    Grasses,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CropType {
    Cereal,
    Legume,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FlowerType {
    Daffodil,
    ForgetMeNot,
    Bella,
    Dandelion,
    BlueIris,
    Aster,
}

/// Legacy tag names retained for migration from old terrain files.
pub fn legacy_flower_name_to_type(name: &str) -> Option<FlowerType> {
    match name.trim().to_lowercase().as_str() {
        "dafidals" | "daffodil" | "daffodils" => Some(FlowerType::Daffodil),
        "forgetmenoghts" | "forget-me-not" | "forgetmenots" => Some(FlowerType::ForgetMeNot),
        "bella" => Some(FlowerType::Bella),
        "dandlions" | "dandelion" | "dandelions" => Some(FlowerType::Dandelion),
        "blue iris" | "blue_iris" | "blueiris" => Some(FlowerType::BlueIris),
        "a" | "aster" => Some(FlowerType::Aster),
        _ => None,
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct EcologicalSuitability {
    pub flora_density: f32,
    pub crop_yield_factor: f32,
    pub flower_density: f32,
}

pub fn estimate_ecological_suitability(
    biome: BiomeWeights,
    terrain_mix: TerrainSurfaceMix,
    moisture: f32,
    temperature: f32,
) -> EcologicalSuitability {
    let flora_density = (biome.temperate * 0.8 + biome.wetland * 0.7 + biome.boreal * 0.6)
        * (terrain_mix.organic + terrain_mix.silt).clamp(0.0, 1.0);
    let crop_yield_factor = (biome.temperate * 0.9 + biome.coastal * 0.3 + biome.arid * 0.2)
        * ((moisture * 0.7 + (1.0 - (temperature - 0.55).abs())) * 0.5).clamp(0.0, 1.0);
    let flower_density = (flora_density * 0.6 + biome.coastal * 0.2 + biome.alpine * 0.2)
        .clamp(0.0, 1.0);
    EcologicalSuitability {
        flora_density: flora_density.clamp(0.0, 1.0),
        crop_yield_factor: crop_yield_factor.clamp(0.0, 1.0),
        flower_density,
    }
}
