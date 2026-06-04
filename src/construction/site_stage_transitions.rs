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
        (Clearing, Some(Stumps) | None) => Some((Foundation, None)),
        (Foundation, _) => Some((UnderConstruction, None)),
        (UnderConstruction, _) => Some((Operational, None)),
        (Operational | Provisioning | Damaged | Offline | Abandoned, _) => None,
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
            (SiteConstructionPhase::Clearing, Some(ClearingSubstep::Trees)),
            (SiteConstructionPhase::Clearing, Some(ClearingSubstep::Stumps)),
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
}
