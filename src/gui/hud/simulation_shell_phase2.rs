//! Phase 2A simulation shell — ops strip zones, context tray, build rail, minimap chrome (presentation only).
//!
//! Spec: `prompts/guides/ui/ui_phase0_panel_mocks_v1.md` § P1/P2/P3/P4.

use bevy::prelude::*;
use crate::gui::DeprecatedInteraction;
use bevy::ui_widgets::Button;
use bevy::window::PrimaryWindow;

use crate::construction::{BuildStripState, ToolContext};
use crate::engine::states::BaseState;
use crate::gui::ui_gates::in_simulation_or_editor;
use crate::gui::view_authority::commit_world_main_map_focus;
use crate::gui::{
    MinimapShellState, UiPalette,
    WorldRepresentationFrame,
};
use crate::render::view_runtime::{ViewProjectionAuthority, ViewRuntimeTrace};
use crate::strategic::{
    ActiveMissions, NarrativeObservationBus, StrategicOverlayDisplayPolicy, WorldFields,
};
use crate::systems::sim_control::{SimControlState, SimTick};
use crate::entities::production::power::failure_modes::RenewableWeatherFactors;
use crate::systems::weather::{
    format_ops_power_derate_line, format_tactical_wx_line, WeatherPlayerReadFrame,
    WeatherPrecipVisualSample,
};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::live_data::HudInfoLiveData;
use super::panel_state::HudPanelState;
use super::icon_atlas::{IconAtlasManifest, IconAtlasUi, IconId};
use crate::render::{
    MinimapCompositorState, MinimapRenderTargetRegistry,
};

const INTEL_FOCUS_CHUNK_TILE_PX: f32 = 32.0;

/// P4 — left context rail width (mock § P4).
pub const CONTEXT_RAIL_W_PX: f32 = 48.0;
/// P4 — build tool rail width (Phase 2B; mock § P4 dual column — **2C-B**).
pub const BUILD_RAIL_W_PX: f32 = 52.0;
/// Gap between columns on `CommandLeftStackOverlay`.
pub const COMMAND_LEFT_STACK_COLUMN_GAP_PX: f32 = 6.0;
/// Build rail container + slot geometry (must match `in_game_hud` spawn).
pub const BUILD_RAIL_CONTAINER_PAD_PX: f32 = 4.0;
/// Hit target ≥48 px (HUD-A-001 presentation contract).
pub const BUILD_RAIL_SLOT_MIN_H_PX: f32 = 42.0;
pub const BUILD_RAIL_SLOT_PAD_Y_PX: f32 = 3.0;
pub const BUILD_RAIL_ROW_GAP_PX: f32 = 4.0;
/// Gap between build rail and egui picker sheets.
pub const BUILD_PICKER_SHEET_GAP_PX: f32 = 8.0;
/// Expanded narrative stack body width (`LeftContextStackBody`).
pub const LEFT_CONTEXT_STACK_BODY_W_PX: f32 = 400.0;

#[must_use]
pub fn build_rail_slot_step_px() -> f32 {
    BUILD_RAIL_SLOT_MIN_H_PX + BUILD_RAIL_SLOT_PAD_Y_PX * 2.0 + BUILD_RAIL_ROW_GAP_PX
}

/// Screen anchor for an egui sheet beside the Bevy build rail (logical px).
#[must_use]
pub fn build_rail_slot_anchor_xy(slot: ToolContext, left_stack_collapsed: bool) -> bevy::prelude::Vec2 {
    use crate::gui::in_game_hud::{CENTER_ROW_EDGE_PAD_PX, SIMULATION_MAP_VIEWPORT_TOP_CHROME_PX};

    let idx = match slot {
        ToolContext::Roads => 0,
        ToolContext::Rail => 1,
        ToolContext::Utilities => 2,
        ToolContext::Military => 3,
        ToolContext::Industry => 4,
        ToolContext::Ecology => 5,
        ToolContext::Civil => 6,
        ToolContext::None => 0,
    };
    let x = CENTER_ROW_EDGE_PAD_PX
        + if left_stack_collapsed {
            CONTEXT_RAIL_W_PX + COMMAND_LEFT_STACK_COLUMN_GAP_PX + BUILD_RAIL_W_PX
        } else {
            BUILD_RAIL_W_PX
        }
        + BUILD_PICKER_SHEET_GAP_PX;
    let y = SIMULATION_MAP_VIEWPORT_TOP_CHROME_PX
        + BUILD_RAIL_CONTAINER_PAD_PX
        + idx as f32 * build_rail_slot_step_px();
    bevy::prelude::Vec2::new(x, y)
}

/// Legacy Y-only helper — prefer [`build_rail_slot_anchor_xy`].
#[must_use]
pub fn build_rail_slot_anchor_y(slot: ToolContext) -> f32 {
    build_rail_slot_anchor_xy(slot, true).y
}

/// Signed Phase 2C layout option (mock § P4).
pub const PHASE_2C_LAYOUT_OPTION: &str = "2C-B";

/// Horizontal overlay footprint on map (excludes `CENTER_ROW_EDGE_PAD_PX` window inset).
#[must_use]
pub fn command_left_stack_footprint_px(collapsed: bool) -> f32 {
    if collapsed {
        CONTEXT_RAIL_W_PX + COMMAND_LEFT_STACK_COLUMN_GAP_PX + BUILD_RAIL_W_PX
    } else {
        BUILD_RAIL_W_PX + COMMAND_LEFT_STACK_COLUMN_GAP_PX + LEFT_CONTEXT_STACK_BODY_W_PX
    }
}

/// Screen rect for build picker sheet (pointer gate hit test).
#[must_use]
pub fn sim_build_rail_submenu_block_rect(
    picker: &crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState,
    left_stack_collapsed: bool,
) -> bevy::prelude::Rect {
    crate::gui::hud::sim_build_picker_sheet::sim_build_picker_sheet_rect(
        picker,
        left_stack_collapsed,
    )
}

/// P3 — map viewport inset frame.
pub const MAP_FRAME_INSET_PX: f32 = 4.0;

pub const OPS_STRIP_TOP_OFFSET_PX: f32 = 2.0;
pub const CONTEXT_TRAY_TAB_H_PX: f32 = 32.0;
pub const CONTEXT_TRAY_BODY_H_PX: f32 = 96.0;
/// Peek preview body — half height for first-click glance (F-06).
pub const CONTEXT_TRAY_PEEK_BODY_H_PX: f32 = 48.0;
/// Selected tab gold accent (mock § P2 / F-07).
pub const CONTEXT_TRAY_TAB_GOLD_BAR_PX: f32 = 2.0;
/// Bevy stroke padding around egui minimap texture (F-09 ≤2px target).
pub const MINIMAP_CHROME_STROKE_PAD_PX: f32 = 1.0;

/// Bottom context tray authority (mock § P2).
#[derive(Resource, Clone, Debug)]
pub struct ContextTrayState {
    pub panel_state: HudPanelState,
    pub active_tab: ContextTrayTab,
}

