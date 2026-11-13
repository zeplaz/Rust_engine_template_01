//! **`WeatherField`** — the one authority for weather truth (plan `src/dev/plan_weather_wind_v1.md` §2).
//!
//! A coarse grid of weather cells (1 km by default, sized from the map) driven by a seeded climate
//! and 2–8 moving pressure systems. The field is solved once per `weather_step_secs` of game time
//! (driven by [`SimStepClock`](crate::systems::sim_control::SimStepClock), never by frames or real
//! time). It keeps two snapshots: `prev` (the state at the last solve) and `next` (the forecast one
//! step ahead). [`WeatherField::sample`] interpolates bilinearly in space and linearly in time.
//!
//! Metric units: m, s, m/s, hPa, °C, mm/h. Deterministic from the world seed.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::tuning::{WeatherPreset, WeatherPresetId, WeatherTuning};

/// Sea-level reference pressure.
pub const BASELINE_PRESSURE_HPA: f32 = 1013.25;

/// Schema of [`WeatherFieldSave`].
pub const WEATHER_FIELD_SAVE_SCHEMA: u32 = 1;

/// Deterministic, serialisable RNG (SplitMix64).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherRng {
    pub state: u64,
}

impl WeatherRng {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self { state: seed ^ 0x5745_4154_4845_5231 }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        mix64(self.state)
    }

    /// Uniform in `[0, 1)`.
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn range(&mut self, r: (f32, f32)) -> f32 {
        r.0 + (r.1 - r.0) * self.unit()
    }
}

#[inline]
fn mix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PressureKind {
    High,
    Low,
}

/// One moving high or low.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PressureSystem {
    pub kind: PressureKind,
    pub centre_m: [f32; 2],
    pub velocity_mps: [f32; 2],
    pub radius_m: f32,
    pub peak_hpa: f32,
    /// 0..1 moisture carried (lows wet, highs dry).
    pub humidity_load: f32,
    pub age_secs: f32,
    pub lifetime_secs: f32,
}

impl PressureSystem {
    /// 0..1 strength envelope over the lifetime: strengthen, hold, fade.
    #[must_use]
    pub fn envelope(&self) -> f32 {
        let t = (self.age_secs / self.lifetime_secs.max(1.0)).clamp(0.0, 1.0);
        let up = smooth01(t / 0.25);
        let down = smooth01((1.0 - t) / 0.25);
        up.min(down)
    }

    /// Signed pressure anomaly at the centre (negative for lows).
    #[must_use]
    pub fn anomaly_hpa(&self) -> f32 {
        let s = self.peak_hpa * self.envelope();
        match self.kind {
            PressureKind::Low => -s,
            PressureKind::High => s,
        }
    }

    /// Gaussian weight of this system at `p` (1 at the centre).
    #[must_use]
    pub fn weight_at(&self, p: Vec2) -> f32 {
        let d = p - Vec2::from(self.centre_m);
        (-(d.length_squared()) / (self.radius_m * self.radius_m).max(1.0)).exp()
    }
}

#[inline]
fn smooth01(x: f32) -> f32 {
    let x = x.clamp(0.0, 1.0);
    x * x * (3.0 - 2.0 * x)
}

/// Slow climate tier (season, base temperature, prevailing wind).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeatherClimate {
    /// 0..1 annual phase.
    pub season_phase: f32,
    /// Sea-level base temperature.
    pub base_temp_c: f32,
    /// Direction the prevailing wind blows **toward** (radians, +x = 0).
    pub prevailing_dir_rad: f32,
    pub prevailing_speed_mps: f32,
    /// Relative humidity the field relaxes toward.
    pub humidity_base: f32,
    pub preset: Option<WeatherPresetId>,
    /// Game seconds since the last climate step.
    pub secs_since_climate_step: u32,
}

impl WeatherClimate {
    #[must_use]
    pub fn prevailing_mps(&self) -> Vec2 {
        Vec2::from_angle(self.prevailing_dir_rad) * self.prevailing_speed_mps
    }
}

/// One solved state of every weather cell (row-major `dims.x * dims.y`).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WeatherSnapshot {
    pub pressure_hpa: Vec<f32>,
    pub wind_mps: Vec<[f32; 2]>,
    pub temp_c: Vec<f32>,
    pub humidity: Vec<f32>,
    pub cloud: Vec<f32>,
    pub rain_mm_h: Vec<f32>,
    pub fog: Vec<f32>,
    pub lightning: Vec<f32>,
    pub visibility_m: Vec<f32>,
}

