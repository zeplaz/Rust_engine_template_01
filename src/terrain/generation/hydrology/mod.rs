//! Canonical hydrology / surface flow (D8 + depression fill + accumulation).
//! Consumed by pass 4 and by legacy ECS `world_generator_enhanced` full-map generation.

mod flow;

pub use flow::{
    compute_hydrology_rect, compute_hydrology_world, flow_direction_grid, HydrologyResult,
    LakeRegion,
};

use crate::terrain::biome::BiomeTuning;

/// Tunables for hydrology passes (chunk + world map).
#[derive(Clone, Debug)]
pub struct HydrologyParams {
    /// Normalized height below which tiles are treated as open water.
    pub water_line: f32,
    /// Fraction of cells (by flow-accumulation rank) that form the river channel mask — the major
    /// drainage trunks. Channel width is derived from flow in [`crate::terrain::generation::water_class`].
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
            // Major drainage trunks only: ~0.5% of cells as channel centrelines (1.5% plus the old
            // max-relative blend drew ~3% of the map as a web of creeks; 0.10 flooded it).
            river_acc_quantile: 0.005,
            erosion_slope_threshold: 0.08,
            silt_moisture_threshold: (t.wetland_moist_threshold * 0.85).min(1.0),
        }
    }
}
