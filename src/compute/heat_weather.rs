//! **DH-03a** — weather RO → dense heat (wind advection + rain damp).
//!
//! Weather never writes fire sim; heat field remains the sole dense writer.
//! Inputs come from [`ClimateVisualAggregate`] (already aggregated — no ChunkWeather scan here).

use bevy::math::Vec2;

use crate::render::ClimateVisualAggregate;

use super::heat_diffusion_dense::DenseHeatGrid;

/// Rain damp scale: at rain01=1, multiply heat/smoke by this floor blend.
pub const WEATHER_RAIN_DAMP: f32 = 0.35;
/// Wind advection fraction of cell mass moved per step at |w|=20 m/s.
pub const WEATHER_WIND_FRAC_AT_20: f32 = 0.35;

/// Apply RO weather to a dense heat grid (after separable diffusion + source inject).
pub fn apply_weather_to_dense_heat(
    grid: &mut DenseHeatGrid,
    wind_mps: Vec2,
    rain01: f32,
) {
    let rain01 = rain01.clamp(0.0, 1.0);
    let damp = (1.0 - WEATHER_RAIN_DAMP * rain01).clamp(0.5, 1.0);
    for v in &mut grid.heat {
        *v = (*v * damp).clamp(0.0, 1.5);
    }
    for v in &mut grid.smoke {
        *v = (*v * damp).clamp(0.0, 1.0);
    }

    let speed = wind_mps.length();
    if speed < 0.5 || grid.width == 0 || grid.height == 0 {
        return;
    }
    let (adx, ady) = if wind_mps.x.abs() >= wind_mps.y.abs() {
        (wind_mps.x.signum() as i32, 0)
    } else {
        (0, wind_mps.y.signum() as i32)
    };
    if adx == 0 && ady == 0 {
        return;
    }
    let frac = (speed / 20.0 * WEATHER_WIND_FRAC_AT_20).clamp(0.0, WEATHER_WIND_FRAC_AT_20);
    advect_field(&mut grid.heat, grid.width as usize, grid.height as usize, adx, ady, frac);
    advect_field(&mut grid.smoke, grid.width as usize, grid.height as usize, adx, ady, frac);
}

/// Convenience: pull means from climate aggregate (RO).
pub fn apply_climate_to_dense_heat(grid: &mut DenseHeatGrid, climate: &ClimateVisualAggregate) {
    apply_weather_to_dense_heat(grid, climate.mean_wind_mps, climate.mean_rain);
}

/// Upwind-ish lattice advection: `dst = (1-f)*src + f*src[upstream]`.
fn advect_field(buf: &mut [f32], w: usize, h: usize, adx: i32, ady: i32, frac: f32) {
    let prev = buf.to_vec();
    for y in 0..h {
        for x in 0..w {
            let ux = (x as i32 - adx).clamp(0, w as i32 - 1) as usize;
            let uy = (y as i32 - ady).clamp(0, h as i32 - 1) as usize;
            let i = y * w + x;
            let up = prev[uy * w + ux];
            buf[i] = prev[i] * (1.0 - frac) + up * frac;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::IVec2;
    use crate::compute::heat_diffusion_dense::DenseHeatGrid;

    #[test]
    fn tac03_dh03a() {
        let mut grid = DenseHeatGrid::new(IVec2::ZERO, 5, 5);
        let c = grid.idx(2, 2);
        grid.heat[c] = 1.0;
        grid.smoke[c] = 0.8;

        // Rain damps.
        let mut wet = grid.clone();
        apply_weather_to_dense_heat(&mut wet, Vec2::ZERO, 1.0);
        assert!(wet.heat[c] < grid.heat[c]);
        assert!(wet.smoke[c] < grid.smoke[c]);

        // East wind moves heat toward +X.
        let mut blown = grid.clone();
        apply_weather_to_dense_heat(&mut blown, Vec2::new(20.0, 0.0), 0.0);
        let east = grid.idx(3, 2);
        assert!(
            blown.heat[east] > 0.0,
            "wind advection should deposit heat downwind"
        );
        assert!(blown.heat[c] < grid.heat[c]);

        // Deterministic.
        let mut a = grid.clone();
        let mut b = grid.clone();
        apply_weather_to_dense_heat(&mut a, Vec2::new(8.0, -2.0), 0.4);
        apply_weather_to_dense_heat(&mut b, Vec2::new(8.0, -2.0), 0.4);
        assert_eq!(a.heat, b.heat);
        assert_eq!(a.smoke, b.smoke);
    }
}