impl WeatherSnapshot {
    #[must_use]
    pub fn len(&self) -> usize {
        self.pressure_hpa.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pressure_hpa.is_empty()
    }

    fn with_len(n: usize) -> Self {
        Self {
            pressure_hpa: vec![BASELINE_PRESSURE_HPA; n],
            wind_mps: vec![[0.0, 0.0]; n],
            temp_c: vec![0.0; n],
            humidity: vec![0.0; n],
            cloud: vec![0.0; n],
            rain_mm_h: vec![0.0; n],
            fog: vec![0.0; n],
            lightning: vec![0.0; n],
            visibility_m: vec![0.0; n],
        }
    }
}

/// Terrain factors per weather cell, baked once per terrain revision (not saved: derived).
#[derive(Clone, Debug, Default)]
pub struct WeatherTerrain {
    /// Wind speed factor from height exposure (ridge > 1, valley < 1).
    pub exposure: Vec<f32>,
    /// Wind speed factor from surface roughness (forest / city < 1).
    pub roughness: Vec<f32>,
    /// Along-valley unit axis and how strongly wind turns onto it (0..`channel_max`).
    pub channel_axis: Vec<Vec2>,
    pub channel_strength: Vec<f32>,
    /// Normalised elevation gradient per km (orographic lift).
    pub elev_grad_per_km: Vec<Vec2>,
    pub elevation_norm: Vec<f32>,
    /// Share of open water (evaporation source).
    pub water_frac: Vec<f32>,
    /// Terrain revision this bake belongs to.
    pub key: Option<(u64, u64)>,
}

impl WeatherTerrain {
    #[must_use]
    pub fn neutral(n: usize) -> Self {
        Self {
            exposure: vec![1.0; n],
            roughness: vec![1.0; n],
            channel_axis: vec![Vec2::X; n],
            channel_strength: vec![0.0; n],
            elev_grad_per_km: vec![Vec2::ZERO; n],
            elevation_norm: vec![0.0; n],
            water_frac: vec![0.0; n],
            key: None,
        }
    }
}

/// Per-tile terrain input to [`WeatherTerrain`] bake, accumulated per weather cell.
#[derive(Clone, Debug, Default)]
pub struct TerrainBakeAccum {
    pub elev_sum: Vec<f64>,
    pub rough_n: Vec<u32>,
    pub water_n: Vec<u32>,
    pub n: Vec<u32>,
}

impl TerrainBakeAccum {
    #[must_use]
    pub fn new(cells: usize) -> Self {
        Self {
            elev_sum: vec![0.0; cells],
            rough_n: vec![0; cells],
            water_n: vec![0; cells],
            n: vec![0; cells],
        }
    }

    pub fn add(&mut self, cell: usize, elevation_norm: f32, rough: bool, water: bool) {
        if cell >= self.n.len() {
            return;
        }
        self.elev_sum[cell] += f64::from(elevation_norm);
        self.n[cell] += 1;
        if rough {
            self.rough_n[cell] += 1;
        }
        if water {
            self.water_n[cell] += 1;
        }
    }
}

/// What a consumer reads at one point and time.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeatherSample {
    pub wind_mps: Vec2,
    pub gust_mps: f32,
    pub temp_c: f32,
    pub humidity: f32,
    pub cloud: f32,
    pub rain_mm_h: f32,
    pub snow: bool,
    pub fog: f32,
    pub visibility_m: f32,
    pub lightning: f32,
    pub pressure_hpa: f32,
}

impl Default for WeatherSample {
    fn default() -> Self {
        Self {
            wind_mps: Vec2::ZERO,
            gust_mps: 0.0,
            temp_c: 12.0,
            humidity: 0.5,
            cloud: 0.0,
            rain_mm_h: 0.0,
            snow: false,
            fog: 0.0,
            visibility_m: 20_000.0,
            lightning: 0.0,
            pressure_hpa: BASELINE_PRESSURE_HPA,
        }
    }
}

/// Persistent weather state (RON save overlay).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WeatherFieldSave {
    pub schema_version: u32,
    pub seed: u64,
    pub cell_size_m: f32,
    pub dims: [u32; 2],
    pub extent_m: [f32; 2],
    pub step_secs: u32,
    pub climate: WeatherClimate,
    pub systems: Vec<PressureSystem>,
    pub rng: WeatherRng,
    pub prev: WeatherSnapshot,
    pub next: WeatherSnapshot,
    pub steps_into_interval: u32,
    pub solves: u64,
}

