//! Per-biome vegetation suitability — what may grow where.
//!
//! The terrain classification is the authority: each cell carries a terrain family
//! ([`ChunkCellMatrix::family`], names from `terrain_family_registry`), and standing water comes from
//! the same gate fire uses ([`SurfaceWaterFireGate`]: shallow water line + P4 `flooded` tag, which
//! covers lakes and river channels). A table maps family name to a canopy ceiling and a growth rate.
//! The table is data (`assets/configs/landscape/_vegetation_suitability.ron`) with code defaults.
//!
//! This module only produces inputs. The canopy writer ([`super::vegetation_field`]) applies the cap,
//! so there is still one writer of `canopy_density`.

use std::collections::HashMap;
use std::fs;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use serde::Deserialize;

use super::landscape_grammar_lg2::SuccessionTopologyStage;
use super::landscape_grammar_map::repo_asset_path;
use super::vegetation_field::CanopyDrivers;
use crate::systems::fire::SurfaceWaterFireGate;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::biome::BiomeId;
use crate::terrain::generation::ChunkCellMatrix;
use crate::terrain::{default_terrain_families, TerrainFamilyRegistry};

pub const VEGETATION_SUITABILITY_RON: &str = "assets/configs/landscape/_vegetation_suitability.ron";

/// Canopy ceiling (0 = nothing grows) and growth rate multiplier for one terrain family.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct SuitabilityEntry {
    pub max_canopy: f32,
    #[serde(default = "one")]
    pub growth: f32,
}

fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Deserialize)]
struct SuitabilityEntryFile {
    family: String,
    max_canopy: f32,
    #[serde(default = "one")]
    growth: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct SuitabilityFile {
    #[allow(dead_code)]
    schema_version: u32,
    /// Ceiling for a family the table does not list.
    #[serde(default = "one")]
    default_max_canopy: f32,
    entries: Vec<SuitabilityEntryFile>,
}

/// Terrain family name → suitability. Families missing from the table use `default_max_canopy`.
#[derive(Resource, Debug, Clone)]
pub struct VegetationSuitabilityTable {
    pub default_max_canopy: f32,
    pub by_family: HashMap<String, SuitabilityEntry>,
}

/// Code defaults (also what the RON ships): water none, wetland low cover, desert/rock/ice/beach
/// sparse or none, tundra/alpine low scrub, grassland scattered, forests full.
const DEFAULT_ROWS: &[(&str, f32, f32)] = &[
    ("DeepWater", 0.0, 0.0),
    ("ShallowWater", 0.0, 0.0),
    ("Beach", 0.05, 0.4),
    ("Desert", 0.05, 0.3),
    ("Grassland", 0.3, 0.9),
    ("Forest", 0.85, 1.0),
    ("DenseForest", 1.0, 1.0),
    ("Mountain", 0.2, 0.5),
    ("SnowCappedMountain", 0.0, 0.0),
    ("Tundra", 0.15, 0.4),
    ("Swamp", 0.35, 0.8),
    ("Cliff", 0.0, 0.0),
    ("Concrete", 0.0, 0.0),
    ("Dirt", 0.1, 0.6),
    ("Snow", 0.0, 0.0),
    ("Stone", 0.0, 0.0),
];

impl Default for VegetationSuitabilityTable {
    fn default() -> Self {
        Self {
            default_max_canopy: 1.0,
            by_family: DEFAULT_ROWS
                .iter()
                .map(|(name, cap, growth)| {
                    (
                        (*name).to_owned(),
                        SuitabilityEntry {
                            max_canopy: *cap,
                            growth: *growth,
                        },
                    )
                })
                .collect(),
        }
    }
}

impl VegetationSuitabilityTable {
    /// Load the RON table; any read or parse failure yields the code defaults.
    #[must_use]
    pub fn load() -> Self {
        let Ok(raw) = fs::read_to_string(repo_asset_path(VEGETATION_SUITABILITY_RON)) else {
            return Self::default();
        };
        let Ok(file) = ron::from_str::<SuitabilityFile>(&raw) else {
            return Self::default();
        };
        Self {
            default_max_canopy: file.default_max_canopy.clamp(0.0, 1.0),
            by_family: file
                .entries
                .into_iter()
                .map(|e| {
                    (
                        e.family,
                        SuitabilityEntry {
                            max_canopy: e.max_canopy.clamp(0.0, 1.0),
                            growth: e.growth.clamp(0.0, 1.0),
                        },
                    )
                })
                .collect(),
        }
    }

