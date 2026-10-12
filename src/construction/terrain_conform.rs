//! Terrain height conform for ghosts and road control points (PHASE2-BUILD-20).
//!
//! Prefer live [`WorldGenDenseTerrainCache`] elevation. Fallback stub is only used when
//! the dense height grid is absent (harness / pre-worldgen) — witness stays Open then.

use bevy::prelude::*;

use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use bevy::ecs::system::SystemParam;
use crate::terrain::generation::WorldGenDenseTerrainCache;

/// World-unit Y scale for normalized height samples.
pub const TERRAIN_CONFORM_Y_SCALE: f32 = 28.0;

/// Blocker note when live height grid is missing (witness / board Open).
pub const TERRAIN_CONFORM_HEIGHT_GRID_BLOCKER: &str =
    "PHASE2-BUILD-20 Open: WorldGenDenseTerrainCache elevation unavailable — stub fallback only";

/// Bundled params for conform sampling (keeps execute systems under the 16-param limit).
#[derive(SystemParam)]
pub struct ConstructionTerrainConform<'w> {
    pub params: Res<'w, WorldGenParams>,
    pub height_grid: Option<Res<'w, WorldGenDenseTerrainCache>>,
}

impl ConstructionTerrainConform<'_> {
    #[must_use]
    pub fn conform_y(&self, x: f32, z: f32) -> f32 {
        conform_world_y(x, z, self.params.as_ref(), self.height_grid.as_deref())
    }

    #[must_use]
    pub fn live_ready(&self) -> bool {
        live_height_grid_ready(self.height_grid.as_deref())
    }
}

/// Deterministic height stub — harness / pre-worldgen only (not production conform authority).
#[must_use]
pub fn height_norm_stub(x: f32, z: f32, params: &WorldGenParams) -> f32 {
    let nx = x / params.width.max(1) as f32;
    let nz = z / params.height.max(1) as f32;
    let h = (nx * 12.7).sin() * 0.35 + (nz * 9.3).cos() * 0.35 + ((nx + nz) * 6.1).sin() * 0.2;
    (h * 0.5 + 0.5).clamp(0.0, 1.0)
}

/// Normalized height: live dense elevation when ready, else stub.
#[must_use]
pub fn sample_height_norm(
    x: f32,
    z: f32,
    params: &WorldGenParams,
    height_grid: Option<&WorldGenDenseTerrainCache>,
) -> f32 {
    if let Some(cache) = height_grid {
        if let Some(h) = cache.sample_elevation_norm(x, z) {
            return h;
        }
    }
    height_norm_stub(x, z, params)
}

#[must_use]
pub fn conform_world_y(
    x: f32,
    z: f32,
    params: &WorldGenParams,
    height_grid: Option<&WorldGenDenseTerrainCache>,
) -> f32 {
    sample_height_norm(x, z, params, height_grid) * TERRAIN_CONFORM_Y_SCALE
}

#[must_use]
pub fn conform_vec3(
    mut world: Vec3,
    params: &WorldGenParams,
    height_grid: Option<&WorldGenDenseTerrainCache>,
) -> Vec3 {
    world.y = conform_world_y(world.x, world.z, params, height_grid);
    world
}

/// Honest witness: dense elevation grid is present and sized.
#[must_use]
pub fn live_height_grid_ready(height_grid: Option<&WorldGenDenseTerrainCache>) -> bool {
    height_grid.is_some_and(|c| c.is_live_height_grid())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conform_y_varies_with_position_stub_fallback() {
        let params = WorldGenParams {
            width: 128,
            height: 128,
            ..Default::default()
        };
        let a = conform_world_y(3.0, 7.0, &params, None);
        let b = conform_world_y(40.0, 12.0, &params, None);
        assert!(a > 0.0 && b > 0.0);
        assert!((a - b).abs() > 0.01);
        assert!(!live_height_grid_ready(None));
    }

    #[test]
    fn live_cache_sample_beats_stub() {
        let params = WorldGenParams {
            width: 4,
            height: 4,
            ..Default::default()
        };
        let mut elevation = vec![0.1f32; 16];
        elevation[1 + 2 * 4] = 0.9;
        let cache = WorldGenDenseTerrainCache {
            width: 4,
            height: 4,
            elevation,
            moisture: vec![0.0; 16],
            temperature: vec![0.0; 16],
            family: vec![crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID; 16],
            region_index: vec![0; 16],
        };
        assert!(live_height_grid_ready(Some(&cache)));
        let live = sample_height_norm(1.2, 2.4, &params, Some(&cache));
        let stub = height_norm_stub(1.2, 2.4, &params);
        assert!((live - 0.9).abs() < 1e-5);
        assert!((live - stub).abs() > 0.01);
    }
}
