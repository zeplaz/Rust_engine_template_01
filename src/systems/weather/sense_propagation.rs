//! SENSE-04 — sense-degradation relaxation over weather-chunk cells.
//!
//! P0: pure kernel. P1+: [`WeatherSenseCellField`] sole-written from stations + smoke.

use bevy::math::{IVec2, UVec2};
use bevy::prelude::*;

use crate::substrate::atmosphere::AtmosphereClipmapStack;
use crate::systems::fire::ChunkSmokeField;
use crate::terrain::generation::Chunk;
use crate::terrain::world_scale_contract::CHUNK_TILES_SIM;

use super::air_ops::{
    collect_chunk_smoke_density, sensor_range_mul, smoke01_resolve, SensorKind,
    SMOKE_SAMPLE_CELLS_PER_CHUNK,
};
use super::field::WeatherSample;
use super::polling_stations::{tile_to_weather_chunk, WeatherPollingStations};

/// Floor matching [`sensor_range_mul`] clamp.
pub const SENSE_MUL_FLOOR: f32 = 0.15;
pub const SENSE_RELAX_ITERS: u32 = 6;
pub const SENSE_RELAX_ALPHA: f32 = 1.0;

/// Seed a cell mul from weather + smoke using the EO air-ops formula (default gameplay modality).
#[must_use]
pub fn sense_seed_mul(sample: &WeatherSample, smoke01: f32, sensor: SensorKind, resist: f32) -> f32 {
    sensor_range_mul(sample, smoke01, sensor, resist)
}

/// Fixed-iteration Gauss–Seidel relaxation of sense mul over a rectangular grid.
///
/// - `seeds[i]` / `seeded[i]`: when `seeded`, cell stays pinned to `seeds[i]` every iteration.
/// - Unseeded cells start at `1.0` and blend toward the mean of existing 4-neighbours.
/// - `alpha ∈ (0,1]`: blend of neighbour mean into current value (`1` = full replace).
/// - Output clamped to `[SENSE_MUL_FLOOR, 1]`. Deterministic for identical inputs.
#[must_use]
pub fn relax_sense_degradation(
    width: usize,
    height: usize,
    seeds: &[f32],
    seeded: &[bool],
    iterations: u32,
    alpha: f32,
) -> Vec<f32> {
    let n = width.saturating_mul(height);
    debug_assert_eq!(seeds.len(), n);
    debug_assert_eq!(seeded.len(), n);
    let alpha = alpha.clamp(1e-6, 1.0);

    let mut grid = vec![1.0_f32; n];
    for i in 0..n {
        if seeded[i] {
            grid[i] = seeds[i].clamp(SENSE_MUL_FLOOR, 1.0);
        }
    }

    for _ in 0..iterations {
        for y in 0..height {
            for x in 0..width {
                let i = y * width + x;
                if seeded[i] {
                    grid[i] = seeds[i].clamp(SENSE_MUL_FLOOR, 1.0);
                    continue;
                }
                let mut sum = 0.0_f32;
                let mut count = 0_u32;
                if x > 0 {
                    sum += grid[i - 1];
                    count += 1;
                }
                if x + 1 < width {
                    sum += grid[i + 1];
                    count += 1;
                }
                if y > 0 {
                    sum += grid[i - width];
                    count += 1;
                }
                if y + 1 < height {
                    sum += grid[i + width];
                    count += 1;
                }
                if count == 0 {
                    continue;
                }
                let mean = sum / count as f32;
                grid[i] = (grid[i] * (1.0 - alpha) + mean * alpha).clamp(SENSE_MUL_FLOOR, 1.0);
            }
        }
    }

    grid
}

/// Build seed buffers from per-cell samples + smoke, then relax.
#[must_use]
pub fn relax_sense_from_samples(
    width: usize,
    height: usize,
    samples: &[WeatherSample],
    smoke01: &[f32],
    seeded: &[bool],
    sensor: SensorKind,
    resist: f32,
    iterations: u32,
    alpha: f32,
) -> Vec<f32> {
    let n = width.saturating_mul(height);
    debug_assert_eq!(samples.len(), n);
    debug_assert_eq!(smoke01.len(), n);
    let mut seeds = vec![1.0_f32; n];
    for i in 0..n {
        if seeded[i] {
            seeds[i] = sense_seed_mul(&samples[i], smoke01[i], sensor, resist);
        }
    }
    relax_sense_degradation(width, height, &seeds, seeded, iterations, alpha)
}

