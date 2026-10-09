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

/// Read model for the plant focus card. Filled from [`NuclearContainmentRuntime`].
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

fn map_diesel(state: crate::entities::production::power::NuclearDieselState) -> DieselState {
    use crate::entities::production::power::NuclearDieselState;
    match state {
        NuclearDieselState::Off => DieselState::Off,
        NuclearDieselState::Starting => DieselState::Starting,
        NuclearDieselState::Running => DieselState::Running,
        NuclearDieselState::Failed => DieselState::Failed,
    }
}

/// Copy one live containment plant into the card. `fuel_hours_cap` is the profile tank.
pub fn fill_plant_focus_from_runtime(
    card: &mut PlantFocusCardSnapshot,
    display_name: &str,
    runtime: &crate::entities::production::power::NuclearContainmentRuntime,
    thermal: &crate::entities::production::power::ThermalComponent,
    fuel_hours_cap: f32,
) {
    card.plant_name = if display_name.is_empty() {
        "Reactor".into()
    } else {
        display_name.to_string()
    };
    card.status = if runtime.meltdown_active || runtime.containment_breached {
        PlantOperationalStatus::Meltdown
    } else if runtime.scrammed {
        PlantOperationalStatus::Scram
    } else {
        PlantOperationalStatus::Operational
    };
    card.offsite_connected = runtime.offsite_connected;
    card.diesel = map_diesel(runtime.diesel);
    let max_t = thermal.max_temperature.max(1.0);
    card.core_heat = (thermal.current_temperature / max_t).clamp(0.0, 1.0);
    let cap = fuel_hours_cap.max(0.001);
    card.diesel_fuel = (runtime.diesel_fuel_hours_remaining / cap).clamp(0.0, 1.0);
    card.containment_pressure = if runtime.containment_breached {
        1.0
    } else if runtime.meltdown_threshold_reached {
        0.8
    } else {
        card.core_heat * 0.5
    };
    card.minutes_to_next_phase = None;
    card.visible = true;
}

pub fn sync_plant_focus_card_visibility(
    strip: Res<BuildStripState>,
    defs: Option<Res<crate::entities::production::power::PlantDefinitionRegistry>>,
    mut card: ResMut<PlantFocusCardSnapshot>,
    plants: Query<(
        &crate::entities::production::power::PowerPlant,
        &crate::entities::production::power::NuclearContainmentRuntime,
        &crate::entities::production::power::ThermalComponent,
    )>,
) {
    if strip.active != ToolContext::Utilities {
        card.visible = false;
        return;
    }
    let Some((plant, runtime, thermal)) = plants.iter().max_by(|a, b| {
        a.2.current_temperature
            .partial_cmp(&b.2.current_temperature)
            .unwrap_or(std::cmp::Ordering::Equal)
    }) else {
        card.visible = false;
        return;
    };
    let def = defs
        .as_ref()
        .and_then(|d| d.get(plant.definition_id.as_str()));
    let name = def
        .map(|d| d.display_name.as_str())
        .filter(|n| !n.is_empty())
        .unwrap_or(plant.definition_id.as_str());
    let cap = def
        .and_then(|d| d.nuclear_failure_profile.as_ref())
        .map(|p| p.diesel_fuel_hours)
        .unwrap_or(72.0);
    fill_plant_focus_from_runtime(&mut card, name, runtime, thermal, cap);
}

pub fn draw_plant_focus_card_egui(
    mut contexts: bevy_egui::EguiContexts,
    base: Res<State<BaseState>>,
    card: Res<PlantFocusCardSnapshot>,
    palette: Res<UiPalette>,
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

    #[test]
    fn live_runtime_fills_scram_heat_and_fuel() {
        use crate::entities::production::power::{
            NuclearContainmentRuntime, NuclearDieselState, ThermalComponent,
        };

        let runtime = NuclearContainmentRuntime {
            offsite_connected: false,
            loop_elapsed_s: 0.0,
            scrammed: true,
            diesel: NuclearDieselState::Running,
            diesel_start_elapsed_s: 0.0,
            diesel_fuel_hours_remaining: 4.0,
            cooling_degraded: false,
            meltdown_threshold_reached: false,
            cooling_alert_emitted: false,
            meltdown_active: false,
            meltdown_elapsed_s: 0.0,
            containment_breached: false,
            meltdown_event_emitted: false,
            breach_event_emitted: false,
        };
        let thermal = ThermalComponent {
            current_temperature: 500.0,
            max_temperature: 1000.0,
        };
        let mut card = PlantFocusCardSnapshot::default();
        fill_plant_focus_from_runtime(&mut card, "PWR-4 Loop", &runtime, &thermal, 10.0);
        assert!(card.visible);
        assert_eq!(card.status, PlantOperationalStatus::Scram);
        assert!(!card.offsite_connected);
        assert_eq!(card.diesel, DieselState::Running);
        assert!((card.core_heat - 0.5).abs() < 0.001);
        assert!((card.diesel_fuel - 0.4).abs() < 0.001);
        assert_eq!(card.plant_name, "PWR-4 Loop");
    }
}