/// The weather authority. Sole writer: `weather_field_step` (in `systems.rs`).
#[derive(Resource, Clone, Debug, Default)]
pub struct WeatherField {
    pub seed: u64,
    pub cell_size_m: f32,
    pub dims: UVec2,
    pub extent_m: Vec2,
    pub step_secs: u32,
    pub climate: WeatherClimate,
    pub systems: Vec<PressureSystem>,
    pub rng: WeatherRng,
    /// State at the last solve (sim time `prev_micros`).
    pub prev: WeatherSnapshot,
    /// Forecast one step ahead (sim time `next_micros`).
    pub next: WeatherSnapshot,
    pub prev_micros: u64,
    pub next_micros: u64,
    /// Sim steps elapsed since the last solve.
    pub steps_into_interval: u32,
    /// Field solves since the field was created (persists through save/load).
    pub solves: u64,
    /// Bumped on every write (init, solve, restore). Projections re-read on change.
    pub revision: u64,
    pub terrain: WeatherTerrain,
    gust_bucket_secs: u32,
    gust_ratio: (f32, f32),
    rough_factor: f32,
}

impl Default for WeatherClimate {
    fn default() -> Self {
        Self {
            season_phase: 0.0,
            base_temp_c: 12.0,
            prevailing_dir_rad: 0.0,
            prevailing_speed_mps: 0.0,
            humidity_base: 0.6,
            preset: None,
            secs_since_climate_step: 0,
        }
    }
}

