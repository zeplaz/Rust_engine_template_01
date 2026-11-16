//! W4 — fire → weather source terms (`plan_shared_atmosphere_weather_fire_v1.md` §2.3).
//!
//! Aggregates L0 heat / smoke / ember into a coarse grid matching [`WeatherField`] dims.
//! Sole writer: [`build_weather_source_terms`] (Frame set, after `build_atmosphere_frame`).
//! `weather_field_step` reads the resource on the next weather solve (one-step lag).

use bevy::math::{IVec2, UVec2};
use bevy::prelude::*;

use crate::substrate::atmosphere::AtmosphereClipmapStack;
use crate::systems::fire::ember_physics::plume_updraft_mps;
use crate::systems::weather::WeatherField;
use crate::terrain::generation::chunk_world_center;
use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

use super::diagnostics::AtmosphereDiagnostics;
use super::field::ATMO_GRID_ORIGIN;
use super::frame::AtmosphereFrame;
use super::pipeline::AtmospherePipelineSet;

/// Heat / smoke / ember must all clear these to count as plume core.
pub const PLUME_HEAT_MIN: f32 = 0.35;
pub const PLUME_SMOKE_MIN: f32 = 0.25;
pub const PLUME_EMBER_MIN: f32 = 0.12;
/// Contiguous L0 cells required before a component is a firestorm plume.
pub const PLUME_MIN_CELLS: u32 = 4;

/// Humidity injected per m/s of plume updraft (scaled by heat).
const MOISTURE_PER_UPDRAFT_MPS: f32 = 0.012;
/// Cloud lift from heat flux × moist air.
const CLOUD_LIFT_PER_HEAT: f32 = 0.40;
/// Aerosol → temperature shade (°C) in the weather solve.
pub const AEROSOL_TEMP_SHADE_C: f32 = 1.5;
/// Smoke / cloud overlap thresholds for dump counter.
const OVERLAP_SMOKE_MIN: f32 = 0.15;
const OVERLAP_CLOUD_MIN: f32 = 0.35;

/// Per weather-cell fire→cloud coupling inputs.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WeatherSourceCell {
    /// Normalised heat flux `0..1` (from L0 heat_distortion).
    pub heat_flux: f32,
    /// Smoke / ash aerosol `0..1`.
    pub aerosol: f32,
    /// Extra humidity to inject this weather step.
    pub moisture: f32,
    /// Peak plume updraft (m/s) contributing to this cell.
    pub updraft_mps: f32,
}

/// Coarse fire→weather terms. Sole writer: [`build_weather_source_terms`].
#[derive(Resource, Clone, Debug, Default)]
pub struct WeatherSourceTerms {
    pub dims: UVec2,
    pub cells: Vec<WeatherSourceCell>,
    /// Dump: contiguous plume components ≥ [`PLUME_MIN_CELLS`].
    pub firestorm_plumes: u32,
    /// Dump: peak `plume_updraft_mps` over plume cores.
    pub plume_updraft_peak_mps: f32,
    /// Pre-solve proposed moisture from fire coupling (CQ-36).
    pub pyro_moisture_injected: f32,
    /// Solve-applied moisture written only by weather advance (CQ-36).
    pub pyro_moisture_applied: f32,
    /// Dump: sum of cloud lift applied by the last weather solve that read these terms.
    pub pyro_cloud_delta: f32,
    /// Dump: L0 tiles with smoke+cloud both above threshold.
    pub smoke_cloud_overlap_cells: u32,
    /// Builds since start.
    pub builds: u64,
}

impl WeatherSourceTerms {
    #[must_use]
    pub fn cell(&self, idx: usize) -> WeatherSourceCell {
        self.cells.get(idx).copied().unwrap_or_default()
    }

    #[must_use]
    pub fn is_ready(&self) -> bool {
        !self.cells.is_empty() && self.dims.x > 0 && self.dims.y > 0
    }

    /// Resize / clear for a weather grid (keeps dump counters until overwritten).
    pub fn ensure_dims(&mut self, dims: UVec2) {
        let n = (dims.x * dims.y) as usize;
        if self.dims != dims || self.cells.len() != n {
            self.dims = dims;
            self.cells = vec![WeatherSourceCell::default(); n];
        } else {
            self.cells.fill(WeatherSourceCell::default());
        }
    }

