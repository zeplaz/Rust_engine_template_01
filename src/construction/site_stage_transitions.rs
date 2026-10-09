//! **CON-P2-002** — pure phase transition table (forest pipeline + default planned entry).

use crate::construction::site_stage::ClearingSubstep;
use crate::strategic::SiteConstructionPhase;

/// Next `(phase, substep)` after the current stage completes (`progress >= 1.0`).
#[must_use]
pub fn next_site_stage(
    current: SiteConstructionPhase,
    substep: Option<ClearingSubstep>,
) -> Option<(SiteConstructionPhase, Option<ClearingSubstep>)> {
    use ClearingSubstep::{Stumps, Trees};
    use SiteConstructionPhase::*;

    match (current, substep) {
        (Planned, _) => Some((Surveying, None)),
        (Surveying, _) => Some((Clearing, Some(Trees))),
        (Clearing, Some(Trees)) => Some((Clearing, Some(Stumps))),
        (Clearing, None) => Some((Clearing, Some(Trees))),
        (Clearing, Some(Stumps)) => Some((Foundation, None)),
        (Foundation, _) => Some((UnderConstruction, None)),
        (UnderConstruction, _) => Some((Operational, None)),
        (Operational | Provisioning | Damaged | Offline | Abandoned, _) => None,
    }
}

/// Progress added on one sim tick, clamped to the open interval. A bad step does not move the site.
#[must_use]
pub fn saturating_phase_progress(progress: f32, step: f32) -> f32 {
    if !progress.is_finite() {
        return 0.0;
    }
    let held = progress.clamp(0.0, 1.0);
    if step <= 0.0 || !step.is_finite() {
        return held;
    }
    (held + step).min(1.0)
}

/// Progress added on one sim tick. Foundation and framing take two ticks. Terminal phases stay put.
#[must_use]
pub fn phase_progress_step(phase: SiteConstructionPhase) -> f32 {
    match phase {
        SiteConstructionPhase::Clearing => 0.5,
        SiteConstructionPhase::Foundation | SiteConstructionPhase::UnderConstruction => 0.5,
        SiteConstructionPhase::Operational
        | SiteConstructionPhase::Provisioning
        | SiteConstructionPhase::Damaged
        | SiteConstructionPhase::Offline
        | SiteConstructionPhase::Abandoned => 0.0,
        _ => 1.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forest_pipeline_phase_order() {
        let steps = [
            (SiteConstructionPhase::Planned, None),
            (SiteConstructionPhase::Surveying, None),
            (
                SiteConstructionPhase::Clearing,
                Some(ClearingSubstep::Trees),
            ),
            (
                SiteConstructionPhase::Clearing,
                Some(ClearingSubstep::Stumps),
            ),
            (SiteConstructionPhase::Foundation, None),
            (SiteConstructionPhase::UnderConstruction, None),
            (SiteConstructionPhase::Operational, None),
        ];
        let mut phase = SiteConstructionPhase::Planned;
        let mut sub = None;
        for (expected_phase, expected_sub) in steps {
            assert_eq!(phase, expected_phase);
            assert_eq!(sub, expected_sub);
            let Some((nphase, nsub)) = next_site_stage(phase, sub) else {
                break;
            };
            phase = nphase;
            sub = nsub;
        }
        assert_eq!(phase, SiteConstructionPhase::Operational);
        assert!(next_site_stage(phase, sub).is_none());
    }

    #[test]
    fn drain_foundation_step_is_slower_than_survey() {
        assert!(
            phase_progress_step(SiteConstructionPhase::Foundation)
                < phase_progress_step(SiteConstructionPhase::Surveying)
        );
        assert_eq!(phase_progress_step(SiteConstructionPhase::Damaged), 0.0);
        assert_eq!(phase_progress_step(SiteConstructionPhase::Surveying), 1.0);
    }

    #[test]
    fn live_clearing_advances_slower_than_planned() {
        assert!(
            phase_progress_step(SiteConstructionPhase::Clearing)
                < phase_progress_step(SiteConstructionPhase::Planned)
        );
        assert_eq!(phase_progress_step(SiteConstructionPhase::Clearing), 0.5);
        assert_eq!(phase_progress_step(SiteConstructionPhase::Planned), 1.0);
    }

    #[test]
    fn next_clearing_without_substep_starts_on_trees() {
        assert_eq!(
            next_site_stage(SiteConstructionPhase::Clearing, None),
            Some((
                SiteConstructionPhase::Clearing,
                Some(ClearingSubstep::Trees)
            ))
        );
        assert_eq!(
            next_site_stage(
                SiteConstructionPhase::Clearing,
                Some(ClearingSubstep::Stumps)
            ),
            Some((SiteConstructionPhase::Foundation, None))
        );
    }

    #[test]
    fn next_provisioning_stage_does_not_move() {
        assert_eq!(phase_progress_step(SiteConstructionPhase::Provisioning), 0.0);
        assert!(next_site_stage(SiteConstructionPhase::Provisioning, None).is_none());
    }

    #[test]
    fn cont_phase_progress_stops_at_one() {
        assert!((saturating_phase_progress(0.75, 0.5) - 1.0).abs() < 1e-6);
        assert!((saturating_phase_progress(0.25, 0.0) - 0.25).abs() < 1e-6);
        assert!((saturating_phase_progress(f32::NAN, 0.5)).abs() < 1e-6);
        assert!((saturating_phase_progress(-0.2, 0.5) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn over_exact_progress_stays_at_one() {
        assert_eq!(saturating_phase_progress(1.0, 0.5), 1.0);
        assert_eq!(saturating_phase_progress(1.0, 0.0), 1.0);
        assert_eq!(saturating_phase_progress(1.0, 1.0), 1.0);
    }
}
