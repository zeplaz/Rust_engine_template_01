//! `AtmosphereFrame` — the one derived, revision-keyed atmospheric read path
//! (`plan_shared_atmosphere_weather_fire_v1.md` §2.2, phase W2).
//!
//! Authorities are unchanged: [`WeatherField`] owns synoptic weather, [`AtmosphereClipmapStack`] owns
//! fire-borne tracers (smoke / ash / ember / heat / toxicity). This frame **only reads** them and is
//! rebuilt by exactly one system, [`build_atmosphere_frame`], in [`AtmospherePipelineSet::Frame`]
//! (after `WindAdvect`, before `VisualExtract`). Consumers read the frame; nothing writes back.
//!
//! - Chunk-indexed like L0 (`atmos_chunk_to_tile`): each tile holds the weather sampled **once** at the
//!   chunk centre (same position as `project_chunk_weather` / L0 advect) plus the L0 tracers and derived
//!   scalars ([`AtmosphereTile::visibility_m_combined`], icing risk, storm class).
//! - Key: `(WeatherField.revision, AtmosphereClipmapStack.revision, SimTimeMicros)`. Rebuild only when a
//!   key part moves. Tracer revision is the stack's native counter (W1), not ECS change detection.
//! - Not saved: rebuilt from the two saved authorities after load.

use bevy::math::{IVec2, UVec2, Vec2};
use bevy::prelude::*;

use crate::substrate::atmosphere::AtmosphereClipmapStack;
use crate::systems::sim_control::SimTimeMicros;
use crate::systems::weather::{
    icing_risk_for_sample, tile_to_weather_chunk, WeatherField, WeatherSample, WeatherTuning,
    CHUNK_RAIN_FULL_SCALE_MM_H, CHUNK_WIND_FULL_SCALE_MPS,
};
use crate::terrain::generation::{chunk_world_center, chunk_world_origin};
use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

use super::coupling_terms::detect_firestorm_plumes;
use super::field::{atmos_chunk_to_tile_in, ATMO_GRID_ORIGIN, ATMO_GRID_SIZE};
use super::pipeline::AtmospherePipelineSet;
use super::storm_class::{classify_storm_with_context, StormClass, StormClassThresholds};

/// Default visibility divisor weight for L0 smoke (matches `weather_v1.ron` `k_smoke`).
pub const VIS_SMOKE_K: f32 = 2.5;
/// Default visibility divisor weight for L0 ash (matches `weather_v1.ron` `k_ash`).
pub const VIS_ASH_K: f32 = 1.5;

/// Metres spanned by one chunk (= one L0 tile) edge.
const CHUNK_M: f32 = CHUNK_TILES_SIM as f32 * TILE_METRES_GAMEPLAY;

/// What the frame was last built from. Equal key ⇒ no rebuild.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AtmosphereFrameKey {
    pub weather_revision: u64,
    pub tracer_revision: u64,
    pub sim_micros: u64,
}

/// One chunk-indexed (L0 tile) atmosphere column.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtmosphereTile {
    /// Weather at the chunk centre, sampled once per build. Meaningful iff [`Self::weather_valid`].
    pub weather: WeatherSample,
    /// False for tiles outside the weather field extent (tracers are still valid).
    pub weather_valid: bool,
    pub smoke: f32,
    pub ash: f32,
    pub ember: f32,
    pub heat: f32,
    pub toxicity: f32,
    /// `weather.visibility_m` reduced by smoke and ash.
    pub visibility_m_combined: f32,
    /// Icing risk `0..1` at [`WeatherTuning::icing_ref_alt_m`] (W3).
    pub icing_risk: f32,
    /// Derived storm class (W3 Developing/Thunder; W4 FirestormPlume; W5 HurricaneClass).
    pub storm_class: StormClass,
    /// Max vortex influence from [`WeatherField`] systems at this tile (W5 classifier input).
    pub vortex_influence: f32,
}

impl Default for AtmosphereTile {
    fn default() -> Self {
        let weather = WeatherSample::default();
        Self {
            visibility_m_combined: weather.visibility_m,
            weather,
            weather_valid: false,
            smoke: 0.0,
            ash: 0.0,
            ember: 0.0,
            heat: 0.0,
            toxicity: 0.0,
            icing_risk: 0.0,
            storm_class: StormClass::Clear,
            vortex_influence: 0.0,
        }
    }
}

/// `weather_visibility / (1 + k_smoke·smoke + k_ash·ash)` with default k weights.
#[inline]
#[must_use]
pub fn combine_visibility_m(weather_visibility_m: f32, smoke: f32, ash: f32) -> f32 {
    combine_visibility_m_k(weather_visibility_m, smoke, ash, VIS_SMOKE_K, VIS_ASH_K)
}

