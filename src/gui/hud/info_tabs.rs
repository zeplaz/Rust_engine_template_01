//! Tabbed **Info** panel body (layers / economy / logistics / alerts / diagnostics).

use bevy::prelude::*;

use bevy_egui::egui;

use crate::construction::PendingConstructionQueue;
use crate::gui::construction_growth_inspector::EcologyGrowthHint;
use crate::gui::logistics_focus::HudLogisticsFocus;
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::gui::world_representation::WorldRepresentationFrame;
use crate::render::{AppStage5ReadinessReport, FireAtmosphereAggregate, GpuRepresentationMetrics,
    infrastructure_overlay_legend_rows, InfrastructureOverlaySettings};
use crate::strategic::{
    ActiveMissions, CityPlanningHints, FractureProbabilityOverlay, LogisticsAiRuntime,
    OperationalTheaterSummary, StrategicOverlayDisplayPolicy, WorldFields, WorldReadSnapshot,
};
use crate::systems::sim_control::{SimControlState, SimTick};

use super::hud_chrome::flat_v2_tray_tab;
use super::overlay_framework::OverlayFrameworkState;
use super::overlay_shell::{mock_overlay_channel_descriptors, OverlayShellState, OverlayToggleGroup};
use crate::gui::style::{error_text, UiPalette};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum HudInfoTab {
    #[default]
    Layers,
    Economy,
    Logistics,
    Alerts,
    Diagnostics,
}

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct HudInfoTabState {
    pub active: HudInfoTab,
    pub request_layout_reset: bool,
}

/// Snapshot for Info panel tabs — refreshed each frame from ops-strip sources.
#[derive(Resource, Clone, Debug, Default)]
pub struct HudInfoLiveData {
    pub sim_tick: u64,
    pub sim_paused: bool,
    pub sim_speed: f32,
    pub economic_pressure: f32,
    pub instability_index: f32,
    pub resource_scarcity: f32,
    pub public_sentiment: f32,
    pub logistics_congestion: f32,
    pub logistics_edge_damage: f32,
    pub logistics_stockpile: f32,
    pub logistics_industrial: f32,
    pub routes_layer_on: bool,
    pub mean_logistics_strength: f32,
    pub mean_threat_slot0: f32,
    pub contested_chunks: u32,
    pub transport_edges: u32,
    pub planning_site_score: f32,
    pub planning_rebuild_pressure: f32,
    pub fracture_mean: f32,
    pub fracture_max: f32,
    pub mission_count: usize,
    pub mission_hint: String,
    pub theater_faction_slots: usize,
    pub pending_total: usize,
    pub pending_unapproved: usize,
    pub fire_smoke_density: f32,
    pub fire_heat_energy: f32,
    pub fire_particle_rows: u32,
    pub focus_has_site: bool,
    pub ecology_program_chunks: u32,
    pub ecology_unique_presets: u32,
    pub ecology_topology_kinds: u32,
}

