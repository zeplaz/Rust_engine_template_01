//! NAV-13 — time-varying edge cost over piecewise-constant weather buckets.
//!
//! P0: pure integrate. P1+: [`LogisticsCostForecastBook`] sole-written from stations + field rhythm.

use bevy::prelude::*;

use crate::economy::logistics::orders::weather_haul_speed_mul;
use crate::systems::sim_control::{SimStepClock, SimTimeMicros};
use crate::systems::weather::{WeatherField, WeatherPollingStations, WeatherSample};

/// One constant speed-mul window starting at `start_tick` (inclusive) until the next bucket.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeatherCostBucket {
    pub start_tick: u64,
    /// Haul speed multiplier in `(0.25..=1.0)` — invert for cost.
    pub mul: f32,
}

impl WeatherCostBucket {
    #[must_use]
    pub fn from_sample(start_tick: u64, sample: &WeatherSample) -> Self {
        Self {
            start_tick,
            mul: weather_haul_speed_mul(sample),
        }
    }
}

pub const FORECAST_BUCKET_COUNT: usize = 4;
pub const FORECAST_MUL_EPS: f32 = 1e-3;

/// DERIVED forecast book — omit from save. Sole writer: [`sync_logistics_cost_forecast`].
#[derive(Resource, Debug, Clone)]
pub struct LogisticsCostForecastBook {
    pub ready: bool,
    pub revision: u64,
    pub bucket_secs: u32,
    pub buckets: [WeatherCostBucket; FORECAST_BUCKET_COUNT],
    pub min_mul: f32,
    pub max_mul: f32,
    /// Last revision consumed by route invalidation (N13-P3).
    pub consumed_revision: u64,
}

impl Default for LogisticsCostForecastBook {
    fn default() -> Self {
        Self {
            ready: false,
            revision: 0,
            bucket_secs: 1,
            buckets: [
                WeatherCostBucket {
                    start_tick: 0,
                    mul: 1.0,
                },
                WeatherCostBucket {
                    start_tick: 0,
                    mul: 1.0,
                },
                WeatherCostBucket {
                    start_tick: 0,
                    mul: 1.0,
                },
                WeatherCostBucket {
                    start_tick: 0,
                    mul: 1.0,
                },
            ],
            min_mul: 1.0,
            max_mul: 1.0,
            consumed_revision: 0,
        }
    }
}

impl LogisticsCostForecastBook {
    #[must_use]
    pub fn as_slice(&self) -> &[WeatherCostBucket] {
        &self.buckets
    }

    /// Integrate using this book's buckets when ready; else static `base_mul * traverse_ticks`.
    #[must_use]
    pub fn edge_cost(&self, base_mul: f32, depart_tick: u64, traverse_ticks: u64) -> f32 {
        if !self.ready {
            return base_mul.max(1e-3) * traverse_ticks as f32;
        }
        integrated_edge_cost(base_mul, depart_tick, traverse_ticks, self.as_slice())
    }
}

/// Cost contribution of one tick at the given haul speed mul.
#[inline]
#[must_use]
pub fn cost_per_tick(base_mul: f32, speed_mul: f32) -> f32 {
    let base = base_mul.max(1e-3);
    let speed = speed_mul.clamp(0.25, 1.0);
    base / speed
}

/// Integrate edge cost over `[depart_tick, depart_tick + traverse_ticks)`.
#[must_use]
pub fn integrated_edge_cost(
    base_mul: f32,
    depart_tick: u64,
    traverse_ticks: u64,
    buckets: &[WeatherCostBucket],
) -> f32 {
    if traverse_ticks == 0 {
        return 0.0;
    }
    let end = depart_tick.saturating_add(traverse_ticks);
    let mut total = 0.0_f32;
    let mut t = depart_tick;

    while t < end {
        let (mul, span_end) = active_mul_until(t, end, buckets);
        let span = span_end.saturating_sub(t);
        if span == 0 {
            break;
        }
        total += cost_per_tick(base_mul, mul) * span as f32;
        t = span_end;
    }
    total
}

#[inline]
#[must_use]
pub fn edge_cost_at(
    base_mul: f32,
    depart_tick: u64,
    traverse_ticks: u64,
    buckets: &[WeatherCostBucket],
) -> f32 {
    integrated_edge_cost(base_mul, depart_tick, traverse_ticks, buckets)
}

