//! H-A-SPIKE-001 — Hanabi validation experiment (Layer 3 embellishment bounds only).

pub mod bounds;
pub mod hanabi_gate;
pub mod report;

pub use bounds::{
    aggregate_spike_metrics, spike_event_catalog, BoundsVerdict, L3EventMetrics, SpikeAggregate,
    LIFETIME_MAX_S, LIFETIME_MIN_S, MAX_INSTANCES_PER_EVENT, PEAK_ALPHA_MAX,
};
pub use report::{build_default_report, render_report_v1};

pub const REPORT_V1_PATH: &str = "experiments/hanabi_validation/report_v1.md";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pass_tune_presets_within_designer_bounds() {
        for e in spike_event_catalog() {
            if e.id == "reject_arcade_muzzle_stack" {
                assert_eq!(e.verdict(), BoundsVerdict::Reject);
                continue;
            }
            assert!(
                e.verdict() != BoundsVerdict::Reject,
                "preset {} should not REJECT",
                e.id
            );
        }
        let agg = aggregate_spike_metrics(&spike_event_catalog());
        assert_eq!(agg.peak_instances_frame, 24);
        assert!(agg.worst_alpha <= PEAK_ALPHA_MAX);
    }

    #[test]
    fn report_v1_matches_aggregate() {
        let body = build_default_report();
        assert!(body.contains("Peak instances / frame | 24"));
        assert!(body.contains("fire_ember_burst"));
        assert!(body.contains("REJECT"));
    }
}