    /// Record cloud / moisture actually applied by a weather solve (does not overwrite proposed).
    pub fn record_solve_deltas(&mut self, moisture_applied: f32, cloud_delta: f32) {
        self.pyro_moisture_applied = moisture_applied;
        self.pyro_cloud_delta = cloud_delta;
    }
}

/// True when an L0 tile is a plume-core candidate.
#[inline]
#[must_use]
pub fn is_plume_core(heat: f32, smoke: f32, ember: f32) -> bool {
    heat >= PLUME_HEAT_MIN && smoke >= PLUME_SMOKE_MIN && ember >= PLUME_EMBER_MIN
}

/// Contiguous 4-connected plume components over L0 lanes.
/// Returns `(mask, plume_count, peak_updraft_mps)`.
#[must_use]
pub fn detect_firestorm_plumes(
    heat: &[f32],
    smoke: &[f32],
    ember: &[f32],
    resolution: UVec2,
) -> (Vec<bool>, u32, f32) {
    let n = (resolution.x * resolution.y) as usize;
    let mut mask = vec![false; n];
    if heat.len() != n || smoke.len() != n || ember.len() != n || n == 0 {
        return (mask, 0, 0.0);
    }
    let mut visited = vec![false; n];
    let mut plume_count = 0u32;
    let mut peak_up = 0.0f32;
    let w = resolution.x as i32;
    let h = resolution.y as i32;

    for y0 in 0..h {
        for x0 in 0..w {
            let start = (y0 * w + x0) as usize;
            if visited[start] || !is_plume_core(heat[start], smoke[start], ember[start]) {
                continue;
            }
            let mut stack = vec![(x0, y0)];
            visited[start] = true;
            let mut component: Vec<usize> = Vec::new();
            let mut local_peak = 0.0f32;
            while let Some((x, y)) = stack.pop() {
                let i = (y * w + x) as usize;
                component.push(i);
                let intensity = heat[i].clamp(0.0, 1.0);
                local_peak = local_peak.max(plume_updraft_mps(intensity));
                for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x + dx;
                    let ny = y + dy;
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    let ni = (ny * w + nx) as usize;
                    if visited[ni] || !is_plume_core(heat[ni], smoke[ni], ember[ni]) {
                        continue;
                    }
                    visited[ni] = true;
                    stack.push((nx, ny));
                }
            }
            if component.len() as u32 >= PLUME_MIN_CELLS {
                plume_count += 1;
                peak_up = peak_up.max(local_peak);
                for i in component {
                    mask[i] = true;
                }
            }
        }
    }
    (mask, plume_count, peak_up)
}

/// Aggregate L0 plume cores into weather-cell source terms.
#[must_use]
pub fn aggregate_source_terms(
    field: &WeatherField,
    heat: &[f32],
    smoke: &[f32],
    _ember: &[f32],
    resolution: UVec2,
    plume_mask: &[bool],
) -> (Vec<WeatherSourceCell>, f32) {
    let n = field.cell_count();
    let mut cells = vec![WeatherSourceCell::default(); n];
    let mut counts = vec![0u32; n];
    let mut moisture_sum = 0.0f32;
    if heat.len() != plume_mask.len() {
        return (cells, 0.0);
    }
    let tiles = UVec2::splat(CHUNK_TILES_SIM);
    let origin = ATMO_GRID_ORIGIN;
    for y in 0..resolution.y {
        for x in 0..resolution.x {
            let i = (y * resolution.x + x) as usize;
            if !plume_mask.get(i).copied().unwrap_or(false) {
                continue;
            }
            let chunk = IVec2::new(origin.x + x as i32, origin.y + y as i32);
            let pos = chunk_world_center(chunk, tiles) * TILE_METRES_GAMEPLAY;
            if pos.x < 0.0 || pos.y < 0.0 || pos.x >= field.extent_m.x || pos.y >= field.extent_m.y
            {
                continue;
            }
            let ci = field.cell_index_at(pos);
            let h = heat[i].clamp(0.0, 1.0);
            let s = smoke[i].clamp(0.0, 1.0);
            let up = plume_updraft_mps(h);
            let moisture = (up * MOISTURE_PER_UPDRAFT_MPS * h).clamp(0.0, 0.25);
            let cell = &mut cells[ci];
            cell.heat_flux = cell.heat_flux.max(h);
            cell.aerosol = cell.aerosol.max(s);
            cell.updraft_mps = cell.updraft_mps.max(up);
            cell.moisture += moisture;
            counts[ci] += 1;
            moisture_sum += moisture;
        }
    }
    for (i, c) in cells.iter_mut().enumerate() {
        if counts[i] > 1 {
            c.moisture /= counts[i] as f32;
        }
        c.moisture = c.moisture.clamp(0.0, 0.25);
    }
    (cells, moisture_sum)
}