/// Combined visibility with explicit smoke/ash weights from weather tuning.
#[inline]
#[must_use]
pub fn combine_visibility_m_k(
    weather_visibility_m: f32,
    smoke: f32,
    ash: f32,
    k_smoke: f32,
    k_ash: f32,
) -> f32 {
    weather_visibility_m
        / (1.0 + k_smoke.max(0.0) * smoke.clamp(0.0, 1.0) + k_ash.max(0.0) * ash.clamp(0.0, 1.0))
}

/// Means over weather-valid tiles (replaces the `ChunkWeather` scan in `ClimateVisualAggregate`).
///
/// Scales match `ChunkWeather` (`rain01 = mm/h / 10`, `wind_speed01 = |wind| / 20 m/s`).
/// `snow01` is **snowfall** intensity (rate when snowing), not ground cover.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AtmosphereWeatherMeans {
    pub tiles: u32,
    pub rain01: f32,
    pub snow01: f32,
    pub fog: f32,
    pub wind_speed01: f32,
    pub wind_mps: Vec2,
    pub lightning: f32,
    pub cloud: f32,
}

/// Cheap counters for the review dump (`atmosphere.*`); written only by the frame builder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtmosphereFrameStats {
    /// Rebuilds since start (`frame_builds`).
    pub builds: u64,
    /// Rebuilds that re-sampled [`WeatherField`] (the rest only re-copied tracers).
    pub weather_resamples: u64,
    /// `WeatherField::sample` calls issued by the most recent weather resample.
    pub weather_samples_last: u32,
    /// Direct `WeatherField` reads by migrated consumers (climate / camera-local / air_ops).
    /// Stays `0` once those paths read [`AtmosphereFrame`] only.
    pub legacy_weather_direct_reads: u64,
    pub vis_weather_min_m: f32,
    pub vis_combined_min_m: f32,
    /// Largest `weather − combined` visibility over tiles (smoke/ash effect).
    pub vis_smoke_delta_max_m: f32,
    /// Max tile `icing_risk` (W3).
    pub icing_risk_max: f32,
    /// Weather-valid tiles with `icing_risk` above the tuning ground threshold (W3).
    pub icing_grounded_samples: u32,
    /// Weather-valid tiles classified Developing (W3).
    pub storm_developing_cells: u32,
    /// Weather-valid tiles classified Thunderstorm (W3).
    pub storm_thunder_cells: u32,
    /// Weather-valid tiles classified FirestormPlume (W4).
    pub storm_firestorm_cells: u32,
    /// Weather-valid tiles classified HurricaneClass (W5).
    pub storm_hurricane_cells: u32,
}

impl Default for AtmosphereFrameStats {
    fn default() -> Self {
        let clear = WeatherSample::default().visibility_m;
        Self {
            builds: 0,
            weather_resamples: 0,
            weather_samples_last: 0,
            legacy_weather_direct_reads: 0,
            vis_weather_min_m: clear,
            vis_combined_min_m: clear,
            vis_smoke_delta_max_m: 0.0,
            icing_risk_max: 0.0,
            icing_grounded_samples: 0,
            storm_developing_cells: 0,
            storm_thunder_cells: 0,
            storm_firestorm_cells: 0,
            storm_hurricane_cells: 0,
        }
    }
}

/// Inputs that decide whether the frame is stale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AtmosphereFrameInputs {
    /// `None` when no [`WeatherField`] resource exists.
    pub weather_revision: Option<u64>,
    pub sim_micros: u64,
    /// [`AtmosphereClipmapStack::revision`] at the candidate build.
    pub tracer_revision: u64,
}

/// Derived per-frame atmosphere snapshot. Sole writer: [`build_atmosphere_frame`].
#[derive(Resource, Clone, Debug)]
pub struct AtmosphereFrame {
    key: AtmosphereFrameKey,
    revision: u64,
    initialised: bool,
    weather_ready: bool,
    origin: IVec2,
    size: UVec2,
    tiles: Vec<AtmosphereTile>,
    centre_chunk: IVec2,
    means: AtmosphereWeatherMeans,
    stats: AtmosphereFrameStats,
}

impl Default for AtmosphereFrame {
    fn default() -> Self {
        Self {
            key: AtmosphereFrameKey::default(),
            revision: 0,
            initialised: false,
            weather_ready: false,
            origin: ATMO_GRID_ORIGIN,
            size: ATMO_GRID_SIZE,
            tiles: Vec::new(),
            centre_chunk: IVec2::ZERO,
            means: AtmosphereWeatherMeans::default(),
            stats: AtmosphereFrameStats::default(),
        }
    }
}

impl AtmosphereFrame {
    /// Bumped on every rebuild; presentation compares against this.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    #[must_use]
    pub fn key(&self) -> AtmosphereFrameKey {
        self.key
    }

