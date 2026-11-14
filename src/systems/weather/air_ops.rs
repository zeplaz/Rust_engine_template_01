//! W5 + drone D0 — air / missile / drone weather hooks (API + RON + status; no airframe entities).
//!
//! Consumers call [`evaluate_grounding`], [`air_ground_speed_mul`], [`missile_weather_drift`],
//! [`link_quality`], [`sensor_range_mul`], and [`airlift_hold`].
//! [`AirOpsWeatherStatus`] is the sole centre-sample projection for dump/AI.
//!
//! Program: [`plan_drone_subsystem_fit_v1.md`](../../dev/plan_drone_subsystem_fit_v1.md).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use bevy::math::{IVec2, UVec2, Vec2};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::substrate::atmosphere::{sample_tactical_smoke_from_l0, AtmosphereClipmapStack};
use crate::systems::atmosphere::atmos_chunk_to_tile;
use crate::systems::fire::ChunkSmokeField;
use crate::systems::sim_control::SimTimeMicros;
use crate::terrain::generation::Chunk;
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

use super::field::{WeatherField, WeatherSample};

/// Fallback cells/chunk when raster config is absent (matches common worldgen).
pub const SMOKE_SAMPLE_CELLS_PER_CHUNK: u32 = 16;

/// Repo-relative path of the air-ops table.
pub const AIR_OPS_TUNING_RON: &str = "assets/configs/weather/air_ops_v1.ron";

const EMBEDDED_AIR_OPS: &str = include_str!("../../../assets/configs/weather/air_ops_v1.ron");

/// Kind of air / missile / drone profile in [`AirOpsTuning`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AirOpsProfileId {
    FixedWing,
    Rotor,
    Missile,
    Airlift,
    DroneFpv,
    DroneRecon,
    DroneCruise,
    DroneInterceptor,
}

/// C2 modality for [`link_quality`] (weather / EW fragility ordering).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkModality {
    Radio,
    Fiber,
    SatRelay,
    /// Onboard autonomy — resists C2 weather loss (still needs sensors to prosecute).
    Autonomy,
}

/// Seeker / ISR kit for [`sensor_range_mul`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensorKind {
    Eo,
    Ir,
    Radar,
    PassiveRf,
}

/// One RON profile: grounding + headwind / missile drift + research resist knobs.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AirOpsProfile {
    pub id: AirOpsProfileId,
    pub max_wind_mps: f32,
    pub max_crosswind_mps: f32,
    pub min_visibility_m: f32,
    pub max_lightning: f32,
    pub headwind_fuel_k: f32,
    pub cruise_mps: f32,
    pub missile_drift_coef: f32,
    pub max_missile_wind_mps: f32,
    /// Research resist on link envelope (0..1).
    #[serde(default)]
    pub link_resist: f32,
    /// Research resist on sensor envelope (0..1).
    #[serde(default)]
    pub sensor_resist: f32,
}

/// Loaded air-ops table (`air_ops_v1.ron`).
#[derive(Resource, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AirOpsTuning {
    pub schema_version: u32,
    pub profiles: Vec<AirOpsProfile>,
}

impl Default for AirOpsTuning {
    fn default() -> Self {
        parse_air_ops_tuning(EMBEDDED_AIR_OPS).expect("embedded air_ops_v1.ron must parse")
    }
}

impl AirOpsTuning {
    #[must_use]
    pub fn profile(&self, id: AirOpsProfileId) -> Option<&AirOpsProfile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    #[must_use]
    pub fn profile_or_default(&self, id: AirOpsProfileId) -> &AirOpsProfile {
        self.profile(id)
            .or_else(|| self.profiles.first())
            .expect("air_ops profiles non-empty")
    }
}

pub fn parse_air_ops_tuning(text: &str) -> Result<AirOpsTuning, String> {
    let t: AirOpsTuning =
        ron::from_str(text).map_err(|e| format!("air_ops RON parse: {e}"))?;
    if t.profiles.is_empty() {
        return Err("air_ops profiles must be non-empty".into());
    }
    Ok(t)
}

fn repo_asset_path(rel: &str) -> PathBuf {
    std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .map(|root| root.join(rel))
        .unwrap_or_else(|| PathBuf::from(rel))
}

