//! Tabbed **Info** panel body (layers / economy / logistics / alerts / diagnostics).

use bevy::prelude::*;


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

pub fn draw_info_tab_bar() {}

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

fn draw_infrastructure_overlay_legend() {}

pub fn draw_info_tab_body() {}

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