impl Default for ContextTrayState {
    fn default() -> Self {
        Self {
            panel_state: HudPanelState::Collapsed,
            active_tab: ContextTrayTab::Alerts,
        }
    }
}

impl ContextTrayState {
    /// Collapsed → Peek → Expanded → Collapsed (pinned stays pinned).
    pub fn cycle_tray_affordance(&mut self) {
        if self.panel_state.is_pinned() {
            return;
        }
        self.panel_state = match self.panel_state {
            HudPanelState::Collapsed => HudPanelState::Peek,
            HudPanelState::Peek => HudPanelState::Expanded,
            HudPanelState::Expanded => HudPanelState::Collapsed,
            HudPanelState::Pinned => HudPanelState::Pinned,
        };
    }

    /// Tab press: first click peeks, second expands; switching tabs keeps engagement level.
    pub fn on_tab_pressed(&mut self, tab: ContextTrayTab) {
        if self.active_tab == tab {
            self.panel_state = match self.panel_state {
                HudPanelState::Collapsed => HudPanelState::Peek,
                HudPanelState::Peek => HudPanelState::Expanded,
                HudPanelState::Expanded | HudPanelState::Pinned => self.panel_state,
            };
        } else {
            self.active_tab = tab;
            if self.panel_state == HudPanelState::Collapsed {
                self.panel_state = HudPanelState::Peek;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ContextTrayTab {
    #[default]
    Alerts,
    Events,
    Intel,
    Logistics,
    Build,
    Diagnostics,
}

impl ContextTrayTab {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Alerts => "Alerts",
            Self::Events => "Events",
            Self::Intel => "Intel",
            Self::Logistics => "Logistics",
            Self::Build => crate::gui::hud::sim_hud_copy::TRAY_BUILD_TAB,
            Self::Diagnostics => "Diag",
        }
    }
}

/// **EVENT-LOG-UI-001** — Events tab + tray/ops format hooks wired in sim chrome.
#[must_use]
pub fn event_log_ui_chrome_wired() -> bool {
    ContextTrayTab::Events.label() == "Events"
        && crate::sim::effects::PLAYER_EVENT_TRAY_BODY_MAX_ROWS >= 4
}

/// P1 ops strip zones: time | alerts | intel | weather | power | tray affordance.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpsStripZone {
    Time,
    Alerts,
    Intel,
    Weather,
    Power,
    TrayAffordance,
}

#[derive(Component)]
pub struct OpsStripTime;

#[derive(Component)]
pub struct OpsStripAlerts;

#[derive(Component)]
pub struct OpsStripIntel;

#[derive(Component)]
pub struct OpsStripWeather;

#[derive(Component)]
pub struct OpsStripPower;

/// Filter control on the ops strip. One flag, not a window.
#[derive(Component)]
pub struct ZoneShadeToggleButton;

#[derive(Component)]
pub struct ZoneShadeToggleLabel;

#[derive(Resource, Clone, Copy, Debug)]
pub struct ZoneShadeToggle {
    pub visible: bool,
}

impl Default for ZoneShadeToggle {
    fn default() -> Self {
        Self { visible: false }
    }
}

#[derive(Component)]
pub struct OpsStripTrayAffordance;

#[derive(Component)]
pub struct OpsStripAlertBadge;

#[derive(Component)]
pub struct OpsStripAlertBadgeText;

/// Orders-pending label for **S7B-M2-001** (optional entity; DTO still wires when absent).
#[derive(Component)]
pub struct OpsStripOrdersPendingText;

#[derive(Component)]
pub struct ContextTrayBodyLine;

#[derive(Component)]
pub struct MinimapChromeRoot;

/// GPU minimap texture host under [`MinimapChromeRoot`] (UX-E01 M1).
#[derive(Component)]
pub struct MinimapGpuImageNode;

/// Title bar of the GPU minimap window (drag-grab strip; mirrors the egui `Minimap` window header).
/// MINIMAP-WIDGET-IMPL-001 chrome — positioned by [`sync_minimap_chrome_layout_system`].
#[derive(Component)]
pub struct MinimapChromeTitleBar;

/// Title caption text inside [`MinimapChromeTitleBar`].
#[derive(Component)]
pub struct MinimapChromeTitleText;

/// Overlay-toggle affordance hint shown in the title bar (mirrors egui toolbar's frame/overlay row).
#[derive(Component)]
pub struct MinimapChromeOverlayHint;

/// Bottom-right resize grip; visible target for the resize path in `minimap_bevy_pointer_system`.
#[derive(Component)]
pub struct MinimapChromeResizeGrip;

#[derive(Component)]
pub struct ContextTrayRoot;

#[derive(Component)]
pub struct ContextTrayTabButton(pub ContextTrayTab);

#[derive(Component)]
pub struct ContextTrayTabLabel;

#[derive(Component)]
pub struct ContextTrayBodyRoot;

#[derive(Component)]
pub struct MapViewportFrameInset;
#[derive(Component)]
pub struct BuildRailRoot;

#[derive(Component, Clone, Copy)]
pub struct BuildRailToolSlot(pub ToolContext);

/// Petroleum industry panel tab (Phase 4.2 — `IconId::P5Br` + label).
#[derive(Component)]
pub struct PetroleumPanelTabRoot;

#[derive(Component)]
pub struct PetroleumPanelTabIcon;

#[derive(Component)]
pub struct PetroleumPanelTabLabel;

/// P4-VEH-01 — vehicle silhouette chips in context tray Logistics tab.
#[derive(Component)]
pub struct LogisticsVehicleChipRow;

#[derive(Component, Clone, Copy)]
pub struct LogisticsVehicleChip(pub IconId);

#[derive(Component)]
pub struct LogisticsVehicleChipIcon;

#[derive(Component)]
pub struct LogisticsVehicleChipLabel;

#[derive(Component)]
pub struct BuildRailToolLabel;

#[derive(Resource, Default, Debug)]
pub struct OpsStripIntelFocusRequest {
    pub pending_world: Option<Vec2>,
}

fn open_alerts_tray(tray: &mut ContextTrayState) {
    tray.panel_state = HudPanelState::Expanded;
    tray.active_tab = ContextTrayTab::Alerts;
}

fn request_intel_focus(
    tray: &mut ContextTrayState,
    world: &WorldRepresentationFrame,
    intel_req: &mut OpsStripIntelFocusRequest,
) {
    tray.on_tab_pressed(ContextTrayTab::Intel);
    let tile = INTEL_FOCUS_CHUNK_TILE_PX;
    let fc = world.focus_chunk;
    intel_req.pending_world = Some(Vec2::new(
        fc.x as f32 * tile + tile * 0.5,
        fc.y as f32 * tile + tile * 0.5,
    ));
}

pub struct SimulationShellPhase2Plugin;

/// Ordering anchor for ops-strip zone line refresh (S7P grid toast runs after).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpsStripZoneLinesSet;

impl Plugin for SimulationShellPhase2Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(super::icon_atlas::IconAtlasPlugin);
        app.add_plugins(super::power_hud_icon_atlas::PowerHudIconAtlasPlugin);
        super::simulation_pointer_gate::add_pointer_gate_systems(app);
        app.configure_sets(
            Update,
            (
                super::simulation_pointer_gate::PointerGateSet::Sync,
                super::simulation_pointer_gate::PointerGateSet::Block,
                super::simulation_pointer_gate::PointerGateSet::Publish,
            )
                .run_if(in_simulation_or_editor),
        );
        app.init_resource::<ContextTrayState>()
            .init_resource::<ZoneShadeToggle>()
            .init_resource::<super::simulation_pointer_gate::SimulationMapPointerGate>()
            .init_resource::<super::minimap_bevy_interaction::MinimapBevyPointerState>()
            .init_resource::<OpsStripIntelFocusRequest>()
            .init_resource::<crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState>()
            .init_resource::<crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState>()
            .init_resource::<crate::gui::hud::sim_power_tool_sheet::SimPowerToolSheetState>()
            .init_resource::<crate::gui::hud::plant_focus_card::PlantFocusCardSnapshot>()
            .configure_sets(Update, OpsStripZoneLinesSet)
            .add_systems(
                Update,
                (
                    super::minimap_bevy_interaction::minimap_bevy_active_input_system
                        .before(crate::gui::tactical::map_camera::MapCameraSystemSet::ApplyInput),
                    super::minimap_bevy_interaction::minimap_bevy_scroll_zoom_system
                        .after(crate::gui::tactical::map_camera::MapCameraSystemSet::ApplyInput),
                    super::minimap_bevy_interaction::pin_minimap_centered_fit_system,
                    ops_strip_zone_click_system,
                    context_tray_tab_click_system,
                    build_rail_tool_click_system,
                    apply_ops_strip_intel_focus_system
                        .before(crate::gui::tactical::map_camera::MapCameraSystemSet::ApplyInput),
                    update_ops_strip_zone_lines_system.in_set(OpsStripZoneLinesSet),
                    sync_ops_strip_zone_hover_system,
                    sync_ops_strip_alert_badge_system,
                    sync_context_tray_visibility_system,
                    sync_context_tray_tab_chrome_system,
                    sync_build_rail_from_strip_system,
                    sync_petroleum_panel_tab_system,
                    sync_logistics_vehicle_chips_system,
                )
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (zone_shade_toggle_click_system, sync_zone_shade_toggle_label)
                    .run_if(in_simulation_or_editor),
            )
            .init_resource::<crate::gui::hud::sim_build_picker_panel::SimBuildPickerCache>()
            .add_systems(
                Update,
                (
                    crate::gui::hud::sim_build_picker_panel::sync_sim_build_picker_panel_layout,
                    crate::gui::hud::sim_build_picker_panel::rebuild_sim_build_picker_panel_list,
                    crate::gui::hud::sim_build_picker_panel::sync_sim_build_picker_panel_selection,
                    crate::gui::hud::sim_build_picker_panel::sim_build_picker_panel_actions,
                )
                    .chain()
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    crate::gui::hud::sim_road_power_sheets_bevy::sync_sim_road_sheet_bevy,
                    crate::gui::hud::sim_road_power_sheets_bevy::sync_sim_power_sheet_bevy,
                    crate::gui::hud::sim_road_power_sheets_bevy::sim_road_sheet_actions,
                    crate::gui::hud::sim_road_power_sheets_bevy::sim_power_sheet_actions,
                )
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    crate::gui::hud::sim_play_bevy_chrome::sync_sim_play_plant_card,
                    crate::gui::hud::sim_play_bevy_chrome::sync_sim_play_power_hover,
                    crate::gui::hud::sim_play_bevy_chrome::sync_sim_play_crosshair,
                    crate::gui::hud::sim_play_bevy_chrome::sync_context_tray_power_repair_list,
                    crate::gui::hud::sim_play_bevy_chrome::context_tray_power_repair_clicks,
                )
                    .chain()
                    .after(super::simulation_pointer_gate::PointerGateSet::Publish)
                    .run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(
                Update,
                (
                    super::plant_focus_card::sync_plant_focus_card_visibility,
                )
                    .run_if(in_simulation_or_editor),
            )
            ;
    }
}