pub fn load_air_ops_tuning_from_path(path: &Path) -> Result<AirOpsTuning, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_air_ops_tuning(&text)
}

/// Disk table when present and valid, else compiled-in default.
#[must_use]
pub fn load_air_ops_tuning() -> (AirOpsTuning, Option<String>) {
    match load_air_ops_tuning_from_path(&repo_asset_path(AIR_OPS_TUNING_RON)) {
        Ok(t) => (t, None),
        Err(err) => (AirOpsTuning::default(), Some(err)),
    }
}

/// Why air ops are grounded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GroundingReason {
    Wind,
    Crosswind,
    Visibility,
    Lightning,
}

/// Result of [`evaluate_grounding`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GroundingVerdict {
    Clear,
    Grounded { reasons: Vec<GroundingReason> },
}

impl GroundingVerdict {
    #[must_use]
    pub fn is_clear(&self) -> bool {
        matches!(self, Self::Clear)
    }

    #[must_use]
    pub fn is_grounded(&self) -> bool {
        !self.is_clear()
    }
}

/// Positive = headwind (wind opposing travel heading).
#[must_use]
pub fn headwind_component(wind_mps: Vec2, heading: Vec2) -> f32 {
    let h = if heading.length_squared() > 1e-8 {
        heading.normalize()
    } else {
        Vec2::X
    };
    -wind_mps.dot(h)
}

/// Absolute crosswind relative to heading (runway or track).
#[must_use]
pub fn crosswind_component(wind_mps: Vec2, heading: Vec2) -> f32 {
    let h = if heading.length_squared() > 1e-8 {
        heading.normalize()
    } else {
        Vec2::X
    };
    let cross = Vec2::new(-h.y, h.x);
    wind_mps.dot(cross).abs()
}

/// Grounding from wind / optional runway crosswind / visibility / lightning.
#[must_use]
pub fn evaluate_grounding(
    sample: &WeatherSample,
    profile: &AirOpsProfile,
    runway_dir: Option<Vec2>,
) -> GroundingVerdict {
    let mut reasons = Vec::new();
    let wind_speed = sample.wind_mps.length().max(sample.gust_mps * 0.9);
    if wind_speed > profile.max_wind_mps {
        reasons.push(GroundingReason::Wind);
    }
    if let Some(dir) = runway_dir {
        if crosswind_component(sample.wind_mps, dir) > profile.max_crosswind_mps {
            reasons.push(GroundingReason::Crosswind);
        }
    }
    if sample.visibility_m < profile.min_visibility_m {
        reasons.push(GroundingReason::Visibility);
    }
    if sample.lightning > profile.max_lightning {
        reasons.push(GroundingReason::Lightning);
    }
    if reasons.is_empty() {
        GroundingVerdict::Clear
    } else {
        GroundingVerdict::Grounded { reasons }
    }
}

/// Ground-speed scale from head/tail wind vs cruise (`0.35..=1.25`).
#[must_use]
pub fn air_ground_speed_mul(sample: &WeatherSample, heading: Vec2, profile: &AirOpsProfile) -> f32 {
    let cruise = profile.cruise_mps.max(1.0);
    let hw = headwind_component(sample.wind_mps, heading);
    let gs = (cruise - hw).max(cruise * 0.35);
    (gs / cruise).clamp(0.35, 1.25)
}

/// Fuel burn scale: `1 + k * max(0, headwind)`.
#[must_use]
pub fn air_fuel_mul(sample: &WeatherSample, heading: Vec2, profile: &AirOpsProfile) -> f32 {
    let hw = headwind_component(sample.wind_mps, heading).max(0.0);
    (1.0 + profile.headwind_fuel_k.max(0.0) * hw).clamp(1.0, 2.0)
}

/// True when Airlift profile would be grounded (high wind / vis / lightning hold).
#[must_use]
pub fn airlift_hold(sample: &WeatherSample, profile: &AirOpsProfile) -> bool {
    evaluate_grounding(sample, profile, None).is_grounded()
}