    #[must_use]
    pub fn entry(&self, family_name: &str) -> SuitabilityEntry {
        self.by_family
            .get(family_name)
            .copied()
            .unwrap_or(SuitabilityEntry {
                max_canopy: self.default_max_canopy,
                growth: 1.0,
            })
    }
}

/// Cached per-chunk result (read by the canopy writer inputs and by extract/render).
#[derive(Component, Debug, Clone, PartialEq)]
pub struct ChunkVegetationSuitability {
    /// Mean per-cell ceiling: the largest chunk canopy density the terrain allows.
    pub max_canopy: f32,
    /// Mean growth multiplier over cells that can grow vegetation (1 when none can).
    pub growth: f32,
    /// Fraction of cells that can carry any vegetation (0 for an all-water chunk).
    pub land_fraction: f32,
    /// Per-cell ceiling, row-major like the cell matrix. Zero on water and bare rock.
    pub cell_caps: Vec<f32>,
}

impl ChunkVegetationSuitability {
    /// Ceiling for one cell; 1 when the cell is out of range (no information, no restriction).
    #[must_use]
    pub fn cell_cap(&self, i: usize) -> f32 {
        self.cell_caps.get(i).copied().unwrap_or(1.0)
    }

    /// Cover to draw on one cell. `chunk_canopy` is a cover fraction over the whole chunk, so it is
    /// re-based onto the land cells (a half-ocean chunk shows the land at twice the chunk value),
    /// then limited by the cell's own ceiling. Water cells always return 0.
    #[must_use]
    pub fn cell_density(&self, chunk_canopy: f32, i: usize) -> f32 {
        let cap = self.cell_cap(i);
        if cap <= 0.0 || self.land_fraction <= 0.0 {
            return 0.0;
        }
        (chunk_canopy / self.land_fraction).clamp(0.0, 1.0).min(cap)
    }
}

/// Everything needed to rate a cell. Missing resources fall back to defaults.
#[derive(SystemParam)]
pub struct SuitabilityParams<'w> {
    table: Option<Res<'w, VegetationSuitabilityTable>>,
    gate: Option<Res<'w, SurfaceWaterFireGate>>,
    handles: Option<Res<'w, TerrainRegistriesHandles>>,
    family_assets: Option<Res<'w, Assets<TerrainFamilyRegistry>>>,
}

impl SuitabilityParams<'_> {
    #[must_use]
    pub fn rate(&self, matrix: &ChunkCellMatrix) -> ChunkVegetationSuitability {
        let default_table = VegetationSuitabilityTable::default();
        let table = self.table.as_deref().unwrap_or(&default_table);
        let default_gate = SurfaceWaterFireGate::default();
        let gate = self.gate.as_deref().unwrap_or(&default_gate);
        let families = self
            .handles
            .as_ref()
            .zip(self.family_assets.as_ref())
            .and_then(|(h, a)| a.get(&h.terrain_families))
            .unwrap_or_else(|| default_terrain_families());
        rate_chunk_vegetation_suitability(matrix, families, table, gate)
    }

    fn changed(&self) -> bool {
        self.table.as_ref().is_some_and(|t| t.is_changed())
            || self.gate.as_ref().is_some_and(|g| g.is_changed())
    }
}

/// Pure rating: standing water and marine cells get 0, otherwise the family row decides.
#[must_use]
pub fn rate_chunk_vegetation_suitability(
    matrix: &ChunkCellMatrix,
    families: &TerrainFamilyRegistry,
    table: &VegetationSuitabilityTable,
    gate: &SurfaceWaterFireGate,
) -> ChunkVegetationSuitability {
    let n = matrix.elevation.len();
    let mut cell_caps = Vec::with_capacity(n);
    let mut growth_sum = 0.0f32;
    let mut growth_cells = 0u32;
    for i in 0..n {
        let marine = matrix
            .weights
            .get(i)
            .is_some_and(|w| w.primary() == BiomeId::Marine);
        if marine || gate.cell_has_standing_water(matrix, i) {
            cell_caps.push(0.0);
            continue;
        }
        let entry = matrix
            .family
            .get(i)
            .and_then(|id| families.def(*id))
            .map(|def| table.entry(&def.name))
            .unwrap_or(SuitabilityEntry {
                max_canopy: table.default_max_canopy,
                growth: 1.0,
            });
        if entry.max_canopy > 0.0 {
            growth_sum += entry.growth;
            growth_cells += 1;
        }
        cell_caps.push(entry.max_canopy.clamp(0.0, 1.0));
    }
    let max_canopy = if n == 0 {
        1.0
    } else {
        cell_caps.iter().sum::<f32>() / n as f32
    };
    let land_cells = cell_caps.iter().filter(|c| **c > 0.0).count();
    ChunkVegetationSuitability {
        max_canopy,
        land_fraction: if n == 0 { 1.0 } else { land_cells as f32 / n as f32 },
        growth: if growth_cells == 0 {
            1.0
        } else {
            growth_sum / growth_cells as f32
        },
        cell_caps,
    }
}