/// Coarse sense mul over weather-polling chunks (1:1 with stations). DERIVED — omit from save.
#[derive(Resource, Debug, Clone)]
pub struct WeatherSenseCellField {
    pub ready: bool,
    /// Min chunk corner of the dense AABB (`origin + (w,h)` exclusive).
    pub origin_chunk: IVec2,
    pub width: u32,
    pub height: u32,
    pub sense_mul: Vec<f32>,
    pub cells_written: u32,
    pub relax_iters: u32,
    pub max_delta: f32,
}

impl Default for WeatherSenseCellField {
    fn default() -> Self {
        Self {
            ready: false,
            origin_chunk: IVec2::ZERO,
            width: 0,
            height: 0,
            sense_mul: Vec::new(),
            cells_written: 0,
            relax_iters: 0,
            max_delta: 0.0,
        }
    }
}

impl WeatherSenseCellField {
    /// Sense mul at a gameplay tile via its weather chunk; `1.0` when not ready / OOB.
    #[must_use]
    pub fn mul_at_tile(&self, tile: IVec2) -> f32 {
        if !self.ready || self.width == 0 || self.height == 0 {
            return 1.0;
        }
        let chunk = tile_to_weather_chunk(tile);
        self.mul_at_chunk(chunk)
    }

    #[must_use]
    pub fn mul_at_chunk(&self, chunk: IVec2) -> f32 {
        if !self.ready || self.width == 0 || self.height == 0 {
            return 1.0;
        }
        let lx = chunk.x - self.origin_chunk.x;
        let ly = chunk.y - self.origin_chunk.y;
        if lx < 0 || ly < 0 || lx >= self.width as i32 || ly >= self.height as i32 {
            return 1.0;
        }
        let i = (ly as u32 * self.width + lx as u32) as usize;
        self.sense_mul.get(i).copied().unwrap_or(1.0)
    }
}

#[inline]
fn weather_chunk_centre_tile(chunk: IVec2) -> IVec2 {
    let c = CHUNK_TILES_SIM as i32;
    IVec2::new(chunk.x * c + c / 2, chunk.y * c + c / 2)
}