/// Guided-missile impact drift metres (downwind). Calm or zero coef → zero.
#[must_use]
pub fn missile_weather_drift(
    sample: &WeatherSample,
    coef: f32,
    max_wind_mps: f32,
    flight_time_s: f32,
) -> Vec2 {
    if coef <= 0.0 || flight_time_s <= 0.0 {
        return Vec2::ZERO;
    }
    let speed = sample.wind_mps.length().min(max_wind_mps.max(0.0));
    if speed < 0.05 {
        return Vec2::ZERO;
    }
    sample.wind_mps.normalize() * (speed * coef * flight_time_s)
}

fn fog_factor(sample: &WeatherSample) -> f32 {
    sample.fog.clamp(0.0, 1.0)
}

fn vis_factor(sample: &WeatherSample) -> f32 {
    (sample.visibility_m / 20_000.0).clamp(0.0, 1.0)
}

fn rain_factor(sample: &WeatherSample) -> f32 {
    (sample.rain_mm_h / 10.0).clamp(0.0, 1.0)
}

/// C2 link quality `0..1` under weather + smoke. `resist` is research 0..1.
#[must_use]
pub fn link_quality(
    sample: &WeatherSample,
    smoke01: f32,
    modality: LinkModality,
    resist: f32,
) -> f32 {
    let smoke = smoke01.clamp(0.0, 1.0);
    let fog = fog_factor(sample);
    let lightning = sample.lightning.clamp(0.0, 1.0);
    let rain = rain_factor(sample);
    let r = resist.clamp(0.0, 1.0);

    let raw = match modality {
        LinkModality::Radio => {
            // Pilot video + RF: smoke, fog, lightning, rain all hurt hard.
            let hit = 0.55 * smoke + 0.35 * fog + 0.55 * lightning + 0.20 * rain;
            (1.0 - hit).clamp(0.0, 1.0)
        }
        LinkModality::Fiber => {
            // Physical fiber: RF/lightning almost ignored; smoke/fog only if EO for nav.
            let hit = 0.08 * smoke + 0.05 * fog + 0.05 * rain;
            (1.0 - hit).clamp(0.0, 1.0)
        }
        LinkModality::SatRelay => {
            let hit = 0.25 * smoke + 0.20 * fog + 0.30 * lightning + 0.15 * rain;
            (1.0 - hit).clamp(0.0, 1.0)
        }
        LinkModality::Autonomy => {
            // C2 weather almost irrelevant; mild rain/fog only.
            let hit = 0.05 * fog + 0.08 * rain + 0.05 * lightning;
            (1.0 - hit).clamp(0.0, 1.0)
        }
    };
    (raw + (1.0 - raw) * r).clamp(0.0, 1.0)
}

/// Sensor range scale `0.15..=1.0` under weather + smoke.
#[must_use]
pub fn sensor_range_mul(
    sample: &WeatherSample,
    smoke01: f32,
    sensor: SensorKind,
    resist: f32,
) -> f32 {
    let smoke = smoke01.clamp(0.0, 1.0);
    let fog = fog_factor(sample);
    let vis = vis_factor(sample);
    let rain = rain_factor(sample);
    let r = resist.clamp(0.0, 1.0);

    let raw = match sensor {
        SensorKind::Eo => {
            let clear = vis * (1.0 - 0.85 * smoke) * (1.0 - 0.70 * fog) * (1.0 - 0.25 * rain);
            clear.clamp(0.0, 1.0)
        }
        SensorKind::Ir => {
            let clear = (0.55 + 0.45 * vis) * (1.0 - 0.45 * smoke) * (1.0 - 0.25 * fog);
            clear.clamp(0.0, 1.0)
        }
        SensorKind::Radar => {
            let clear = (0.75 + 0.25 * vis) * (1.0 - 0.15 * smoke) * (1.0 - 0.35 * rain);
            clear.clamp(0.0, 1.0)
        }
        SensorKind::PassiveRf => {
            // Mostly EW/lightning, not optical smoke.
            let clear = 1.0 - 0.40 * sample.lightning.clamp(0.0, 1.0);
            clear.clamp(0.0, 1.0)
        }
    };
    let scaled = raw + (1.0 - raw) * r;
    scaled.clamp(0.15, 1.0)
}

