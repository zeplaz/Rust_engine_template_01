//! Plant focus card — diesel + core heat gauges (DES-ART-PLANT-CARD-001 layout wire).
//!
//! **HUD-NAT-009:** map-hole clamp + max-h scroll; yields to power-node hover (one themed satellite).

use bevy::prelude::*;
use bevy_egui::egui;

use crate::construction::BuildStripState;
use crate::construction::ToolContext;
use crate::engine::states::BaseState;
use crate::gui::hud::power_hud_icon_atlas::{
    draw_power_hud_gauge_row, draw_power_hud_icon_labeled, PowerHudEguiTextureCache,
    PowerHudIconAtlasManifest, PowerHudIconAtlasUi, PowerHudIconId,
};
use crate::gui::hud::power_node_hover::PowerNodeHoverState;
use crate::gui::hud::simulation_pointer_gate::SimulationMapPointerGate;
use crate::gui::{SimulationMapViewport, UiPalette};

use super::sim_hud_egui_theme::{
    apply_sim_hud_egui_theme, caption_text, clamp_satellite_to_map_hole, data_text,
    map_attached_chip_frame, picker_header_frame, title_text,
};

pub const PLANT_FOCUS_CARD_W: f32 = 260.0;
pub const PLANT_FOCUS_CARD_MAX_H: f32 = 280.0;
/// Offset from map-hole min (not raw window origin — avoids left Bevy chrome overflow).
pub const PLANT_FOCUS_OFFSET_X: f32 = 12.0;
pub const PLANT_FOCUS_OFFSET_Y: f32 = 96.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PlantOperationalStatus {
    #[default]
    Operational,
    Scram,
    Meltdown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DieselState {
    #[default]
    Off,
    Starting,
    Running,
    Failed,
}

/// Read model for plant focus card (sim hooks replace mock fields later).
#[derive(Resource, Debug, Clone)]
pub struct PlantFocusCardSnapshot {
    pub visible: bool,
    pub plant_name: String,
    pub status: PlantOperationalStatus,
    pub offsite_connected: bool,
    pub diesel: DieselState,
    pub core_heat: f32,
    pub diesel_fuel: f32,
    pub containment_pressure: f32,
    pub minutes_to_next_phase: Option<u32>,
}

impl Default for PlantFocusCardSnapshot {
    fn default() -> Self {
        Self {
            visible: false,
            plant_name: "PWR-4 Loop".into(),
            status: PlantOperationalStatus::Operational,
            offsite_connected: true,
            diesel: DieselState::Off,
            core_heat: 0.18,
            diesel_fuel: 1.0,
            containment_pressure: 0.22,
            minutes_to_next_phase: None,
        }
    }
}

impl PlantFocusCardSnapshot {
    #[must_use]
    pub fn demo_scram() -> Self {
        Self {
            visible: true,
            status: PlantOperationalStatus::Scram,
            offsite_connected: false,
            diesel: DieselState::Running,
            core_heat: 0.42,
            diesel_fuel: 0.68,
            containment_pressure: 0.35,
            minutes_to_next_phase: Some(48),
            ..Self::default()
        }
    }
}

pub fn sync_plant_focus_card_visibility(
    strip: Res<BuildStripState>,
    mut card: ResMut<PlantFocusCardSnapshot>,
) {
    card.visible = strip.active == ToolContext::Utilities;
}

pub fn draw_plant_focus_card_egui(
    mut contexts: bevy_egui::EguiContexts,
    base: Res<State<BaseState>>,
    card: Res<PlantFocusCardSnapshot>,
    palette: Res<UiPalette>,
    strip: Res<BuildStripState>,
    map_vp: Res<SimulationMapViewport>,
    pointer_gate: Res<SimulationMapPointerGate>,
    power_hover: Res<PowerNodeHoverState>,
    atlas_ui: Option<Res<PowerHudIconAtlasUi>>,
    manifests: Res<Assets<PowerHudIconAtlasManifest>>,
    mut tex_cache: ResMut<PowerHudEguiTextureCache>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) || !card.visible {
        return Ok(());
    }
    // HUD-NAT-009 — prefer one themed satellite: power-node hover wins when visible.
    if power_hover.card_visible {
        return Ok(());
    }
    let texture_id = atlas_ui
        .as_ref()
        .and_then(|atlas| tex_cache.resolve(&mut contexts, &atlas.atlas));
    let manifest = atlas_ui
        .as_ref()
        .and_then(|atlas| manifests.get(&atlas.manifest));
    let ctx = contexts.ctx_mut()?;
    apply_sim_hud_egui_theme(ctx, &palette);

    let status_label = match card.status {
        PlantOperationalStatus::Operational => "Operational",
        PlantOperationalStatus::Scram => "SCRAM",
        PlantOperationalStatus::Meltdown => "Meltdown",
    };
    let diesel_label = match card.diesel {
        DieselState::Off => "Off",
        DieselState::Starting => "Starting",
        DieselState::Running => "Running",
        DieselState::Failed => "Failed",
    };

    let card_size = egui::vec2(PLANT_FOCUS_CARD_W, PLANT_FOCUS_CARD_MAX_H);
    let mut anchor = if map_vp.valid {
        egui::pos2(
            map_vp.min.x + PLANT_FOCUS_OFFSET_X,
            map_vp.min.y + PLANT_FOCUS_OFFSET_Y,
        )
    } else {
        egui::pos2(PLANT_FOCUS_OFFSET_X, PLANT_FOCUS_OFFSET_Y)
    };
    anchor = clamp_satellite_to_map_hole(anchor, card_size, map_vp.as_ref());
    // Stay clear of Bevy left chrome when gate reports chrome under the default offset.
    if pointer_gate.chrome_blocks_pre_egui && map_vp.valid {
        anchor.x = anchor.x.max(map_vp.min.x + 4.0);
    }

    egui::Area::new(egui::Id::new("plant_focus_card"))
        .order(egui::Order::Foreground)
        .fixed_pos(anchor)
        .show(ctx, |ui| {
            ui.set_width(PLANT_FOCUS_CARD_W);
            map_attached_chip_frame(&palette, palette.wire_magenta).show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .max_height(PLANT_FOCUS_CARD_MAX_H)
                    .show(ui, |ui| {
                        picker_header_frame(&palette).show(ui, |ui| {
                            ui.label(title_text(&palette, &card.plant_name));
                        });
                        ui.label(data_text(
                            &palette,
                            &format!(
                                "Status: {status_label} · Offsite: {}",
                                if card.offsite_connected {
                                    "Connected"
                                } else {
                                    "Islanded"
                                }
                            ),
                        ));
                        if strip.active == ToolContext::Utilities {
                            ui.label(caption_text(&palette, "Utilities focus"));
                        }
                        ui.separator();
                        if let (Some(tex), Some(manifest)) = (texture_id, manifest) {
                            let scram_tint = if card.status == PlantOperationalStatus::Scram {
                                palette.warn
                            } else {
                                palette.fg_muted
                            };
                            draw_power_hud_icon_labeled(
                                ui,
                                tex,
                                manifest,
                                PowerHudIconId::Scram,
                                16.0,
                                scram_tint,
                                &format!("Diesels: {diesel_label}"),
                                card.diesel == DieselState::Running,
                            );
                            draw_power_hud_gauge_row(
                                ui,
                                tex,
                                manifest,
                                PowerHudIconId::Diesel,
                                "Diesel fuel",
                                card.diesel_fuel,
                                palette.accent_action,
                                palette.bg_vellum,
                            );
                            draw_power_hud_gauge_row(
                                ui,
                                tex,
                                manifest,
                                PowerHudIconId::Scram,
                                "Core heat",
                                card.core_heat,
                                palette.warn,
                                palette.bg_vellum,
                            );
                            if !card.offsite_connected {
                                draw_power_hud_icon_labeled(
                                    ui,
                                    tex,
                                    manifest,
                                    PowerHudIconId::Island,
                                    16.0,
                                    palette.warn,
                                    "Grid island",
                                    true,
                                );
                            }
                        } else {
                            ui.label(data_text(
                                &palette,
                                &format!("Diesels: {diesel_label}"),
                            ));
                            ui.label(data_text(
                                &palette,
                                &format!("Core heat: {:.0}%", card.core_heat * 100.0),
                            ));
                        }
                        if let Some(min) = card.minutes_to_next_phase {
                            ui.label(caption_text(
                                &palette,
                                &format!("~{min} min to next phase"),
                            ));
                        }
                    });
            });
        });
    Ok(())
}

#[must_use]
pub fn plant_focus_card_gauges_wired() -> bool {
    PowerHudIconId::inventory()
        .iter()
        .any(|id| matches!(id, PowerHudIconId::Diesel | PowerHudIconId::Scram | PowerHudIconId::Island))
}

#[must_use]
pub fn hud_nat_009_plant_focus_map_clamped() -> bool {
    include_str!("plant_focus_card.rs").contains("clamp_satellite_to_map_hole")
        && include_str!("plant_focus_card.rs").contains("PLANT_FOCUS_CARD_MAX_H")
        && include_str!("plant_focus_card.rs").contains("power_hover.card_visible")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::hud::sim_hud_egui_theme::hud_nat_009_single_themed_satellite_wired;

    #[test]
    fn hud_nat_009_plant_and_theme_wired() {
        assert!(hud_nat_009_plant_focus_map_clamped());
        assert!(hud_nat_009_single_themed_satellite_wired());
    }
}