fn active_mul_until(t: u64, end: u64, buckets: &[WeatherCostBucket]) -> (f32, u64) {
    if buckets.is_empty() {
        return (1.0, end);
    }

    let mut idx = None;
    for (i, b) in buckets.iter().enumerate() {
        if b.start_tick <= t {
            idx = Some(i);
        } else {
            break;
        }
    }

    match idx {
        None => {
            let first = &buckets[0];
            let until = first.start_tick.min(end);
            (first.mul, if until > t { until } else { end })
        }
        Some(i) => {
            let mul = buckets[i].mul;
            let next = if i + 1 < buckets.len() {
                buckets[i + 1].start_tick
            } else {
                end
            };
            let until = next.min(end);
            let until = if until <= t {
                t.saturating_add(1).min(end)
            } else {
                until
            };
            (mul, until)
        }
    }
}

fn micros_to_tick(micros: u64, step_micros: u64) -> u64 {
    micros / step_micros.max(1)
}

/// Build four buckets: prev → now → next → far, using haul speed muls.
#[must_use]
pub fn build_forecast_buckets(
    prev: &WeatherSample,
    now: &WeatherSample,
    next: &WeatherSample,
    far: &WeatherSample,
    prev_tick: u64,
    now_tick: u64,
    next_tick: u64,
    far_tick: u64,
) -> [WeatherCostBucket; FORECAST_BUCKET_COUNT] {
    let t0 = prev_tick;
    let mut t1 = now_tick.max(t0);
    let mut t2 = next_tick.max(t1);
    let mut t3 = far_tick.max(t2);
    // Ensure strictly non-decreasing starts for the integrator.
    if t1 == t0 {
        t1 = t0.saturating_add(1);
    }
    if t2 <= t1 {
        t2 = t1.saturating_add(1);
    }
    if t3 <= t2 {
        t3 = t2.saturating_add(1);
    }
    [
        WeatherCostBucket::from_sample(t0, prev),
        WeatherCostBucket::from_sample(t1, now),
        WeatherCostBucket::from_sample(t2, next),
        WeatherCostBucket::from_sample(t3, far),
    ]
}

fn buckets_changed(a: &[WeatherCostBucket; 4], b: &[WeatherCostBucket; 4]) -> bool {
    for i in 0..4 {
        if (a[i].mul - b[i].mul).abs() > FORECAST_MUL_EPS {
            return true;
        }
        if a[i].start_tick != b[i].start_tick {
            return true;
        }
    }
    false
}

/// Sole writer of [`LogisticsCostForecastBook`].
pub fn sync_logistics_cost_forecast(
    stations: Res<WeatherPollingStations>,
    field: Option<Res<WeatherField>>,
    sim_time: Option<Res<SimTimeMicros>>,
    clock: Option<Res<SimStepClock>>,
    mut book: ResMut<LogisticsCostForecastBook>,
) {
    let step_micros = clock.as_ref().map(|c| c.step_micros).unwrap_or(16_667);
    let steps = clock.as_ref().map(|c| c.steps).unwrap_or(0);

    let centre = if stations.ready {
        stations.centre
    } else {
        WeatherSample::default()
    };

    if let Some(field) = field.filter(|f| f.is_ready()) {
        let centre_m = field.extent_m * 0.5;
        let prev = field.sample(centre_m, SimTimeMicros(field.prev_micros));
        let next = field.sample(centre_m, SimTimeMicros(field.next_micros));
        let far_t = field
            .next_micros
            .saturating_add(u64::from(field.step_secs) * 1_000_000);
        let far = field.sample(centre_m, SimTimeMicros(far_t));
        let now = if stations.ready {
            centre
        } else {
            let t = sim_time
                .as_deref()
                .copied()
                .unwrap_or(SimTimeMicros(field.prev_micros));
            field.sample(centre_m, t)
        };
        let prev_tick = micros_to_tick(field.prev_micros, step_micros);
        let now_tick = steps.max(prev_tick);
        let next_tick = micros_to_tick(field.next_micros, step_micros).max(now_tick);
        let far_tick = micros_to_tick(far_t, step_micros).max(next_tick);
        let buckets =
            build_forecast_buckets(&prev, &now, &next, &far, prev_tick, now_tick, next_tick, far_tick);
        apply_buckets(&mut book, buckets, field.step_secs.max(1));
        return;
    }

    let now_tick = steps;
    let buckets = build_forecast_buckets(
        &centre,
        &centre,
        &centre,
        &centre,
        now_tick.saturating_sub(1),
        now_tick,
        now_tick.saturating_add(1),
        now_tick.saturating_add(2),
    );
    apply_buckets(&mut book, buckets, 1);
}

