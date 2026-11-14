//! EXEC-01 (plan_ref_books E3): shared **(period, phase)** gate for heavy periodic sim work.
//!
//! Each consumer owns its constants; [`heavy_periodic_catalog`] lists them for spike scheduling tests.

/// Returns true when `step` is on the periodic fire tick for `(period, phase)`.
#[inline]
#[must_use]
pub fn periodic_step_fires(step: u64, period: u64, phase: u64) -> bool {
    let period = period.max(1);
    step % period == phase % period
}

/// One registered heavy periodic consumer (theoretical fire tick, not runtime tier gates).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HeavyPeriodicSpec {
    pub label: &'static str,
    pub period: u64,
    pub phase: u64,
    /// When set, the consumer also runs on sim step `0` (bootstrap), regardless of phase.
    pub fires_on_zero: bool,
}

/// LCM of two positive integers (for one scheduling window).
#[must_use]
pub fn lcm_u64(a: u64, b: u64) -> u64 {
    let a = a.max(1);
    let b = b.max(1);
    a / gcd_u64(a, b) * b
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

/// LCM across the catalog periods — one full repeat of all relative phases.
#[must_use]
pub fn heavy_periodic_window_ticks() -> u64 {
    heavy_periodic_catalog()
        .iter()
        .map(|s| s.period.max(1))
        .fold(1_u64, lcm_u64)
}

/// Catalog of heavy periodic consumers (periods unchanged from gameplay; phases staggered).
#[must_use]
pub fn heavy_periodic_catalog() -> &'static [HeavyPeriodicSpec] {
    use crate::combat::pressure::{
        PRESSURE_AGGREGATE_RESCORE_PERIOD_TICKS, PRESSURE_AGGREGATE_RESCORE_PHASE_TICKS,
    };
    use crate::economy::logistics::orders::plugin::{
        LOGISTICS_ORDER_AGGREGATE_PERIOD, LOGISTICS_ORDER_AGGREGATE_PHASE,
    };
    use crate::strategic::{
        FACTION_MESO_INTERNAL_PERIOD_TICKS, FACTION_MESO_INTERNAL_PHASE_TICKS,
        STAGE7_BELIEF_DECAY_PERIOD_TICKS, STAGE7_BELIEF_DECAY_PHASE_TICKS,
    };

    &[
        HeavyPeriodicSpec {
            label: "stage7_belief_decay",
            period: STAGE7_BELIEF_DECAY_PERIOD_TICKS,
            phase: STAGE7_BELIEF_DECAY_PHASE_TICKS,
            fires_on_zero: true,
        },
        HeavyPeriodicSpec {
            label: "faction_meso_internal",
            period: FACTION_MESO_INTERNAL_PERIOD_TICKS,
            phase: FACTION_MESO_INTERNAL_PHASE_TICKS,
            fires_on_zero: false,
        },
        HeavyPeriodicSpec {
            label: "pressure_aggregate_rescore",
            period: PRESSURE_AGGREGATE_RESCORE_PERIOD_TICKS,
            phase: PRESSURE_AGGREGATE_RESCORE_PHASE_TICKS,
            fires_on_zero: false,
        },
        HeavyPeriodicSpec {
            label: "logistics_order_aggregate",
            period: LOGISTICS_ORDER_AGGREGATE_PERIOD,
            phase: LOGISTICS_ORDER_AGGREGATE_PHASE,
            fires_on_zero: false,
        },
    ]
}

/// Strategic periodics that always gate full-tick work (recon spike source).
#[must_use]
pub fn heavy_periodic_strategic_catalog() -> &'static [HeavyPeriodicSpec] {
    &heavy_periodic_catalog()[..2]
}

#[must_use]
pub fn coincident_heavy_count_at(step: u64) -> u32 {
    heavy_periodic_strategic_catalog()
        .iter()
        .filter(|s| {
            (s.fires_on_zero && step == 0)
                || periodic_step_fires(step, s.period, s.phase)
        })
        .count() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lcm_window_max_coincident_heavy_systems_at_most_two() {
        let window = heavy_periodic_window_ticks();
        assert!(window >= 4, "catalog window should cover all periods");
        let max = (0..=window)
            .map(coincident_heavy_count_at)
            .max()
            .unwrap_or(0);
        assert!(
            max <= 2,
            "EXEC-01: at most 2 heavy periodic systems per tick (window={window}, max={max})"
        );
    }

    #[test]
    fn periodic_step_fires_examples() {
        assert!(periodic_step_fires(17, 4, 1));
        assert!(!periodic_step_fires(16, 4, 1));
        assert!(periodic_step_fires(22, 5, 2));
    }

    #[test]
    fn aggregate_half_rate_gates_are_out_of_phase() {
        use crate::combat::pressure::{
            PRESSURE_AGGREGATE_RESCORE_PERIOD_TICKS, PRESSURE_AGGREGATE_RESCORE_PHASE_TICKS,
        };
        use crate::economy::logistics::orders::plugin::{
            LOGISTICS_ORDER_AGGREGATE_PERIOD, LOGISTICS_ORDER_AGGREGATE_PHASE,
        };
        assert_ne!(
            PRESSURE_AGGREGATE_RESCORE_PHASE_TICKS % PRESSURE_AGGREGATE_RESCORE_PERIOD_TICKS,
            LOGISTICS_ORDER_AGGREGATE_PHASE % LOGISTICS_ORDER_AGGREGATE_PERIOD,
        );
        let window = lcm_u64(
            PRESSURE_AGGREGATE_RESCORE_PERIOD_TICKS,
            LOGISTICS_ORDER_AGGREGATE_PERIOD,
        );
        for step in 0..=window {
            assert!(
                !(periodic_step_fires(step, PRESSURE_AGGREGATE_RESCORE_PERIOD_TICKS, PRESSURE_AGGREGATE_RESCORE_PHASE_TICKS)
                    && periodic_step_fires(step, LOGISTICS_ORDER_AGGREGATE_PERIOD, LOGISTICS_ORDER_AGGREGATE_PHASE)),
                "aggregate pressure/logistics must not stack on tick {step}"
            );
        }
    }
}