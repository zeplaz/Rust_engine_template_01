//! Weather series for the sim run dump (counts only, no verdicts).
//!
//! Sampled once per weather field solve from [`WeatherField`] (the authority). The dump reads
//! [`WeatherDumpSeries`] through one hook in `sim_run_dump.rs`; the matrix scores it.

use std::collections::BTreeMap;

use bevy::prelude::*;
use serde_json::{json, Value};

use crate::systems::weather::WeatherField;

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

fn sample_weather_series(field: Option<Res<WeatherField>>, mut s: ResMut<WeatherDumpSeries>) {
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
    s.last = json!({
        "prevailing_dir_deg": field.climate.prevailing_dir_rad.to_degrees(),
        "prevailing_mps": field.climate.prevailing_speed_mps,
        "base_temp_c": field.climate.base_temp_c,
        "preset": field.climate.preset.map(|p| format!("{p:?}")),
        "lows": field.systems.iter().filter(|s| s.kind == crate::systems::weather::PressureKind::Low).count(),
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
