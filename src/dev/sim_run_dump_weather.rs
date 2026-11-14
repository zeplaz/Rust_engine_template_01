//! Weather series for the sim run dump (counts only, no verdicts).
//!
//! Sampled once per weather field solve from [`WeatherField`] (the authority). The dump reads
//! [`WeatherDumpSeries`] through one hook in `sim_run_dump.rs`; the matrix scores it.

use std::collections::BTreeMap;

use bevy::prelude::*;
use serde_json::{json, Value};

use crate::combat::{ArtilleryWeatherDump, DroneCargoDemandBook, DroneCombatModifiers, DroneMissionDump};
use crate::economy::logistics::orders::HaulWeatherDump;
use crate::entities::production::power::RenewableWeatherFactors;
use crate::systems::weather::{
    AirOpsWeatherStatus, CameraLocalWeatherSample, WeatherField, WeatherOverlayBakeState,
};

/// Rain rate (mm/h) at which a cell counts as raining.
const RAIN_COVERAGE_MM_H: f32 = 0.1;

#[derive(Clone, Copy, Debug, Default)]
struct WxSeries {
    first: Option<f64>,
    last: f64,
    min: f64,
    max: f64,
}

impl WxSeries {
    fn push(&mut self, v: f64) {
        if self.first.is_none() {
            self.first = Some(v);
            self.min = v;
            self.max = v;
        }
        self.last = v;
        self.min = self.min.min(v);
        self.max = self.max.max(v);
    }

    fn json(&self) -> Value {
        json!({ "first": self.first.unwrap_or(0.0), "last": self.last, "min": self.min, "max": self.max })
    }
}

/// Run-over-run accumulation of weather statistics (one sample per field revision).
#[derive(Resource, Debug, Default)]
pub struct WeatherDumpSeries {
    last_revision: u64,
    samples: u64,
    solves: u64,
    cells: usize,
    cell_size_m: f32,
    series: BTreeMap<&'static str, WxSeries>,
    last: Value,
}

fn track(s: &mut WeatherDumpSeries, key: &'static str, v: f64) {
    s.series.entry(key).or_default().push(v);
}