impl WeatherField {
    /// Whether the field has been initialised for a world.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        !self.next.is_empty() && self.dims.x > 0 && self.dims.y > 0
    }

    #[must_use]
    pub fn cell_count(&self) -> usize {
        (self.dims.x * self.dims.y) as usize
    }

    #[must_use]
    pub fn cell_center_m(&self, idx: usize) -> Vec2 {
        let w = self.dims.x.max(1) as usize;
        Vec2::new(
            ((idx % w) as f32 + 0.5) * self.cell_size_m,
            ((idx / w) as f32 + 0.5) * self.cell_size_m,
        )
    }

    /// Weather cell index containing `pos_m` (clamped into the grid).
    #[must_use]
    pub fn cell_index_at(&self, pos_m: Vec2) -> usize {
        let cx = ((pos_m.x / self.cell_size_m).floor() as i64).clamp(0, i64::from(self.dims.x.max(1)) - 1);
        let cy = ((pos_m.y / self.cell_size_m).floor() as i64).clamp(0, i64::from(self.dims.y.max(1)) - 1);
        (cy as usize) * self.dims.x as usize + cx as usize
    }

    /// Fresh field for a world: seeded climate, systems, first solve (`prev == next`).
    #[must_use]
    pub fn new(
        seed: u64,
        extent_m: Vec2,
        tuning: &WeatherTuning,
        preset: Option<WeatherPresetId>,
        terrain: WeatherTerrain,
        now_micros: u64,
    ) -> Self {
        let cell = tuning.cell_size_m.max(100.0);
        let dims = UVec2::new(
            (extent_m.x / cell).ceil().max(1.0) as u32,
            (extent_m.y / cell).ceil().max(1.0) as u32,
        );
        let n = (dims.x * dims.y) as usize;
        let mut rng = WeatherRng::new(seed);
        let preset_row = preset.and_then(|id| tuning.preset(id)).cloned();
        let season_phase = rng.unit();
        let mut climate = WeatherClimate {
            season_phase,
            base_temp_c: 0.0,
            prevailing_dir_rad: rng.unit() * std::f32::consts::TAU,
            prevailing_speed_mps: rng.range(tuning.prevailing_speed_mps),
            humidity_base: tuning.humidity_base,
            preset: preset_row.as_ref().map(|p| p.id),
            secs_since_climate_step: 0,
        };
        if let Some(p) = &preset_row {
            climate.prevailing_speed_mps = p.prevailing_mps;
            climate.humidity_base = p.humidity;
        }
        climate.base_temp_c = base_temp_for(&climate, tuning, preset_row.as_ref());

        let mut field = Self {
            seed,
            cell_size_m: cell,
            dims,
            extent_m,
            step_secs: tuning.weather_step_secs.max(1),
            climate,
            systems: Vec::new(),
            rng,
            prev: WeatherSnapshot::default(),
            next: WeatherSnapshot::default(),
            prev_micros: now_micros,
            next_micros: now_micros,
            steps_into_interval: 0,
            solves: 0,
            revision: 0,
            terrain: if terrain.exposure.len() == n { terrain } else { WeatherTerrain::neutral(n) },
            gust_bucket_secs: tuning.gust_bucket_secs.max(1),
            gust_ratio: tuning.gust_ratio,
            rough_factor: tuning.rough_factor,
        };
        let count = field.target_system_count(tuning, preset_row.as_ref());
        for _ in 0..count {
            let mut s = field.spawn_system(tuning, preset_row.as_ref(), false);
            // Stagger lifetimes so systems do not all fade together.
            s.age_secs = s.lifetime_secs * field.rng.range((0.15, 0.6));
            field.systems.push(s);
        }
        let mut seed_snap = WeatherSnapshot::with_len(n);
        seed_snap.humidity.fill(field.climate.humidity_base);
        let first = field.solve(&seed_snap, tuning, 0.0);
        field.prev = first.clone();
        field.next = first;
        field.next_micros = now_micros + u64::from(field.step_secs) * 1_000_000;
        field.revision = 1;
        field
    }

    fn target_system_count(&self, tuning: &WeatherTuning, preset: Option<&WeatherPreset>) -> u32 {
        let (lo, hi) = tuning.systems_count;
        if let Some(p) = preset {
            return p.systems.clamp(lo, hi);
        }
        let area_km2 = self.extent_m.x * self.extent_m.y / 1.0e6;
        let extra = (area_km2 / tuning.systems_km2_per_extra.max(1.0)).floor() as u32;
        let jitter = (self.seed % 2) as u32;
        (lo + extra + jitter).clamp(lo, hi)
    }

    fn spawn_system(
        &mut self,
        tuning: &WeatherTuning,
        preset: Option<&WeatherPreset>,
        upwind_edge: bool,
    ) -> PressureSystem {
        let low_fraction = preset.map_or(tuning.low_fraction, |p| p.low_fraction);
        let kind = if self.rng.unit() < low_fraction {
            PressureKind::Low
        } else {
            PressureKind::High
        };
        let extent = self.extent_m.max_element().max(1.0);
        let radius_m = (extent * self.rng.range(tuning.system_radius_frac))
            .clamp(tuning.system_radius_m.0, tuning.system_radius_m.1);
        let peak_hpa = self.rng.range(preset.map_or(tuning.system_strength_hpa, |p| p.strength_hpa));
        let dir = Vec2::from_angle(self.climate.prevailing_dir_rad);
        let perp = dir.perp();
        let centre_map = self.extent_m * 0.5;
        let centre = if upwind_edge {
            let half = self.extent_m.length() * 0.5;
            centre_map - dir * (half + radius_m * 0.3) + perp * (self.rng.unit() - 0.5) * extent
        } else {
            Vec2::new(
                (self.rng.unit() * 1.5 - 0.25) * self.extent_m.x,
                (self.rng.unit() * 1.5 - 0.25) * self.extent_m.y,
            )
        };
        let cross = (self.rng.unit() * 2.0 - 1.0) * tuning.system_cross_drift_mps;
        let velocity = dir * self.climate.prevailing_speed_mps * tuning.system_steering_frac + perp * cross;
        let humidity_load = match kind {
            PressureKind::Low => self.rng.range((0.5, 1.0)),
            PressureKind::High => self.rng.range((0.0, 0.3)),
        };
        PressureSystem {
            kind,
            centre_m: centre.into(),
            velocity_mps: velocity.into(),
            radius_m,
            peak_hpa,
            humidity_load,
            age_secs: 0.0,
            lifetime_secs: self.rng.range(tuning.system_lifetime_secs),
        }
    }

    /// One weather step: climate tier, move / age / respawn systems, solve the next forecast.
    pub fn advance(&mut self, tuning: &WeatherTuning, now_micros: u64) {
        let dt = self.step_secs as f32;
        let preset_row = self.climate.preset.and_then(|id| tuning.preset(id)).cloned();

        self.climate.secs_since_climate_step += self.step_secs;
        if self.climate.secs_since_climate_step >= tuning.climate_step_secs.max(1) {
            let secs = self.climate.secs_since_climate_step as f32;
            self.climate.secs_since_climate_step = 0;
            self.climate.season_phase =
                (self.climate.season_phase + secs / tuning.season_length_secs.max(1.0)).fract();
            let veer = (self.rng.unit() * 2.0 - 1.0) * tuning.prevailing_veer_rad;
            self.climate.prevailing_dir_rad =
                (self.climate.prevailing_dir_rad + veer).rem_euclid(std::f32::consts::TAU);
            if preset_row.is_none() {
                let drift = 1.0 + (self.rng.unit() * 2.0 - 1.0) * 0.15;
                self.climate.prevailing_speed_mps = (self.climate.prevailing_speed_mps * drift)
                    .clamp(tuning.prevailing_speed_mps.0, tuning.prevailing_speed_mps.1);
            }
            self.climate.base_temp_c = base_temp_for(&self.climate, tuning, preset_row.as_ref());
        }

        let map_centre = self.extent_m * 0.5;
        let reach = self.extent_m.length() * 0.5;
        for i in 0..self.systems.len() {
            let s = &mut self.systems[i];
            let c = Vec2::from(s.centre_m) + Vec2::from(s.velocity_mps) * dt;
            s.centre_m = c.into();
            s.age_secs += dt;
            let gone = (c - map_centre).length() > reach + 1.5 * s.radius_m;
            if s.age_secs >= s.lifetime_secs || gone {
                self.systems[i] = self.spawn_system(tuning, preset_row.as_ref(), true);
            }
        }

        let next = self.solve(&self.next, tuning, dt);
        self.prev = std::mem::replace(&mut self.next, next);
        self.prev_micros = now_micros;
        self.next_micros = now_micros + u64::from(self.step_secs) * 1_000_000;
        self.solves += 1;
        self.revision = self.revision.wrapping_add(1);
    }

    /// Pressure → wind → humidity → cloud → precipitation → visibility for every cell.
    #[must_use]
    pub fn solve(&self, from: &WeatherSnapshot, t: &WeatherTuning, dt: f32) -> WeatherSnapshot {
        let n = self.cell_count();
        let mut out = WeatherSnapshot::with_len(n);
        if from.len() != n || self.terrain.exposure.len() != n {
            return out;
        }
        let prevailing = self.climate.prevailing_mps();
        let (sin_t, cos_t) = t.friction_turn_deg.to_radians().sin_cos();
        for i in 0..n {
            let p = self.cell_center_m(i);
            // 1. Pressure (hPa) and its gradient (hPa/km).
            let mut pressure = BASELINE_PRESSURE_HPA;
            let mut grad = Vec2::ZERO;
            let mut low_w = 0.0f32;
            let mut high_w = 0.0f32;
            for s in &self.systems {
                let a = s.anomaly_hpa();
                let g = s.weight_at(p);
                pressure += a * g;
                let r2 = (s.radius_m * s.radius_m).max(1.0);
                grad += a * g * (-2.0) * (p - Vec2::from(s.centre_m)) / r2 * 1000.0;
                match s.kind {
                    PressureKind::Low => low_w += g * s.envelope() * s.humidity_load,
                    PressureKind::High => high_w += g * s.envelope(),
                }
            }
            // 2. Wind: prevailing + gradient wind (low on the left), turned toward low pressure.
            let geo = Vec2::new(-grad.y, grad.x) * t.gradient_mps_per_hpa_km;
            let geo = Vec2::new(geo.x * cos_t - geo.y * sin_t, geo.x * sin_t + geo.y * cos_t);
            let mut wind = prevailing + geo;
            let speed = wind.length();
            if speed > 1e-4 {
                let mut dir = wind / speed;
                let strength = self.terrain.channel_strength[i];
                if strength > 0.0 {
                    let axis = self.terrain.channel_axis[i];
                    let along = if dir.dot(axis) >= 0.0 { axis } else { -axis };
                    dir = dir.lerp(along, strength).normalize_or(dir);
                }
                let factor = self.terrain.exposure[i] * self.terrain.roughness[i];
                wind = dir * (speed * factor).min(t.max_wind_mps);
            }
            let speed = wind.length();
            // Temperature: base, lapse rate, cloud shade.
            let elev_m = self.terrain.elevation_norm[i] * t.elevation_range_m;
            let temp = self.climate.base_temp_c - t.lapse_rate_c_per_km * elev_m / 1000.0
                - 2.0 * from.cloud[i];
            // 3. Humidity: semi-Lagrangian advection on the coarse grid, sources, sinks.
            let back = p - wind * dt;
            let mut h = bilinear(&from.humidity, self.dims, self.cell_size_m, back);
            let warm = ((temp + 5.0) / 25.0).clamp(0.2, 1.5);
            h += t.water_evaporation * self.terrain.water_frac[i] * warm;
            h += t.low_humidity_load * low_w;
            h -= t.high_drying * high_w;
            h += (self.climate.humidity_base - h) * t.humidity_relax;
            h -= t.rain_sink_per_mm_h * from.rain_mm_h[i];
            let h = h.clamp(0.02, 1.0);
            // 4. Cloud: humidity, low-pressure lift, orographic lift (rain shadow on the lee).
            let moist = ((h - 0.35) / 0.3).clamp(0.0, 1.0);
            let oro = (wind / 10.0).dot(self.terrain.elev_grad_per_km[i]) * t.orographic_lift;
            let mut cloud = (h - t.cloud_humidity_min) / t.cloud_humidity_span.max(1e-3);
            cloud += t.low_lift_per_hpa * (BASELINE_PRESSURE_HPA - pressure).max(0.0) * moist;
            cloud += if oro > 0.0 { oro * moist } else { oro * 0.5 };
            let cloud = cloud.clamp(0.0, 1.0);
            // 5. Rain / fog / lightning.
            let rain = if cloud > t.rain_cloud_min {
                (cloud - t.rain_cloud_min) / (1.0 - t.rain_cloud_min).max(1e-3) * h * t.rain_max_mm_h
            } else {
                0.0
            };
            let fog = ((h - 0.8) / 0.2).clamp(0.0, 1.0)
                * (1.0 - speed / 4.0).clamp(0.0, 1.0)
                * ((15.0 - temp) / 10.0).clamp(0.0, 1.0);
            let lightning = ((temp - 12.0) / 10.0).clamp(0.0, 1.0)
                * ((h - 0.7) / 0.3).clamp(0.0, 1.0)
                * ((BASELINE_PRESSURE_HPA - pressure) / 10.0).clamp(0.0, 1.0)
                * cloud;
            // 6. Visibility (fog, rain; smoke joins in W2).
            let vis = t.visibility_clear_m / (1.0 + fog * t.visibility_fog_k) / (1.0 + rain * t.visibility_rain_k);

            out.pressure_hpa[i] = pressure;
            out.wind_mps[i] = wind.into();
            out.temp_c[i] = temp;
            out.humidity[i] = h;
            out.cloud[i] = cloud;
            out.rain_mm_h[i] = rain;
            out.fog[i] = fog;
            out.lightning[i] = lightning;
            out.visibility_m[i] = vis.clamp(50.0, t.visibility_clear_m);
        }
        out
    }

    /// Bilinear in space, linear in time between `prev` and `next`. O(1).
    #[must_use]
    pub fn sample(&self, pos_m: Vec2, sim_time: crate::systems::sim_control::SimTimeMicros) -> WeatherSample {
        if !self.is_ready() || self.prev.len() != self.next.len() {
            return WeatherSample::default();
        }
        let span = self.next_micros.saturating_sub(self.prev_micros).max(1);
        let a = (sim_time.0.saturating_sub(self.prev_micros) as f64 / span as f64).clamp(0.0, 1.0) as f32;
        let (d, c, s) = (self.dims, self.cell_size_m, pos_m);
        let lerp_f = |p: &Vec<f32>, q: &Vec<f32>| {
            let x = bilinear(p, d, c, s);
            x + (bilinear(q, d, c, s) - x) * a
        };
        let wp = bilinear_vec(&self.prev.wind_mps, d, c, s);
        let wind = wp + (bilinear_vec(&self.next.wind_mps, d, c, s) - wp) * a;
        let temp_c = lerp_f(&self.prev.temp_c, &self.next.temp_c);
        let rain_mm_h = lerp_f(&self.prev.rain_mm_h, &self.next.rain_mm_h);
        let speed = wind.length();
        let cell = self.cell_index_at(pos_m);
        let bucket = (sim_time.0 / 1_000_000) / u64::from(self.gust_bucket_secs.max(1));
        let u = (mix64(self.seed ^ (cell as u64).wrapping_mul(0x9E37_79B9) ^ bucket.wrapping_mul(0xC2B2_AE35))
            >> 40) as f32
            / (1u64 << 24) as f32;
        let rough = self.terrain.roughness.get(cell).copied().unwrap_or(1.0);
        let rough01 = ((1.0 - rough) / (1.0 - self.rough_factor).max(1e-3)).clamp(0.0, 1.0);
        let ratio = self.gust_ratio.0 + (self.gust_ratio.1 - self.gust_ratio.0) * rough01;
        WeatherSample {
            wind_mps: wind,
            gust_mps: speed * (1.0 + ratio * u),
            temp_c,
            humidity: lerp_f(&self.prev.humidity, &self.next.humidity),
            cloud: lerp_f(&self.prev.cloud, &self.next.cloud),
            rain_mm_h,
            snow: temp_c < 0.0 && rain_mm_h > 0.05,
            fog: lerp_f(&self.prev.fog, &self.next.fog),
            visibility_m: lerp_f(&self.prev.visibility_m, &self.next.visibility_m),
            lightning: lerp_f(&self.prev.lightning, &self.next.lightning),
            pressure_hpa: lerp_f(&self.prev.pressure_hpa, &self.next.pressure_hpa),
        }
    }

    /// Replace the terrain factors (same grid). Does not re-solve.
    pub fn set_terrain(&mut self, terrain: WeatherTerrain) {
        if terrain.exposure.len() == self.cell_count() {
            self.terrain = terrain;
        }
    }

    #[must_use]
    pub fn to_save(&self) -> WeatherFieldSave {
        WeatherFieldSave {
            schema_version: WEATHER_FIELD_SAVE_SCHEMA,
            seed: self.seed,
            cell_size_m: self.cell_size_m,
            dims: [self.dims.x, self.dims.y],
            extent_m: [self.extent_m.x, self.extent_m.y],
            step_secs: self.step_secs,
            climate: self.climate.clone(),
            systems: self.systems.clone(),
            rng: self.rng,
            prev: self.prev.clone(),
            next: self.next.clone(),
            steps_into_interval: self.steps_into_interval,
            solves: self.solves,
        }
    }

    /// Field from a save. `now_micros` re-anchors the interpolation window to the current sim time.
    pub fn from_save(save: &WeatherFieldSave, tuning: &WeatherTuning, now_micros: u64) -> Result<Self, String> {
        if save.schema_version != WEATHER_FIELD_SAVE_SCHEMA {
            return Err(format!("weather save schema {} unsupported", save.schema_version));
        }
        let n = (save.dims[0] * save.dims[1]) as usize;
        if save.prev.len() != n || save.next.len() != n || save.next.humidity.len() != n {
            return Err(format!("weather save snapshot len != {n}"));
        }
        let step = save.step_secs.max(1);
        let prev_micros = now_micros.saturating_sub(u64::from(save.steps_into_interval) * 1_000_000);
        Ok(Self {
            seed: save.seed,
            cell_size_m: save.cell_size_m,
            dims: UVec2::new(save.dims[0], save.dims[1]),
            extent_m: Vec2::new(save.extent_m[0], save.extent_m[1]),
            step_secs: step,
            climate: save.climate.clone(),
            systems: save.systems.clone(),
            rng: save.rng,
            prev: save.prev.clone(),
            next: save.next.clone(),
            prev_micros,
            next_micros: prev_micros + u64::from(step) * 1_000_000,
            steps_into_interval: save.steps_into_interval,
            solves: save.solves,
            revision: 1,
            terrain: WeatherTerrain::neutral(n),
            gust_bucket_secs: tuning.gust_bucket_secs.max(1),
            gust_ratio: tuning.gust_ratio,
            rough_factor: tuning.rough_factor,
        })
    }
}

