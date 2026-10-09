//! Site-phase fills the map paints after commit.
//!
//! Distinct from the place-feedback pulse, which only crossfades locked-valid green into
//! Planned blue and never shows Clearing, Foundation, or Offline.

use crate::strategic::SiteConstructionPhase;

/// Unmultiplied RGBA for one construction phase.
#[must_use]
pub fn site_phase_rgba(phase: SiteConstructionPhase) -> [u8; 4] {
    match phase {
        SiteConstructionPhase::Planned => [100, 160, 220, 200],
        SiteConstructionPhase::Surveying => [64, 196, 214, 200],
        SiteConstructionPhase::Clearing => [220, 170, 60, 210],
        SiteConstructionPhase::Foundation => [186, 96, 42, 220],
        SiteConstructionPhase::UnderConstruction => [220, 120, 40, 230],
        SiteConstructionPhase::Provisioning => [232, 176, 64, 220],
        SiteConstructionPhase::Operational => [70, 170, 95, 200],
        SiteConstructionPhase::Damaged => [160, 70, 55, 220],
        SiteConstructionPhase::Offline => [72, 86, 140, 220],
        SiteConstructionPhase::Abandoned => [120, 120, 120, 180],
    }
}

/// Ghost fill when a footprint is rejected for occupancy. Not a place-feedback settle color.
#[must_use]
pub fn occupancy_reject_rgba() -> [u8; 4] {
    [96, 42, 128, 230]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Endpoints of `place_feedback_fill_at` (unmultiplied valid green and Planned settle).
    const PLACE_FEEDBACK_VALID: [u8; 4] = [48, 140, 72, 220];
    const PLACE_FEEDBACK_SETTLE: [u8; 4] = [100, 144, 220, 200];

    #[test]
    fn sweep_foundation_rgba_is_not_a_place_feedback_color() {
        let foundation = site_phase_rgba(SiteConstructionPhase::Foundation);
        assert_ne!(foundation, PLACE_FEEDBACK_VALID);
        assert_ne!(foundation, PLACE_FEEDBACK_SETTLE);
        assert_ne!(foundation, site_phase_rgba(SiteConstructionPhase::Planned));
    }

    #[test]
    fn sweep_clearing_rgba_differs_from_foundation() {
        assert_ne!(
            site_phase_rgba(SiteConstructionPhase::Clearing),
            site_phase_rgba(SiteConstructionPhase::Foundation)
        );
    }

    #[test]
    fn sweep_offline_rgba_differs_from_damaged() {
        let offline = site_phase_rgba(SiteConstructionPhase::Offline);
        let damaged = site_phase_rgba(SiteConstructionPhase::Damaged);
        assert_ne!(offline, damaged);
        assert_ne!(offline, PLACE_FEEDBACK_VALID);
        assert_ne!(offline, PLACE_FEEDBACK_SETTLE);
    }

    #[test]
    fn drain_surveying_rgba_differs_from_planned() {
        assert_ne!(
            site_phase_rgba(SiteConstructionPhase::Surveying),
            site_phase_rgba(SiteConstructionPhase::Planned)
        );
    }

    #[test]
    fn drain_provisioning_rgba_differs_from_building() {
        assert_ne!(
            site_phase_rgba(SiteConstructionPhase::Provisioning),
            site_phase_rgba(SiteConstructionPhase::UnderConstruction)
        );
    }
}