fn sample_weather_series(
    field: Option<Res<WeatherField>>,
    local: Option<Res<CameraLocalWeatherSample>>,
    bake: Option<Res<WeatherOverlayBakeState>>,
    arty: Option<Res<ArtilleryWeatherDump>>,
    haul: Option<Res<HaulWeatherDump>>,
    renew: Option<Res<RenewableWeatherFactors>>,
    air_ops: Option<Res<AirOpsWeatherStatus>>,
    drone: Option<Res<DroneMissionDump>>,
    drone_mods: Option<Res<DroneCombatModifiers>>,
    drone_demand: Option<Res<DroneCargoDemandBook>>,
    mut s: ResMut<WeatherDumpSeries>,
) {
    if !super::sim_run_dump::sim_dump_enabled(None) {
        return;
    }
    let Some(field) = field.filter(|f| f.is_ready()) else {
        return;
    };
    if field.revision == s.last_revision {
        return;
    }
    s.last_revision = field.revision;
    let snap = &field.next;
    let n = snap.wind_mps.len().max(1) as f64;
    let speeds: Vec<f64> = snap
        .wind_mps
        .iter()
        .map(|w| f64::from(Vec2::from(*w).length()))
        .collect();
    let mean = speeds.iter().sum::<f64>() / n;
    let max = speeds.iter().copied().fold(0.0, f64::max);
    let var = speeds.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n;
    let rain_pct = snap.rain_mm_h.iter().filter(|r| **r >= RAIN_COVERAGE_MM_H).count() as f64 / n * 100.0;
    let mean_vis = snap.visibility_m.iter().map(|v| f64::from(*v)).sum::<f64>() / n;
    let mean_cloud = snap.cloud.iter().map(|v| f64::from(*v)).sum::<f64>() / n;
    let mean_temp = snap.temp_c.iter().map(|v| f64::from(*v)).sum::<f64>() / n;
    let max_rain = snap.rain_mm_h.iter().copied().fold(0.0f32, f32::max);
    let max_lightning = snap.lightning.iter().copied().fold(0.0f32, f32::max);
    let (p_min, p_max) = snap
        .pressure_hpa
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), p| (a.min(*p), b.max(*p)));
    // Mean per-cell change of the wind vector across the last solve (prev -> next).
    let change = if field.prev.wind_mps.len() == snap.wind_mps.len() {
        snap.wind_mps
            .iter()
            .zip(&field.prev.wind_mps)
            .map(|(a, b)| f64::from((Vec2::from(*a) - Vec2::from(*b)).length()))
            .sum::<f64>()
            / n
    } else {
        0.0
    };
    s.samples += 1;
    s.solves = field.solves;
    s.cells = field.cell_count();
    s.cell_size_m = field.cell_size_m;
    track(&mut s, "mean_wind_mps", mean);
    track(&mut s, "max_wind_mps", max);
    track(&mut s, "wind_speed_stddev_mps", var.sqrt());
    track(&mut s, "wind_change_mps", change);
    track(&mut s, "rain_coverage_pct", rain_pct);
    track(&mut s, "systems", field.systems.len() as f64);
    track(&mut s, "mean_visibility_m", mean_vis);
    track(&mut s, "mean_cloud", mean_cloud);
    track(&mut s, "mean_temp_c", mean_temp);
    track(&mut s, "max_rain_mm_h", f64::from(max_rain));
    track(&mut s, "max_lightning", f64::from(max_lightning));
    track(&mut s, "pressure_span_hpa", f64::from(p_max - p_min));
    // W3: camera-local rain vs world mean (absolute delta in rain01 units).
    let local_rain = local
        .as_ref()
        .filter(|l| l.ready)
        .map(|l| f64::from(l.rain01))
        .unwrap_or(0.0);
    let mean_rain01 = snap
        .rain_mm_h
        .iter()
        .map(|r| f64::from((*r / 10.0).clamp(0.0, 1.0)))
        .sum::<f64>()
        / n;
    track(
        &mut s,
        "local_rain_vs_mean_abs",
        (local_rain - mean_rain01).abs(),
    );
    track(
        &mut s,
        "overlay_wind_published",
        if bake.as_ref().is_some_and(|b| b.published) {
            1.0
        } else {
            0.0
        },
    );
    // W4 combat / logistics / power evidence.
    if let Some(a) = arty.as_ref() {
        track(&mut s, "artillery_drift_tiles", f64::from(a.max_drift_tiles));
        track(&mut s, "artillery_radius_mul", f64::from(a.last_radius_mul));
    }
    if let Some(h) = haul.as_ref() {
        track(&mut s, "haul_weather_speed_mul", f64::from(h.last_mul));
        track(&mut s, "haul_weather_speed_mul_min", f64::from(h.min_mul));
    }
    if let Some(r) = renew.as_ref() {
        if r.wind_n > 0 {
            track(&mut s, "wind_cf_min", f64::from(r.wind_min));
            track(&mut s, "wind_cf_max", f64::from(r.wind_max));
        }
        if r.solar_n > 0 {
            track(&mut s, "solar_cf_min", f64::from(r.solar_min));
            track(&mut s, "solar_cf_max", f64::from(r.solar_max));
        }
    }
    // W5 + drone D0 air / missile / link-sensor hooks (centre-sample; soft on calm maps).
    if let Some(a) = air_ops.as_ref().filter(|a| a.ready) {
        track(
            &mut s,
            "air_ops_grounded",
            if a.grounded_fixed_wing { 1.0 } else { 0.0 },
        );
        track(
            &mut s,
            "airlift_hold",
            if a.airlift_hold { 1.0 } else { 0.0 },
        );
        track(&mut s, "air_ground_speed_mul", f64::from(a.ground_speed_mul));
        track(&mut s, "missile_drift_m", f64::from(a.missile_drift_m));
        track(&mut s, "smoke01_centre", f64::from(a.smoke01_centre));
        track(&mut s, "link_quality_radio", f64::from(a.link_quality_radio));
        track(&mut s, "link_quality_fiber", f64::from(a.link_quality_fiber));
        track(
            &mut s,
            "sensor_range_mul_eo",
            f64::from(a.sensor_range_mul_eo),
        );
        track(
            &mut s,
            "drone_cruise_grounded",
            if a.drone_cruise_grounded { 1.0 } else { 0.0 },
        );
    }
    // D1 mission book dump (soft when no AOs).
    if let Some(d) = drone.as_ref() {
        track(&mut s, "missions_active", f64::from(d.missions_active));
        track(&mut s, "sorties_aborted_wx", d.sorties_aborted_wx as f64);
        track(&mut s, "sorties_ok", d.sorties_ok as f64);
        track(&mut s, "ao_count", f64::from(d.ao_count));
    }
    if let Some(m) = drone_mods.as_ref() {
        track(&mut s, "cuas_intercepts", f64::from(m.cuas_intercepts));
        track(&mut s, "cruise_threat", f64::from(m.cruise_threat));
    }
    if let Some(d) = drone_demand.as_ref() {
        track(&mut s, "drone_cargo_demand_units", f64::from(d.total_units));
    }
    s.last = json!({
        "prevailing_dir_deg": field.climate.prevailing_dir_rad.to_degrees(),
        "prevailing_mps": field.climate.prevailing_speed_mps,
        "base_temp_c": field.climate.base_temp_c,
        "preset": field.climate.preset.map(|p| format!("{p:?}")),
        "lows": field.systems.iter().filter(|s| s.kind == crate::systems::weather::PressureKind::Low).count(),
        "local_rain01": local_rain,
        "overlay_bake_revision": bake.as_ref().map(|b| b.revision).unwrap_or(0),
        "arty_samples": arty.as_ref().map(|a| a.samples).unwrap_or(0),
        "haul_samples": haul.as_ref().map(|h| h.samples).unwrap_or(0),
        "air_ops_samples": air_ops.as_ref().map(|a| a.samples).unwrap_or(0),
        "drone_mission_samples": drone.as_ref().map(|d| d.samples).unwrap_or(0),
    });
}

/// `weather` block of the dump (`samples: 0` when the field never solved).
#[must_use]
pub fn weather_json(s: Option<&WeatherDumpSeries>) -> Value {
    let Some(s) = s else {
        return json!({ "samples": 0 });
    };
    let mut out = serde_json::Map::new();
    out.insert("samples".into(), json!(s.samples));
    out.insert("solves".into(), json!(s.solves));
    out.insert("cells".into(), json!(s.cells));
    out.insert("cell_size_m".into(), json!(s.cell_size_m));
    for (k, v) in &s.series {
        out.insert((*k).into(), v.json());
    }
    out.insert("climate".into(), s.last.clone());
    Value::Object(out)
}

/// Collector; registered by `SimRunDumpPlugin` (which owns the debug / feature gate).
pub struct WeatherDumpPlugin;

impl Plugin for WeatherDumpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherDumpSeries>()
            .add_systems(Last, sample_weather_series);
    }
}
