//! Canonical hydrology / surface flow (D8 + depression fill + accumulation).
//! Consumed by pass 4 and by legacy ECS `world_generator_enhanced` full-map generation.

mod erosion;
mod flow;

pub use erosion::{apply_hydraulic_erosion, apply_thermal_erosion, erode_dem};
pub use flow::{
    compute_hydrology_rect, compute_hydrology_world, flow_direction_grid, lake_touches_river,
    HydrologyResult, LakeRegion,
};

use crate::terrain::biome::BiomeTuning;
use crate::terrain::world_map_scale::{LandFeatureRhythm, WorldMapScale};

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
    /// Chebyshev reach (tiles) from a lake cell to a river channel cell that counts as **attached**.
    /// `0` disables attachment ranking (size-only lake truncation).
    pub lake_attach_radius_tiles: u32,
    /// Rank bonus for attached lakes (`cells × (1 + weight)`). `0` disables the bonus.
    pub lake_attach_weight: f32,
    /// **PCG-03** — run thermal→hydraulic before D8 (user world-look signed 2026-11-15).
    pub apply_erosion: bool,
    pub thermal_iters: u32,
    /// Min steepest-descent drop before thermal moves material.
    pub thermal_talus: f32,
    /// Fraction of (drop − talus) moved per thermal iter.
    pub thermal_rate: f32,
    pub hydraulic_iters: u32,
    /// Fraction of steepest drop transported per hydraulic iter.
    pub hydraulic_capacity: f32,
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
            // Neutral: size-only lake truncation unless `with_rhythm` opts in.
            lake_attach_radius_tiles: 0,
            lake_attach_weight: 0.0,
            // PCG-03 default-on (modest); env/tests may set `apply_erosion = false`.
            apply_erosion: true,
            thermal_iters: 2,
            thermal_talus: 0.04,
            thermal_rate: 0.35,
            hydraulic_iters: 2,
            hydraulic_capacity: 0.2,
        }
    }

    /// Apply the km-scale lake attachment knobs of a [`LandFeatureRhythm`].
    #[must_use]
    pub fn with_rhythm(mut self, rhythm: &LandFeatureRhythm, scale: &WorldMapScale) -> Self {
        self.lake_attach_radius_tiles = if rhythm.lake_river_attach_km > 0.0 {
            scale.tiles_for_km(rhythm.lake_river_attach_km)
        } else {
            0
        };
        self.lake_attach_weight = rhythm.lake_river_attach_weight.max(0.0);
        self
    }
}