/// Mission gate: flight clear and link×sensor above floor.
#[must_use]
pub fn sortie_envelopes_ok(
    sample: &WeatherSample,
    profile: &AirOpsProfile,
    smoke01: f32,
    modality: LinkModality,
    sensor: SensorKind,
    link_floor: f32,
    sensor_floor: f32,
) -> bool {
    if evaluate_grounding(sample, profile, None).is_grounded() {
        return false;
    }
    let lq = link_quality(sample, smoke01, modality, profile.link_resist);
    let sq = sensor_range_mul(sample, smoke01, sensor, profile.sensor_resist);
    lq >= link_floor && sq >= sensor_floor
}

/// Map a gameplay tile to the chunk that owns its smoke scalar.
#[inline]
#[must_use]
pub fn tile_to_smoke_chunk(tile: IVec2, cells_per_chunk: UVec2) -> IVec2 {
    let cw = cells_per_chunk.x.max(1) as i32;
    let ch = cells_per_chunk.y.max(1) as i32;
    IVec2::new(tile.x.div_euclid(cw), tile.y.div_euclid(ch))
}

/// Build chunk → density from live [`ChunkSmokeField`] rows (L0-pulled gameplay smoke).
#[must_use]
pub fn collect_chunk_smoke_density(
    q: &Query<'_, '_, (&Chunk, &ChunkSmokeField)>,
) -> HashMap<IVec2, f32> {
    let mut map = HashMap::new();
    for (chunk, smoke) in q.iter() {
        map.insert(chunk.coord, smoke.density.clamp(0.0, 1.0));
    }
    map
}

/// Envelope smoke `0..1`: prefer [`ChunkSmokeField`] density, else L0 clipmap, else `0`.
#[must_use]
pub fn smoke01_resolve(
    chunk: IVec2,
    by_chunk: &HashMap<IVec2, f32>,
    stack: Option<&AtmosphereClipmapStack>,
) -> f32 {
    if let Some(&d) = by_chunk.get(&chunk) {
        return d.clamp(0.0, 1.0);
    }
    if let Some(stack) = stack {
        if let Some((x, y)) = atmos_chunk_to_tile(chunk) {
            if let Some(d) = sample_tactical_smoke_from_l0(stack, x, y) {
                return d.clamp(0.0, 1.0);
            }
        }
    }
    0.0
}

/// Smoke at a gameplay tile for link/sensor envelopes.
#[must_use]
pub fn smoke01_at_tile(
    tile: IVec2,
    cells_per_chunk: UVec2,
    by_chunk: &HashMap<IVec2, f32>,
    stack: Option<&AtmosphereClipmapStack>,
) -> f32 {
    smoke01_resolve(tile_to_smoke_chunk(tile, cells_per_chunk), by_chunk, stack)
}

/// Centre-sample air-ops projection for dump / future AI (sole writer: [`sync_air_ops_weather_status`]).
#[derive(Resource, Debug, Clone)]
pub struct AirOpsWeatherStatus {
    pub ready: bool,
    pub grounded_fixed_wing: bool,
    pub airlift_hold: bool,
    pub ground_speed_mul: f32,
    pub missile_drift_m: f32,
    pub wind_mps: Vec2,
    pub samples: u64,
    /// Centre smoke `0..1` from ChunkSmoke / L0 (feeds link/sensor probes).
    pub smoke01_centre: f32,
    /// Radio FPV link at centre (weather + smoke).
    pub link_quality_radio: f32,
    pub link_quality_fiber: f32,
    pub sensor_range_mul_eo: f32,
    pub drone_cruise_grounded: bool,
}

impl Default for AirOpsWeatherStatus {
    fn default() -> Self {
        Self {
            ready: false,
            grounded_fixed_wing: false,
            airlift_hold: false,
            ground_speed_mul: 1.0,
            missile_drift_m: 0.0,
            wind_mps: Vec2::ZERO,
            samples: 0,
            smoke01_centre: 0.0,
            link_quality_radio: 1.0,
            link_quality_fiber: 1.0,
            sensor_range_mul_eo: 1.0,
            drone_cruise_grounded: false,
        }
    }
}