/// Cloud lift a weather solve should apply from one source cell (for tests / solve).
/// Heat / updraft always contribute a dry buoyant floor; moisture amplifies.
#[inline]
#[must_use]
pub fn pyro_cloud_lift(cell: &WeatherSourceCell, moist01: f32) -> f32 {
    let moist = moist01.clamp(0.0, 1.0);
    let heat = cell.heat_flux.clamp(0.0, 1.0);
    let up = (cell.updraft_mps / 12.0).clamp(0.0, 1.0);
    let dry = heat * CLOUD_LIFT_PER_HEAT * 0.35 + up * 0.08;
    let wet = heat * CLOUD_LIFT_PER_HEAT * moist + up * 0.15 * moist;
    dry.max(wet).clamp(0.0, 0.6)
}

fn count_smoke_cloud_overlap(frame: &AtmosphereFrame) -> u32 {
    let mut n = 0u32;
    for tile in frame.tiles() {
        if !tile.weather_valid {
            continue;
        }
        if tile.smoke >= OVERLAP_SMOKE_MIN && tile.weather.cloud >= OVERLAP_CLOUD_MIN {
            n += 1;
        }
    }
    n
}

/// Sole writer of [`WeatherSourceTerms`]. Runs after [`super::frame::build_atmosphere_frame`].
pub fn build_weather_source_terms(
    field: Option<Res<WeatherField>>,
    stack: Res<AtmosphereClipmapStack>,
    frame: Res<AtmosphereFrame>,
    mut terms: ResMut<WeatherSourceTerms>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    let Some(field) = field.filter(|f| f.is_ready()) else {
        terms.ensure_dims(UVec2::ZERO);
        terms.cells.clear();
        terms.firestorm_plumes = 0;
        terms.plume_updraft_peak_mps = 0.0;
        terms.pyro_moisture_injected = 0.0;
        // Keep last solve-applied deltas; overlap still readable from frame.
        terms.smoke_cloud_overlap_cells = count_smoke_cloud_overlap(&frame);
        diag.smoke_cloud_overlap_cells = terms.smoke_cloud_overlap_cells;
        diag.firestorm_plumes = 0;
        diag.plume_updraft_peak_mps = 0.0;
        // CQ-36: diagnostics hold solve-applied only — leave pyro_moisture_injected alone here.
        return;
    };

    let Some(level0) = stack.levels.first() else {
        return;
    };
    let res = level0.resolution;
    let mask = frame.firestorm_plume_mask();
    let plumes = frame.firestorm_plume_count();
    let peak_up = frame.plume_updraft_peak_mps();
    let (cells, moisture_sum) = aggregate_source_terms(
        &field,
        &level0.heat_distortion,
        &level0.smoke_density,
        &level0.ember_density,
        res,
        mask,
    );

    terms.ensure_dims(field.dims);
    terms.cells = cells;
    terms.firestorm_plumes = plumes;
    terms.plume_updraft_peak_mps = peak_up;
    // CQ-36: pre-solve proposed on terms only; diag gets applied from weather solve.
    terms.pyro_moisture_injected = moisture_sum;
    terms.smoke_cloud_overlap_cells = count_smoke_cloud_overlap(&frame);
    terms.builds = terms.builds.wrapping_add(1);

    diag.firestorm_plumes = plumes;
    diag.plume_updraft_peak_mps = peak_up;
    diag.smoke_cloud_overlap_cells = terms.smoke_cloud_overlap_cells;
}