    /// True once built from a ready [`WeatherField`].
    #[must_use]
    pub fn is_weather_ready(&self) -> bool {
        self.weather_ready
    }

    /// True once built at least once (tracers readable).
    #[must_use]
    pub fn is_built(&self) -> bool {
        self.initialised
    }

    #[must_use]
    pub fn stats(&self) -> &AtmosphereFrameStats {
        &self.stats
    }

    #[must_use]
    pub fn weather_means(&self) -> &AtmosphereWeatherMeans {
        &self.means
    }

    /// Chunk containing the weather field centre (air-ops station probe).
    #[must_use]
    pub fn centre_chunk(&self) -> IVec2 {
        self.centre_chunk
    }

    #[must_use]
    pub fn tiles(&self) -> &[AtmosphereTile] {
        &self.tiles
    }

    #[must_use]
    pub fn size(&self) -> UVec2 {
        self.size
    }

    /// Column for a chunk, `None` outside the L0 window or before the first build.
    #[must_use]
    pub fn tile_at_chunk(&self, chunk: IVec2) -> Option<&AtmosphereTile> {
        let (x, y) = atmos_chunk_to_tile_in(self.origin, self.size, chunk)?;
        self.tiles.get((y * self.size.x + x) as usize)
    }

    /// Column for a gameplay tile (via its owning chunk).
    #[must_use]
    pub fn tile_at_tile(&self, tile: IVec2) -> Option<&AtmosphereTile> {
        self.tile_at_chunk(tile_to_weather_chunk(tile))
    }

    #[must_use]
    pub fn smoke01_at_chunk(&self, chunk: IVec2) -> Option<f32> {
        self.tile_at_chunk(chunk).map(|t| t.smoke.clamp(0.0, 1.0))
    }

    #[must_use]
    pub fn smoke01_at_tile(&self, tile: IVec2) -> Option<f32> {
        self.tile_at_tile(tile).map(|t| t.smoke.clamp(0.0, 1.0))
    }

    /// Weather at a chunk centre; `None` when the chunk has no weather sample.
    #[must_use]
    pub fn weather_at_chunk(&self, chunk: IVec2) -> Option<WeatherSample> {
        self.tile_at_chunk(chunk)
            .filter(|t| t.weather_valid)
            .map(|t| t.weather)
    }

    /// Weather at a world position (metres): bilinear over the four nearest chunk-centre samples,
    /// skipping tiles without weather. `None` when no neighbour has a sample.
    #[must_use]
    pub fn weather_at_pos_m(&self, pos_m: Vec2) -> Option<WeatherSample> {
        if !self.weather_ready || self.tiles.is_empty() {
            return None;
        }
        let fx = pos_m.x / CHUNK_M - self.origin.x as f32 - 0.5;
        let fy = pos_m.y / CHUNK_M - self.origin.y as f32 - 0.5;
        let (x0, y0) = (fx.floor(), fy.floor());
        let (tx, ty) = (fx - x0, fy - y0);
        let max_x = self.size.x as i32 - 1;
        let max_y = self.size.y as i32 - 1;

        let mut acc = WeatherAccum::default();
        for (dx, dy, w) in [
            (0, 0, (1.0 - tx) * (1.0 - ty)),
            (1, 0, tx * (1.0 - ty)),
            (0, 1, (1.0 - tx) * ty),
            (1, 1, tx * ty),
        ] {
            if w <= 0.0 {
                continue;
            }
            let ix = (x0 as i32 + dx).clamp(0, max_x) as u32;
            let iy = (y0 as i32 + dy).clamp(0, max_y) as u32;
            let tile = &self.tiles[(iy * self.size.x + ix) as usize];
            if tile.weather_valid {
                acc.add(&tile.weather, w);
            }
        }
        acc.finish()
    }

    /// FNV-1a over every tile's float bits in index order: identical inputs ⇒ identical hash.
    #[must_use]
    pub fn content_hash(&self) -> u64 {
        let mut h = Fnv::new();
        h.u32(self.size.x);
        h.u32(self.size.y);
        h.u32(u32::from(self.weather_ready));
        for t in &self.tiles {
            h.u32(u32::from(t.weather_valid));
            let w = &t.weather;
            for v in [
                w.wind_mps.x,
                w.wind_mps.y,
                w.gust_mps,
                w.temp_c,
                w.humidity,
                w.cloud,
                w.rain_mm_h,
                w.fog,
                w.visibility_m,
                w.lightning,
                w.pressure_hpa,
                t.smoke,
                t.ash,
                t.ember,
                t.heat,
                t.toxicity,
                t.visibility_m_combined,
                t.icing_risk,
            ] {
                h.u32(v.to_bits());
            }
            h.u32(u32::from(w.snow));
            h.u32(t.storm_class as u32);
        }
        h.finish()
    }

