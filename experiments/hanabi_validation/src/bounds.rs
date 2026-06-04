//! Designer bounds from [`hanabi_event_vfx_style_bounds_v1.md`](../../../src/dev/hanabi_event_vfx_style_bounds_v1.md).

/// Max instances per local burst (DESIGN-HANABI-BOUNDS-001).
pub const MAX_INSTANCES_PER_EVENT: u32 = 32;
/// Lifetime window (seconds).
pub const LIFETIME_MIN_S: f32 = 0.2;
pub const LIFETIME_MAX_S: f32 = 1.2;
/// Peak alpha per particle.
pub const PEAK_ALPHA_MAX: f32 = 0.45;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundsVerdict {
    Pass,
    Tune,
    Reject,
}

#[derive(Clone, Debug)]
pub struct L3EventMetrics {
    pub id: &'static str,
    pub domain: &'static str,
    pub peak_instances: u32,
    pub lifetime_min_s: f32,
    pub lifetime_max_s: f32,
    pub peak_alpha: f32,
    pub designer_intent: &'static str,
}

impl L3EventMetrics {
    #[must_use]
    pub fn verdict(&self) -> BoundsVerdict {
        if self.peak_instances > MAX_INSTANCES_PER_EVENT
            || self.peak_alpha > PEAK_ALPHA_MAX
            || self.lifetime_min_s < LIFETIME_MIN_S - f32::EPSILON
            || self.lifetime_max_s > LIFETIME_MAX_S + f32::EPSILON
        {
            return BoundsVerdict::Reject;
        }
        let tune = self.peak_instances > 24
            || self.peak_alpha > 0.38
            || self.lifetime_max_s > 1.0;
        if tune {
            BoundsVerdict::Tune
        } else {
            BoundsVerdict::Pass
        }
    }
}

#[must_use]
pub fn spike_event_catalog() -> Vec<L3EventMetrics> {
    vec![
        L3EventMetrics {
            id: "fire_ember_burst",
            domain: "fire_edge",
            peak_instances: 24,
            lifetime_min_s: 0.35,
            lifetime_max_s: 0.85,
            peak_alpha: 0.35,
            designer_intent: "ACCEPT — material kick-up at fire edge",
        },
        L3EventMetrics {
            id: "water_splash_mist",
            domain: "water_surface",
            peak_instances: 16,
            lifetime_min_s: 0.25,
            lifetime_max_s: 0.55,
            peak_alpha: 0.28,
            designer_intent: "ACCEPT — local splash mist",
        },
        L3EventMetrics {
            id: "construction_micro_spark",
            domain: "construction_commit",
            peak_instances: 8,
            lifetime_min_s: 0.2,
            lifetime_max_s: 0.35,
            peak_alpha: 0.22,
            designer_intent: "ACCEPT — one-shot commit spark",
        },
        L3EventMetrics {
            id: "reject_arcade_muzzle_stack",
            domain: "anti_pattern",
            peak_instances: 96,
            lifetime_min_s: 0.1,
            lifetime_max_s: 2.5,
            peak_alpha: 0.92,
            designer_intent: "REJECT — neon muzzle / screen-fill (not for merge)",
        },
    ]
}

#[derive(Clone, Debug, Default)]
pub struct SpikeAggregate {
    pub peak_instances_frame: u32,
    pub worst_alpha: f32,
    pub lifetime_histogram: Vec<(String, u32)>,
}

#[must_use]
pub fn aggregate_spike_metrics(events: &[L3EventMetrics]) -> SpikeAggregate {
    let pass_tune: Vec<_> = events
        .iter()
        .filter(|e| e.verdict() != BoundsVerdict::Reject)
        .collect();
    let peak_instances_frame = pass_tune
        .iter()
        .map(|e| e.peak_instances)
        .max()
        .unwrap_or(0);
    let worst_alpha = pass_tune
        .iter()
        .map(|e| e.peak_alpha)
        .fold(0.0_f32, f32::max);
    let mut buckets = [
        ("0.2-0.4s", 0_u32),
        ("0.4-0.8s", 0_u32),
        ("0.8-1.2s", 0_u32),
    ];
    for e in &pass_tune {
        let mid = (e.lifetime_min_s + e.lifetime_max_s) * 0.5;
        let slot = if mid < 0.4 {
            0
        } else if mid < 0.8 {
            1
        } else {
            2
        };
        buckets[slot].1 += 1;
    }
    SpikeAggregate {
        peak_instances_frame,
        worst_alpha,
        lifetime_histogram: buckets
            .into_iter()
            .map(|(k, v)| (k.to_string(), v))
            .collect(),
    }
}