pub fn sync_hud_info_live_data(
    tick: Res<SimTick>,
    ctrl: Res<SimControlState>,
    world_fields: Option<Res<WorldFields>>,
    world_read: Option<Res<WorldReadSnapshot>>,
    theater: Option<Res<OperationalTheaterSummary>>,
    logistics: Option<Res<LogisticsAiRuntime>>,
    missions: Option<Res<ActiveMissions>>,
    fracture: Option<Res<FractureProbabilityOverlay>>,
    policy: Option<Res<StrategicOverlayDisplayPolicy>>,
    planning: Option<Res<CityPlanningHints>>,
    fire_atm: Option<Res<FireAtmosphereAggregate>>,
    metrics: Option<Res<GpuRepresentationMetrics>>,
    pending: Option<Res<PendingConstructionQueue>>,
    focus: Option<Res<HudLogisticsFocus>>,
    ecology: Option<Res<EcologyGrowthHint>>,
    mut live: ResMut<HudInfoLiveData>,
) {
    live.sim_tick = tick.0;
    live.sim_paused = ctrl.paused;
    live.sim_speed = ctrl.speed;
    if let Some(w) = world_fields.as_deref() {
        live.economic_pressure = w.economic_pressure;
        live.instability_index = w.instability_index;
        live.resource_scarcity = w.resource_scarcity;
        live.public_sentiment = w.public_sentiment;
    }
    if let Some(s) = world_read.as_deref() {
        live.mean_logistics_strength = s.mean_logistics_strength;
        live.mean_threat_slot0 = s.mean_threat_slot0;
        live.contested_chunks = s.contested_chunk_count;
        live.transport_edges = s.transport_edge_count;
    }
    if let Some(l) = logistics.as_deref() {
        live.logistics_congestion = l.congestion_proxy;
        live.logistics_edge_damage = l.mean_edge_damage;
        live.logistics_stockpile = l.stockpile_fill_ratio;
        live.logistics_industrial = l.industrial_output_proxy;
    }
    live.routes_layer_on = policy
        .as_deref()
        .map(|p| p.apply_routing_congestion)
        .unwrap_or(false);
    if let Some(t) = theater.as_deref() {
        live.theater_faction_slots = t.active_faction_slots;
        live.mean_threat_slot0 = t.mean_threat_by_slot[0];
    }
    if let Some(p) = planning.as_deref() {
        live.planning_site_score = p.last_best_site_score;
        live.planning_rebuild_pressure = p.adaptive_rebuild_pressure;
    }
    if let Some(f) = fracture.as_deref() {
        live.fracture_mean = f.mean_heuristic;
        live.fracture_max = f.max_heuristic;
    }
    live.mission_count = missions.as_deref().map(|m| m.missions.len()).unwrap_or(0);
    live.mission_hint = missions
        .as_deref()
        .and_then(|m| {
            m.missions.first().and_then(|row| {
                row.success_readout_label
                    .as_deref()
                    .or(row.objectives.first().map(|o| o.label.as_str()))
            })
        })
        .unwrap_or("—")
        .to_string();
    if let Some(q) = pending.as_deref() {
        live.pending_total = q.entries.len();
        live.pending_unapproved = q.entries.iter().filter(|e| !e.approved).count();
    }
    if let Some(f) = fire_atm.as_deref() {
        live.fire_smoke_density = f.smoke_density;
        live.fire_heat_energy = f.heat_energy;
    }
    live.fire_particle_rows = metrics.as_deref().map(|m| m.particle_rows).unwrap_or(0);
    live.focus_has_site = focus.as_deref().and_then(|f| f.tracked_entity).is_some();
    if let Some(h) = ecology.as_deref() {
        live.ecology_program_chunks = h.program_chunks;
        live.ecology_unique_presets = h.unique_presets;
        live.ecology_topology_kinds = h.topology_kind_count;
    }
}

pub fn draw_info_tab_bar(ui: &mut egui::Ui, palette: &UiPalette, state: &mut HudInfoTabState) {
    ui.horizontal(|ui| {
        for (tab, label) in [
            (HudInfoTab::Layers, "Layers"),
            (HudInfoTab::Economy, "Economy"),
            (HudInfoTab::Logistics, "Logistics"),
            (HudInfoTab::Alerts, "Alerts"),
            (HudInfoTab::Diagnostics, "Diag"),
        ] {
            if flat_v2_tray_tab(ui, palette, label, state.active == tab).clicked() {
                state.active = tab;
            }
            ui.add_space(4.0);
        }
    });
    ui.separator();
}

fn sync_infra_overlay_from_utility_group(
    shell: &OverlayShellState,
    settings: &mut InfrastructureOverlaySettings,
) {
    let utility_on = shell.groups[3];
    if utility_on && !settings.enabled {
        settings.enabled = true;
        settings.road = true;
        settings.rail = true;
    } else if !utility_on {
        settings.enabled = false;
    }
}