    /// Test-only: weather field from `weather_seed`, smoke at L0 tile (0, 0), sim time `0`.
    #[cfg(test)]
    pub fn rebuild_for_tests_with_smoke(&mut self, weather_seed: u64, smoke: f32) {
        let tuning = WeatherTuning::default();
        let extent = Vec2::splat(8.0 * CHUNK_M);
        let field = WeatherField::new(
            weather_seed,
            extent,
            &tuning,
            None,
            Default::default(),
            0,
        );
        let mut stack = AtmosphereClipmapStack::default();
        let l0 = &mut stack.levels[0];
        let i = (0 * l0.resolution.x + 0) as usize;
        l0.smoke_density[i] = smoke;
        stack.bump_revision();
        let _ = self.rebuild(Some(&field), &stack, 0, &tuning);
    }

    /// Would a rebuild with these inputs change the frame? Read-only (does not flag ECS change).
    #[must_use]
    pub fn rebuild_needed(&self, inputs: &AtmosphereFrameInputs) -> bool {
        !self.initialised
            || inputs.tracer_revision != self.key.tracer_revision
            || self.weather_stale(inputs)
    }

    fn weather_stale(&self, inputs: &AtmosphereFrameInputs) -> bool {
        match inputs.weather_revision {
            Some(rev) => rev != self.key.weather_revision || inputs.sim_micros != self.key.sim_micros,
            None => false,
        }
    }

    /// Rebuild from the authorities when [`Self::rebuild_needed`]. Returns whether it rebuilt.
    ///
    /// Weather is re-sampled only when its revision or the sim time moved; otherwise only the tracer
    /// lanes are re-copied. Tracer staleness is keyed by [`AtmosphereClipmapStack::revision`].
    /// `tuning` supplies combined-visibility weights, icing band, and storm-class thresholds (W3).
    pub fn rebuild(
        &mut self,
        weather: Option<&WeatherField>,
        stack: &AtmosphereClipmapStack,
        sim_micros: u64,
        tuning: &WeatherTuning,
    ) -> bool {
        let inputs = AtmosphereFrameInputs {
            weather_revision: weather.map(|f| f.revision),
            sim_micros,
            tracer_revision: stack.revision,
        };
        if !self.rebuild_needed(&inputs) {
            return false;
        }

        let n = (self.size.x * self.size.y) as usize;
        if self.tiles.len() != n {
            self.tiles = vec![AtmosphereTile::default(); n];
        }

        let resample = !self.initialised || self.weather_stale(&inputs);
        if resample {
            self.resample_weather(weather, sim_micros);
        }
        self.copy_tracers(stack);
        self.derive(tuning);

        self.key = AtmosphereFrameKey {
            weather_revision: weather.map_or(0, |f| f.revision),
            tracer_revision: stack.revision,
            sim_micros,
        };
        self.initialised = true;
        self.revision = self.revision.wrapping_add(1);
        self.stats.builds = self.stats.builds.wrapping_add(1);
        true
    }

    fn resample_weather(&mut self, weather: Option<&WeatherField>, sim_micros: u64) {
        let field = weather.filter(|f| f.is_ready());
        self.weather_ready = field.is_some();
        let mut samples = 0u32;

        if let Some(field) = field {
            let t = SimTimeMicros(sim_micros);
            let chunk_size = UVec2::splat(CHUNK_TILES_SIM);
            for y in 0..self.size.y {
                for x in 0..self.size.x {
                    let chunk = IVec2::new(self.origin.x + x as i32, self.origin.y + y as i32);
                    let tile = &mut self.tiles[(y * self.size.x + x) as usize];
                    let o = chunk_world_origin(chunk, chunk_size) * TILE_METRES_GAMEPLAY;
                    if o.x < 0.0 || o.y < 0.0 || o.x >= field.extent_m.x || o.y >= field.extent_m.y {
                        tile.weather_valid = false;
                        tile.weather = WeatherSample::default();
                        tile.vortex_influence = 0.0;
                        continue;
                    }
                    let pos = chunk_world_center(chunk, chunk_size) * TILE_METRES_GAMEPLAY;
                    tile.weather = field.sample(pos, t);
                    tile.vortex_influence = field.vortex_influence_at(pos);
                    tile.weather_valid = true;
                    samples += 1;
                }
            }
            let centre_m = field.extent_m * 0.5;
            self.centre_chunk = tile_to_weather_chunk(IVec2::new(
                (centre_m.x / TILE_METRES_GAMEPLAY).floor() as i32,
                (centre_m.y / TILE_METRES_GAMEPLAY).floor() as i32,
            ));
        } else {
            for tile in &mut self.tiles {
                tile.weather_valid = false;
                tile.weather = WeatherSample::default();
                tile.vortex_influence = 0.0;
            }
            self.centre_chunk = IVec2::ZERO;
        }
        self.stats.weather_resamples = self.stats.weather_resamples.wrapping_add(1);
        self.stats.weather_samples_last = samples;
    }