/// Highest succession stage a chunk ceiling can support. Bare ground stays grass, scrub stays shrub.
#[must_use]
pub fn max_stage_for_canopy_cap(cap: f32) -> SuccessionTopologyStage {
    match cap {
        c if c < 0.2 => SuccessionTopologyStage::Grass,
        c if c < 0.45 => SuccessionTopologyStage::Shrub,
        c if c < 0.75 => SuccessionTopologyStage::YoungForest,
        _ => SuccessionTopologyStage::OldGrowth,
    }
}

/// Order of the growth stages for capping. `BurnScar` is a disturbance, not a rung, so it is never capped.
#[must_use]
pub fn clamp_stage_to_cap(stage: SuccessionTopologyStage, cap: f32) -> SuccessionTopologyStage {
    fn rank(s: SuccessionTopologyStage) -> Option<u8> {
        match s {
            SuccessionTopologyStage::Grass => Some(0),
            SuccessionTopologyStage::Shrub => Some(1),
            SuccessionTopologyStage::YoungForest => Some(2),
            SuccessionTopologyStage::OldGrowth => Some(3),
            SuccessionTopologyStage::BurnScar => None,
        }
    }
    let max = max_stage_for_canopy_cap(cap);
    match (rank(stage), rank(max)) {
        (Some(s), Some(m)) if s > m => max,
        _ => stage,
    }
}

pub(crate) fn init_vegetation_suitability_table(mut commands: Commands) {
    commands.insert_resource(VegetationSuitabilityTable::load());
}

