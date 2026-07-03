//! Dense terrain field storage — authoritative when [`TerrainFieldStorage::ChunkCellMatrixAuthoritative`].
//!
//! Replaces one-ECS-entity-per-tile for full world generation. Chunk [`ChunkCellMatrix`] components
//! are hydrated from this cache; raster / preview read chunks or this resource instead of iterating
//! 100k+ [`TileMarker`] entities.

use bevy::prelude::*;

use crate::terrain::family::{default_terrain_families, TerrainFamilyId, DEFAULT_TERRAIN_FAMILY_ID};
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;
use crate::terrain::generation::chunk::Chunk;
use crate::terrain::generation::hydrology::HydrologyResult;
use crate::terrain::generation::world_generator_enhanced::{TileSpawnData, WorldGenParams};
use crate::terrain::world_map_scale::TerrainFieldStorage;
use crate::render::TileWorldFallbackRasterDirty;

/// Prevents re-copying dense terrain into chunks every frame.
#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub struct DenseTerrainHydrationGate {
    stamped: Option<(u32, u32, u64)>,
}

impl DenseTerrainHydrationGate {
    pub fn reset(&mut self) {
        self.stamped = None;
    }
}

/// Row-major `width × height` terrain fields (same layout as world-gen height grid).
#[derive(Resource, Clone, Debug)]
pub struct WorldGenDenseTerrainCache {
    pub width: u32,
    pub height: u32,
    pub elevation: Vec<f32>,
    pub moisture: Vec<f32>,
    pub temperature: Vec<f32>,
    pub family: Vec<TerrainFamilyId>,
    pub region_index: Vec<u32>,
}

impl WorldGenDenseTerrainCache {
    #[must_use]
    pub fn tile_count(&self) -> u32 {
        self.width.saturating_mul(self.height)
    }

    #[inline]
    fn idx_xy(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        Some((y * self.width + x) as usize)
    }

    #[must_use]
    pub(crate) fn from_world_gen_raster(
        width: u32,
        height: u32,
        elevation: Vec<f32>,
        moisture: Vec<f32>,
        cells: &[TileSpawnData],
        region_index: Vec<u32>,
    ) -> Self {
        let n = (width as usize).saturating_mul(height as usize);
        let mut family = vec![DEFAULT_TERRAIN_FAMILY_ID; n];
        for (i, cell) in cells.iter().enumerate().take(n) {
            family[i] = cell.terrain_family;
        }
        Self {
            width,
            height,
            elevation,
            moisture,
            temperature: cells.iter().map(|c| c.temperature).collect(),
            family,
            region_index,
        }
    }

    pub fn apply_hydrology(&mut self, params: &WorldGenParams, hydro: &HydrologyResult) {
        let shallow = default_terrain_families()
            .id("ShallowWater")
            .expect("terrain family registry must define ShallowWater");
        let water_line = params.biome_tuning.shallow_water_height_max;
        let river_depth = (water_line * 0.92).clamp(0.02, 0.98);
        let lake_depth = ((params.biome_tuning.deep_water_height_max + water_line) * 0.5)
            .clamp(0.02, 0.98);

        for path in &hydro.rivers {
            for &(tx, ty) in path {
                let Some(i) = self.idx_xy(tx, ty) else {
                    continue;
                };
                self.elevation[i] = river_depth;
                self.moisture[i] = 0.95;
                self.family[i] = shallow;
            }
        }
        for lake in &hydro.lakes {
            for &(lx, ly) in &lake.cells {
                let Some(i) = self.idx_xy(lx, ly) else {
                    continue;
                };
                self.elevation[i] = lake_depth;
                self.moisture[i] = 0.98;
                self.family[i] = shallow;
            }
        }
    }
}

/// Copy dense cache into live chunk matrices (sim / fire / materialize path).
pub fn hydrate_chunk_matrices_from_dense_terrain(
    params: Res<WorldGenParams>,
    cache: Option<Res<WorldGenDenseTerrainCache>>,
    mut chunks: Query<(&Chunk, &mut ChunkCellMatrix)>,
    gate: Option<ResMut<DenseTerrainHydrationGate>>,
    mut raster_dirty: Option<ResMut<TileWorldFallbackRasterDirty>>,
) {
    if params.field_storage != TerrainFieldStorage::ChunkCellMatrixAuthoritative {
        return;
    }
    let Some(cache) = cache else {
        return;
    };
    if cache.width == 0 || cache.height == 0 {
        return;
    }
    let Some(mut gate) = gate else {
        return;
    };
    let key = (cache.width, cache.height, params.seed);
    if gate.stamped == Some(key) {
        return;
    }
    if chunks.is_empty() {
        return;
    }

    for (chunk, mut matrix) in chunks.iter_mut() {
        let sx = matrix.size.x;
        let sy = matrix.size.y;
        if sx == 0 || sy == 0 {
            continue;
        }
        for y in 0..sy {
            for x in 0..sx {
                let wx = chunk.coord.x * sx as i32 + x as i32;
                let wy = chunk.coord.y * sy as i32 + y as i32;
                if wx < 0 || wy < 0 {
                    continue;
                }
                let ux = wx as u32;
                let uy = wy as u32;
                let Some(src) = cache.idx_xy(ux, uy) else {
                    continue;
                };
                let dst = matrix.idx(x, y);
                matrix.elevation[dst] = cache.elevation[src];
                matrix.moisture[dst] = cache.moisture[src];
                matrix.temperature[dst] = cache.temperature[src];
                matrix.family[dst] = cache.family[src];
            }
        }
    }

    gate.stamped = Some(key);

    if let Some(dirty) = raster_dirty.as_mut() {
        dirty.bump();
    }
}
