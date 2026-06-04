//! Rolling 60s perf attribution for readiness witness (**PERF-VIS-004**).

use bevy::prelude::*;

use crate::gui::hud::frame_budget_diagnostics::{FrameBudgetBucket, FrameBudgetDiagnostics};

/// ~60s at 60 Hz — rolling window for p95 attribution.
pub const PERF_ATTRIBUTION_WINDOW: usize = 3600;

#[derive(Clone, Debug, Default)]
struct SampleRing {
    buf: Vec<f32>,
    head: usize,
    len: usize,
}

impl SampleRing {
    fn with_capacity(cap: usize) -> Self {
        Self {
            buf: vec![0.0; cap],
            head: 0,
            len: 0,
        }
    }

    fn push(&mut self, value: f32) {
        if value <= 0.0 {
            return;
        }
        let cap = self.buf.len();
        if self.len < cap {
            self.buf[self.len] = value;
            self.len += 1;
        } else {
            self.buf[self.head] = value;
            self.head = (self.head + 1) % cap;
        }
    }

    fn p95(&self) -> f32 {
        percentile_from_slice(sample_slice(self), 0.95)
    }
}

fn sample_slice(ring: &SampleRing) -> Vec<f32> {
    if ring.len == 0 {
        return Vec::new();
    }
    let cap = ring.buf.len();
    if ring.len < cap {
        return ring.buf[..ring.len].to_vec();
    }
    let mut out = Vec::with_capacity(cap);
    out.extend_from_slice(&ring.buf[ring.head..]);
    out.extend_from_slice(&ring.buf[..ring.head]);
    out
}

#[must_use]
pub fn percentile_from_slice(mut values: Vec<f32>, p: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((values.len() as f32 - 1.0) * p.clamp(0.0, 1.0)).round() as usize;
    values[idx.min(values.len() - 1)]
}

/// Rolling p95 samples for frame wall, minimap raster bucket, and fire pipeline attrib.
#[derive(Resource, Clone, Debug)]
pub struct PerfAttributionWitness {
    frame_ms: SampleRing,
    raster_b_ms: SampleRing,
    view_fire_ms: SampleRing,
    pub frames_recorded: u64,
}

impl Default for PerfAttributionWitness {
    fn default() -> Self {
        Self {
            frame_ms: SampleRing::with_capacity(PERF_ATTRIBUTION_WINDOW),
            raster_b_ms: SampleRing::with_capacity(PERF_ATTRIBUTION_WINDOW),
            view_fire_ms: SampleRing::with_capacity(PERF_ATTRIBUTION_WINDOW),
            frames_recorded: 0,
        }
    }
}

impl PerfAttributionWitness {
    pub fn record_frame(
        &mut self,
        frame_ms: f32,
        raster_b_ms: f32,
        view_fire_ms: f32,
    ) {
        self.frame_ms.push(frame_ms);
        self.raster_b_ms.push(raster_b_ms);
        self.view_fire_ms.push(view_fire_ms);
        self.frames_recorded = self.frames_recorded.saturating_add(1);
    }

    #[must_use]
    pub fn p95_frame_ms(&self) -> f32 {
        self.frame_ms.p95()
    }

    #[must_use]
    pub fn p95_raster_b_ms(&self) -> f32 {
        self.raster_b_ms.p95()
    }

    #[must_use]
    pub fn p95_view_fire_ms(&self) -> f32 {
        self.view_fire_ms.p95()
    }

    #[must_use]
    pub fn window_samples(&self) -> usize {
        self.frame_ms.len.min(PERF_ATTRIBUTION_WINDOW)
    }
}

#[must_use]
pub fn perf_attribution_witness_json(w: &PerfAttributionWitness) -> serde_json::Value {
    serde_json::json!({
        "window_target_frames": PERF_ATTRIBUTION_WINDOW,
        "window_samples": w.window_samples(),
        "frames_recorded": w.frames_recorded,
        "p95_frame_ms": w.p95_frame_ms(),
        "p95_raster_b_ms": w.p95_raster_b_ms(),
        "p95_view_fire_ms": w.p95_view_fire_ms(),
    })
}

pub fn reset_perf_attribution_witness_on_enter_simulation(mut w: ResMut<PerfAttributionWitness>) {
    *w = PerfAttributionWitness::default();
}

pub fn sync_perf_attribution_witness_system(
    budget: Option<Res<FrameBudgetDiagnostics>>,
    attrib: Option<Res<crate::render::FrameUpdateAttrib>>,
    mut witness: ResMut<PerfAttributionWitness>,
) {
    let Some(budget) = budget else {
        return;
    };
    let raster_b = budget.buckets[FrameBudgetBucket::MinimapRaster.index()].last_ms;
    let view_fire = attrib
        .as_deref()
        .map(|a| a.fire_pipeline_ms)
        .unwrap_or(0.0);
    witness.record_frame(budget.frame_time_ms, raster_b, view_fire);
}

/// Lib / disk refresh fixture — rolling p95 samples without `--test visual`.
#[must_use]
pub fn perf_attribution_witness_lib_fixture() -> PerfAttributionWitness {
    let mut witness = PerfAttributionWitness::default();
    for frame_ms in [14.0_f32, 16.0, 18.0, 20.0, 22.0, 24.0] {
        for _ in 0..120 {
            witness.record_frame(frame_ms, frame_ms * 0.35, frame_ms * 0.12);
        }
    }
    witness
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percentile_p95_matches_sorted_index() {
        let values: Vec<f32> = (1..=100).map(|v| v as f32).collect();
        let p95 = percentile_from_slice(values, 0.95);
        assert!((p95 - 95.0).abs() < 0.01);
    }

    #[test]
    fn ring_p95_tracks_spikes() {
        let mut w = PerfAttributionWitness::default();
        for _ in 0..94 {
            w.record_frame(10.0, 2.0, 1.0);
        }
        for _ in 0..6 {
            w.record_frame(10.0, 50.0, 20.0);
        }
        assert!(w.p95_raster_b_ms() >= 50.0);
        assert!(w.p95_view_fire_ms() >= 20.0);
    }
}