/// Input system: rate each chunk's terrain and hand the ceiling to the canopy writer.
/// Runs when the matrix changes, the chunk is new, or the table/water gate changed.
pub(crate) fn update_chunk_vegetation_suitability(
    params: SuitabilityParams,
    mut commands: Commands,
    mut q: Query<(
        Entity,
        Ref<ChunkCellMatrix>,
        Option<&mut ChunkVegetationSuitability>,
        Option<&mut CanopyDrivers>,
    )>,
) {
    let all = params.changed();
    for (entity, matrix, suit, drivers) in &mut q {
        let needs = all
            || matrix.is_changed()
            || suit.is_none()
            || drivers.as_ref().is_some_and(|d| d.is_added());
        if !needs {
            continue;
        }
        let rated = params.rate(&matrix);
        if let Some(mut drivers) = drivers {
            if drivers.suitability_cap != rated.max_canopy || drivers.growth_scale != rated.growth {
                drivers.suitability_cap = rated.max_canopy;
                drivers.growth_scale = rated.growth;
            }
        }
        match suit {
            Some(mut existing) => {
                if *existing != rated {
                    *existing = rated;
                }
            }
            None => {
                commands.entity(entity).insert(rated);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::TerrainFamilyId;

    fn family(name: &str) -> TerrainFamilyId {
        default_terrain_families()
            .id(name)
            .unwrap_or_else(|| panic!("family {name}"))
    }

    fn matrix_of(name: &str, elevation: f32) -> ChunkCellMatrix {
        let mut m = ChunkCellMatrix::new(UVec2::new(4, 4));
        for i in 0..16 {
            m.elevation[i] = elevation;
            m.family[i] = family(name);
        }
        m
    }

    #[test]
    fn ron_table_matches_code_defaults() {
        let loaded = VegetationSuitabilityTable::load();
        let defaults = VegetationSuitabilityTable::default();
        assert_eq!(loaded.by_family.len(), defaults.by_family.len());
        for (name, entry) in &defaults.by_family {
            assert_eq!(loaded.by_family.get(name), Some(entry), "{name}");
        }
    }

    #[test]
    fn water_and_flooded_cells_allow_nothing_and_forest_allows_full() {
        let table = VegetationSuitabilityTable::default();
        let gate = SurfaceWaterFireGate::default();
        let families = default_terrain_families();
        for name in ["DeepWater", "ShallowWater"] {
            let r = rate_chunk_vegetation_suitability(&matrix_of(name, 0.1), families, &table, &gate);
            assert_eq!(r.max_canopy, 0.0, "{name}");
        }
        // A forest-labelled cell below the water line is still water.
        let drowned = rate_chunk_vegetation_suitability(&matrix_of("Forest", 0.05), families, &table, &gate);
        assert_eq!(drowned.max_canopy, 0.0);
        let forest = rate_chunk_vegetation_suitability(&matrix_of("DenseForest", 0.6), families, &table, &gate);
        assert_eq!(forest.max_canopy, 1.0);
        let swamp = rate_chunk_vegetation_suitability(&matrix_of("Swamp", 0.6), families, &table, &gate);
        assert!(swamp.max_canopy > 0.0 && swamp.max_canopy < 0.5);
    }

    #[test]
    fn stage_is_capped_by_ceiling() {
        assert_eq!(
            clamp_stage_to_cap(SuccessionTopologyStage::OldGrowth, 0.0),
            SuccessionTopologyStage::Grass
        );
        assert_eq!(
            clamp_stage_to_cap(SuccessionTopologyStage::OldGrowth, 0.35),
            SuccessionTopologyStage::Shrub
        );
        assert_eq!(
            clamp_stage_to_cap(SuccessionTopologyStage::OldGrowth, 1.0),
            SuccessionTopologyStage::OldGrowth
        );
        assert_eq!(
            clamp_stage_to_cap(SuccessionTopologyStage::BurnScar, 0.0),
            SuccessionTopologyStage::BurnScar
        );
    }

    #[test]
    fn water_chunk_never_grows_canopy_and_forest_chunk_does() {
        use super::super::vegetation_field::integrate_vegetation_field_step_with_drivers;
        use super::super::ChunkEcology;
        use super::super::VegetationField;
        use crate::systems::weather::ChunkWeather;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, update_chunk_vegetation_suitability);
        let lake = app
            .world_mut()
            .spawn((
                matrix_of("DeepWater", 0.1),
                VegetationField {
                    canopy_density: 0.9,
                    understory_density: 0.5,
                    ..VegetationField::default()
                },
            ))
            .id();
        let forest = app
            .world_mut()
            .spawn((
                matrix_of("DenseForest", 0.6),
                VegetationField {
                    canopy_density: 0.1,
                    ..VegetationField::default()
                },
            ))
            .id();
        app.update();
        app.update();

        let eco = ChunkEcology {
            biomass: 0.8,
            regrowth_rate: 0.3,
            shade_factor: 0.6,
            ..ChunkEcology::default()
        };
        let wx = ChunkWeather::default();
        for (entity, expect_growth) in [(lake, false), (forest, true)] {
            let drivers = *app.world().get::<CanopyDrivers>(entity).unwrap();
            let mut veg = *app.world().get::<VegetationField>(entity).unwrap();
            // Population target and succession floor push hard toward cover on both.
            let drivers = CanopyDrivers {
                population_density: Some(1.0),
                succession_floor: 0.5,
                ..drivers
            };
            for _ in 0..60 {
                integrate_vegetation_field_step_with_drivers(0.5, None, &eco, &wx, 0.0, &drivers, &mut veg);
                if !expect_growth {
                    assert_eq!(veg.canopy_density, 0.0);
                }
            }
            if expect_growth {
                assert!(veg.canopy_density > 0.5, "forest canopy {}", veg.canopy_density);
            } else {
                assert_eq!(veg.understory_density, 0.0);
                assert_eq!(veg.old_growth, 0.0);
                assert_eq!(veg.ground_fuel, 0.0);
            }
        }
        let suit = app.world().get::<ChunkVegetationSuitability>(lake).unwrap();
        assert!(suit.cell_caps.iter().all(|c| *c == 0.0));
        assert_eq!(suit.cell_density(0.9, 0), 0.0);
    }

    #[test]
    fn coastal_chunk_density_reflects_land_cells_only() {
        let table = VegetationSuitabilityTable::default();
        let gate = SurfaceWaterFireGate::default();
        let families = default_terrain_families();
        let mut m = matrix_of("DenseForest", 0.6);
        for i in 0..8 {
            m.family[i] = family("DeepWater");
            m.elevation[i] = 0.1;
        }
        let r = rate_chunk_vegetation_suitability(&m, families, &table, &gate);
        assert_eq!(r.land_fraction, 0.5);
        assert_eq!(r.max_canopy, 0.5);
        assert_eq!(r.cell_density(0.4, 0), 0.0);
        assert!((r.cell_density(0.4, 12) - 0.8).abs() < 1e-6);
    }
}