#[must_use]
pub fn format_sim_tick_line(tick: u64, paused: bool, speed: f32) -> String {
    let run = if paused { "PAUSE" } else { "RUN" };
    format!("T+{:05}  {:<5}  v={:.1}x", tick, run, speed)
}

#[derive(Default, Clone)]
struct OpsStripZoneCache {
    time_fp: Option<(u64, bool, i32)>,
    time_line: String,
    alerts_fp: Option<(usize, u32, u32, u64)>,
    alerts_line: String,
    intel_fp: Option<(bool, i32, i32)>,
    intel_line: String,
    weather_fp: Option<(i32, i32, i32)>,
    weather_line: String,
    power_fp: Option<i32>,
    power_line: String,
    tray_line: String,
}

fn update_ops_strip_zone_lines_system(
    tick: Res<SimTick>,
    ctrl: Res<SimControlState>,
    policy: Res<StrategicOverlayDisplayPolicy>,
    logistics: Option<Res<crate::strategic::LogisticsAiRuntime>>,
    missions: Option<Res<ActiveMissions>>,
    narrative: Option<Res<NarrativeObservationBus>>,
    player_log: Option<Res<crate::sim::effects::PlayerEventLog>>,
    weather: Option<Res<WeatherPrecipVisualSample>>,
    wx_read: Option<Res<WeatherPlayerReadFrame>>,
    renewables: Option<Res<RenewableWeatherFactors>>,
    world_fields: Option<Res<WorldFields>>,
    tray: Res<ContextTrayState>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut qs: ParamSet<(
        Query<&mut Text, With<OpsStripTime>>,
        Query<&mut Text, With<OpsStripAlerts>>,
        Query<&mut Text, With<OpsStripIntel>>,
        Query<&mut Text, With<OpsStripWeather>>,
        Query<&mut Text, With<OpsStripPower>>,
        Query<&mut Text, With<OpsStripTrayAffordance>>,
    )>,
    mut cache: Local<OpsStripZoneCache>,
) {
    // The tick readout only needs ~15 Hz: re-shaping the text every sim tick costs more than
    // everything else on the strip (bevy_ui text layout).
    let time_fp = (tick.0 / 4, ctrl.paused, (ctrl.speed * 100.0).round() as i32);
    if cache.time_fp != Some(time_fp) {
        cache.time_fp = Some(time_fp);
        cache.time_line = format_sim_tick_line(tick.0, ctrl.paused, ctrl.speed);
        for mut t in qs.p0().iter_mut() {
            crate::gui::style::widgets::set_text_if_changed(&mut t, cache.time_line.clone());
        }
    }

    let n_m = missions.as_deref().map(|m| m.missions.len()).unwrap_or(0);
    let narrative_fp = narrative
        .as_deref()
        .and_then(|b| b.recent.back())
        .map(|o| o.generated_text.len() as u32)
        .unwrap_or(0);
    let crit_unread = player_log.as_deref().map(|l| l.unread_crit).unwrap_or(0);
    let last_event_id = player_log
        .as_deref()
        .and_then(|l| l.rows.back().map(|r| r.effect_id))
        .unwrap_or(0);
    let alerts_fp = (n_m, narrative_fp, crit_unread, last_event_id);
    if cache.alerts_fp != Some(alerts_fp) {
        cache.alerts_fp = Some(alerts_fp);
        cache.alerts_line = crate::sim::effects::format_ops_strip_alerts_line(
            n_m,
            player_log.as_deref().unwrap_or(&crate::sim::effects::PlayerEventLog::default()),
        );
        for mut t in qs.p1().iter_mut() {
            crate::gui::style::widgets::set_text_if_changed(&mut t, cache.alerts_line.clone());
        }
    }

    let log = logistics.as_deref();
    let intel_fp = (
        policy.apply_routing_congestion,
        // Fingerprint at the displayed precision (`{:.2}`), not finer: sub-display changes
        // re-shaped identical text every frame. `mean_edge_damage` is not shown here.
        (log.map(|l| l.congestion_proxy).unwrap_or(0.0) * 100.0).round() as i32,
        0,
    );
    if cache.intel_fp != Some(intel_fp) {
        cache.intel_fp = Some(intel_fp);
        cache.intel_line = format!(
            "INTEL  routes {}  c {:.2}",
            if policy.apply_routing_congestion { "on" } else { "off" },
            log.map(|l| l.congestion_proxy).unwrap_or(0.0),
        );
        for mut t in qs.p2().iter_mut() {
            crate::gui::style::widgets::set_text_if_changed(&mut t, cache.intel_line.clone());
        }
    }

    let w = weather.as_deref();
    let read = wx_read.as_deref();
    let weather_fp = if let Some(r) = read.filter(|r| r.ready) {
        (
            (r.display.wind_mps.length() * 10.0).round() as i32,
            (r.display.rain_mm_h * 10.0).round() as i32,
            (r.display.visibility_m / 100.0).round() as i32,
        )
    } else {
        (
            (w.map(|s| s.rain).unwrap_or(0.0) * 100.0).round() as i32,
            (w.map(|s| s.snow).unwrap_or(0.0) * 100.0).round() as i32,
            (w.map(|s| s.fog).unwrap_or(0.0) * 100.0).round() as i32,
        )
    };
    if cache.weather_fp != Some(weather_fp) {
        cache.weather_fp = Some(weather_fp);
        let compact = window
            .single()
            .map(|w| w.width() < 1920.0)
            .unwrap_or(false);
        cache.weather_line = if compact {
            format!(
                "WX r{:.1} s{:.1}",
                w.map(|s| s.rain).unwrap_or(0.0),
                w.map(|s| s.snow).unwrap_or(0.0),
            )
        } else if let Some(r) = read.filter(|r| r.ready) {
            format!("WX  {}", format_tactical_wx_line(&r.display))
        } else {
            format!(
                "WX  r {:.2}  s {:.2}  f {:.2}",
                w.map(|s| s.rain).unwrap_or(0.0),
                w.map(|s| s.snow).unwrap_or(0.0),
                w.map(|s| s.fog).unwrap_or(0.0),
            )
        };
        for mut t in qs.p3().iter_mut() {
            crate::gui::style::widgets::set_text_if_changed(&mut t, cache.weather_line.clone());
        }
    }

    let scarcity = world_fields.as_deref().map(|f| f.resource_scarcity).unwrap_or(0.5);
    let power_proxy = (1.0 - scarcity).clamp(0.0, 1.0);
    let wind_f = renewables
        .as_ref()
        .filter(|r| r.wind_n > 0)
        .map(|r| r.wind_mean)
        .unwrap_or(1.0);
    let solar_f = renewables
        .as_ref()
        .filter(|r| r.solar_n > 0)
        .map(|r| r.solar_mean)
        .unwrap_or(1.0);
    let power_fp = (power_proxy * 100.0).round() as i32
        + (wind_f * 100.0).round() as i32 * 1_000
        + (solar_f * 100.0).round() as i32 * 1_000_000;
    if cache.power_fp != Some(power_fp) {
        cache.power_fp = Some(power_fp);
        cache.power_line = format_ops_power_derate_line(power_proxy * 100.0, wind_f, solar_f);
        for mut t in qs.p4().iter_mut() {
            crate::gui::style::widgets::set_text_if_changed(&mut t, cache.power_line.clone());
        }
    }

    let tray_label = match tray.panel_state {
        HudPanelState::Collapsed => "▼ TRAY",
        HudPanelState::Peek => "◧ TRAY",
        HudPanelState::Expanded | HudPanelState::Pinned => "▲ TRAY",
    };
    if cache.tray_line != tray_label {
        cache.tray_line = tray_label.to_string();
        for mut t in qs.p5().iter_mut() {
            crate::gui::style::widgets::set_text_if_changed(&mut t, cache.tray_line.clone());
        }
    }
}