fn apply_buckets(
    book: &mut LogisticsCostForecastBook,
    buckets: [WeatherCostBucket; FORECAST_BUCKET_COUNT],
    bucket_secs: u32,
) {
    let min_mul = buckets.iter().map(|b| b.mul).fold(f32::INFINITY, f32::min);
    let max_mul = buckets.iter().map(|b| b.mul).fold(0.0_f32, f32::max);
    let changed = !book.ready || buckets_changed(&book.buckets, &buckets);
    book.ready = true;
    book.bucket_secs = bucket_secs;
    book.buckets = buckets;
    book.min_mul = if min_mul.is_finite() { min_mul } else { 1.0 };
    book.max_mul = max_mul;
    if changed {
        book.revision = book.revision.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_clear_buckets_equal_base_times_traverse() {
        let buckets = [
            WeatherCostBucket {
                start_tick: 0,
                mul: 1.0,
            },
            WeatherCostBucket {
                start_tick: 100,
                mul: 1.0,
            },
        ];
        let c = integrated_edge_cost(2.0, 10, 40, &buckets);
        assert!((c - 2.0 * 40.0).abs() < 1e-4, "got {c}");
    }

    #[test]
    fn bad_window_mid_trip_costs_more_than_departing_earlier() {
        let buckets = [
            WeatherCostBucket {
                start_tick: 0,
                mul: 1.0,
            },
            WeatherCostBucket {
                start_tick: 50,
                mul: 0.25,
            },
        ];
        let late = integrated_edge_cost(1.0, 40, 30, &buckets);
        let early = integrated_edge_cost(1.0, 0, 30, &buckets);
        assert!(late > early, "late={late} early={early}");
        assert!((early - 30.0).abs() < 1e-4);
        assert!((late - 90.0).abs() < 1e-3, "got {late}");
    }

    #[test]
    fn identical_calls_bit_identical() {
        let buckets = [
            WeatherCostBucket {
                start_tick: 0,
                mul: 0.8,
            },
            WeatherCostBucket {
                start_tick: 25,
                mul: 0.5,
            },
            WeatherCostBucket {
                start_tick: 80,
                mul: 1.0,
            },
        ];
        let a = integrated_edge_cost(1.5, 10, 100, &buckets);
        let b = integrated_edge_cost(1.5, 10, 100, &buckets);
        assert_eq!(a.to_bits(), b.to_bits());
    }

    #[test]
    fn from_sample_matches_haul_helper() {
        let s = WeatherSample {
            rain_mm_h: 10.0,
            humidity: 0.95,
            ..Default::default()
        };
        let b = WeatherCostBucket::from_sample(0, &s);
        assert!((b.mul - weather_haul_speed_mul(&s)).abs() < 1e-6);
    }

    #[test]
    fn forecast_buckets_stable_revision() {
        let clear = WeatherSample::default();
        let a = build_forecast_buckets(&clear, &clear, &clear, &clear, 0, 10, 20, 30);
        let b = build_forecast_buckets(&clear, &clear, &clear, &clear, 0, 10, 20, 30);
        assert!(!buckets_changed(&a, &b));
        let mut book = LogisticsCostForecastBook::default();
        apply_buckets(&mut book, a, 60);
        let rev = book.revision;
        apply_buckets(&mut book, b, 60);
        assert_eq!(book.revision, rev);
        let storm = WeatherSample {
            rain_mm_h: 12.0,
            humidity: 0.99,
            snow: true,
            ..Default::default()
        };
        let c = build_forecast_buckets(&clear, &storm, &storm, &storm, 0, 10, 20, 30);
        apply_buckets(&mut book, c, 60);
        assert!(book.revision > rev);
    }
}
