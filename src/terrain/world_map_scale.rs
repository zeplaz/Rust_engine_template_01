//! Symbolic map scale — ties tile grid to design-time **kilometres** without changing sim units.
//!
//! **Sim contract:** 1 tile = 1 logical world unit on XZ (unchanged). [`WorldMapScale::meters_per_tile`]
//! is the lore/design layer used to derive world-gen rhythm (regions, relief wavelength, hydrology density)
//! and UI labels. Buildings may remain oversized in tiles for gameplay.

use serde::{Deserialize, Serialize};

/// Symbolic metres per tile (not survey-grade). Default **100 m** → 320×320 tiles ≈ **32 km** square.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorldMapScale {
    pub meters_per_tile: f32,
}

impl Default for WorldMapScale {
    fn default() -> Self {
        Self {
            meters_per_tile: crate::terrain::world_scale_contract::TILE_METERS_SYMBOLIC,
        }
    }
}

impl WorldMapScale {
    pub const DEFAULT_METERS_PER_TILE: f32 = crate::terrain::world_scale_contract::TILE_METERS_SYMBOLIC;

    #[must_use]
    pub fn extent_m(&self, tiles: u32) -> f32 {
        tiles as f32 * self.meters_per_tile
    }

    #[must_use]
    pub fn extent_km(&self, tiles: u32) -> f32 {
        self.extent_m(tiles) / 1000.0
    }

    #[must_use]
    pub fn area_km2(&self, width: u32, height: u32) -> f32 {
        let w_km = self.extent_km(width);
        let h_km = self.extent_km(height);
        w_km * h_km
    }

    #[must_use]
    pub fn tiles_for_km(&self, km: f32) -> u32 {
        if self.meters_per_tile <= 0.0 {
            return 1;
        }
        ((km * 1000.0) / self.meters_per_tile).round().max(1.0) as u32
    }

    #[must_use]
    pub fn extent_label(&self, width: u32, height: u32) -> String {
        if width == height {
            format!("{:.0} km × {:.0} km", self.extent_km(width), self.extent_km(height))
        } else {
            format!(
                "{:.0} km × {:.0} km",
                self.extent_km(width),
                self.extent_km(height)
            )
        }
    }
}

/// Named tile-grid presets (count of tiles per axis). Pair with [`WorldMapScale`] for km extent.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileExtentPreset {
    /// 192×192 — frame / maneuver tests (~19 km @ 100 m/tile).
    TacticalSmall,
    /// 320×320 — visual harness / **medium-small** play (~32 km).
    MediumSmall,
    /// 512×512 — editor default (~51 km).
    Standard,
    /// 1024×1024 — large strategic patch (~102 km); chunk-authoritative path required at full gen.
    LargeStrategic,
}

impl TileExtentPreset {
    #[must_use]
    pub const fn tiles_per_axis(self) -> u32 {
        match self {
            Self::TacticalSmall => 192,
            Self::MediumSmall => 320,
            Self::Standard => 512,
            Self::LargeStrategic => 1024,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::TacticalSmall => "tactical small (192²)",
            Self::MediumSmall => "medium-small (320²)",
            Self::Standard => "standard (512²)",
            Self::LargeStrategic => "large strategic (1024²)",
        }
    }
}

/// Design-time land-feature rhythm in **symbolic km** (rivers, ridges, macro patches).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LandFeatureRhythm {
    /// Typical diameter of a Voronoi macro patch (strategic region).
    pub macro_patch_km: f32,
    /// Dominant relief / ridge spacing for height noise sampling.
    pub relief_wavelength_km: f32,
    /// Target spacing between major river systems.
    pub major_river_spacing_km: f32,
    /// Lake count density: lakes ≈ `area_km² / 100 × this`.
    pub lakes_per_100km2: f32,
    /// A lake counts as **river-attached** when a river channel cell lies within this many km of any lake
    /// cell (hydrology lake ranking only; see `compute_hydrology_world`). `0` = size-only ranking (default).
    pub lake_river_attach_km: f32,
    /// Rank bonus for attached lakes: score = `cells × (1 + weight)` when attached. `0` = size-only ranking
    /// (default).
    pub lake_river_attach_weight: f32,
    /// Soft marsh belt reach (km) around open water: wet ground within this range becomes `Wetland`
    /// (never open water, never a channel). `0` = no belts (default).
    pub marsh_belt_km: f32,
    /// `0..=1` extra reach / lower moisture threshold inside [`Self::marsh_region`]. Ignored without a region.
    pub marsh_emphasis: f32,
    /// Normalized map rectangle `[x0, y0, x1, y1]` (fractions of width / height, half-open, tile `y` up). When
    /// set, marsh belts form **only** inside it. The region comes from the shared partition
    /// (`engine::playquad::quadrant_aabb`); terrain never defines its own quadrants. `None` = uniform world.
    pub marsh_region: Option<[f32; 4]>,
}

