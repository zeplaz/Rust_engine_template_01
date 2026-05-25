//! Construction phase tiles on the map (Syx-style occupation + optional labels).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::engine::states::BaseState;
use crate::gui::{
    construction_phase_on_instanced_path, MapCameraDesired, SimulationMapViewport,
    TileDebugInstanceMap, TileGpuDebugSettings,
};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::construction::map_egui_projection::{tile_screen_extent, world_to_sim_map_egui};
use crate::construction::tile_visual::ConstructionTileVisualSettings;
use crate::strategic::{
    ConstructionSite, PlannedSite, SiteArchetype, SiteConstructionPhase, SiteFootprint,
};

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
        SiteArchetype::FuelDepot => "Fuel depot",
        SiteArchetype::WaterPlant => "Water",
    }
}

fn phase_label(phase: SiteConstructionPhase) -> &'static str {
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
    match phase {
        SiteConstructionPhase::Planned | SiteConstructionPhase::Surveying => {
            egui::Color32::from_rgba_unmultiplied(100, 160, 220, 200)
        }
        SiteConstructionPhase::Clearing | SiteConstructionPhase::Foundation => {
            egui::Color32::from_rgba_unmultiplied(220, 170, 60, 210)
        }
        SiteConstructionPhase::UnderConstruction | SiteConstructionPhase::Provisioning => {
            egui::Color32::from_rgba_unmultiplied(220, 120, 40, 230)
        }
        SiteConstructionPhase::Operational => {
            egui::Color32::from_rgba_unmultiplied(70, 170, 95, 200)
        }
        SiteConstructionPhase::Damaged | SiteConstructionPhase::Offline => {
            egui::Color32::from_rgba_unmultiplied(160, 70, 55, 220)
        }
        _ => egui::Color32::from_rgba_unmultiplied(120, 120, 120, 180),
    }
}

pub fn draw_construction_phase_labels_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    settings: Res<ConstructionTileVisualSettings>,
    tile_debug: Option<Res<TileDebugInstanceMap>>,
    gpu_tile: Option<Res<TileGpuDebugSettings>>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    params: Res<WorldGenParams>,
    sites: Query<(&ConstructionSite, &PlannedSite, &SiteFootprint)>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation | BaseState::Editor)
        || !map_vp.is_adequate_for_camera()
        || !settings.show_site_phase_tiles
    {
        return Ok(());
    }
    if let (Some(map), Some(tile_settings)) = (tile_debug.as_deref(), gpu_tile.as_deref()) {
        if construction_phase_on_instanced_path(map, tile_settings) {
            return Ok(());
        }
    }
    let ctx = contexts.ctx_mut()?;
    let layer = egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("construction_phase_tiles"),
    );
    let painter = ctx.layer_painter(layer);
    let tile_px = tile_screen_extent(
        authority.as_deref(),
        desired.as_ref(),
        map_vp.as_ref(),
        params.as_ref(),
    );
    let side = tile_px * 0.9;

    for (site, planned, footprint) in sites.iter() {
        if matches!(
            site.phase,
            SiteConstructionPhase::Abandoned
        ) {
            continue;
        }
        let color = phase_color(site.phase);
        for local in &footprint.tiles {
            let world = Vec3::new(
                planned.origin.x as f32 + local.x as f32 + 0.5,
                0.0,
                planned.origin.z as f32 + local.y as f32 + 0.5,
            );
            let Some(pos) = world_to_sim_map_egui(
                world,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            ) else {
                continue;
            };
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
            if let Some(local) = footprint.tiles.first() {
                let label_world = Vec3::new(
                    planned.origin.x as f32 + local.x as f32 + 0.5,
                    0.0,
                    planned.origin.z as f32 + local.y as f32 + 0.5,
                );
                if let Some(label_pos) = world_to_sim_map_egui(
                    label_world,
                    authority.as_deref(),
                    desired.as_ref(),
                    map_vp.as_ref(),
                    params.as_ref(),
                ) {
                    let text = format!(
                        "{} · {} · {}×{}",
                        site_archetype_label(site.archetype),
                        phase_label(site.phase),
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