pub fn coupling_terms_systems(app: &mut App) {
    app.init_resource::<WeatherSourceTerms>().add_systems(
        Update,
        build_weather_source_terms
            .after(super::frame::build_atmosphere_frame)
            .in_set(AtmospherePipelineSet::Frame),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::weather::WeatherTuning;

    fn level_fixture(w: u32, h: u32, cores: &[(u32, u32)]) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let n = (w * h) as usize;
        let mut heat = vec![0.0; n];
        let mut smoke = vec![0.0; n];
        let mut ember = vec![0.0; n];
        for &(x, y) in cores {
            let i = (y * w + x) as usize;
            heat[i] = 0.8;
            smoke[i] = 0.6;
            ember[i] = 0.4;
        }
        (heat, smoke, ember)
    }

    #[test]
    fn plume_detects_contiguous_block() {
        let (heat, smoke, ember) = level_fixture(
            8,
            8,
            &[
                (2, 2),
                (3, 2),
                (2, 3),
                (3, 3),
                (4, 2), // 5 cells
            ],
        );
        let (mask, count, peak) =
            detect_firestorm_plumes(&heat, &smoke, &ember, UVec2::new(8, 8));
        assert_eq!(count, 1);
        assert!(peak >= plume_updraft_mps(0.8) - 1e-3);
        assert_eq!(mask.iter().filter(|m| **m).count(), 5);
    }

    #[test]
    fn small_cluster_is_not_a_plume() {
        let (heat, smoke, ember) = level_fixture(8, 8, &[(1, 1), (1, 2)]);
        let (_, count, _) = detect_firestorm_plumes(&heat, &smoke, &ember, UVec2::new(8, 8));
        assert_eq!(count, 0);
    }

    #[test]
    fn fire_terms_raise_cloud_vs_baseline() {
        let tune = WeatherTuning::default();
        let extent = Vec2::splat(8.0 * 32.0 * 16.0);
        let field = WeatherField::new(42, extent, &tune, None, Default::default(), 0);
        let n = field.cell_count();
        let mut terms = WeatherSourceTerms {
            dims: field.dims,
            cells: vec![WeatherSourceCell::default(); n],
            ..Default::default()
        };
        // Inject a strong plume into the centre weather cell.
        let mid = field.cell_index_at(extent * 0.5);
        terms.cells[mid] = WeatherSourceCell {
            heat_flux: 0.9,
            aerosol: 0.7,
            moisture: 0.12,
            updraft_mps: plume_updraft_mps(0.9),
        };
        // Dry mid-range start so spin-up saturation does not hide the pyro lift.
        let mut from = field.next.clone();
        from.humidity.fill(0.45);
        from.cloud.fill(0.15);
        from.rain_mm_h.fill(0.0);
        let base = field.solve(&from, &tune, field.step_secs as f32);
        let (with, (moisture, cloud_delta)) =
            field.solve_with_terms(&from, &tune, field.step_secs as f32, Some(&terms));
        assert!(moisture > 0.0, "terms must inject moisture");
        assert!(cloud_delta > 0.0, "terms must record pyro cloud lift");
        assert!(
            with.cloud[mid] > base.cloud[mid],
            "pyro cloud should lift over fire cell ({} vs {})",
            with.cloud[mid],
            base.cloud[mid]
        );
        assert!(with.humidity[mid] > base.humidity[mid]);
    }

    #[test]
    fn solve_with_terms_is_deterministic() {
        let tune = WeatherTuning::default();
        let extent = Vec2::splat(4.0 * 32.0 * 16.0);
        let field = WeatherField::new(7, extent, &tune, None, Default::default(), 0);
        let mut terms = WeatherSourceTerms {
            dims: field.dims,
            cells: vec![WeatherSourceCell::default(); field.cell_count()],
            ..Default::default()
        };
        terms.cells[0] = WeatherSourceCell {
            heat_flux: 0.7,
            aerosol: 0.5,
            moisture: 0.08,
            updraft_mps: 8.0,
        };
        let (a, _) = field.solve_with_terms(&field.next, &tune, 60.0, Some(&terms));
        let (b, _) = field.solve_with_terms(&field.next, &tune, 60.0, Some(&terms));
        assert_eq!(a.cloud, b.cloud);
        assert_eq!(a.humidity, b.humidity);
        assert_eq!(a.temp_c, b.temp_c);
    }
}