impl Default for LandFeatureRhythm {
    fn default() -> Self {
        Self {
            macro_patch_km: 8.0,
            relief_wavelength_km: 4.0,
            major_river_spacing_km: 6.0,
            lakes_per_100km2: 0.3,
            // Neutral: legacy lake choice, no marsh belts (PQ-H1). Only PlayQuad opts in.
            lake_river_attach_km: 0.0,
            lake_river_attach_weight: 0.0,
            marsh_belt_km: 0.0,
            marsh_emphasis: 0.0,
            marsh_region: None,
        }
    }
}

impl LandFeatureRhythm {
    /// Play-test quadrant world (PLAYTEST-QUADRANT-WORLD-001 §3.3): SW lake basin (river-attached lakes
    /// ranked first, world-wide candidates) and an SE marsh belt confined to `marsh_region`.
    /// `marsh_region` is the SE quadrant as map fractions — built by the caller from the shared partition.
    /// PlayQuad-only; the default rhythm stays neutral.
    #[must_use]
    pub fn playquad_hydrology(marsh_region: [f32; 4]) -> Self {
        Self {
            lakes_per_100km2: 0.5,
            lake_river_attach_km: 0.4,
            lake_river_attach_weight: 4.0,
            marsh_belt_km: 0.3,
            marsh_emphasis: 0.75,
            marsh_region: Some(marsh_region),
            ..Self::default()
        }
    }
}

/// Derived symbolic land params applied to world-gen sliders (`num_regions`, `noise_scale`, hydrology counts).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DerivedLandFeatures {
    pub num_regions: u32,
    pub noise_scale: f32,
    pub river_count: u32,
    pub lake_count: u32,
}

/// How terrain fields are stored after generation (ECS migration).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainFieldStorage {
    /// One ECS entity per tile (`TileMarker`). Legacy; breaks past ~100k tiles.
    #[default]
    PerTileEntities,
    /// Chunk `ChunkCellMatrix` authoritative — target for 100+ km battlefields.
    ChunkCellMatrixAuthoritative,
}

#[must_use]
pub fn derive_land_features(
    width: u32,
    height: u32,
    scale: &WorldMapScale,
    rhythm: &LandFeatureRhythm,
) -> DerivedLandFeatures {
    let w = width.max(1);
    let h = height.max(1);
    let extent_km = scale.extent_km(w.max(h));
    let area_km2 = scale.area_km2(w, h);

    let patch_km = rhythm.macro_patch_km.max(1.0);
    let regions_per_axis = (extent_km / patch_km).ceil().max(1.0);
    let num_regions = (regions_per_axis * regions_per_axis).round() as u32;
    let num_regions = num_regions.clamp(4, 64);

    let wavelength_tiles = scale.tiles_for_km(rhythm.relief_wavelength_km.max(0.5)) as f32;
    let noise_scale = (1.0 / wavelength_tiles).clamp(0.008, 0.12);

    let river_spacing = rhythm.major_river_spacing_km.max(1.0);
    let river_count = (area_km2.sqrt() / river_spacing).round() as u32;
    let river_count = river_count.clamp(0, 16);

    let lake_count = (area_km2 / 100.0 * rhythm.lakes_per_100km2.max(0.0)).round() as u32;
    let lake_count = lake_count.clamp(0, 24);

    DerivedLandFeatures {
        num_regions,
        noise_scale,
        river_count,
        lake_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn medium_small_map_extent() {
        let scale = WorldMapScale::default();
        assert!((scale.extent_km(320) - 32.0).abs() < 0.01);
        assert!((scale.area_km2(320, 320) - 1024.0).abs() < 1.0);
    }

    #[test]
    fn derive_320_matches_symbolic_targets() {
        let scale = WorldMapScale::default();
        let rhythm = LandFeatureRhythm::default();
        let d = derive_land_features(320, 320, &scale, &rhythm);
        assert_eq!(d.num_regions, 16);
        assert!((d.noise_scale - 0.025).abs() < 0.002);
        assert_eq!(d.river_count, 5);
        assert_eq!(d.lake_count, 3);
    }

    #[test]
    fn tiles_for_km_at_100m() {
        let scale = WorldMapScale::default();
        assert_eq!(scale.tiles_for_km(4.0), 40);
        assert_eq!(scale.tiles_for_km(32.0), 320);
    }
}
