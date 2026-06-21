//! egui draw — map-attached power node hover card (COD-POWER-NODE-HOVER-001).

use bevy::prelude::*;
use bevy_egui::egui;
use bevy_egui::EguiContexts;

use crate::construction::{
    world_to_sim_map_egui, ActiveBuildTool, BuildStripState, BuildTool, ToolContext,
};
use crate::engine::states::BaseState;
use crate::gui::hud::simulation_pointer_gate::SimulationMapPointerGate;
use crate::gui::{MapCameraDesired, SimulationMapViewport, UiPalette};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::strategic::SiteFootprint;

use super::power_grid_copy::{POWER_HOVER_CAPACITY, POWER_HOVER_FEEDS, POWER_HOVER_LINKS, POWER_HOVER_LOAD};
use super::power_node_hover::{
    footprint_center_world, PowerNodeHoverState, HOVER_CARD_MAX_W, HOVER_CARD_MIN_W, HOVER_OFFSET_X,
    HOVER_OFFSET_Y,
};
use super::sim_hud_egui_theme::{
    apply_sim_hud_egui_theme, body_text, caption_text, data_text, map_attached_chip_frame,
    title_text,
};

#[must_use]
pub fn power_node_hover_card_wired() -> bool {
    true
}

fn load_bar_color(palette: &UiPalette, pct: f32) -> egui::Color32 {
    if pct > 90.0 {
        palette.warn
    } else if pct >= 70.0 {
        palette.accent_gold
    } else {
        palette.fg_data
    }
}

fn clamp_card_pos(mut pos: egui::Pos2, size: egui::Vec2, map_vp: &SimulationMapViewport) -> egui::Pos2 {
    if !map_vp.valid {
        return pos;
    }
    let min_x = map_vp.min.x + 4.0;
    let min_y = map_vp.min.y + 4.0;
    let max_x = map_vp.max.x - size.x - 4.0;
    let max_y = map_vp.max.y - size.y - 4.0;
    pos.x = pos.x.clamp(min_x, max_x);
    pos.y = pos.y.clamp(min_y, max_y);
    if pos.y + size.y > map_vp.max.y - 4.0 {
        pos.y = map_vp.max.y - size.y - 12.0;
        pos.y = pos.y.clamp(min_y, max_y);
    }
    pos
}

pub fn draw_power_node_hover_egui(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    hover: Res<PowerNodeHoverState>,
    palette: Res<UiPalette>,
    pointer_gate: Res<SimulationMapPointerGate>,
    map_vp: Res<SimulationMapViewport>,
    strip: Res<BuildStripState>,
    tool: Res<ActiveBuildTool>,
    authority: Option<Res<ViewProjectionAuthority>>,
    desired: Res<MapCameraDesired>,
    params: Res<crate::terrain::generation::world_generator_enhanced::WorldGenParams>,
    sites: Query<&SiteFootprint>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) || !hover.card_visible {
        return Ok(());
    }
    let Some(card) = hover.card.as_ref() else {
        return Ok(());
    };

    let ctx = contexts.ctx_mut()?;
    apply_sim_hud_egui_theme(ctx, &palette);

    let card_size = egui::vec2(HOVER_CARD_MIN_W, 120.0);
    let mut anchor = egui::pos2(
        pointer_gate.cursor.x + HOVER_OFFSET_X,
        pointer_gate.cursor.y + HOVER_OFFSET_Y,
    );
    anchor = clamp_card_pos(anchor, card_size, map_vp.as_ref());

    egui::Area::new(egui::Id::new("power_node_hover_card"))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor)
        .interactable(false)
        .show(ctx, |ui| {
            map_attached_chip_frame(&palette, palette.wire_magenta).show(ui, |ui| {
                ui.set_min_width(HOVER_CARD_MIN_W);
                ui.set_max_width(HOVER_CARD_MAX_W);
                ui.horizontal(|ui| {
                    ui.label(title_text(&palette, &card.title));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(body_text(&palette, &card.voltage_label));
                    });
                });
                ui.label(body_text(&palette, card.status.label()));
                ui.horizontal(|ui| {
                    ui.label(caption_text(&palette, POWER_HOVER_LOAD));
                    let bar_w = ui.available_width() - 48.0;
                    let (rect, _) = ui.allocate_exact_size(
                        egui::vec2(bar_w.max(80.0), 10.0),
                        egui::Sense::hover(),
                    );
                    let fill_w = rect.width() * (card.load_pct / 100.0).clamp(0.0, 1.0);
                    let fill = egui::Rect::from_min_size(rect.min, egui::vec2(fill_w, rect.height()));
                    ui.painter().rect_filled(rect, 2.0, palette.bg_vellum);
                    ui.painter()
                        .rect_filled(fill, 2.0, load_bar_color(&palette, card.load_pct));
                    ui.label(data_text(&palette, &format!("{:.0}%", card.load_pct)));
                });
                ui.horizontal(|ui| {
                    ui.label(caption_text(&palette, POWER_HOVER_CAPACITY));
                    ui.label(data_text(&palette, &card.capacity_line));
                });
                ui.horizontal(|ui| {
                    ui.label(caption_text(&palette, POWER_HOVER_FEEDS));
                    ui.label(data_text(&palette, &card.feeds_line));
                });
                if let Some(links) = card.links_line.as_ref() {
                    ui.horizontal(|ui| {
                        ui.label(caption_text(&palette, POWER_HOVER_LINKS));
                        ui.label(data_text(&palette, links));
                    });
                }
                if let Some(yard) = card.yard_line.as_ref() {
                    ui.label(caption_text(&palette, yard));
                }
            });
        });

    let power_line_tool = matches!(tool.tool, BuildTool::PowerLine(_));
    if power_line_tool && strip.active == ToolContext::Utilities {
        if let Ok(footprint) = sites.get(card.entity) {
            let world = footprint_center_world(footprint);
            if let Some(center) = world_to_sim_map_egui(
                world,
                authority.as_deref(),
                desired.as_ref(),
                map_vp.as_ref(),
                params.as_ref(),
            ) {
                let layer = egui::LayerId::new(
                    egui::Order::Foreground,
                    egui::Id::new("power_node_hover_snap_ring"),
                );
                let painter = ctx.layer_painter(layer);
                let radius = 14.0;
                painter.circle_stroke(
                    center,
                    radius,
                    egui::Stroke::new(2.0, palette.accent_gold),
                );
            }
        }
    }

    Ok(())
}