fn base_temp_for(climate: &WeatherClimate, t: &WeatherTuning, preset: Option<&WeatherPreset>) -> f32 {
    t.base_temp_mean_c
        + t.base_temp_amplitude_c * (climate.season_phase * std::f32::consts::TAU).sin()
        + preset.map_or(0.0, |p| p.base_temp_offset_c)
}

/// Bake terrain factors from per-cell accumulations (`accum` laid out like the field grid).
#[must_use]
pub fn bake_weather_terrain(dims: UVec2, cell_size_m: f32, accum: &TerrainBakeAccum, t: &WeatherTuning) -> WeatherTerrain {
    let (w, h) = (dims.x as i32, dims.y as i32);
    let n = (dims.x * dims.y) as usize;
    let mut out = WeatherTerrain::neutral(n);
    if accum.n.len() != n {
        return out;
    }
    let total: f64 = accum.elev_sum.iter().sum();
    let count: u64 = accum.n.iter().map(|c| u64::from(*c)).sum();
    let mean_all = if count > 0 { (total / count as f64) as f32 } else { 0.0 };
    let elev: Vec<f32> = (0..n)
        .map(|i| if accum.n[i] > 0 { (accum.elev_sum[i] / f64::from(accum.n[i])) as f32 } else { mean_all })
        .collect();
    let at = |x: i32, y: i32| elev[(y.clamp(0, h - 1) * w + x.clamp(0, w - 1)) as usize];
    let cell_km = (cell_size_m / 1000.0).max(1e-3);
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) as usize;
            let e = elev[i];
            let mut sum = 0.0;
            let mut k = 0.0;
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let (nx, ny) = (x + dx, y + dy);
                    if nx < 0 || ny < 0 || nx >= w || ny >= h {
                        continue;
                    }
                    sum += elev[(ny * w + nx) as usize];
                    k += 1.0;
                }
            }
            let nbr = if k > 0.0 { sum / k } else { e };
            let d = ((e - nbr) / t.exposure_span.max(1e-4)).clamp(-1.0, 1.0);
            out.exposure[i] = if d >= 0.0 {
                1.0 + d * (t.ridge_factor - 1.0)
            } else {
                1.0 + d * (1.0 - t.valley_factor)
            };
            let cnt = accum.n[i].max(1) as f32;
            let rough_frac = accum.rough_n[i] as f32 / cnt;
            out.roughness[i] = 1.0 - rough_frac * (1.0 - t.rough_factor);
            out.water_frac[i] = accum.water_n[i] as f32 / cnt;
            out.elevation_norm[i] = e;
            out.elev_grad_per_km[i] = Vec2::new(
                (at(x + 1, y) - at(x - 1, y)) / (2.0 * cell_km),
                (at(x, y + 1) - at(x, y - 1)) / (2.0 * cell_km),
            );
            // Valley channelling: along-valley axis = Hessian eigenvector of least curvature.
            if d < 0.0 {
                let hxx = at(x + 1, y) - 2.0 * e + at(x - 1, y);
                let hyy = at(x, y + 1) - 2.0 * e + at(x, y - 1);
                let hxy = (at(x + 1, y + 1) - at(x + 1, y - 1) - at(x - 1, y + 1) + at(x - 1, y - 1)) * 0.25;
                let mean = (hxx + hyy) * 0.5;
                let r = (((hxx - hyy) * 0.5).powi(2) + hxy * hxy).sqrt();
                let l1 = mean - r;
                let l2 = mean + r;
                let axis = if hxy.abs() > 1e-6 {
                    Vec2::new(hxy, l1 - hxx).normalize_or(Vec2::X)
                } else if hxx <= hyy {
                    Vec2::X
                } else {
                    Vec2::Y
                };
                let aniso = ((l2 - l1) / (l1.abs() + l2.abs() + 1e-6)).clamp(0.0, 1.0);
                out.channel_axis[i] = axis;
                out.channel_strength[i] = t.channel_max * (-d) * aniso;
            }
        }
    }
    out
}

