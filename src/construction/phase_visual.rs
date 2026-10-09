//! Construction phase tiles on the map (Syx-style occupation + optional labels).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::construction::map_egui_projection::{
    tile_screen_extent_rendered, world_to_sim_map_egui_rendered,
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

fn phase_color(phase: SiteConstructionPhase) -> egui::Color32 {
    let [r, g, b, a] = super::phase_fill::site_phase_rgba(phase);
    egui::Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn phase_point_inside_map(map_vp: &SimulationMapViewport, pos: egui::Pos2) -> bool {
    pos.x >= map_vp.min.x && pos.x <= map_vp.max.x && pos.y >= map_vp.min.y && pos.y <= map_vp.max.y
}

pub fn draw_construction_phase_labels_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    settings: Res<ConstructionTileVisualSettings>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesiredRes>,
    map_vp: Res<SimulationMapViewport>,
    ortho: Res<MainWorldCameraOrthoTrace>,
    params: Res<WorldGenParams>,
    sites: Query<(&ConstructionSite, &PlannedSite, &SiteFootprint)>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation | BaseState::Editor)
        || !map_vp.is_adequate_for_camera()
        || !settings.show_site_phase_tiles
    {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let layer = egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("construction_phase_tiles"),
    );
    let map_rect = egui::Rect::from_min_max(
        egui::pos2(map_vp.min.x, map_vp.min.y),
        egui::pos2(map_vp.max.x, map_vp.max.y),
    );
    let painter = ctx.layer_painter(layer).with_clip_rect(map_rect);
    let tile_px = tile_screen_extent_rendered(
        Some(ortho.as_ref()),
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    );
    let side = tile_px * 0.9;

    for (site, planned, footprint) in sites.iter() {
        if matches!(site.phase, SiteConstructionPhase::Abandoned) {
            continue;
        }
        let color = phase_color(site.phase);
        // `SiteFootprint.tiles` stores absolute world tile indices (see `footprint_tiles` in site commit).
        for tile in &footprint.tiles {
            let world = Vec3::new(tile.x as f32 + 0.5, 0.0, tile.y as f32 + 0.5);
            let Some(pos) = world_to_sim_map_egui_rendered(
                world,
                Some(ortho.as_ref()),
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            ) else {
                continue;
            };
            if !phase_point_inside_map(map_vp.as_ref(), pos) {
                continue;
            }
            let rect = egui::Rect::from_center_size(pos, egui::vec2(side, side));
            painter.rect_filled(rect, 1.0, color);
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(1.0, egui::Color32::from_gray(240)),
                egui::epaint::StrokeKind::Inside,
            );
        }

        if settings.show_tile_info_labels
            && !matches!(site.phase, SiteConstructionPhase::Operational)
        {
            if let Some(tile) = footprint.tiles.first() {
                let label_world = Vec3::new(tile.x as f32 + 0.5, 0.0, tile.y as f32 + 0.5);
                if let Some(label_pos) = world_to_sim_map_egui_rendered(
                    label_world,
                    Some(ortho.as_ref()),
                    authority.as_deref(),
                    desired.as_ref(),
                    map_vp.as_ref(),
                    params.as_ref(),
                )
                .filter(|label_pos| phase_point_inside_map(map_vp.as_ref(), *label_pos))
                {
                    let text = format!(
                        "{} · {} · {}×{}",
                        site_archetype_label(site.archetype),
                        phase_map_word(site.phase),
                        planned.footprint.width,
                        planned.footprint.depth
                    );
                    let bg = egui::Rect::from_min_max(
                        label_pos + egui::vec2(tile_px * 0.5, -tile_px * 0.4),
                        label_pos + egui::vec2(tile_px * 2.8, tile_px * 0.15),
                    );
                    painter.rect_filled(
                        bg,
                        2.0,
                        egui::Color32::from_rgba_unmultiplied(10, 12, 16, 200),
                    );
                    painter.text(
                        label_pos + egui::vec2(tile_px * 0.55, -tile_px * 0.35),
                        egui::Align2::LEFT_TOP,
                        text,
                        egui::FontId::proportional(11.0),
                        egui::Color32::WHITE,
                    );
                }
            }
        }
    }
    Ok(())
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
        let on_footprint = bevy_egui::egui::pos2(420.0, 310.0);
        let window_corner = bevy_egui::egui::pos2(6.0, 930.0);
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