    fn copy_tracers(&mut self, stack: &AtmosphereClipmapStack) {
        let level0 = stack.levels.first();
        for y in 0..self.size.y {
            for x in 0..self.size.x {
                let tile = &mut self.tiles[(y * self.size.x + x) as usize];
                let lane = |data: Option<&Vec<f32>>| -> f32 {
                    let Some(level) = level0 else { return 0.0 };
                    if x >= level.resolution.x || y >= level.resolution.y {
                        return 0.0;
                    }
                    data.and_then(|d| d.get((y * level.resolution.x + x) as usize))
                        .copied()
                        .unwrap_or(0.0)
                };
                tile.smoke = lane(level0.map(|l| &l.smoke_density));
                tile.ash = lane(level0.map(|l| &l.ash_density));
                tile.ember = lane(level0.map(|l| &l.ember_density));
                tile.heat = lane(level0.map(|l| &l.heat_distortion));
                tile.toxicity = lane(level0.map(|l| &l.toxicity));
            }
        }
    }

    /// Derived scalars + aggregates from the tile payload (pure function of tiles + tuning).
    fn derive(&mut self, tuning: &WeatherTuning) {
        let mut means = AtmosphereWeatherMeans::default();
        let mut vis_w_min = f32::INFINITY;
        let mut vis_c_min = f32::INFINITY;
        let mut delta_max = 0.0f32;
        let mut icing_max = 0.0f32;
        let mut icing_grounded = 0u32;
        let mut storm_developing = 0u32;
        let mut storm_thunder = 0u32;
        let mut storm_firestorm = 0u32;
        let mut storm_hurricane = 0u32;
        let storm_thr = StormClassThresholds::from_tuning(tuning);
        let icing_alt = tuning.icing_ref_alt_m;
        let icing_ground_thr = tuning.icing_ground_threshold;

        // W4: contiguous plume mask over the tile tracer lanes (same indexing as L0).
        let n = self.tiles.len();
        let mut heat = vec![0.0f32; n];
        let mut smoke = vec![0.0f32; n];
        let mut ember = vec![0.0f32; n];
        for (i, tile) in self.tiles.iter().enumerate() {
            heat[i] = tile.heat;
            smoke[i] = tile.smoke;
            ember[i] = tile.ember;
        }
        let (plume_mask, _, _) = detect_firestorm_plumes(&heat, &smoke, &ember, self.size);

        for (i, tile) in self.tiles.iter_mut().enumerate() {
            tile.visibility_m_combined = combine_visibility_m_k(
                tile.weather.visibility_m,
                tile.smoke,
                tile.ash,
                tuning.k_smoke,
                tuning.k_ash,
            );
            // Contiguous mask only — isolated cores stay non-firestorm.
            let in_plume = plume_mask.get(i).copied().unwrap_or(false);
            if tile.weather_valid {
                tile.icing_risk = icing_risk_for_sample(&tile.weather, icing_alt, tuning);
                tile.storm_class = classify_storm_with_context(
                    &tile.weather,
                    &storm_thr,
                    in_plume,
                    tile.vortex_influence,
                );
            } else {
                tile.icing_risk = 0.0;
                tile.storm_class = StormClass::Clear;
            }
            if !tile.weather_valid {
                continue;
            }
            let w = &tile.weather;
            let rate01 = (w.rain_mm_h / CHUNK_RAIN_FULL_SCALE_MM_H).clamp(0.0, 1.0);
            means.tiles += 1;
            means.rain01 += if w.snow { 0.0 } else { rate01 };
            means.snow01 += if w.snow { rate01 } else { 0.0 };
            means.fog += w.fog.clamp(0.0, 1.0);
            means.wind_speed01 += (w.wind_mps.length() / CHUNK_WIND_FULL_SCALE_MPS).clamp(0.0, 1.0);
            means.wind_mps += w.wind_mps;
            means.lightning += w.lightning.clamp(0.0, 1.0);
            means.cloud += w.cloud.clamp(0.0, 1.0);
            vis_w_min = vis_w_min.min(w.visibility_m);
            vis_c_min = vis_c_min.min(tile.visibility_m_combined);
            delta_max = delta_max.max(w.visibility_m - tile.visibility_m_combined);
            icing_max = icing_max.max(tile.icing_risk);
            if tile.icing_risk > icing_ground_thr {
                icing_grounded += 1;
            }
            match tile.storm_class {
                StormClass::Developing => storm_developing += 1,
                StormClass::Thunderstorm => storm_thunder += 1,
                StormClass::FirestormPlume => storm_firestorm += 1,
                StormClass::HurricaneClass => storm_hurricane += 1,
                StormClass::Clear => {}
            }
        }

        if means.tiles > 0 {
            let inv = 1.0 / means.tiles as f32;
            means.rain01 *= inv;
            means.snow01 *= inv;
            means.fog *= inv;
            means.wind_speed01 *= inv;
            means.wind_mps *= inv;
            means.lightning *= inv;
            means.cloud *= inv;
        }
        self.means = means;
        if vis_w_min.is_finite() {
            self.stats.vis_weather_min_m = vis_w_min;
            self.stats.vis_combined_min_m = vis_c_min;
        } else {
            let clear = WeatherSample::default().visibility_m;
            self.stats.vis_weather_min_m = clear;
            self.stats.vis_combined_min_m = clear;
        }
        self.stats.vis_smoke_delta_max_m = delta_max;
        self.stats.icing_risk_max = icing_max;
        self.stats.icing_grounded_samples = icing_grounded;
        self.stats.storm_developing_cells = storm_developing;
        self.stats.storm_thunder_cells = storm_thunder;
        self.stats.storm_firestorm_cells = storm_firestorm;
        self.stats.storm_hurricane_cells = storm_hurricane;
    }
}