/// Sole writer of [`AirOpsWeatherStatus`] from the weather-field centre sample.
pub fn sync_air_ops_weather_status(
    field: Option<Res<WeatherField>>,
    sim_time: Option<Res<SimTimeMicros>>,
    tuning: Res<AirOpsTuning>,
    stack: Option<Res<AtmosphereClipmapStack>>,
    smoke_q: Query<(&Chunk, &ChunkSmokeField)>,
    mut status: ResMut<AirOpsWeatherStatus>,
) {
    let Some(field) = field.filter(|f| f.is_ready()) else {
        *status = AirOpsWeatherStatus::default();
        return;
    };
    let t = sim_time
        .as_deref()
        .copied()
        .unwrap_or(SimTimeMicros(field.prev_micros));
    let centre_m = field.extent_m * 0.5;
    let sample = field.sample(centre_m, t);
    let fw = tuning.profile_or_default(AirOpsProfileId::FixedWing);
    let lift = tuning.profile_or_default(AirOpsProfileId::Airlift);
    let missile = tuning.profile_or_default(AirOpsProfileId::Missile);
    let fpv = tuning.profile_or_default(AirOpsProfileId::DroneFpv);
    let cruise = tuning.profile_or_default(AirOpsProfileId::DroneCruise);
    let heading = Vec2::X;
    let drift = missile_weather_drift(
        &sample,
        missile.missile_drift_coef,
        missile.max_missile_wind_mps,
        20.0,
    );
    let by_chunk = collect_chunk_smoke_density(&smoke_q);
    let cells = UVec2::splat(SMOKE_SAMPLE_CELLS_PER_CHUNK);
    let centre_tile = IVec2::new(
        (centre_m.x / TILE_METRES_GAMEPLAY).floor() as i32,
        (centre_m.y / TILE_METRES_GAMEPLAY).floor() as i32,
    );
    let smoke01 = smoke01_at_tile(centre_tile, cells, &by_chunk, stack.as_deref());
    status.ready = true;
    status.grounded_fixed_wing = evaluate_grounding(&sample, fw, Some(heading)).is_grounded();
    status.airlift_hold = airlift_hold(&sample, lift);
    status.ground_speed_mul = air_ground_speed_mul(&sample, heading, fw);
    status.missile_drift_m = drift.length();
    status.wind_mps = sample.wind_mps;
    status.smoke01_centre = smoke01;
    status.link_quality_radio = link_quality(&sample, smoke01, LinkModality::Radio, fpv.link_resist);
    status.link_quality_fiber = link_quality(&sample, smoke01, LinkModality::Fiber, fpv.link_resist);
    status.sensor_range_mul_eo =
        sensor_range_mul(&sample, smoke01, SensorKind::Eo, fpv.sensor_resist);
    status.drone_cruise_grounded = evaluate_grounding(&sample, cruise, None).is_grounded();
    status.samples = status.samples.wrapping_add(1);
}

