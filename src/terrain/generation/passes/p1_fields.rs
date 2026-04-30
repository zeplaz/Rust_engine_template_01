use bevy::prelude::IVec2;

use super::super::cell_matrix::ChunkCellMatrix;
use super::super::terrain_noise::NoiseSamplingTuning;
use super::super::world_generator_enhanced::{
    build_world_noise_kernels, sample_fields_at_world_tile, WorldGenParams,
};

/// Fill `elevation` / `moisture` / `temperature` from the same noise path as `generate_world`.
///
/// `tuning_overlay`: when `None`, uses [`WorldGenParams::noise_sampling`] (recommended for parity).
pub fn fill_fields(
    matrix: &mut ChunkCellMatrix,
    chunk_xy: IVec2,
    params: &WorldGenParams,
    tuning_overlay: Option<&NoiseSamplingTuning>,
) {
    let tuning = tuning_overlay.unwrap_or(&params.noise_sampling);
    let kernels = build_world_noise_kernels(params, tuning);
    for y in 0..matrix.size.y {
        for x in 0..matrix.size.x {
            let wx = chunk_xy.x * matrix.size.x as i32 + x as i32;
            let wy = chunk_xy.y * matrix.size.y as i32 + y as i32;
            let i = matrix.idx(x, y);
            let (e, m, t) = sample_fields_at_world_tile(wx, wy, params, &kernels, tuning);
            matrix.elevation[i] = e;
            matrix.moisture[i] = m;
            matrix.temperature[i] = t;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::UVec2;

    #[test]
    fn pass1_fields_deterministic() {
        let mut params = WorldGenParams::default();
        params.seed = 0xC0FFEE;
        params.width = 64;
        params.height = 64;
        let size = UVec2::new(8, 8);
        let chunk = IVec2::new(1, 2);

        let mut m1 = ChunkCellMatrix::new(size);
        let mut m2 = ChunkCellMatrix::new(size);
        fill_fields(&mut m1, chunk, &params, None);
        fill_fields(&mut m2, chunk, &params, None);
        assert_eq!(m1.elevation, m2.elevation);
        assert_eq!(m1.moisture, m2.moisture);
        assert_eq!(m1.temperature, m2.temperature);
    }

    #[test]
    fn pass1_matches_shared_sampler_for_corner_chunk() {
        let mut params = WorldGenParams::default();
        params.seed = 42;
        params.width = 32;
        params.height = 32;
        params.island_mode = false;
        let size = UVec2::new(4, 4);
        let chunk_xy = IVec2::ZERO;
        let tuning = &params.noise_sampling;
        let kernels = build_world_noise_kernels(&params, tuning);

        let mut matrix = ChunkCellMatrix::new(size);
        fill_fields(&mut matrix, chunk_xy, &params, None);

        for y in 0..size.y {
            for x in 0..size.x {
                let i = matrix.idx(x, y);
                let (e, m, t) = sample_fields_at_world_tile(
                    x as i32,
                    y as i32,
                    &params,
                    &kernels,
                    tuning,
                );
                assert_eq!(matrix.elevation[i], e);
                assert_eq!(matrix.moisture[i], m);
                assert_eq!(matrix.temperature[i], t);
            }
        }
    }
}