/// Weighted sum of weather samples for [`AtmosphereFrame::weather_at_pos_m`].
#[derive(Default)]
struct WeatherAccum {
    w: f32,
    wind: Vec2,
    gust: f32,
    temp: f32,
    humidity: f32,
    cloud: f32,
    rain: f32,
    fog: f32,
    vis: f32,
    lightning: f32,
    pressure: f32,
}

impl WeatherAccum {
    fn add(&mut self, s: &WeatherSample, w: f32) {
        self.w += w;
        self.wind += s.wind_mps * w;
        self.gust += s.gust_mps * w;
        self.temp += s.temp_c * w;
        self.humidity += s.humidity * w;
        self.cloud += s.cloud * w;
        self.rain += s.rain_mm_h * w;
        self.fog += s.fog * w;
        self.vis += s.visibility_m * w;
        self.lightning += s.lightning * w;
        self.pressure += s.pressure_hpa * w;
    }

    fn finish(self) -> Option<WeatherSample> {
        if self.w <= 1e-6 {
            return None;
        }
        let inv = 1.0 / self.w;
        let temp_c = self.temp * inv;
        let rain_mm_h = self.rain * inv;
        Some(WeatherSample {
            wind_mps: self.wind * inv,
            gust_mps: self.gust * inv,
            temp_c,
            humidity: self.humidity * inv,
            cloud: self.cloud * inv,
            rain_mm_h,
            // Same rule as `WeatherField::sample`, applied to the blended values.
            snow: temp_c < 0.0 && rain_mm_h > 0.05,
            fog: self.fog * inv,
            visibility_m: self.vis * inv,
            lightning: self.lightning * inv,
            pressure_hpa: self.pressure * inv,
        })
    }
}

struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
    fn u32(&mut self, v: u32) {
        for b in v.to_le_bytes() {
            self.0 ^= u64::from(b);
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

/// Sole writer of [`AtmosphereFrame`]. Reads [`WeatherField`] + [`AtmosphereClipmapStack`] only.
pub fn build_atmosphere_frame(
    field: Option<Res<WeatherField>>,
    stack: Res<AtmosphereClipmapStack>,
    sim_time: Option<Res<SimTimeMicros>>,
    tuning: Res<WeatherTuning>,
    mut frame: ResMut<AtmosphereFrame>,
) {
    let sim_micros = sim_time
        .as_deref()
        .map_or_else(|| field.as_deref().map_or(0, |f| f.prev_micros), |t| t.0);
    let inputs = AtmosphereFrameInputs {
        weather_revision: field.as_deref().map(|f| f.revision),
        sim_micros,
        tracer_revision: stack.revision,
    };
    // Read through `Deref` so an idle frame never flags ECS change.
    if !frame.rebuild_needed(&inputs) {
        return;
    }
    frame.rebuild(field.as_deref(), &stack, sim_micros, &tuning);
}

pub fn atmosphere_frame_systems(app: &mut App) {
    // Idempotent if `WeatherSimulationPlugin` already inserted the table.
    app.init_resource::<WeatherTuning>()
        .init_resource::<AtmosphereFrame>()
        .add_systems(
            Update,
            build_atmosphere_frame.in_set(AtmospherePipelineSet::Frame),
        );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::weather::{WeatherField, WeatherPresetId, WeatherTuning};

    fn extent_m() -> Vec2 {
        Vec2::splat(8.0 * CHUNK_M)
    }

    fn field(seed: u64) -> WeatherField {
        WeatherField::new(seed, extent_m(), &WeatherTuning::default(), None, Default::default(), 0)
    }

    fn stack_with_smoke(chunk: IVec2, v: f32) -> AtmosphereClipmapStack {
        let mut s = AtmosphereClipmapStack::default();
        let l0 = &mut s.levels[0];
        let i = (chunk.y as u32 * l0.resolution.x + chunk.x as u32) as usize;
        l0.smoke_density[i] = v;
        s.bump_revision();
        s
    }

    fn tune() -> WeatherTuning {
        WeatherTuning::default()
    }

    fn built(seed: u64, smoke: f32) -> AtmosphereFrame {
        let f = field(seed);
        let stack = stack_with_smoke(IVec2::new(2, 3), smoke);
        let mut frame = AtmosphereFrame::default();
        assert!(frame.rebuild(Some(&f), &stack, 5_000_000, &tune()));
        frame
    }

    #[test]
    fn frame_hash_is_deterministic_per_seed() {
        let a = built(42, 0.4);
        let b = built(42, 0.4);
        assert!(a.is_weather_ready());
        assert_eq!(a.content_hash(), b.content_hash());
        assert_ne!(a.content_hash(), built(43, 0.4).content_hash(), "seed must matter");
        assert_ne!(a.content_hash(), built(42, 0.5).content_hash(), "tracers must matter");
    }

    #[test]
    fn tile_weather_is_one_field_sample_at_chunk_centre() {
        let f = field(7);
        let mut stack = AtmosphereClipmapStack::default();
        stack.bump_revision();
        let mut frame = AtmosphereFrame::default();
        frame.rebuild(Some(&f), &stack, 3_000_000, &tune());
        let chunk = IVec2::new(1, 2);
        let pos = chunk_world_center(chunk, UVec2::splat(CHUNK_TILES_SIM)) * TILE_METRES_GAMEPLAY;
        let want = f.sample(pos, SimTimeMicros(3_000_000));
        assert_eq!(frame.weather_at_chunk(chunk), Some(want));
        // Outside the field extent: no weather, tracers still addressable.
        assert!(frame.weather_at_chunk(IVec2::new(100, 100)).is_none());
        assert!(frame.tile_at_chunk(IVec2::new(100, 100)).is_some());
        // Outside the L0 window: nothing.
        assert!(frame.tile_at_chunk(IVec2::new(-1, 0)).is_none());
    }

    #[test]
    fn rebuild_is_revision_keyed() {
        let mut f = field(9);
        let mut stack = AtmosphereClipmapStack::default();
        stack.bump_revision();
        let t = tune();
        let mut frame = AtmosphereFrame::default();
        assert!(frame.rebuild(Some(&f), &stack, 1_000_000, &t));
        let rev = frame.revision();
        let builds = frame.stats().builds;

        assert!(!frame.rebuild(Some(&f), &stack, 1_000_000, &t), "same key must not rebuild");
        assert_eq!(frame.revision(), rev);

        stack.bump_revision();
        assert!(frame.rebuild(Some(&f), &stack, 1_000_000, &t), "stack.revision rebuilds");
        let resamples = frame.stats().weather_resamples;
        assert_eq!(resamples, 1, "tracer-only rebuild must not re-sample weather");
        assert_eq!(frame.key().tracer_revision, stack.revision);

        assert!(frame.rebuild(Some(&f), &stack, 2_000_000, &t), "sim time rebuilds");
        assert_eq!(frame.stats().weather_resamples, 2);

        f.revision = f.revision.wrapping_add(1);
        assert!(frame.rebuild(Some(&f), &stack, 2_000_000, &t), "weather revision rebuilds");
        assert_eq!(frame.stats().weather_resamples, 3);
        assert!(frame.stats().builds > builds);
        assert!(frame.revision() > rev);
        assert_eq!(frame.stats().legacy_weather_direct_reads, 0);
    }

    #[test]
    fn smoke_lowers_combined_visibility() {
        let clear = built(11, 0.0);
        let smoky = built(11, 0.8);
        let chunk = IVec2::new(2, 3);
        let c = clear.tile_at_chunk(chunk).unwrap();
        let s = smoky.tile_at_chunk(chunk).unwrap();
        assert!((c.visibility_m_combined - c.weather.visibility_m).abs() < 1e-3);
        assert!(s.visibility_m_combined < s.weather.visibility_m);
        assert!(smoky.stats().vis_smoke_delta_max_m > 0.0);
        assert!(smoky.stats().vis_combined_min_m < smoky.stats().vis_weather_min_m + 1e-3);
        assert_eq!(clear.stats().vis_smoke_delta_max_m, 0.0);
        assert!((smoky.smoke01_at_chunk(chunk).unwrap() - 0.8).abs() < 1e-6);
        // Per-tile lookup resolves through the owning chunk.
        let tile = chunk * CHUNK_TILES_SIM as i32 + IVec2::new(5, 7);
        assert!((smoky.smoke01_at_tile(tile).unwrap() - 0.8).abs() < 1e-6);
    }

    #[test]
    fn pos_sample_is_exact_at_tile_centre_and_smooth_between() {
        let frame = built(21, 0.0);
        let chunk = IVec2::new(3, 3);
        let centre = (chunk.as_vec2() + 0.5) * CHUNK_M;
        let exact = frame.weather_at_chunk(chunk).unwrap();
        let got = frame.weather_at_pos_m(centre).unwrap();
        assert!((got.rain_mm_h - exact.rain_mm_h).abs() < 1e-4);
        assert!((got.temp_c - exact.temp_c).abs() < 1e-3);
        assert!((got.wind_mps - exact.wind_mps).length() < 1e-4);

        let right = frame.weather_at_chunk(IVec2::new(4, 3)).unwrap();
        let mid = frame.weather_at_pos_m(centre + Vec2::new(CHUNK_M * 0.5, 0.0)).unwrap();
        let want = (exact.temp_c + right.temp_c) * 0.5;
        assert!((mid.temp_c - want).abs() < 1e-3, "{} vs {want}", mid.temp_c);
    }

    #[test]
    fn means_follow_chunk_weather_scales() {
        let frame = built(5, 0.0);
        let m = frame.weather_means();
        assert!(m.tiles > 0);
        let mut rain = 0.0f32;
        let mut n = 0u32;
        for t in frame.tiles().iter().filter(|t| t.weather_valid) {
            let r = (t.weather.rain_mm_h / CHUNK_RAIN_FULL_SCALE_MM_H).clamp(0.0, 1.0);
            rain += if t.weather.snow { 0.0 } else { r };
            n += 1;
        }
        assert_eq!(n, m.tiles);
        assert!((m.rain01 - rain / n as f32).abs() < 1e-5);
        assert!((0.0..=1.0).contains(&m.wind_speed01));
    }

    #[test]
    fn without_weather_field_frame_still_serves_tracers() {
        let stack = stack_with_smoke(IVec2::new(1, 1), 0.6);
        let t = tune();
        let mut frame = AtmosphereFrame::default();
        assert!(frame.rebuild(None, &stack, 0, &t));
        assert!(!frame.is_weather_ready());
        assert_eq!(frame.weather_means().tiles, 0);
        assert!(frame.weather_at_pos_m(Vec2::splat(100.0)).is_none());
        assert!((frame.smoke01_at_chunk(IVec2::new(1, 1)).unwrap() - 0.6).abs() < 1e-6);
        // Idle: stack revision unchanged, no field → no rebuild.
        assert!(!frame.rebuild(None, &stack, 0, &t));
    }

    #[test]
    fn plugin_builds_frame_from_live_resources() {
        use bevy::input::InputPlugin;

        use crate::gui::InputBindings;
        use crate::systems::atmosphere::AtmospherePlugin;
        use crate::systems::sim_control::SimControlPlugin;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(InputPlugin);
        app.init_resource::<InputBindings>();
        app.add_plugins(SimControlPlugin);
        app.add_plugins(AtmospherePlugin);
        app.init_resource::<SimTimeMicros>();
        app.insert_resource(field(3));

        app.update();
        app.update();
        let frame = app.world().resource::<AtmosphereFrame>();
        assert!(frame.stats().builds > 0, "frame_builds must advance");
        assert!(frame.is_weather_ready());
        assert!(frame.weather_means().tiles > 0);
    }

    #[test]
    fn hurricane_preset_marks_hurricane_class_cells_else_zero() {
        let t = tune();
        let mut stack = AtmosphereClipmapStack::default();
        stack.bump_revision();

        let calm = WeatherField::new(21, extent_m(), &t, Some(WeatherPresetId::Calm), Default::default(), 0);
        let mut frame = AtmosphereFrame::default();
        assert!(frame.rebuild(Some(&calm), &stack, 0, &t));
        assert_eq!(
            frame.stats().storm_hurricane_cells, 0,
            "non-vortex preset must not count hurricane cells"
        );

        let cane = WeatherField::new(
            21,
            extent_m(),
            &t,
            Some(WeatherPresetId::HurricaneClass),
            Default::default(),
            0,
        );
        assert!(cane.systems.iter().any(|s| s.vortex_mps > 0.0));
        stack.bump_revision();
        let mut frame = AtmosphereFrame::default();
        assert!(frame.rebuild(Some(&cane), &stack, 0, &t));
        assert!(
            frame.stats().storm_hurricane_cells > 0,
            "HurricaneClass preset must yield hurricane_class_cells > 0"
        );
    }
}