/// Load air-ops RON once (disk over compiled-in).
pub fn load_air_ops_tuning_on_startup(mut commands: Commands) {
    let (tuning, err) = load_air_ops_tuning();
    if let Some(err) = err {
        warn!("air_ops tuning: using compiled-in default ({err})");
    }
    commands.insert_resource(tuning);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn storm_sample() -> WeatherSample {
        WeatherSample {
            wind_mps: Vec2::new(20.0, 0.0),
            gust_mps: 24.0,
            visibility_m: 800.0,
            fog: 0.85,
            lightning: 0.8,
            rain_mm_h: 6.0,
            ..Default::default()
        }
    }

    #[test]
    fn embedded_ron_parses_with_drones() {
        let t = AirOpsTuning::default();
        assert!(t.profile(AirOpsProfileId::FixedWing).is_some());
        assert!(t.profile(AirOpsProfileId::Missile).is_some());
        assert!(t.profile(AirOpsProfileId::DroneFpv).is_some());
        assert!(t.profile(AirOpsProfileId::DroneCruise).is_some());
        assert!(t.schema_version >= 2);
    }

    #[test]
    fn calm_is_clear() {
        let t = AirOpsTuning::default();
        let p = t.profile_or_default(AirOpsProfileId::FixedWing);
        let v = evaluate_grounding(&WeatherSample::default(), p, Some(Vec2::X));
        assert!(v.is_clear());
    }

    #[test]
    fn storm_grounds_fixed_wing() {
        let t = AirOpsTuning::default();
        let p = t.profile_or_default(AirOpsProfileId::FixedWing);
        let v = evaluate_grounding(&storm_sample(), p, Some(Vec2::Y));
        assert!(v.is_grounded());
        if let GroundingVerdict::Grounded { reasons } = v {
            assert!(reasons.contains(&GroundingReason::Wind));
            assert!(reasons.contains(&GroundingReason::Crosswind));
            assert!(reasons.contains(&GroundingReason::Visibility));
            assert!(reasons.contains(&GroundingReason::Lightning));
        }
    }

    #[test]
    fn headwind_slows_tailwind_speeds() {
        let t = AirOpsTuning::default();
        let p = t.profile_or_default(AirOpsProfileId::FixedWing);
        let head = WeatherSample {
            wind_mps: Vec2::new(-10.0, 0.0),
            ..Default::default()
        };
        let tail = WeatherSample {
            wind_mps: Vec2::new(10.0, 0.0),
            ..Default::default()
        };
        let mh = air_ground_speed_mul(&head, Vec2::X, p);
        let mt = air_ground_speed_mul(&tail, Vec2::X, p);
        assert!(mh < 1.0, "headwind mul {mh}");
        assert!(mt > 1.0, "tailwind mul {mt}");
        assert!(air_fuel_mul(&head, Vec2::X, p) > 1.0);
    }

    #[test]
    fn missile_calm_zero_storm_drifts() {
        let t = AirOpsTuning::default();
        let m = t.profile_or_default(AirOpsProfileId::Missile);
        let calm = missile_weather_drift(&WeatherSample::default(), m.missile_drift_coef, 30.0, 20.0);
        assert_eq!(calm, Vec2::ZERO);
        let d = missile_weather_drift(&storm_sample(), m.missile_drift_coef, 30.0, 20.0);
        assert!(d.x > 0.0, "expected east drift, got {d:?}");
    }

    #[test]
    fn airlift_hold_in_storm() {
        let t = AirOpsTuning::default();
        let p = t.profile_or_default(AirOpsProfileId::Airlift);
        assert!(!airlift_hold(&WeatherSample::default(), p));
        assert!(airlift_hold(&storm_sample(), p));
    }

    #[test]
    fn radio_dies_in_smoke_and_fog_fiber_holds() {
        let s = storm_sample();
        let radio = link_quality(&s, 0.9, LinkModality::Radio, 0.0);
        let fiber = link_quality(&s, 0.9, LinkModality::Fiber, 0.0);
        assert!(radio < 0.35, "radio link {radio}");
        assert!(fiber > 0.75, "fiber link {fiber}");
        assert!(fiber > radio);
    }

    #[test]
    fn eo_sensor_shrinks_in_fog_smoke() {
        let clear = sensor_range_mul(&WeatherSample::default(), 0.0, SensorKind::Eo, 0.0);
        let bad = sensor_range_mul(&storm_sample(), 0.9, SensorKind::Eo, 0.0);
        assert!((clear - 1.0).abs() < 0.05);
        assert!(bad < 0.4, "eo mul {bad}");
    }

    #[test]
    fn wind_grounds_drone_cruise_launch() {
        let t = AirOpsTuning::default();
        let p = t.profile_or_default(AirOpsProfileId::DroneCruise);
        assert!(evaluate_grounding(&WeatherSample::default(), p, None).is_clear());
        assert!(evaluate_grounding(&storm_sample(), p, None).is_grounded());
        assert!(!sortie_envelopes_ok(
            &storm_sample(),
            p,
            0.9,
            LinkModality::Radio,
            SensorKind::Eo,
            0.35,
            0.35,
        ));
    }

    #[test]
    fn smoke01_prefers_chunk_density_over_empty_l0() {
        let mut map = HashMap::new();
        map.insert(IVec2::new(1, 2), 0.72);
        assert!((smoke01_resolve(IVec2::new(1, 2), &map, None) - 0.72).abs() < 1e-5);
        assert_eq!(smoke01_resolve(IVec2::new(0, 0), &map, None), 0.0);
        let cells = UVec2::splat(16);
        assert_eq!(tile_to_smoke_chunk(IVec2::new(17, -3), cells), IVec2::new(1, -1));
        assert!((smoke01_at_tile(IVec2::new(17, 32), cells, &map, None) - 0.72).abs() < 1e-5);
    }
}