fn zone_shade_toggle_click_system(
    q: Query<&DeprecatedInteraction, (Changed<DeprecatedInteraction>, With<ZoneShadeToggleButton>)>,
    mut shade: ResMut<ZoneShadeToggle>,
) {
    for interaction in &q {
        if *interaction == DeprecatedInteraction::Pressed {
            shade.visible = !shade.visible;
        }
    }
}

fn sync_zone_shade_toggle_label(
    shade: Res<ZoneShadeToggle>,
    mut text: Query<&mut Text, With<ZoneShadeToggleLabel>>,
) {
    if !shade.is_changed() {
        return;
    }
    let label = if shade.visible { "SHADE ON" } else { "SHADE" };
    for mut line in &mut text {
        crate::gui::style::widgets::set_text_if_changed(&mut line, label);
    }
}

fn ops_strip_zone_click_system(
    q: Query<(&DeprecatedInteraction, &OpsStripZone), Changed<DeprecatedInteraction>>,
    mut tray: ResMut<ContextTrayState>,
    mut intel_req: ResMut<OpsStripIntelFocusRequest>,
    world: Option<Res<WorldRepresentationFrame>>,
) {
    for (interaction, zone) in &q {
        if *interaction != DeprecatedInteraction::Pressed {
            continue;
        }
        match zone {
            OpsStripZone::Alerts => {
                open_alerts_tray(tray.as_mut());
            }
            OpsStripZone::Intel => {
                if let Some(w) = world.as_deref() {
                    request_intel_focus(tray.as_mut(), w, intel_req.as_mut());
                }
            }
            OpsStripZone::TrayAffordance => {
                tray.cycle_tray_affordance();
            }
            OpsStripZone::Time | OpsStripZone::Weather | OpsStripZone::Power => {}
        }
    }
}

