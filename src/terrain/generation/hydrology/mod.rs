//! Canonical hydrology / surface flow (D8 + depression fill + accumulation).
//! Consumed by pass 4 and by legacy ECS `world_generator_enhanced` full-map generation.

mod flow;

pub use flow::{
    compute_hydrology_rect, compute_hydrology_world, HydrologyResult, LakeRegion,
};

use crate::terrain::biome::BiomeTuning;

/// Tunables for hydrology passes (chunk + world map).
#[derive(Clone, Debug)]
pub struct HydrologyParams {
    /// Normalized height below which tiles are treated as open water.
    pub water_line: f32,
    /// River channel sensitivity: blended against **max** accumulation and a **rank** threshold so
    /// a single mega–catchment cell does not starve smaller maps of visible channels.
    pub river_acc_quantile: f32,
    /// Min absolute slope (neighbour height delta) to tag erosion on river cells.
    pub erosion_slope_threshold: f32,
    /// Moisture at or above this marks silting candidates (low-gradient wet cells).
    pub silt_moisture_threshold: f32,
}

impl Default for HydrologyParams {
    fn default() -> Self {
        Self::from_biome_tuning(&BiomeTuning::default())
    }
}

impl HydrologyParams {
    pub fn from_biome_tuning(t: &BiomeTuning) -> Self {
        Self {
            water_line: t.shallow_water_height_max,
            river_acc_quantile: 0.12,
            erosion_slope_threshold: 0.08,
            silt_moisture_threshold: (t.wetland_moist_threshold * 0.85).min(1.0),
        }
    }
}