fn draw_infrastructure_overlay_legend(
    ui: &mut egui::Ui,
    settings: &mut InfrastructureOverlaySettings,
) {
    ui.separator();
    ui.label(egui::RichText::new("Infrastructure network").strong());
    ui.checkbox(&mut settings.enabled, "Show network overlay");
    if !settings.enabled {
        return;
    }
    ui.checkbox(&mut settings.road, "Roads");
    ui.checkbox(&mut settings.rail, "Rail");
    ui.checkbox(&mut settings.power, "Power");
    ui.checkbox(&mut settings.water, "Water");
    ui.checkbox(&mut settings.sewer, "Sewer");
    ui.separator();
    ui.label(egui::RichText::new("Stroke legend").small());
    for row in infrastructure_overlay_legend_rows() {
        ui.horizontal(|ui| {
            let [r, g, b] = row.stroke.color_rgb;
            let color = egui::Color32::from_rgb(r, g, b);
            let height = row.stroke.weight_px.max(2.0);
            let (rect, _) =
                ui.allocate_exact_size(egui::vec2(28.0, height + 4.0), egui::Sense::hover());
            let y = rect.center().y;
            ui.painter().line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(row.stroke.weight_px, color),
            );
            ui.label(row.label);
        });
    }
}

pub fn draw_info_tab_body(
    ui: &mut egui::Ui,
    palette: &UiPalette,
    tab: HudInfoTab,
    tabs: &mut HudInfoTabState,
    shell: &mut OverlayShellState,
    framework: &mut OverlayFrameworkState,
    world: Option<&WorldRepresentationFrame>,
    readiness: Option<&AppStage5ReadinessReport>,
    live: Option<&HudInfoLiveData>,
    minimap_legend: Option<&str>,
    infra_settings: &mut InfrastructureOverlaySettings,
) {
    match tab {
        HudInfoTab::Layers => {
            ui.label(egui::RichText::new("Map overlays").strong());
            ui.label(egui::RichText::new("Tactical layers").small().weak());
            ui.checkbox(shell.group_mut(OverlayToggleGroup::Threat), "Threat");
            ui.checkbox(shell.group_mut(OverlayToggleGroup::Logistics), "Logistics routes");
            ui.checkbox(shell.group_mut(OverlayToggleGroup::Recon), "Recon");
            ui.separator();
            ui.label(egui::RichText::new("Utilities & networks").small().weak());
            ui.checkbox(shell.group_mut(OverlayToggleGroup::Utility), "Utilities");
            sync_infra_overlay_from_utility_group(shell, infra_settings);
            if shell.groups[3] {
                draw_infrastructure_overlay_legend(ui, infra_settings);
            }
            ui.separator();
            ui.checkbox(&mut shell.legend_open, "Channel legend");
            if shell.legend_open {
                ui.label(egui::RichText::new("Minimap / map channels").small().weak());
                if let Some(legend) = minimap_legend {
                    ui.label(egui::RichText::new(legend).small());
                }
                for channel in &mut framework.channels {
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut channel.enabled, format!("{:?}", channel.descriptor.overlay));
                        ui.add(egui::Slider::new(&mut channel.opacity, 0.0..=1.0).text("α"));
                        ui.add(egui::Slider::new(&mut channel.blend_weight, 0.0..=1.0).text("blend"));
                    });
                }
            }
        }
        HudInfoTab::Economy => {
            ui.label(egui::RichText::new("Economy & pressure").strong());
            if let Some(d) = live {
                ui.label(format!(
                    "SIM n={}  {}  v={:.1}x",
                    d.sim_tick,
                    if d.sim_paused { "PAUSE" } else { "RUN" },
                    d.sim_speed
                ));
                ui.separator();
                ui.label(format!("Economic pressure {:.2}", d.economic_pressure));
                ui.label(format!("Instability {:.2}", d.instability_index));
                ui.label(format!("Resource scarcity {:.2}", d.resource_scarcity));
                ui.label(format!("Public sentiment {:.2}", d.public_sentiment));
                ui.label(format!(
                    "Logistics strength μ {:.2}  contested chunks {}",
                    d.mean_logistics_strength, d.contested_chunks
                ));
                ui.label(format!(
                    "Planning score {:.2}  rebuild pressure {:.2}",
                    d.planning_site_score, d.planning_rebuild_pressure
                ));
            } else {
                ui.label(egui::RichText::new("Live economy feed not ready.").small().weak());
            }
            if let Some(w) = world {
                ui.label(format!("LOD band: {:?}", w.global_band()));
            }
        }
        HudInfoTab::Logistics => {
            ui.label(egui::RichText::new("Logistics throughput").strong());
            if let Some(d) = live {
                ui.label(format!(
                    "Routes layer {}  congestion {:.2}  edge damage {:.2}",
                    if d.routes_layer_on { "on" } else { "off" },
                    d.logistics_congestion,
                    d.logistics_edge_damage
                ));
                ui.label(format!(
                    "Stockpile {:.2}  industrial {:.2}  transport edges {}",
                    d.logistics_stockpile, d.logistics_industrial, d.transport_edges
                ));
            }
            ui.separator();
            for desc in mock_overlay_channel_descriptors() {
                ui.label(format!("{:?} · {:?}", desc.overlay, desc.utility));
            }
        }
        HudInfoTab::Alerts => {
            ui.label(egui::RichText::new("Active alerts").strong());
            if let Some(d) = live {
                ui.label(format!(
                    "Missions {}  |  T0 threat {:.2}  factions {}",
                    d.mission_count, d.mean_threat_slot0, d.theater_faction_slots
                ));
                ui.label(format!("Primary objective: {}", d.mission_hint));
                ui.label(format!(
                    "Fracture μ {:.2}  max {:.2}",
                    d.fracture_mean, d.fracture_max
                ));
                ui.label(format!(
                    "Pending construction {}/{} approved",
                    d.pending_total.saturating_sub(d.pending_unapproved),
                    d.pending_total
                ));
                if d.fire_smoke_density > 0.05 || d.fire_heat_energy > 0.05 {
                    ui.label(format!(
                        "Fire atmosphere smoke {:.2}  heat {:.2}  GPU particles {}",
                        d.fire_smoke_density, d.fire_heat_energy, d.fire_particle_rows
                    ));
                }
                if d.focus_has_site {
                    ui.label(egui::RichText::new("Logistics focus active — see CAUSE strip.").small());
                }
            } else {
                ui.label(egui::RichText::new("Live alert feed not ready.").small().weak());
            }
        }
        HudInfoTab::Diagnostics => {
            ui.label(egui::RichText::new("Runtime diagnostics").strong());
            if let Some(d) = live {
                if d.ecology_program_chunks > 0 {
                    ui.label(format!(
                        "Ecology programs: {} chunks · {} presets · {} topology kinds",
                        d.ecology_program_chunks, d.ecology_unique_presets, d.ecology_topology_kinds
                    ));
                    ui.separator();
                }
            }
            if let Some(r) = readiness {
                ui.label(format!(
                    "Stage5 passes={} vt4={} vt5={} phase_f={}",
                    r.vt4_ok && r.vt5_ok && r.phase_f_ok,
                    r.vt4_ok,
                    r.vt5_ok,
                    r.phase_f_ok
                ));
                if !r.violations.is_empty() {
                    for v in r.violations.iter().take(6) {
                        error_text(ui, palette, v);
                    }
                }
            } else {
                ui.label(egui::RichText::new("Readiness report not wired this frame.").small().weak());
            }
            if ui.button("Reset HUD layout (defaults)").clicked() {
                tabs.request_layout_reset = true;
            }
        }
    }
}

pub struct HudInfoTabPlugin;

impl Plugin for HudInfoTabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudInfoTabState>()
            .init_resource::<HudInfoLiveData>()
            .add_systems(
                PreUpdate,
                sync_hud_info_live_data.run_if(in_simulation_or_editor),
            );
    }
}