fn apply_ops_strip_intel_focus_system(
    mut req: ResMut<OpsStripIntelFocusRequest>,
    mut authority: ResMut<ViewProjectionAuthority>,
    mut trace: ResMut<ViewRuntimeTrace>,
) {
    let Some(world) = req.pending_world.take() else {
        return;
    };
    commit_world_main_map_focus(authority.as_mut(), trace.as_mut(), world);
}

fn context_tray_tab_click_system(
    q: Query<(&DeprecatedInteraction, &ContextTrayTabButton), Changed<DeprecatedInteraction>>,
    mut tray: ResMut<ContextTrayState>,
    mut player_log: Option<ResMut<crate::sim::effects::PlayerEventLog>>,
) {
    for (interaction, tab) in &q {
        if *interaction == DeprecatedInteraction::Pressed {
            tray.on_tab_pressed(tab.0);
            if tab.0 == ContextTrayTab::Events {
                if let Some(log) = player_log.as_mut() {
                    crate::sim::effects::clear_player_event_crit_unread(log.as_mut());
                }
            }
        }
    }
}

fn build_rail_tool_click_system(
    q: Query<(&DeprecatedInteraction, &BuildRailToolSlot), (Changed<DeprecatedInteraction>, With<Button>)>,
    mut strip: ResMut<BuildStripState>,
    mut tool: ResMut<crate::construction::ActiveBuildTool>,
    mut picker: ResMut<crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState>,
    mut road_sheet: ResMut<crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState>,
    mut power_sheet: ResMut<crate::gui::hud::sim_power_tool_sheet::SimPowerToolSheetState>,
) {
    for (interaction, slot) in &q {
        if *interaction != DeprecatedInteraction::Pressed {
            continue;
        }
        let already_open = picker.open && picker.anchor_slot == slot.0;
        if already_open {
            strip.active = ToolContext::None;
            crate::construction::apply_build_rail_tool_selection(&mut tool, ToolContext::None, true);
            picker.close();
            road_sheet.close();
            power_sheet.close();
        } else {
            strip.active = slot.0;
            crate::construction::apply_build_rail_tool_selection(&mut tool, slot.0, false);
            picker.open_for_slot(slot.0);
            if matches!(slot.0, ToolContext::Roads | ToolContext::Rail) {
                road_sheet.open = true;
                power_sheet.close();
            } else if slot.0 == ToolContext::Utilities {
                power_sheet.sync_from_tool(&tool);
                road_sheet.close();
                picker.close();
            } else {
                road_sheet.close();
                power_sheet.close();
            }
        }
    }
}

fn sync_ops_strip_zone_hover_system(
    palette: Res<UiPalette>,
    mut zones: Query<
        (&DeprecatedInteraction, &mut BorderColor, &mut Node),
        (With<Button>, With<OpsStripZone>),
    >,
) {
    let hot = palette.bevy_accent_hot();
    let idle = palette.bevy_border_subtle();
    for (interaction, mut border, mut node) in &mut zones {
        let emphasized = matches!(*interaction, DeprecatedInteraction::Hovered | DeprecatedInteraction::Pressed);
        *border = if emphasized {
            BorderColor::all(hot)
        } else {
            BorderColor::all(idle)
        };
        node.border = UiRect::all(Val::Px(1.0));
    }
}

#[must_use]
pub fn format_ops_strip_alert_badge(count: usize) -> String {
    if count > 99 {
        "◆99+".to_string()
    } else {
        format!("◆{count}")
    }
}

fn sync_ops_strip_alert_badge_system(
    missions: Option<Res<ActiveMissions>>,
    mut count_q: Query<&mut Text, With<OpsStripAlertBadgeText>>,
) {
    let count = missions.as_deref().map(|m| m.missions.len()).unwrap_or(0);
    let label = format_ops_strip_alert_badge(count);
    for mut text in &mut count_q {
        crate::gui::style::widgets::set_text_if_changed(&mut text, label.clone());
    }
}