#[inline]
fn grid_coords(dims: UVec2, cell: f32, pos: Vec2) -> (usize, usize, usize, usize, f32, f32) {
    let w = dims.x.max(1) as f32;
    let h = dims.y.max(1) as f32;
    let fx = (pos.x / cell - 0.5).clamp(0.0, w - 1.0);
    let fy = (pos.y / cell - 0.5).clamp(0.0, h - 1.0);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(dims.x.max(1) as usize - 1);
    let y1 = (y0 + 1).min(dims.y.max(1) as usize - 1);
    (x0, y0, x1, y1, fx - x0 as f32, fy - y0 as f32)
}

fn bilinear(v: &[f32], dims: UVec2, cell: f32, pos: Vec2) -> f32 {
    let n = (dims.x * dims.y) as usize;
    if v.len() != n || n == 0 {
        return 0.0;
    }
    let (x0, y0, x1, y1, tx, ty) = grid_coords(dims, cell, pos);
    let w = dims.x as usize;
    let a = v[y0 * w + x0] + (v[y0 * w + x1] - v[y0 * w + x0]) * tx;
    let b = v[y1 * w + x0] + (v[y1 * w + x1] - v[y1 * w + x0]) * tx;
    a + (b - a) * ty
}

fn bilinear_vec(v: &[[f32; 2]], dims: UVec2, cell: f32, pos: Vec2) -> Vec2 {
    let n = (dims.x * dims.y) as usize;
    if v.len() != n || n == 0 {
        return Vec2::ZERO;
    }
    let (x0, y0, x1, y1, tx, ty) = grid_coords(dims, cell, pos);
    let w = dims.x as usize;
    let g = |i: usize| Vec2::from(v[i]);
    let a = g(y0 * w + x0).lerp(g(y0 * w + x1), tx);
    let b = g(y1 * w + x0).lerp(g(y1 * w + x1), tx);
    a.lerp(b, ty)
}