/// Sole writer of [`WeatherSenseCellField`] from stations + smoke (after polling stations).
pub fn build_weather_sense_cell_field(
    stations: Res<WeatherPollingStations>,
    stack: Option<Res<AtmosphereClipmapStack>>,
    smoke_q: Query<(&Chunk, &ChunkSmokeField)>,
    mut field: ResMut<WeatherSenseCellField>,
) {
    if !stations.ready || stations.by_chunk.is_empty() {
        *field = WeatherSenseCellField::default();
        return;
    }

    let mut min_c = IVec2::new(i32::MAX, i32::MAX);
    let mut max_c = IVec2::new(i32::MIN, i32::MIN);
    for &c in stations.by_chunk.keys() {
        min_c = min_c.min(c);
        max_c = max_c.max(c);
    }
    let width = (max_c.x - min_c.x + 1).max(0) as usize;
    let height = (max_c.y - min_c.y + 1).max(0) as usize;
    if width == 0 || height == 0 {
        *field = WeatherSenseCellField::default();
        return;
    }

    let n = width * height;
    let by_smoke = collect_chunk_smoke_density(&smoke_q);
    let stack_ref = stack.as_deref();
    let cells = UVec2::splat(SMOKE_SAMPLE_CELLS_PER_CHUNK);

    let mut seeds = vec![1.0_f32; n];
    let mut seeded = vec![false; n];
    for (&chunk, sample) in stations.by_chunk.iter() {
        let lx = (chunk.x - min_c.x) as usize;
        let ly = (chunk.y - min_c.y) as usize;
        let i = ly * width + lx;
        let smoke = smoke01_resolve(chunk, &by_smoke, stack_ref);
        // Prefer chunk-keyed smoke; fall back via centre tile if needed.
        let smoke = if smoke > 0.0 {
            smoke
        } else {
            let tile = weather_chunk_centre_tile(chunk);
            super::air_ops::smoke01_at_tile(tile, cells, &by_smoke, stack_ref)
        };
        seeds[i] = sense_seed_mul(sample, smoke, SensorKind::Eo, 0.0);
        seeded[i] = true;
    }

    let prev = field.sense_mul.clone();
    let out = relax_sense_degradation(
        width,
        height,
        &seeds,
        &seeded,
        SENSE_RELAX_ITERS,
        SENSE_RELAX_ALPHA,
    );

    let mut max_delta = 0.0_f32;
    if prev.len() == out.len()
        && field.width == width as u32
        && field.height == height as u32
        && field.origin_chunk == min_c
    {
        for (a, b) in prev.iter().zip(out.iter()) {
            max_delta = max_delta.max((a - b).abs());
        }
    } else {
        max_delta = 1.0;
    }

    field.ready = true;
    field.origin_chunk = min_c;
    field.width = width as u32;
    field.height = height as u32;
    field.cells_written = out.len() as u32;
    field.relax_iters = SENSE_RELAX_ITERS;
    field.max_delta = max_delta;
    field.sense_mul = out;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clear_sample() -> WeatherSample {
        WeatherSample::default()
    }

    #[test]
    fn clear_weather_all_ones() {
        let w = 3usize;
        let h = 3usize;
        let n = w * h;
        let samples = vec![clear_sample(); n];
        let smoke = vec![0.0_f32; n];
        let seeded = vec![true; n];
        let out = relax_sense_from_samples(
            w,
            h,
            &samples,
            &smoke,
            &seeded,
            SensorKind::Eo,
            0.0,
            4,
            1.0,
        );
        assert_eq!(out.len(), n);
        for v in &out {
            assert!((v - 1.0).abs() < 1e-5, "got {v}");
        }
    }

    #[test]
    fn single_degraded_station_diffuses() {
        let w = 3usize;
        let h = 3usize;
        let n = w * h;
        let mut seeds = vec![1.0_f32; n];
        let mut seeded = vec![false; n];
        seeds[4] = 0.2;
        seeded[4] = true;
        let out = relax_sense_degradation(w, h, &seeds, &seeded, 8, 1.0);
        assert!((out[4] - 0.2).abs() < 1e-5);
        for &i in &[1usize, 3, 5, 7] {
            assert!(out[i] < 0.999, "neighbour {i} stayed {v}", v = out[i]);
            assert!(out[i] > out[4], "neighbour should be above seed");
        }
    }

    #[test]
    fn identical_inputs_byte_identical() {
        let w = 4usize;
        let h = 3usize;
        let n = w * h;
        let mut seeds = vec![1.0_f32; n];
        let mut seeded = vec![false; n];
        seeds[0] = 0.4;
        seeded[0] = true;
        seeds[n - 1] = 0.55;
        seeded[n - 1] = true;
        let a = relax_sense_degradation(w, h, &seeds, &seeded, 6, 0.75);
        let b = relax_sense_degradation(w, h, &seeds, &seeded, 6, 0.75);
        assert_eq!(a, b);
    }

    /// S4-P4: gameplay consumers under strategic/combat/economy must not call `WeatherField::sample`.
    #[test]
    fn gameplay_modules_do_not_sample_weather_field() {
        let roots = [
            "src/strategic",
            "src/combat",
            "src/economy",
        ];
        let mut offenders = Vec::new();
        for root in roots {
            let Ok(entries) = std::fs::read_dir(root) else {
                continue;
            };
            fn walk(dir: &std::path::Path, out: &mut Vec<String>) {
                let Ok(rd) = std::fs::read_dir(dir) else {
                    return;
                };
                for e in rd.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        walk(&p, out);
                        continue;
                    }
                    if p.extension().and_then(|s| s.to_str()) != Some("rs") {
                        continue;
                    }
                    let Ok(text) = std::fs::read_to_string(&p) else {
                        continue;
                    };
                    for (i, line) in text.lines().enumerate() {
                        let t = line.trim();
                        if t.starts_with("//") {
                            continue;
                        }
                        if t.contains("WeatherField::sample")
                            || (t.contains(".sample(")
                                && text.contains("WeatherField")
                                && (t.contains("field.sample")
                                    || t.contains("weather.sample")
                                    || t.contains("f.sample")))
                        {
                            out.push(format!("{}:{}", p.display(), i + 1));
                        }
                    }
                }
            }
            walk(std::path::Path::new(root), &mut offenders);
            let _ = entries;
        }
        assert!(
            offenders.is_empty(),
            "WeatherField::sample in gameplay modules: {offenders:?}"
        );
    }

    #[test]
    fn cell_field_3x3_fixture_lookup() {
        let mut seeds = vec![1.0_f32; 9];
        let mut seeded = vec![false; 9];
        seeds[4] = 0.3;
        seeded[4] = true;
        let mul = relax_sense_degradation(3, 3, &seeds, &seeded, 4, 1.0);
        let field = WeatherSenseCellField {
            ready: true,
            origin_chunk: IVec2::new(-1, -1),
            width: 3,
            height: 3,
            sense_mul: mul.clone(),
            cells_written: 9,
            relax_iters: 4,
            max_delta: 0.0,
        };
        // Chunk (0,0) is local (1,1) → index 4
        assert!((field.mul_at_chunk(IVec2::new(0, 0)) - 0.3).abs() < 1e-5);
        assert!(field.mul_at_chunk(IVec2::new(0, -1)) < 1.0);
        assert!((field.mul_at_chunk(IVec2::new(99, 99)) - 1.0).abs() < 1e-5);
    }
}