fn sync_context_tray_visibility_system(
    tray: Res<ContextTrayState>,
    live: Option<Res<HudInfoLiveData>>,
    player_log: Option<Res<crate::sim::effects::PlayerEventLog>>,
    tool: Option<Res<crate::construction::ActiveBuildTool>>,
    ghost: Option<Res<crate::construction::BuildGhostState>>,
    preview: Option<Res<crate::construction::BuildPlacementPreview>>,
    registry: Option<Res<crate::construction::BuildingDefinitionRegistry>>,
    staging_mode: Option<Res<crate::construction::StagedPlacementMode>>,
    staging_book: Option<Res<crate::construction::StagedPlacementBook>>,
    path_feedback: Option<Res<crate::construction::ConstructionPathFeedback>>,
    mut q: ParamSet<(
        Query<(&mut Node, &mut Visibility), With<ContextTrayRoot>>,
        Query<&mut Visibility, With<ContextTrayBodyRoot>>,
    )>,
    mut body_q: Query<&mut Text, With<ContextTrayBodyLine>>,
) {
    let show = tray.panel_state.shows_content();
    let show_body = matches!(
        tray.panel_state,
        HudPanelState::Peek | HudPanelState::Expanded | HudPanelState::Pinned
    );
    let body_h = if tray.panel_state == HudPanelState::Peek {
        CONTEXT_TRAY_PEEK_BODY_H_PX
    } else {
        CONTEXT_TRAY_BODY_H_PX
    };
    let h = if show {
        CONTEXT_TRAY_TAB_H_PX + if show_body { body_h } else { 0.0 }
    } else {
        CONTEXT_TRAY_TAB_H_PX
    };

    if let Ok((mut node, mut vis)) = q.p0().single_mut() {
        *vis = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        node.height = Val::Px(h);
        node.min_height = Val::Px(h);
        node.max_height = Val::Px(h);
    }

    for mut v in q.p1().iter_mut() {
        *v = if show_body {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if let Some(d) = live.as_deref() {
        let body = match tray.active_tab {
            ContextTrayTab::Alerts => format!(
                "ALERTS · msn {} · T0 {:.2} · {}",
                d.mission_count, d.mean_threat_slot0, d.mission_hint
            ),
            ContextTrayTab::Events => crate::sim::effects::format_player_event_tray_body(
                player_log.as_deref().unwrap_or(&crate::sim::effects::PlayerEventLog::default()),
            ),
            ContextTrayTab::Intel => format!(
                "INTEL · routes {} · fract μ {:.2} · factions {}",
                if d.routes_layer_on { "on" } else { "off" },
                d.fracture_mean,
                d.theater_faction_slots
            ),
            ContextTrayTab::Logistics => format!(
                "LOGISTICS · congest {:.2} · stock {:.2} · edges {}",
                d.logistics_congestion, d.logistics_stockpile, d.transport_edges
            ),
            ContextTrayTab::Build => {
                let pending = d.pending_total;
                let unapproved = d.pending_unapproved;
                let subject = tool
                    .as_deref()
                    .and_then(super::sim_hud_copy::active_place_subject);
                let locked = ghost.as_deref().is_some_and(|g| {
                    g.placement_mode == crate::construction::BuildPlacementMode::Adjust
                        && g.origin.is_some()
                });
                let snap = path_feedback.as_deref().and_then(|p| p.snap_hint.as_deref());
                let mut body = crate::gui::hud::sim_hud_copy::tray_build_status_line(
                    pending,
                    unapproved,
                    subject.as_deref(),
                    locked,
                    snap,
                );
                if let (Some(t), Some(g), Some(p), Some(reg)) = (
                    tool.as_deref(),
                    ghost.as_deref(),
                    preview.as_deref(),
                    registry.as_deref(),
                ) {
                    if let Some(param) =
                        crate::construction::parametric_active_ghost_hud_line(t, reg, g, p)
                    {
                        body.push('\n');
                        body.push_str(&param);
                    }
                }
                if let (Some(mode), Some(book)) =
                    (staging_mode.as_deref(), staging_book.as_deref())
                {
                    if mode.enabled || book.staged_count() > 0 {
                        body.push('\n');
                        body.push_str(&crate::construction::staged_placements_tray_lines(
                            mode, book,
                        ));
                    }
                }
                body
            }
            ContextTrayTab::Diagnostics => format!(
                "DIAG · sim n={} · fire parts {} · pending {}/{}",
                d.sim_tick,
                d.fire_particle_rows,
                d.pending_total.saturating_sub(d.pending_unapproved),
                d.pending_total
            ),
        };
        for mut text in &mut body_q {
            crate::gui::style::widgets::set_text_if_changed(&mut text, body.clone());
        }
    } else if tray.active_tab == ContextTrayTab::Events {
        let body = crate::sim::effects::format_player_event_tray_body(
            player_log.as_deref().unwrap_or(&crate::sim::effects::PlayerEventLog::default()),
        );
        for mut text in &mut body_q {
            crate::gui::style::widgets::set_text_if_changed(&mut text, body.clone());
        }
    }
}

fn sync_context_tray_tab_chrome_system(
    tray: Res<ContextTrayState>,
    palette: Res<UiPalette>,
    mut tabs: Query<
        (
            &ContextTrayTabButton,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut Node,
            &Children,
        ),
        (With<Button>, With<ContextTrayTabButton>),
    >,
    mut labels: Query<&mut TextColor, With<ContextTrayTabLabel>>,
) {
    for (btn, mut bg, mut border, mut node, children) in &mut tabs {
        let selected = btn.0 == tray.active_tab;
        if selected {
            *bg = BackgroundColor(palette.bevy_bg_vellum());
            *border = BorderColor {
                left: palette.bevy_accent_gold(),
                top: palette.bevy_border_subtle(),
                right: palette.bevy_border_subtle(),
                bottom: palette.bevy_border_subtle(),
            };
            node.border = UiRect {
                left: Val::Px(CONTEXT_TRAY_TAB_GOLD_BAR_PX),
                top: Val::Px(1.0),
                right: Val::Px(1.0),
                bottom: Val::Px(1.0),
            };
        } else {
            *bg = BackgroundColor(palette.bevy_hud_panel_fill());
            *border = BorderColor::all(palette.bevy_border_subtle());
            node.border = UiRect::all(Val::Px(1.0));
        }
        for child in children.iter() {
            if let Ok(mut color) = labels.get_mut(child) {
                *color = if selected {
                    TextColor(palette.bevy_accent_gold())
                } else {
                    TextColor(palette.bevy_text_muted())
                };
            }
        }
    }
}

fn build_rail_icon_tint(palette: &UiPalette, interaction: &DeprecatedInteraction, selected: bool) -> Color {
    if selected {
        palette.bevy_accent_gold()
    } else if matches!(*interaction, DeprecatedInteraction::Hovered | DeprecatedInteraction::Pressed) {
        palette.bevy_accent_hot()
    } else {
        palette.bevy_text_muted().with_alpha(0.72)
    }
}

#[must_use]
fn build_rail_slot_border_color(
    palette: &UiPalette,
    interaction: &DeprecatedInteraction,
    selected: bool,
) -> Color {
    if selected {
        palette.bevy_accent_gold()
    } else if matches!(*interaction, DeprecatedInteraction::Hovered | DeprecatedInteraction::Pressed) {
        palette.bevy_accent_hot()
    } else {
        palette.bevy_border_subtle()
    }
}

fn sync_build_rail_from_strip_system(
    strip: Res<BuildStripState>,
    picker: Res<crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState>,
    palette: Res<UiPalette>,
    mut slots: Query<
        (
            &BuildRailToolSlot,
            &DeprecatedInteraction,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mut labels: Query<(&mut Text, &mut TextColor, &mut Visibility), With<BuildRailToolLabel>>,
) {
    for (slot, interaction, children, mut bg, mut border) in &mut slots {
        let selected = if picker.open {
            slot.0 == picker.anchor_slot
        } else {
            slot.0 == strip.active && slot.0 != ToolContext::None
        };
        let next_bg = if selected {
            BackgroundColor(palette.bevy_bg_vellum())
        } else {
            BackgroundColor(palette.bevy_hud_panel_fill())
        };
        if *bg != next_bg {
            *bg = next_bg;
        }
        let next_border = BorderColor::all(build_rail_slot_border_color(
            &palette,
            interaction,
            selected,
        ));
        if *border != next_border {
            *border = next_border;
        }
        let icon_tint = build_rail_icon_tint(&palette, interaction, selected);
        let word = crate::gui::hud::sim_hud_copy::rail_player_label(slot.0);
        let next_color = TextColor(icon_tint);
        for child in children.iter() {
            if let Ok((mut text, mut color, mut vis)) = labels.get_mut(child) {
                if text.as_str() != word {
                    crate::gui::style::widgets::set_text_if_changed(&mut text, word);
                }
                if *color != next_color {
                    *color = next_color;
                }
                if *vis != Visibility::Visible {
                    *vis = Visibility::Visible;
                }
            }
        }
    }
}

/// P4-VEH-01: TRUCK / URAL / BUS chips when Logistics context tray tab is active.
fn sync_logistics_vehicle_chips_system(
    tray: Res<ContextTrayState>,
    palette: Res<UiPalette>,
    atlas_ui: Option<Res<IconAtlasUi>>,
    manifests: Res<Assets<IconAtlasManifest>>,
    mut rows: Query<&mut Visibility, With<LogisticsVehicleChipRow>>,
    chips: Query<(&LogisticsVehicleChip, &DeprecatedInteraction, &Children)>,
    mut icons: Query<
        &mut bevy::ui::widget::ImageNode,
        (With<LogisticsVehicleChipIcon>, Without<LogisticsVehicleChipRow>),
    >,
) {
    let show = tray.active_tab == ContextTrayTab::Logistics
        && matches!(
            tray.panel_state,
            HudPanelState::Peek | HudPanelState::Expanded | HudPanelState::Pinned
        );
    for mut vis in &mut rows {
        *vis = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    let Some(atlas) = atlas_ui.as_ref() else {
        return;
    };
    if !show {
        return;
    }
    for (chip, interaction, children) in &chips {
        let Some(node) = atlas.image_node_for_id(&manifests, chip.0) else {
            continue;
        };
        let tint = build_rail_icon_tint(&palette, interaction, false);
        for child in children.iter() {
            if let Ok(mut icon) = icons.get_mut(child) {
                *icon = node.clone().with_color(tint);
            }
        }
    }
}

/// P4-P5-01: petroleum panel tab visible when Industry build context + expanded tray.
#[must_use]
pub fn petroleum_panel_tab_visible(strip: &BuildStripState, tray: &ContextTrayState) -> bool {
    strip.active == ToolContext::Industry && tray.panel_state != HudPanelState::Collapsed
}

/// P4-P5-01: petroleum panel tab icon when Industry build context is active.
fn sync_petroleum_panel_tab_system(
    strip: Res<BuildStripState>,
    tray: Res<ContextTrayState>,
    palette: Res<UiPalette>,
    atlas_ui: Option<Res<IconAtlasUi>>,
    manifests: Res<Assets<IconAtlasManifest>>,
    mut roots: Query<(&DeprecatedInteraction, &mut Visibility), With<PetroleumPanelTabRoot>>,
    mut icons: Query<
        &mut bevy::ui::widget::ImageNode,
        (With<PetroleumPanelTabIcon>, Without<PetroleumPanelTabRoot>),
    >,
) {
    let show = petroleum_panel_tab_visible(&strip, &tray);
    let mut icon_tint = palette.bevy_text_muted().with_alpha(0.72);
    for (interaction, mut vis) in &mut roots {
        *vis = if show {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if show {
            icon_tint = build_rail_icon_tint(&palette, interaction, false);
        }
    }
    if !show {
        return;
    }
    let Some(atlas) = atlas_ui.as_ref() else {
        return;
    };
    if let Ok(mut icon) = icons.single_mut() {
        if let Some(node) = atlas.image_node_for_id(&manifests, IconId::P5Br) {
            *icon = node.with_color(icon_tint);
        }
    }
}

#[allow(dead_code)] // egui-retired; kept for Bevy port / export
fn sync_minimap_gpu_image_node_system(
    registry: Res<MinimapRenderTargetRegistry>,
    fallback: Res<crate::render::TileWorldFallbackState>,
    mut shell: ResMut<MinimapShellState>,
    mut compositor: ResMut<MinimapCompositorState>,
    map_views: Res<crate::gui::MapViewInstances>,
    params: Res<WorldGenParams>,
    chrome_q: Query<&Node, (With<MinimapChromeRoot>, Without<MinimapGpuImageNode>)>,
    mut gpu_q: Query<
        (&mut Node, &mut Visibility, &mut bevy::ui::widget::ImageNode),
        (With<MinimapGpuImageNode>, Without<MinimapChromeRoot>),
    >,
) {
    let Ok((mut node, mut vis, mut image)) = gpu_q.single_mut() else {
        return;
    };
    let display = crate::gui::map_view::resolve_minimap_bevy_display_handle(
        &shell,
        &fallback,
        &registry,
        compositor.stamp,
    );
    if display == Handle::default() {
        *vis = Visibility::Hidden;
        compositor.dual_minimap_present = false;
        return;
    }
    let using_gpu_rt =
        display == registry.committed_image && compositor.stamp > 0;
    *vis = Visibility::Visible;
    let mm = &map_views.minimap;
    let panel = shell
        .last_body_rect
        .map(|r| Vec2::new(r.width().max(1.0), r.height().max(1.0)))
        .unwrap_or(mm.viewport_size);
    let tex_extent = crate::gui::map_view::minimap_bevy_display_texel_extent(
        &shell,
        &registry,
        compositor.stamp,
        &params,
    );
    let tex_w = tex_extent.x;
    let tex_h = tex_extent.y;
    let crop = if using_gpu_rt
        && crate::gui::map_presentation_fit::minimap_gpu_presentation_uses_crop(mm.follow_mode)
    {
        crate::gui::map_presentation_fit::minimap_gpu_texture_pixel_rect_from_world(
            mm.camera_center,
            mm.zoom,
            params.width.max(1),
            params.height.max(1),
            tex_w,
            tex_h,
            panel,
        )
    } else {
        bevy::math::Rect {
            min: Vec2::ZERO,
            max: Vec2::new(tex_w as f32, tex_h as f32),
        }
    };
    *image = bevy::ui::widget::ImageNode {
        rect: Some(crop),
        ..bevy::ui::widget::ImageNode::from(display)
    };

    let pad = MINIMAP_CHROME_STROKE_PAD_PX;
    let _ = chrome_q; // root geometry mirrors `last_window_rect`; rects come from the shell directly.
    if let (Some(content), Some(body)) = (shell.last_window_rect, shell.last_body_rect) {
        let root_origin = bevy::prelude::Vec2::new(content.min.x - pad, content.min.y - pad);
        node.position_type = PositionType::Absolute;
        node.left = Val::Px(body.min.x - root_origin.x);
        node.top = Val::Px(body.min.y - root_origin.y);
        node.width = Val::Px(body.width().max(1.0));
        node.height = Val::Px(body.height().max(1.0));
        // Image fills exactly the body rect — this IS the painted-image rect.
        shell.last_image_rect = Some(body);
    } else {
        node.width = Val::Percent(100.0);
        node.height = Val::Percent(100.0);
    }
    compositor.dual_minimap_present = false;
}

#[allow(dead_code)] // egui-retired; kept for Bevy port / export
fn sync_minimap_chrome_root_system(
    mut minimap: ResMut<MinimapShellState>,
    win: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&mut Node, &mut Visibility), With<MinimapChromeRoot>>,
) {
    let Ok((mut node, mut vis)) = q.single_mut() else {
        return;
    };
    let window = win.single().ok();
    if !minimap.visible || minimap.minimized {
        *vis = Visibility::Hidden;
        return;
    }
    // MINIMAP-SIZE-AUTHORITY-001: the panel CONTENT logical size is owned by
    // `resolve_minimap_panel_viewport` (mirrored into `panel_viewport_suggestion_logical_size` and
    // applied to `viewport_size`). The chrome lays out INWARD from that content size — we must never
    // feed the outer (content + stroke pad) box back as the next content rect. The old code read
    // `last_window_rect` (which the removed `apply_chrome_outer_rect` set to the OUTER box) as
    // `content`, then re-added `pad*2` every frame → a +2px/frame ratchet that never settled.
    let origin = match minimap.panel_screen_origin {
        Some(o) => o,
        None => {
            let Some(content) = window.map(|w| {
                crate::gui::simulation_minimap_bootstrap_rect(
                    w.width(),
                    w.height(),
                    minimap.viewport_size,
                )
            }) else {
                *vis = Visibility::Hidden;
                return;
            };
            // Seed the persistent drag origin once, then fall through using `viewport_size`.
            minimap.panel_screen_origin = Some(Vec2::new(content.min.x, content.min.y));
            Vec2::new(content.min.x, content.min.y)
        }
    };
    // CONTENT rect = origin + authoritative content size. `sync_layout_rects_from_panel_origin`
    // writes `last_window_rect`/`last_image_rect`/title/rails/grip as the CONTENT box (not the outer
    // stroke box), so the next-frame read is stable.
    minimap.sync_layout_rects_from_panel_origin();
    let content = minimap
        .last_window_rect
        .unwrap_or_else(|| bevy::prelude::Rect::from_corners(bevy::prelude::Vec2::new(origin.x, origin.y), bevy::prelude::Vec2::new(origin.x, origin.y) + bevy::prelude::Vec2::new(minimap.viewport_size.x, minimap.viewport_size.y),));
    *vis = Visibility::Visible;
    // Layout rects and cursor_position share logical client pixels (see `minimap_window_logical_size`).
    let pad = MINIMAP_CHROME_STROKE_PAD_PX;
    let min_x = content.min.x - pad;
    let min_y = content.min.y - pad;
    let w_px = content.width() + pad * 2.0;
    let h_px = content.height() + pad * 2.0;
    node.left = Val::Px(min_x);
    node.top = Val::Px(min_y);
    node.width = Val::Px(w_px);
    node.height = Val::Px(h_px);
    minimap.sync_panel_viewport_suggestion_from_layout();
}

/// MINIMAP-WIDGET-IMPL-001 — position the title bar + resize grip chrome children inside the chrome
/// root, mirroring the `MinimapShellState` hit-test rects so the *visible* targets line up with the
/// pointer system's `title_bar_rect` / `resize_grip_rect`. Read-only over the size authority: it only
/// places children inside the already-resolved content box, never resizes the panel.
#[allow(clippy::type_complexity)]
#[allow(dead_code)] // egui-retired; kept for Bevy port / export
fn sync_minimap_chrome_layout_system(
    minimap: Res<MinimapShellState>,
    mut title_q: Query<
        &mut Node,
        (
            With<MinimapChromeTitleBar>,
            Without<MinimapChromeResizeGrip>,
        ),
    >,
    mut grip_q: Query<
        &mut Node,
        (
            With<MinimapChromeResizeGrip>,
            Without<MinimapChromeTitleBar>,
        ),
    >,
) {
    let Some(content) = minimap.last_window_rect else {
        return;
    };
    let pad = MINIMAP_CHROME_STROKE_PAD_PX;
    // Chrome root top-left in screen space (the outer stroke box).
    let root_origin = bevy::prelude::Vec2::new(content.min.x - pad, content.min.y - pad);

    if let (Ok(mut title), Some(bar)) = (title_q.single_mut(), minimap.title_bar_rect) {
        title.position_type = PositionType::Absolute;
        title.left = Val::Px(bar.min.x - root_origin.x);
        title.top = Val::Px(bar.min.y - root_origin.y);
        title.width = Val::Px(bar.width().max(1.0));
        title.height = Val::Px(bar.height().max(1.0));
    }
    if let (Ok(mut grip), Some(g)) = (grip_q.single_mut(), minimap.resize_grip_rect) {
        grip.position_type = PositionType::Absolute;
        grip.left = Val::Px(g.min.x - root_origin.x);
        grip.top = Val::Px(g.min.y - root_origin.y);
        grip.width = Val::Px(g.width().max(1.0));
        grip.height = Val::Px(g.height().max(1.0));
    }
}

pub fn collapse_context_tray_on_escape(tray: &mut ContextTrayState) {
    tray.panel_state.collapse_unpinned();
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sim_tick_line_format() {
        assert_eq!(format_sim_tick_line(42, false, 1.0), "T+00042  RUN    v=1.0x");
    }

    #[test]
    fn context_tray_cycle_collapsed_peek_expanded() {
        let mut tray = ContextTrayState::default();
        assert_eq!(tray.panel_state, HudPanelState::Collapsed);
        tray.cycle_tray_affordance();
        assert_eq!(tray.panel_state, HudPanelState::Peek);
        tray.cycle_tray_affordance();
        assert_eq!(tray.panel_state, HudPanelState::Expanded);
        tray.cycle_tray_affordance();
        assert_eq!(tray.panel_state, HudPanelState::Collapsed);
    }

    #[test]
    fn ops_strip_alert_badge_diamond_and_count() {
        assert_eq!(format_ops_strip_alert_badge(0), "◆0");
        assert_eq!(format_ops_strip_alert_badge(42), "◆42");
        assert_eq!(format_ops_strip_alert_badge(100), "◆99+");
    }

    #[test]
    fn build_rail_hover_border_uses_accent_hot() {
        let palette = UiPalette::default();
        let hot = build_rail_slot_border_color(&palette, &DeprecatedInteraction::Hovered, false);
        assert_eq!(hot, palette.bevy_accent_hot());
        let gold = build_rail_slot_border_color(&palette, &DeprecatedInteraction::None, true);
        assert_eq!(gold, palette.bevy_accent_gold());
        let subtle = build_rail_slot_border_color(&palette, &DeprecatedInteraction::None, false);
        assert_eq!(subtle, palette.bevy_border_subtle());
    }

    #[test]
    fn context_tray_tab_peek_then_expand() {
        let mut tray = ContextTrayState::default();
        tray.on_tab_pressed(ContextTrayTab::Alerts);
        assert_eq!(tray.panel_state, HudPanelState::Peek);
        tray.on_tab_pressed(ContextTrayTab::Alerts);
        assert_eq!(tray.panel_state, HudPanelState::Expanded);
        tray.on_tab_pressed(ContextTrayTab::Intel);
        assert_eq!(tray.active_tab, ContextTrayTab::Intel);
        assert_eq!(tray.panel_state, HudPanelState::Expanded);
    }




























    #[test]
    fn p4_p5_01_petroleum_panel_tab_visible_when_industry_and_tray_open() {
        let strip = BuildStripState {
            active: ToolContext::Industry,
            ..Default::default()
        };
        let mut tray = ContextTrayState::default();
        tray.panel_state = HudPanelState::Expanded;
        assert!(petroleum_panel_tab_visible(&strip, &tray));
        tray.panel_state = HudPanelState::Collapsed;
        assert!(!petroleum_panel_tab_visible(&strip, &tray));
        let strip_roads = BuildStripState {
            active: ToolContext::Roads,
            ..Default::default()
        };
        tray.panel_state = HudPanelState::Expanded;
        assert!(!petroleum_panel_tab_visible(&strip_roads, &tray));
    }




}
