//! Ghost footprint validity hint over the simulation map viewport.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::engine::states::BaseState;
use crate::construction::map_egui_projection::{tile_screen_extent, world_to_sim_map_egui};
use crate::construction::tile_visual::ConstructionTileVisualSettings;
use crate::gui::{MapCameraDesired, SimulationMapViewport};
use crate::render::view_runtime::ViewProjectionAuthority;

use super::build_confidence::{confidence_from_validation, BuildConfidence};
use super::build_state::{BuildGhostState, BuildPlacementPreview};
use super::build_strip::{BuildStripState, ToolContext};
use super::build_tool_authority::ActiveBuildTool;
use super::ghost_visual::{footprint_invalid_color, footprint_risky_color, footprint_valid_color};

pub fn build_footprint_validity_overlay_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    ghost: Res<BuildGhostState>,
    preview: Res<BuildPlacementPreview>,
    settings: Res<ConstructionTileVisualSettings>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    map_vp: Res<SimulationMapViewport>,
    footprint_gpu: Option<Res<super::footprint_tile_instances::FootprintTileWitness>>,
    params: Res<crate::terrain::generation::world_generator_enhanced::WorldGenParams>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) {
        return Ok(());
    }
    if strip.active == ToolContext::None || ghost.origin.is_none() || !map_vp.is_adequate_for_camera() {
        return Ok(());
    }

    let confidence = confidence_from_validation(&preview.report);
    let color = match confidence {
        BuildConfidence::Perfect | BuildConfidence::Good => footprint_valid_color(),
        BuildConfidence::Risky => footprint_risky_color(),
        BuildConfidence::Invalid => footprint_invalid_color(),
    };
    let origin = ghost.origin.unwrap();
    let footprint = ghost.footprint;
    let world = Vec3::new(origin.x as f32 + 0.5, 0.0, origin.z as f32 + 0.5);
    let gpu_active = footprint_gpu.as_deref().is_some_and(|w| w.gpu_path_active);

    if settings.show_tile_info_labels && !gpu_active {
        if let Some(anchor) = world_to_sim_map_egui(
            world,
            authority.as_deref(),
            desired.as_ref(),
            map_vp.as_ref(),
            params.as_ref(),
        ) {
            let tile_px = tile_screen_extent(
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            );
            let label = if let Some(intent) = tool.building_intent.as_ref() {
                format!(
                    "{} · {}×{} · {} · {}",
                    intent.label,
                    footprint.width,
                    footprint.depth,
                    confidence.label(),
                    if preview.report.errors.is_empty() {
                        "ok"
                    } else {
                        "blocked"
                    }
                )
            } else {
                format!(
                    "Place · {}×{} rot {} · {}",
                    footprint.width,
                    footprint.depth,
                    ghost.rotation_quarter_turns,
                    confidence.label()
                )
            };
            egui::Area::new(egui::Id::new("build_footprint_validity_hint"))
                .order(egui::Order::Tooltip)
                .fixed_pos(anchor + egui::vec2(tile_px * 0.45, -tile_px * 0.55))
                .show(contexts.ctx_mut()?, |ui| {
                    egui::Frame::new()
                        .fill(egui::Color32::from_rgba_unmultiplied(14, 16, 22, 230))
                        .stroke(egui::Stroke::new(1.0, color))
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(label).color(egui::Color32::WHITE));
                            if !preview.report.errors.is_empty() {
                                ui.label(
                                    egui::RichText::new(preview.report.errors.join(", "))
                                        .small()
                                        .color(footprint_invalid_color()),
                                );
                            }
                        });
                });
        }
    }
    Ok(())
}
