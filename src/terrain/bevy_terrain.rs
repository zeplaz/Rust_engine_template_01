use bevy::prelude::Vec2;
use serde::{Deserialize, Serialize};

/// Legacy parsing label set from `base_terrains.dat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[doc(alias = "Parameter_labels")]
pub enum LegacyTerrainType {
    Grass,
    Forest,
    Swamp,
    Water,
    Cliff,
    Concrete,
    Sand,
    Dirt,
    Snow,
    Stone,
}

/// Legacy alias maintained so old parsing code can migrate gradually.
#[deprecated(note = "Use terrain::biome::TerrainClass for canonical storage/generation")]
pub type TerrainType = LegacyTerrainType;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct TerrainFeatures {
    pub road: bool,
    pub track: bool,
}

/// Legacy environment descriptor retained for migration/readback.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LegacyTerrainEnvironment {
    pub ambient_temperature: Vec2,
    pub roughness: f64,
    pub moisture: f64,
    pub elevation: f64,
    pub sunlight: f64,
    pub cloud_coverage: f64,
    pub water_density: f64,
    pub water_salinity: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LegacyTerrainTag {
    SedgePath,
    Moss,
    Vines,
    Shrubs,
    BroadLeafedTrees,
    ConiferousTrees,
    Grass,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LegacyCropSubtag {
    Cereal,
    Legumes,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum LegacyFlowerSubtag {
    Dafidals,
    Forgetmenoghts,
    Bella,
    Dandlions,
    BlueIris,
    A,
}
