//! Construction phase tiles on the map (Syx-style occupation + optional labels).

use bevy::prelude::*;

use crate::construction::map_egui_projection::{
    tile_screen_extent_rendered, world_to_sim_map_screen_rendered,
};
use crate::construction::tile_visual::ConstructionTileVisualSettings;
use crate::engine::states::BaseState;
use crate::gui::{MainWorldCameraOrthoTrace, MapCameraDesiredRes, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::strategic::{
    ConstructionSite, PlannedSite, SiteArchetype, SiteConstructionPhase, SiteFootprint,
};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

fn site_archetype_label(archetype: SiteArchetype) -> &'static str {
    match archetype {
        SiteArchetype::CivilHousing => "Housing",
        SiteArchetype::Factory => "Factory",
        SiteArchetype::PowerPlant => "Power",
        SiteArchetype::RailDepot => "Depot",
        SiteArchetype::MilitaryBase => "Military",
        SiteArchetype::RadarSite => "Radar",
        SiteArchetype::SensorPost => "Sensor",
        SiteArchetype::TrenchLine => "Trench",
        SiteArchetype::BunkerComplex => "Bunker",
        SiteArchetype::DefensiveWall => "Wall",
        SiteArchetype::DragonTeeth => "Dragon's teeth",
        SiteArchetype::Minefield => "Minefield",
        SiteArchetype::FuelDepot => "Fuel depot",
        SiteArchetype::WaterPlant => "Water",
    }
}

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

fn phase_color(phase: SiteConstructionPhase) -> crate::gui::UiRgba {
    let [r, g, b, a] = super::phase_fill::site_phase_rgba(phase);
    crate::gui::UiRgba::from_rgba_unmultiplied(r, g, b, a)
}

fn phase_point_inside_map(map_vp: &SimulationMapViewport, pos: bevy::prelude::Vec2) -> bool {
    pos.x >= map_vp.min.x && pos.x <= map_vp.max.x && pos.y >= map_vp.min.y && pos.y <= map_vp.max.y
}


#[cfg(test)]
mod tests {
    use super::{phase_map_word, phase_point_inside_map};
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
}
