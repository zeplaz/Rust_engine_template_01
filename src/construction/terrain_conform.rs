//! Terrain height conform for ghosts and road control points (PHASE2-BUILD-20).

use bevy::prelude::*;

use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// World-unit Y scale for normalized height samples.
pub const TERRAIN_CONFORM_Y_SCALE: f32 = 28.0;

/// Deterministic height stub (until live height_grid resource is public on sim terrain).
#[must_use]
pub fn height_norm_stub(x: f32, z: f32, params: &WorldGenParams) -> f32 {
    let nx = x / params.width.max(1) as f32;
    let nz = z / params.height.max(1) as f32;
    let h = (nx * 12.7).sin() * 0.35 + (nz * 9.3).cos() * 0.35 + ((nx + nz) * 6.1).sin() * 0.2;
    (h * 0.5 + 0.5).clamp(0.0, 1.0)
}

#[must_use]
pub fn conform_world_y(x: f32, z: f32, params: &WorldGenParams) -> f32 {
    height_norm_stub(x, z, params) * TERRAIN_CONFORM_Y_SCALE
}

#[must_use]
pub fn conform_vec3(mut world: Vec3, params: &WorldGenParams) -> Vec3 {
    world.y = conform_world_y(world.x, world.z, params);
    world
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conform_y_varies_with_position() {
        let params = WorldGenParams {
            width: 128,
            height: 128,
            ..Default::default()
        };
        let a = conform_world_y(3.0, 7.0, &params);
        let b = conform_world_y(40.0, 12.0, &params);
        assert!(a > 0.0 && b > 0.0);
        assert!((a - b).abs() > 0.01);
    }
}
