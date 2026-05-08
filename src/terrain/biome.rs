use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum BiomeId {
    Marine,
    Coastal,
    Arid,
    Temperate,
    Boreal,
    Alpine,
    Wetland,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct BiomeWeights {
    pub marine: f32,
    pub coastal: f32,
    pub arid: f32,
    pub temperate: f32,
    pub boreal: f32,
    pub alpine: f32,
    pub wetland: f32,
}

impl Default for BiomeWeights {
    fn default() -> Self {
        Self {
            marine: 0.0,
            coastal: 0.0,
            arid: 0.0,
            temperate: 1.0,
            boreal: 0.0,
            alpine: 0.0,
            wetland: 0.0,
        }
    }
}

impl BiomeWeights {
    pub fn normalize(mut self) -> Self {
        let sum = self.marine
            + self.coastal
            + self.arid
            + self.temperate
            + self.boreal
            + self.alpine
            + self.wetland;
        if sum <= f32::EPSILON {
            return Self::default();
        }
        self.marine /= sum;
        self.coastal /= sum;
        self.arid /= sum;
        self.temperate /= sum;
        self.boreal /= sum;
        self.alpine /= sum;
        self.wetland /= sum;
        self
    }

    pub fn primary(self) -> BiomeId {
        let mut best = (BiomeId::Temperate, self.temperate);
        for (id, val) in [
            (BiomeId::Marine, self.marine),
            (BiomeId::Coastal, self.coastal),
            (BiomeId::Arid, self.arid),
            (BiomeId::Boreal, self.boreal),
            (BiomeId::Alpine, self.alpine),
            (BiomeId::Wetland, self.wetland),
        ] {
            if val > best.1 {
                best = (id, val);
            }
        }
        best.0
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TerrainSurfaceMix {
    pub sand: f32,
    pub silt: f32,
    pub clay: f32,
    pub rock: f32,
    pub organic: f32,
}

impl Default for TerrainSurfaceMix {
    fn default() -> Self {
        Self {
            sand: 0.2,
            silt: 0.3,
            clay: 0.2,
            rock: 0.2,
            organic: 0.1,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct TileEnvironmentProfile {
    pub biome_weights: BiomeWeights,
    pub terrain_mix: TerrainSurfaceMix,
}

/// Registry tag names for pass 2 (threshold tags). Defaults match `tag_registry.example.json`; override via `BiomeTuning` JSON.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ThresholdTagNames {
    pub lowland: String,
    pub highland: String,
    pub wet: String,
    pub dry: String,
    pub hot: String,
    pub cold: String,
}

impl Default for ThresholdTagNames {
    fn default() -> Self {
        Self {
            lowland: "lowland".into(),
            highland: "highland".into(),
            wet: "wet".into(),
            dry: "dry".into(),
            hot: "hot".into(),
            cold: "cold".into(),
        }
    }
}

/// All thresholds / gains that couple height → moisture/temperature → biome weights and terrain **family** id
/// from [`crate::terrain::family::classify_biome`].
/// Tune via editor sliders, JSON overlay, or scripts — keeps generation and classification in sync.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct BiomeTuning {
    pub sea_level: f32,
    pub marine_height_sensitivity: f32,
    pub coastal_height_center: f32,
    pub coastal_band_sharpness: f32,
    pub alpine_height_threshold: f32,
    pub alpine_height_gain: f32,
    pub alpine_cold_reference_temp: f32,
    pub alpine_cold_gain: f32,
    pub arid_hot_threshold: f32,
    pub arid_hot_gain: f32,
    pub arid_dry_moisture_ceiling: f32,
    pub arid_dry_gain: f32,
    pub wetland_moist_threshold: f32,
    pub wetland_moist_gain: f32,
    pub wetland_temp_center: f32,
    pub boreal_cold_reference_temp: f32,
    pub boreal_cold_gain: f32,
    pub boreal_moist_floor: f32,
    pub boreal_moist_gain: f32,
    pub temperate_marine_weight: f32,
    pub temperate_alpine_suppress: f32,
    pub temperate_floor: f32,

    pub deep_water_height_max: f32,
    pub shallow_water_height_max: f32,
    pub beach_height_max: f32,
    pub mountain_height_min: f32,
    pub snow_peak_temperature_max: f32,
    pub tundra_temperature_max: f32,
    pub hot_lowlands_temperature_min: f32,
    pub desert_moisture_max: f32,
    pub swamp_moisture_min: f32,
    pub grassland_moisture_max: f32,
    pub forest_moisture_max: f32,
    pub threshold_tag_names: ThresholdTagNames,
}

impl Default for BiomeTuning {
    fn default() -> Self {
        Self {
            sea_level: 0.40,
            marine_height_sensitivity: 2.5,
            coastal_height_center: 0.38,
            coastal_band_sharpness: 8.0,
            alpine_height_threshold: 0.70,
            alpine_height_gain: 3.0,
            alpine_cold_reference_temp: 0.25,
            alpine_cold_gain: 1.5,
            arid_hot_threshold: 0.60,
            arid_hot_gain: 2.0,
            arid_dry_moisture_ceiling: 0.45,
            arid_dry_gain: 2.0,
            wetland_moist_threshold: 0.70,
            wetland_moist_gain: 2.0,
            wetland_temp_center: 0.55,
            boreal_cold_reference_temp: 0.40,
            boreal_cold_gain: 2.0,
            boreal_moist_floor: 0.35,
            boreal_moist_gain: 1.4,
            temperate_marine_weight: 1.0,
            temperate_alpine_suppress: 0.4,
            temperate_floor: 0.05,
            deep_water_height_max: 0.2,
            shallow_water_height_max: 0.35,
            beach_height_max: 0.38,
            mountain_height_min: 0.75,
            snow_peak_temperature_max: 0.2,
            tundra_temperature_max: 0.3,
            hot_lowlands_temperature_min: 0.7,
            desert_moisture_max: 0.3,
            swamp_moisture_min: 0.8,
            grassland_moisture_max: 0.4,
            forest_moisture_max: 0.7,
            threshold_tag_names: ThresholdTagNames::default(),
        }
    }
}

/// Whittaker-style soft weights + normalization (shared with [`crate::terrain::family::classify_biome`]).
pub fn compute_biome_weights(
    height: f32,
    moisture: f32,
    temperature: f32,
    tuning: &BiomeTuning,
) -> BiomeWeights {
    let mut weights = BiomeWeights::default();
    weights.marine =
        ((tuning.sea_level - height) * tuning.marine_height_sensitivity).max(0.0);
    weights.coastal = (1.0
        - (height - tuning.coastal_height_center).abs() * tuning.coastal_band_sharpness)
        .max(0.0);
    weights.alpine = ((height - tuning.alpine_height_threshold) * tuning.alpine_height_gain)
        .max(0.0)
        + ((tuning.alpine_cold_reference_temp - temperature) * tuning.alpine_cold_gain).max(0.0);
    weights.arid = ((temperature - tuning.arid_hot_threshold) * tuning.arid_hot_gain).max(0.0)
        * ((tuning.arid_dry_moisture_ceiling - moisture) * tuning.arid_dry_gain).max(0.0);
    weights.wetland = ((moisture - tuning.wetland_moist_threshold) * tuning.wetland_moist_gain)
        .max(0.0)
        * (1.0 - (temperature - tuning.wetland_temp_center).abs());
    weights.boreal = ((tuning.boreal_cold_reference_temp - temperature) * tuning.boreal_cold_gain)
        .max(0.0)
        * ((moisture - tuning.boreal_moist_floor) * tuning.boreal_moist_gain).max(0.0);
    weights.temperate = (1.0
        - weights.marine * tuning.temperate_marine_weight
        - weights.alpine * tuning.temperate_alpine_suppress)
        .max(tuning.temperate_floor);
    weights.normalize()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum BiomeBucket {
    Water,
    Beach,
    Plains,
    Forest,
    Mountain,
    Snow,
    Desert,
    Swamp,
}

/// Parse aggregate bucket from `terrain_family_registry.example.json`.
pub fn biome_bucket_from_str(s: &str) -> Result<BiomeBucket, String> {
    match s {
        "Water" => Ok(BiomeBucket::Water),
        "Beach" => Ok(BiomeBucket::Beach),
        "Plains" => Ok(BiomeBucket::Plains),
        "Forest" => Ok(BiomeBucket::Forest),
        "Mountain" => Ok(BiomeBucket::Mountain),
        "Snow" => Ok(BiomeBucket::Snow),
        "Desert" => Ok(BiomeBucket::Desert),
        "Swamp" => Ok(BiomeBucket::Swamp),
        _ => Err(format!(
            "unknown biome_bucket {s:?}; use Water|Beach|Plains|Forest|Mountain|Snow|Desert|Swamp"
        )),
    }
}
