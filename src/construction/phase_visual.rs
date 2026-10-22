//! Construction phase tiles on the map (Syx-style occupation + optional labels).


use crate::gui::SimulationMapViewport;
use crate::strategic::SiteConstructionPhase;

pub fn phase_map_word(phase: SiteConstructionPhase) -> &'static str {
    match phase {
        SiteConstructionPhase::Planned => "Planned",
        SiteConstructionPhase::Surveying => "Survey",
        SiteConstructionPhase::Clearing => "Clearing",
        SiteConstructionPhase::Foundation => "Foundation",
        SiteConstructionPhase::UnderConstruction => "Building",
        SiteConstructionPhase::Provisioning => "Provisioning",
        SiteConstructionPhase::Operational => "Built",
        SiteConstructionPhase::Damaged => "Damaged",
        SiteConstructionPhase::Offline => "Offline",
        SiteConstructionPhase::Abandoned => "Abandoned",
    }
}

/// Ebbtide FOB-style modular stack band (shell → circulation → aperture → fittings).
/// Presentation copy for phase readability without relying on hue alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhaseStackBand {
    Shell,
    Circulation,
    Aperture,
    Fittings,
    Complete,
    Compromised,
}

#[must_use]
pub fn phase_stack_band(phase: SiteConstructionPhase) -> PhaseStackBand {
    match phase {
        SiteConstructionPhase::Planned | SiteConstructionPhase::Surveying => PhaseStackBand::Shell,
        SiteConstructionPhase::Clearing | SiteConstructionPhase::Foundation => {
            PhaseStackBand::Circulation
        }
        SiteConstructionPhase::UnderConstruction => PhaseStackBand::Aperture,
        SiteConstructionPhase::Provisioning => PhaseStackBand::Fittings,
        SiteConstructionPhase::Operational => PhaseStackBand::Complete,
        SiteConstructionPhase::Damaged
        | SiteConstructionPhase::Offline
        | SiteConstructionPhase::Abandoned => PhaseStackBand::Compromised,
    }
}

#[must_use]
pub fn phase_stack_band_word(band: PhaseStackBand) -> &'static str {
    match band {
        PhaseStackBand::Shell => "Shell",
        PhaseStackBand::Circulation => "Circulation",
        PhaseStackBand::Aperture => "Aperture",
        PhaseStackBand::Fittings => "Fittings",
        PhaseStackBand::Complete => "Complete",
        PhaseStackBand::Compromised => "Compromised",
    }
}

#[allow(dead_code)] // egui-retired; kept for Bevy port / export
fn phase_point_inside_map(map_vp: &SimulationMapViewport, pos: bevy::prelude::Vec2) -> bool {
    pos.x >= map_vp.min.x && pos.x <= map_vp.max.x && pos.y >= map_vp.min.y && pos.y <= map_vp.max.y
}


#[cfg(test)]
mod tests {
    use super::{
        phase_map_word, phase_point_inside_map, phase_stack_band, phase_stack_band_word,
        PhaseStackBand,
    };
    use crate::gui::SimulationMapViewport;
    use crate::strategic::SiteConstructionPhase;

    #[test]
    fn phase_quads_outside_the_map_viewport_are_not_painted() {
        let map = SimulationMapViewport {
            valid: true,
            min: bevy::prelude::Vec2::new(180.0, 72.0),
            max: bevy::prelude::Vec2::new(1100.0, 760.0),
            ..SimulationMapViewport::default()
        };
        let on_footprint = bevy::prelude::Vec2::new(420.0, 310.0);
        let window_corner = bevy::prelude::Vec2::new(6.0, 930.0);
        assert!(phase_point_inside_map(&map, on_footprint));
        assert!(!phase_point_inside_map(&map, window_corner));
    }

    #[test]
    fn push_phase_words_keep_damaged_off_built() {
        assert_ne!(
            phase_map_word(SiteConstructionPhase::Damaged),
            phase_map_word(SiteConstructionPhase::Operational)
        );
        assert_ne!(
            phase_map_word(SiteConstructionPhase::Abandoned),
            phase_map_word(SiteConstructionPhase::Offline)
        );
        assert!(!phase_map_word(SiteConstructionPhase::Provisioning).is_empty());
    }

    #[test]
    fn over_survey_word_stays_off_planned() {
        assert_eq!(phase_map_word(SiteConstructionPhase::Surveying), "Survey");
        assert_ne!(
            phase_map_word(SiteConstructionPhase::Surveying),
            phase_map_word(SiteConstructionPhase::Planned)
        );
        assert_ne!(
            phase_map_word(SiteConstructionPhase::Foundation),
            phase_map_word(SiteConstructionPhase::UnderConstruction)
        );
    }

    #[test]
    fn fob_style_stack_bands_progress_without_hue() {
        assert_eq!(
            phase_stack_band(SiteConstructionPhase::Planned),
            PhaseStackBand::Shell
        );
        assert_eq!(
            phase_stack_band(SiteConstructionPhase::Foundation),
            PhaseStackBand::Circulation
        );
        assert_eq!(
            phase_stack_band(SiteConstructionPhase::UnderConstruction),
            PhaseStackBand::Aperture
        );
        assert_eq!(
            phase_stack_band(SiteConstructionPhase::Provisioning),
            PhaseStackBand::Fittings
        );
        assert_eq!(
            phase_stack_band(SiteConstructionPhase::Operational),
            PhaseStackBand::Complete
        );
        assert_ne!(
            phase_stack_band_word(PhaseStackBand::Shell),
            phase_stack_band_word(PhaseStackBand::Fittings)
        );
    }
}
