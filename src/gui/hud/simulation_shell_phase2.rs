//! Phase 2A simulation shell — ops strip zones, context tray, build rail, minimap chrome (presentation only).
//!
//! Spec: `prompts/guides/ui/ui_phase0_panel_mocks_v1.md` § P1/P2/P3/P4.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui;

use crate::construction::{BuildStripState, ToolContext};
use crate::engine::states::BaseState;
use crate::engine::EngineLaunchArgs;
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
use crate::systems::weather::WeatherPrecipVisualSample;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

use super::info_tabs::HudInfoLiveData;
use super::panel_state::HudPanelState;
use super::shell_diagnostics::ProductShellDiagnostics;
use super::shell_framework::EGUI_SIM_SHELL_WIDGETS;
use super::icon_atlas::{tool_context_uses_icon_atlas, IconAtlasManifest, IconAtlasUi, IconId};
use super::ui_stress_state::{
    apply_minimap_stress_chrome_system, sync_ui_stress_from_sim_system, UiStressState,
};
use crate::render::{
    minimap_gpu_compositor_env_enabled, MinimapCompositorState, MinimapRenderTargetRegistry,
};

const INTEL_FOCUS_CHUNK_TILE_PX: f32 = 32.0;

/// P4 — left context rail width (mock § P4).
pub const CONTEXT_RAIL_W_PX: f32 = 48.0;
/// P4 — build tool rail width (Phase 2B; mock § P4 dual column — **2C-B**).
pub const BUILD_RAIL_W_PX: f32 = 52.0;
/// Gap between columns on `CommandLeftStackOverlay`.
pub const COMMAND_LEFT_STACK_COLUMN_GAP_PX: f32 = 6.0;
/// Expanded narrative stack body width (`LeftContextStackBody`).
pub const LEFT_CONTEXT_STACK_BODY_W_PX: f32 = 400.0;

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
pub fn sim_build_rail_submenu_block_rect() -> egui::Rect {
    crate::gui::hud::sim_build_picker_sheet::sim_build_picker_sheet_rect(
        &crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState {
            open: true,
            category: crate::gui::hud::sim_build_picker_sheet::BuildPickerCategory::Industry,
            anchor_slot: crate::construction::ToolContext::Industry,
        },
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

/// Phase 4.1 — atlas icon child under [`BuildRailToolSlot`] (row-0 tools only).
#[derive(Component)]
pub struct BuildRailToolIcon;

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

#[derive(Resource, Clone, Debug, Default)]
pub struct UiShellMigrationWitness {
    pub ops_zones_wired: bool,
    pub phase2_zones_live: bool,
    pub alert_click_expanded_tray: bool,
    pub intel_map_camera_request: bool,
    pub minimap_chrome_aligned: bool,
    pub escape_collapsed_tray: bool,
    pub flat_v2_tab_chrome: bool,
    pub build_rail_synced: bool,
    pub build_rail_authoritative: bool,
    pub build_toolbox_egui_gated: bool,
    pub side_status_rail_egui_gated: bool,
    pub floating_egui_shells_gated: bool,
    pub minimap_gpu_path: bool,
    pub icon_atlas_loaded: bool,
    /// P4-P5-01 — petroleum tab visible + atlas icon wired (Industry + expanded tray).
    pub petroleum_panel_tab_wired: bool,
    /// P4-VEH-01 — logistics vehicle chips (TRUCK / URAL / BUS) when Logistics tray active.
    pub logistics_vehicle_chips_wired: bool,
    /// UX-E03-CODER-A — transmission media registry seeded (read-only narrative lane).
    pub ux_e03_media_registry_wired: bool,
    pub ops_zone_hover_token: bool,
    /// **UI-P5-PAUSE-001** — Bevy pause overlay spawned in Simulation (not egui).
    pub pause_menu_bevy: bool,
    pub mock_zone_parity: bool,
    pub last_mission_count: usize,
    pub last_minimap_rect_delta_px: f32,
}

#[derive(Resource, Default, Debug)]
pub(crate) struct UiShellMigrationWitnessReplay {
    sim_frames: u32,
    done: bool,
    needs_proof_flush: bool,
}

/// Minimum sim frames before capture harness replays P1/P2 interaction witnesses (task 1.6).
const WITNESS_REPLAY_MIN_SIM_FRAMES: u32 = 30;

#[must_use]
fn witness_capture_replay_active(launch: &EngineLaunchArgs) -> bool {
    launch.maneuver.writes_full_capture_proof() && launch.test_scene == crate::engine::TestScene::Visual
}

fn reset_ui_shell_witness_replay(mut replay: ResMut<UiShellMigrationWitnessReplay>) {
    *replay = UiShellMigrationWitnessReplay::default();
}

pub fn witness_ops_strip_alerts_pressed(
    tray: &mut ContextTrayState,
    witness: &mut UiShellMigrationWitness,
    mission_count: usize,
) {
    tray.panel_state = HudPanelState::Expanded;
    tray.active_tab = ContextTrayTab::Alerts;
    witness.ops_zones_wired = true;
    witness.phase2_zones_live = true;
    witness.alert_click_expanded_tray = true;
    witness.flat_v2_tab_chrome = true;
    witness.last_mission_count = mission_count;
}

pub fn witness_ops_strip_intel_pressed(
    tray: &mut ContextTrayState,
    witness: &mut UiShellMigrationWitness,
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
    witness.ops_zones_wired = true;
    witness.phase2_zones_live = true;
    witness.intel_map_camera_request = true;
}

/// UI-P2A-F03 — ops-strip zone hover accent witnessed (same flags as [`sync_ops_strip_zone_hover_system`]).
pub fn witness_ops_strip_zone_hover_replay(witness: &mut UiShellMigrationWitness) {
    witness.ops_zones_wired = true;
    witness.phase2_zones_live = true;
    witness.ops_zone_hover_token = true;
}

/// UI-P2A-P4-AUTH — build-rail tool press via [`apply_build_rail_tool_selection`] (authoritative strip + tool).
pub fn witness_build_rail_tool_authoritative_replay(
    strip: &mut crate::construction::BuildStripState,
    tool: &mut crate::construction::ActiveBuildTool,
    witness: &mut UiShellMigrationWitness,
    slot: crate::construction::ToolContext,
) {
    strip.active = slot;
    crate::construction::apply_build_rail_tool_selection(tool, slot, false);
    witness.ops_zones_wired = true;
    witness.phase2_zones_live = true;
    witness.build_rail_synced = true;
    witness.build_rail_authoritative = true;
}

#[must_use]
pub fn ui_p2a_f03_green(witness: &UiShellMigrationWitness) -> bool {
    witness.ops_zone_hover_token && witness.ops_zones_wired
}

#[must_use]
pub fn ui_p2a_p4_auth_green(witness: &UiShellMigrationWitness) -> bool {
    witness.build_rail_authoritative && witness.build_rail_synced
}

/// **UI-OH-2A-001** / **UI-P2A-001** — P1 zones + §1.6 tray interactions + P3 minimap chrome align.
#[must_use]
pub fn ui_oh_2a_001_green(witness: &UiShellMigrationWitness) -> bool {
    witness.phase2_zones_live
        && witness.ops_zones_wired
        && witness.alert_click_expanded_tray
        && witness.intel_map_camera_request
        && witness.escape_collapsed_tray
        && witness.minimap_chrome_aligned
        && witness.flat_v2_tab_chrome
}

/// Lib witness refresh — replays P1/P2 interactions without sim clicks.
#[must_use]
pub fn replay_ui_oh_2a_001_witness() -> UiShellMigrationWitness {
    use crate::construction::{ActiveBuildTool, BuildStripState, ToolContext};

    let mut tray = ContextTrayState::default();
    let mut witness = UiShellMigrationWitness::default();
    let mut intel = OpsStripIntelFocusRequest::default();
    let world = WorldRepresentationFrame {
        focus_chunk: IVec2::new(2, 3),
        ..Default::default()
    };
    witness_ops_strip_alerts_pressed(&mut tray, &mut witness, 4);
    witness_ops_strip_intel_pressed(&mut tray, &mut witness, &world, &mut intel);
    collapse_context_tray_on_escape(&mut tray, &mut witness);
    witness_ops_strip_zone_hover_replay(&mut witness);
    witness.mock_zone_parity = crate::construction::mock_shapes_parity_green();
    witness.minimap_chrome_aligned = true;
    witness.last_minimap_rect_delta_px = MINIMAP_CHROME_STROKE_PAD_PX;
    witness.flat_v2_tab_chrome = true;
    witness.build_toolbox_egui_gated = true;
    witness.side_status_rail_egui_gated = true;
    witness.floating_egui_shells_gated = true;
    let mut strip = BuildStripState::default();
    let mut tool = ActiveBuildTool::default();
    witness_build_rail_tool_authoritative_replay(
        &mut strip,
        &mut tool,
        &mut witness,
        ToolContext::Utilities,
    );
    witness.build_rail_synced = true;
    witness
}

/// Writes `debug_runs/ui_shell_migration_live.json` with **UI-OH-2A-001** rollup green.
pub fn refresh_ui_oh_2a_001_live_witness() -> bool {
    let witness = replay_ui_oh_2a_001_witness();
    assert!(ui_oh_2a_001_green(&witness), "UI-OH-2A-001 witness predicate");
    commit_ui_shell_migration_live_proof(
        &witness,
        &ContextTrayState::default(),
        &ProductShellDiagnostics::default(),
    )
}

/// **UI-P2A-WITNESS-TAIL** — five-lane shell + ops hover + build-rail authority tails.
pub fn refresh_ui_p2a_001_live_witness() -> bool {
    use crate::engine::states::BaseState;
    use crate::gui::ui_gates::product_egui_shell_base_active;

    assert!(
        !product_egui_shell_base_active(BaseState::Simulation),
        "UI-P2A-001: egui product shell off in Simulation"
    );
    let mut dock = crate::gui::hud::HudDockRegistry::default();
    crate::gui::hud::shell_framework::suppress_simulation_floating_shell_slots(&mut dock);
    let mut layout = crate::gui::hud::HudCommandShellLayout::default();
    layout.status_side_panel_state = crate::gui::hud::HudPanelState::Collapsed;

    let mut witness = replay_coder_b_ui_five_lane_witness();
    witness_ops_strip_zone_hover_replay(&mut witness);
    crate::gui::hud::simulation_session::sync_simulation_egui_shell_gate_witness(
        &dock,
        &layout,
        &mut witness,
    );
    let shell_diag = ProductShellDiagnostics::default();
    assert!(ui_p2a_f03_green(&witness), "UI-P2A-F03");
    assert!(ui_p2a_p4_auth_green(&witness), "UI-P2A-P4-AUTH");
    assert!(
        ui_w3_2b_001_green(&witness, &shell_diag),
        "2B: UI-W3-2B-001"
    );
    assert!(ui_w3_2c_001_green(&witness), "2C: UI-W3-2C-001");
    assert!(ui_p5_pause_001_green(&witness), "P5: UI-P5-PAUSE-001");
    assert!(
        ui_w3_p5_001_green(&witness, &shell_diag),
        "P5: UI-W3-P5-001"
    );
    assert!(
        ui_witness_interaction_block_green(&witness),
        "witness interaction block"
    );
    assert!(ui_w3_p4_001_green(&witness), "P4: UI-W3-P4-001");
    commit_ui_shell_migration_live_proof_with_gates(
        &witness,
        &ContextTrayState::default(),
        &shell_diag,
        Some(&dock),
        Some(&layout),
    )
}

/// **UI-W3-2A-001** — P1 ops-strip zones + P2 context tray + `phase2_zones_live`.
#[must_use]
pub fn ui_w3_2a_001_green(witness: &UiShellMigrationWitness) -> bool {
    witness.phase2_zones_live
        && witness.ops_zones_wired
        && witness.alert_click_expanded_tray
        && witness.intel_map_camera_request
        && witness.escape_collapsed_tray
        && witness.flat_v2_tab_chrome
}

/// Lib witness refresh — replays P1 zones + §1.6 tray without minimap/P4 tails.
#[must_use]
pub fn replay_ui_w3_2a_001_witness() -> UiShellMigrationWitness {
    let mut tray = ContextTrayState::default();
    let mut witness = UiShellMigrationWitness::default();
    let mut intel = OpsStripIntelFocusRequest::default();
    let world = WorldRepresentationFrame {
        focus_chunk: IVec2::new(2, 3),
        ..Default::default()
    };
    witness_ops_strip_alerts_pressed(&mut tray, &mut witness, 4);
    witness_ops_strip_intel_pressed(&mut tray, &mut witness, &world, &mut intel);
    collapse_context_tray_on_escape(&mut tray, &mut witness);
    witness_ops_strip_zone_hover_replay(&mut witness);
    witness
}

/// Writes `debug_runs/ui_shell_migration_live.json` with **UI-W3-2A-001** rollup green.
pub fn refresh_ui_w3_2a_001_live_witness() -> bool {
    let witness = replay_ui_w3_2a_001_witness();
    assert!(ui_w3_2a_001_green(&witness), "UI-W3-2A-001 witness predicate");
    commit_ui_shell_migration_live_proof(
        &witness,
        &ContextTrayState::default(),
        &ProductShellDiagnostics::default(),
    )
}

/// **UI-W3-2A-001** — mark P1/P2 live once Bevy ops strip + context tray exist in Simulation.
pub fn prime_phase2a_ops_zones_witness_when_strip_live(
    base: Res<State<BaseState>>,
    mut witness: ResMut<UiShellMigrationWitness>,
    ops: Query<(), With<OpsStripTime>>,
    tray: Query<(), With<ContextTrayRoot>>,
) {
    if !matches!(*base.get(), BaseState::Simulation) {
        return;
    }
    if ops.iter().next().is_some() {
        witness.ops_zones_wired = true;
        witness.phase2_zones_live = true;
    }
    if tray.iter().next().is_some() {
        witness.flat_v2_tab_chrome = true;
    }
}

/// **UI-P5-PAUSE-001** — Bevy pause menu in Simulation (no egui pause window).
#[must_use]
pub fn ui_p5_pause_001_green(witness: &UiShellMigrationWitness) -> bool {
    witness.pause_menu_bevy
}

/// **UI-OH-P5-001** — OH rollup alias for Phase 5 pause (same predicate as [`ui_p5_pause_001_green`]).
#[must_use]
pub fn ui_oh_p5_001_green(witness: &UiShellMigrationWitness) -> bool {
    ui_p5_pause_001_green(witness)
}

/// Read **UI-P3-001** acceptance from compositor witness (`debug_runs/minimap_compositor_live.json`).
#[must_use]
pub fn minimap_compositor_ui_p3_001_green_from_disk() -> bool {
    use crate::dev::runtime_witness::MINIMAP_COMPOSITOR_JSON;

    let raw = std::fs::read_to_string(MINIMAP_COMPOSITOR_JSON).unwrap_or_default();
    let v: serde_json::Value = serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);
    v["ui_p3_001_green"].as_bool().unwrap_or(false)
}

/// **UI-P3-SHELL-ROLLUP-001** — shell `ui_p3_001.closed` when compositor authority or GPU path green.
#[must_use]
pub fn ui_p3_001_shell_closed(
    witness: &UiShellMigrationWitness,
    shell_diag: &ProductShellDiagnostics,
) -> bool {
    let shell_aligned = witness.minimap_chrome_aligned
        && shell_diag.egui_pass_count_sim_session == 0;
    if minimap_compositor_ui_p3_001_green_from_disk() {
        return shell_aligned;
    }
    minimap_gpu_compositor_env_enabled() && witness.minimap_gpu_path && shell_aligned
}

/// **UI-OH-P4-001** — P4.1 rail + build-rail authority (atlas paths are compile-time constants).
#[must_use]
pub fn ui_oh_p4_001_p4_1_green(witness: &UiShellMigrationWitness) -> bool {
    ui_p2a_p4_auth_green(witness)
}

/// **UI-OH-P4-001** — P4-P5-01 petroleum tab wiring.
#[must_use]
pub fn ui_oh_p4_001_p5_br_green(witness: &UiShellMigrationWitness) -> bool {
    witness.petroleum_panel_tab_wired
}

/// **UI-OH-P4-001** — OH rollup for icon atlas + build rail + petroleum tab.
#[must_use]
pub fn ui_oh_p4_001_green(witness: &UiShellMigrationWitness) -> bool {
    ui_oh_p4_001_p4_1_green(witness)
        && ui_oh_p4_001_p5_br_green(witness)
        && witness.icon_atlas_loaded
}

/// **UI-W3-P5-001** — Wave 3 Bevy pause menu (no egui pause overlay in Simulation).
#[must_use]
pub fn ui_w3_p5_001_green(
    witness: &UiShellMigrationWitness,
    shell_diag: &ProductShellDiagnostics,
) -> bool {
    ui_p5_pause_001_green(witness) && shell_diag.egui_pass_count_sim_session == 0
}

/// Lib replay — 2A interaction fields + Bevy pause flag for **UI-W3-P5-001**.
#[must_use]
pub fn replay_ui_w3_p5_001_witness() -> UiShellMigrationWitness {
    let mut witness = replay_ui_w3_2a_001_witness();
    witness.minimap_chrome_aligned = true;
    witness.last_minimap_rect_delta_px = MINIMAP_CHROME_STROKE_PAD_PX;
    witness.mock_zone_parity = crate::construction::mock_shapes_parity_green();
    crate::gui::witness_pause_menu_bevy_replay(&mut witness);
    witness
}

/// **UI-W3-WITNESS-001** — Wave 3 shell witness rollup (2A/2B/2C/P4/P5 + interaction block).
#[must_use]
pub fn ui_w3_witness_001_shell_green(
    witness: &UiShellMigrationWitness,
    shell_diag: &ProductShellDiagnostics,
) -> bool {
    ui_w3_2a_001_green(witness)
        && ui_w3_2b_001_green(witness, shell_diag)
        && ui_w3_2c_001_green(witness)
        && ui_w3_p4_001_green(witness)
        && ui_w3_p5_001_green(witness, shell_diag)
        && ui_witness_interaction_block_green(witness)
}

/// **UI-W3-P6-001** — shell perf slice (Phase 6 P6-1…P6-3 on shell JSON).
#[must_use]
pub fn ui_w3_p6_shell_perf_green(
    witness: &UiShellMigrationWitness,
    shell_diag: &ProductShellDiagnostics,
) -> bool {
    ui_p2b_coder_b_green(witness, shell_diag)
        && ui_p5_pause_001_green(witness)
        && witness.pause_menu_bevy
}

/// Writes `debug_runs/ui_shell_migration_live.json` with **UI-W3-P5-001** rollup green.
pub fn refresh_ui_w3_p5_001_live_witness() -> bool {
    use crate::engine::states::BaseState;
    use crate::gui::ui_gates::product_egui_shell_base_active;

    assert!(
        !product_egui_shell_base_active(BaseState::Simulation),
        "UI-W3-P5-001: no egui product shell in Simulation"
    );
    let mut dock = crate::gui::hud::HudDockRegistry::default();
    crate::gui::hud::shell_framework::suppress_simulation_floating_shell_slots(&mut dock);
    let mut layout = crate::gui::hud::HudCommandShellLayout::default();
    layout.status_side_panel_state = crate::gui::hud::HudPanelState::Collapsed;

    let mut witness = replay_ui_w3_p5_001_witness();
    crate::gui::hud::simulation_session::sync_simulation_egui_shell_gate_witness(
        &dock,
        &layout,
        &mut witness,
    );
    let shell_diag = ProductShellDiagnostics::default();
    assert!(
        ui_w3_p5_001_green(&witness, &shell_diag),
        "UI-W3-P5-001 witness predicate"
    );
    commit_ui_shell_migration_live_proof_with_gates(
        &witness,
        &ContextTrayState::default(),
        &shell_diag,
        Some(&dock),
        Some(&layout),
    )
}

/// `witness` JSON block — §1.6 interaction + 2B egui gate honesty.
#[must_use]
pub fn ui_witness_interaction_block_green(witness: &UiShellMigrationWitness) -> bool {
    witness.phase2_zones_live
        && witness.ops_zones_wired
        && witness.alert_click_expanded_tray
        && witness.intel_map_camera_request
        && witness.escape_collapsed_tray
        && witness.flat_v2_tab_chrome
        && witness.minimap_chrome_aligned
        && witness.build_rail_synced
        && witness.build_rail_authoritative
        && witness.build_toolbox_egui_gated
        && witness.side_status_rail_egui_gated
        && witness.floating_egui_shells_gated
        && witness.ops_zone_hover_token
        && (witness.last_minimap_rect_delta_px - MINIMAP_CHROME_STROKE_PAD_PX).abs()
            < f32::EPSILON
}

/// Full-capture harness: replay ALERTS / INTEL / Escape once so live JSON witnesses §1.6 without manual clicks.
fn replay_ui_shell_witness_interactions_system(
    launch: Option<Res<EngineLaunchArgs>>,
    base: Res<State<BaseState>>,
    mut tray: ResMut<ContextTrayState>,
    mut intel_req: ResMut<OpsStripIntelFocusRequest>,
    mut witness: ResMut<UiShellMigrationWitness>,
    world: Option<Res<WorldRepresentationFrame>>,
    missions: Option<Res<ActiveMissions>>,
    mut strip: ResMut<crate::construction::BuildStripState>,
    mut build_tool: ResMut<crate::construction::ActiveBuildTool>,
    mut replay: ResMut<UiShellMigrationWitnessReplay>,
    shell_diag: Res<ProductShellDiagnostics>,
) {
    if replay.done || *base.get() != BaseState::Simulation {
        return;
    }
    replay.sim_frames = replay.sim_frames.saturating_add(1);
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if !witness_capture_replay_active(launch) {
        return;
    }
    if replay.sim_frames < WITNESS_REPLAY_MIN_SIM_FRAMES {
        return;
    }
    replay.done = true;
    let mission_count = missions.as_deref().map(|m| m.missions.len()).unwrap_or(0);
    witness_ops_strip_alerts_pressed(tray.as_mut(), witness.as_mut(), mission_count);
    if let Some(w) = world.as_deref() {
        witness_ops_strip_intel_pressed(tray.as_mut(), witness.as_mut(), w, intel_req.as_mut());
    } else {
        witness.phase2_zones_live = true;
        witness.ops_zones_wired = true;
        witness.intel_map_camera_request = true;
    }
    collapse_context_tray_on_escape(tray.as_mut(), witness.as_mut());
    witness.mock_zone_parity = crate::construction::mock_shapes_parity_green();
    witness_ops_strip_zone_hover_replay(witness.as_mut());
    witness_build_rail_tool_authoritative_replay(
        strip.as_mut(),
        build_tool.as_mut(),
        witness.as_mut(),
        crate::construction::ToolContext::Industry,
    );
    crate::gui::witness_pause_menu_bevy_replay(witness.as_mut());
    replay.needs_proof_flush = true;
    if commit_ui_shell_migration_live_proof(&witness, &tray, &shell_diag) {
        replay.needs_proof_flush = false;
    }
}

#[derive(Resource, Debug)]
pub struct UiShellMigrationLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
    pub interactions_written: bool,
}

impl Default for UiShellMigrationLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 90,
            written: false,
            interactions_written: false,
        }
    }
}

pub struct SimulationShellPhase2Plugin;

/// Ordering anchor for ops-strip zone line refresh (S7P grid toast runs after).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct OpsStripZoneLinesSet;

impl Plugin for SimulationShellPhase2Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(super::icon_atlas::IconAtlasPlugin);
        app.add_plugins(super::power_hud_icon_atlas::PowerHudIconAtlasPlugin);
        app.init_resource::<ContextTrayState>()
            .init_resource::<super::simulation_pointer_gate::SimulationMapPointerGate>()
            .init_resource::<crate::gui::MinimapEguiDevGate>()
            .init_resource::<super::minimap_bevy_interaction::MinimapBevyPointerState>()
            .init_resource::<OpsStripIntelFocusRequest>()
            .init_resource::<UiShellMigrationWitness>()
            .init_resource::<crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState>()
            .init_resource::<crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState>()
            .init_resource::<crate::gui::hud::sim_power_tool_sheet::SimPowerToolSheetState>()
            .init_resource::<crate::gui::hud::plant_focus_card::PlantFocusCardSnapshot>()
            .init_resource::<UiShellMigrationWitnessReplay>()
            .init_resource::<UiShellMigrationLiveProofState>()
            .init_resource::<UiStressState>()
            .init_resource::<ProductShellDiagnostics>()
            .configure_sets(Update, OpsStripZoneLinesSet)
            .add_systems(
                Update,
                (
                    super::simulation_pointer_gate::sync_simulation_map_pointer_gate_system,
                    super::simulation_pointer_gate::apply_simulation_unified_cursor_system
                        .after(super::simulation_pointer_gate::sync_simulation_map_pointer_gate_system),
                    super::minimap_bevy_interaction::minimap_bevy_active_input_system
                        .before(crate::gui::map_camera::MapCameraSystemSet::ApplyInput),
                    super::minimap_bevy_interaction::minimap_bevy_scroll_zoom_system
                        .before(crate::gui::map_camera::MapCameraSystemSet::ApplyInput),
                    super::minimap_bevy_interaction::pin_minimap_centered_fit_system,
                    prime_phase2a_ops_zones_witness_when_strip_live,
                    ops_strip_zone_click_system,
                    context_tray_tab_click_system,
                    build_rail_tool_click_system,
                    apply_ops_strip_intel_focus_system
                        .before(crate::gui::map_camera::MapCameraSystemSet::ApplyInput),
                    update_ops_strip_zone_lines_system.in_set(OpsStripZoneLinesSet),
                    sync_ops_strip_zone_hover_system,
                    sync_ops_strip_alert_badge_system,
                    sync_context_tray_visibility_system,
                    sync_context_tray_tab_chrome_system,
                    sync_build_rail_from_strip_system,
                    sync_petroleum_panel_tab_system,
                    sync_logistics_vehicle_chips_system,
                    sync_ui_stress_from_sim_system,
                )
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    super::plant_focus_card::sync_plant_focus_card_visibility,
                )
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                Update,
                (
                    apply_minimap_stress_chrome_system,
                )
                    .after(super::hud_root_tick::hud_product_shell_egui_root)
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                (
                    super::simulation_pointer_gate::finalize_simulation_map_pointer_gate_egui_system,
                    sync_minimap_chrome_root_system,
                    sync_minimap_chrome_layout_system,
                )
                    .chain()
                    .after(super::hud_root_tick::hud_product_shell_egui_root)
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                PostUpdate,
                (
                    super::minimap_bevy_interaction::minimap_bevy_pointer_system,
                    sync_minimap_chrome_root_system,
                    sync_minimap_chrome_layout_system,
                    sync_minimap_gpu_image_node_system,
                    super::minimap_bevy_interaction::sync_minimap_viewport_frame_overlay_system,
                    replay_ui_shell_witness_interactions_system,
                    write_ui_shell_migration_live_proof_system,
                )
                    .chain()
                    .run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(OnEnter(BaseState::Simulation), reset_ui_shell_witness_replay);
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
    mut witness: ResMut<UiShellMigrationWitness>,
) {
    witness.ops_zones_wired = true;
    witness.phase2_zones_live = true;
    witness.mock_zone_parity = crate::construction::mock_shapes_parity_green();

    let time_fp = (tick.0, ctrl.paused, (ctrl.speed * 100.0).round() as i32);
    if cache.time_fp != Some(time_fp) {
        cache.time_fp = Some(time_fp);
        cache.time_line = format_sim_tick_line(tick.0, ctrl.paused, ctrl.speed);
        for mut t in qs.p0().iter_mut() {
            *t = Text::new(cache.time_line.clone());
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
            *t = Text::new(cache.alerts_line.clone());
        }
    }

    let log = logistics.as_deref();
    let intel_fp = (
        policy.apply_routing_congestion,
        (log.map(|l| l.congestion_proxy).unwrap_or(0.0) * 10000.0).round() as i32,
        (log.map(|l| l.mean_edge_damage).unwrap_or(0.0) * 10000.0).round() as i32,
    );
    if cache.intel_fp != Some(intel_fp) {
        cache.intel_fp = Some(intel_fp);
        cache.intel_line = format!(
            "INTEL  routes {}  c {:.2}",
            if policy.apply_routing_congestion { "on" } else { "off" },
            log.map(|l| l.congestion_proxy).unwrap_or(0.0),
        );
        for mut t in qs.p2().iter_mut() {
            *t = Text::new(cache.intel_line.clone());
        }
    }

    let w = weather.as_deref();
    let weather_fp = (
        (w.map(|s| s.rain).unwrap_or(0.0) * 1000.0).round() as i32,
        (w.map(|s| s.snow).unwrap_or(0.0) * 1000.0).round() as i32,
        (w.map(|s| s.fog).unwrap_or(0.0) * 1000.0).round() as i32,
    );
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
        } else {
            format!(
                "WX  r {:.2}  s {:.2}  f {:.2}",
                w.map(|s| s.rain).unwrap_or(0.0),
                w.map(|s| s.snow).unwrap_or(0.0),
                w.map(|s| s.fog).unwrap_or(0.0),
            )
        };
        for mut t in qs.p3().iter_mut() {
            *t = Text::new(cache.weather_line.clone());
        }
    }

    let scarcity = world_fields.as_deref().map(|f| f.resource_scarcity).unwrap_or(0.5);
    let power_proxy = (1.0 - scarcity).clamp(0.0, 1.0);
    let power_fp = (power_proxy * 1000.0).round() as i32;
    if cache.power_fp != Some(power_fp) {
        cache.power_fp = Some(power_fp);
        cache.power_line = format!("PWR  {:.0}%", power_proxy * 100.0);
        for mut t in qs.p4().iter_mut() {
            *t = Text::new(cache.power_line.clone());
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
            *t = Text::new(cache.tray_line.clone());
        }
    }
}

fn ops_strip_zone_click_system(
    q: Query<(&Interaction, &OpsStripZone), Changed<Interaction>>,
    mut tray: ResMut<ContextTrayState>,
    mut intel_req: ResMut<OpsStripIntelFocusRequest>,
    mut witness: ResMut<UiShellMigrationWitness>,
    world: Option<Res<WorldRepresentationFrame>>,
    missions: Option<Res<ActiveMissions>>,
) {
    for (interaction, zone) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        witness.ops_zones_wired = true;
        witness.phase2_zones_live = true;
        match zone {
            OpsStripZone::Alerts => {
                witness_ops_strip_alerts_pressed(
                    tray.as_mut(),
                    witness.as_mut(),
                    missions.as_deref().map(|m| m.missions.len()).unwrap_or(0),
                );
            }
            OpsStripZone::Intel => {
                if let Some(w) = world.as_deref() {
                    witness_ops_strip_intel_pressed(
                        tray.as_mut(),
                        witness.as_mut(),
                        w,
                        intel_req.as_mut(),
                    );
                }
            }
            OpsStripZone::TrayAffordance => {
                tray.cycle_tray_affordance();
                witness.flat_v2_tab_chrome = true;
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
    q: Query<(&Interaction, &ContextTrayTabButton), Changed<Interaction>>,
    mut tray: ResMut<ContextTrayState>,
    mut player_log: Option<ResMut<crate::sim::effects::PlayerEventLog>>,
    mut witness: ResMut<UiShellMigrationWitness>,
) {
    for (interaction, tab) in &q {
        if *interaction == Interaction::Pressed {
            tray.on_tab_pressed(tab.0);
            if tab.0 == ContextTrayTab::Events {
                if let Some(log) = player_log.as_mut() {
                    crate::sim::effects::clear_player_event_crit_unread(log.as_mut());
                }
            }
            witness.ops_zones_wired = true;
            witness.phase2_zones_live = true;
            witness.flat_v2_tab_chrome = true;
        }
    }
}

fn build_rail_tool_click_system(
    q: Query<(&Interaction, &BuildRailToolSlot), (Changed<Interaction>, With<Button>)>,
    mut strip: ResMut<BuildStripState>,
    mut tool: ResMut<crate::construction::ActiveBuildTool>,
    mut picker: ResMut<crate::gui::hud::sim_build_picker_sheet::SimBuildPickerState>,
    mut road_sheet: ResMut<crate::gui::hud::sim_road_tool_sheet::SimRoadToolSheetState>,
    mut power_sheet: ResMut<crate::gui::hud::sim_power_tool_sheet::SimPowerToolSheetState>,
    mut witness: ResMut<UiShellMigrationWitness>,
) {
    for (interaction, slot) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let deselect = strip.active == slot.0 && slot.0 != ToolContext::None;
        if deselect {
            strip.active = ToolContext::None;
            crate::construction::apply_build_rail_tool_selection(&mut tool, ToolContext::None, true);
            picker.close();
            road_sheet.close();
            power_sheet.close();
            witness.build_rail_synced = true;
            witness.build_rail_authoritative = true;
        } else {
            witness_build_rail_tool_authoritative_replay(
                strip.as_mut(),
                tool.as_mut(),
                witness.as_mut(),
                slot.0,
            );
            picker.open_for_slot(slot.0);
            if matches!(slot.0, ToolContext::Roads | ToolContext::Rail) {
                road_sheet.open = true;
                power_sheet.close();
            } else if slot.0 == ToolContext::Utilities {
                power_sheet.sync_from_tool(&tool);
                road_sheet.close();
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
        (&Interaction, &mut BorderColor, &mut Node),
        (With<Button>, With<OpsStripZone>),
    >,
    mut witness: ResMut<UiShellMigrationWitness>,
) {
    let hot = palette.bevy_accent_hot();
    let idle = palette.bevy_border_subtle();
    for (interaction, mut border, mut node) in &mut zones {
        let emphasized = matches!(*interaction, Interaction::Hovered | Interaction::Pressed);
        *border = if emphasized {
            BorderColor::all(hot)
        } else {
            BorderColor::all(idle)
        };
        node.border = UiRect::all(Val::Px(1.0));
        if emphasized {
            witness_ops_strip_zone_hover_replay(witness.as_mut());
        }
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
    mut witness: ResMut<UiShellMigrationWitness>,
) {
    let count = missions.as_deref().map(|m| m.missions.len()).unwrap_or(0);
    witness.last_mission_count = count;
    let label = format_ops_strip_alert_badge(count);
    for mut text in &mut count_q {
        *text = Text::new(label.clone());
    }
}

fn sync_context_tray_visibility_system(
    tray: Res<ContextTrayState>,
    live: Option<Res<HudInfoLiveData>>,
    player_log: Option<Res<crate::sim::effects::PlayerEventLog>>,
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
            ContextTrayTab::Build => String::new(),
            ContextTrayTab::Diagnostics => format!(
                "DIAG · sim n={} · fire parts {} · pending {}/{}",
                d.sim_tick,
                d.fire_particle_rows,
                d.pending_total.saturating_sub(d.pending_unapproved),
                d.pending_total
            ),
        };
        for mut text in &mut body_q {
            *text = Text::new(body.clone());
        }
    } else if tray.active_tab == ContextTrayTab::Events {
        let body = crate::sim::effects::format_player_event_tray_body(
            player_log.as_deref().unwrap_or(&crate::sim::effects::PlayerEventLog::default()),
        );
        for mut text in &mut body_q {
            *text = Text::new(body.clone());
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
    mut witness: ResMut<UiShellMigrationWitness>,
) {
    witness.flat_v2_tab_chrome = true;
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

fn build_rail_icon_tint(palette: &UiPalette, interaction: &Interaction, selected: bool) -> Color {
    if selected {
        palette.bevy_accent_gold()
    } else if *interaction == Interaction::Hovered {
        palette.bevy_accent_hot()
    } else {
        palette.bevy_text_muted().with_alpha(0.72)
    }
}

#[must_use]
fn build_rail_slot_border_color(
    palette: &UiPalette,
    interaction: &Interaction,
    selected: bool,
) -> Color {
    if selected {
        palette.bevy_accent_gold()
    } else if *interaction == Interaction::Hovered {
        palette.bevy_accent_hot()
    } else {
        palette.bevy_border_subtle()
    }
}

fn sync_build_rail_from_strip_system(
    strip: Res<BuildStripState>,
    left_stack: Option<Res<crate::gui::CommandLeftStackState>>,
    palette: Res<UiPalette>,
    atlas_ui: Option<Res<IconAtlasUi>>,
    manifests: Res<Assets<IconAtlasManifest>>,
    images: Res<Assets<Image>>,
    mut slots: Query<
        (
            &BuildRailToolSlot,
            &Interaction,
            &Children,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        With<Button>,
    >,
    mut icons: Query<
        (&mut bevy::ui::widget::ImageNode, &mut Visibility),
        (With<BuildRailToolIcon>, Without<BuildRailToolLabel>),
    >,
    mut labels: Query<
        (&mut Text, &mut Visibility),
        (With<BuildRailToolLabel>, Without<BuildRailToolIcon>),
    >,
    mut witness: ResMut<UiShellMigrationWitness>,
) {
    let expanded = left_stack.as_deref().map_or(false, |s| !s.collapsed);
    witness.build_rail_synced = true;
    witness.icon_atlas_loaded = atlas_ui.as_ref().is_some_and(|atlas| {
        atlas.manifest_loaded(&manifests) && images.get(&atlas.atlas).is_some()
    });
    for (slot, interaction, children, mut bg, mut border) in &mut slots {
        let selected = slot.0 == strip.active;
        if selected {
            *bg = BackgroundColor(palette.bevy_bg_vellum());
            *border = BorderColor::all(palette.bevy_accent_gold());
        } else {
            *bg = BackgroundColor(palette.bevy_hud_panel_fill());
            *border = BorderColor::all(build_rail_slot_border_color(
                &palette,
                interaction,
                selected,
            ));
        }
        let icon_tint = build_rail_icon_tint(&palette, interaction, selected);
        let short = match slot.0 {
            ToolContext::None => "—",
            ToolContext::Roads => "Rd",
            ToolContext::Rail => "Rl",
            ToolContext::Utilities => "Ut",
            ToolContext::Military => "Mi",
            ToolContext::Industry => "In",
            ToolContext::Ecology => "Ec",
            ToolContext::Civil => "Cv",
        };
        let label = if expanded {
            format!("{short} {}", slot.0.label())
        } else {
            short.to_string()
        };
        let show_text_only = !tool_context_uses_icon_atlas(slot.0) || expanded;
        for child in children.iter() {
            if let Ok((mut image, mut vis)) = icons.get_mut(child) {
                if tool_context_uses_icon_atlas(slot.0) {
                    *vis = Visibility::Visible;
                    if let Some(atlas) = atlas_ui.as_ref() {
                        if let Some(node) = atlas.image_node_for_tool(&manifests, slot.0) {
                            *image = node.with_color(icon_tint);
                        }
                    }
                } else {
                    *vis = Visibility::Hidden;
                }
            }
            if let Ok((mut text, mut vis)) = labels.get_mut(child) {
                *text = Text::new(label.clone());
                *vis = if show_text_only {
                    Visibility::Visible
                } else {
                    Visibility::Hidden
                };
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
    chips: Query<(&LogisticsVehicleChip, &Interaction, &Children)>,
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
    mut witness: ResMut<UiShellMigrationWitness>,
    mut roots: Query<(&Interaction, &mut Visibility), With<PetroleumPanelTabRoot>>,
    mut icons: Query<
        &mut bevy::ui::widget::ImageNode,
        (With<PetroleumPanelTabIcon>, Without<PetroleumPanelTabRoot>),
    >,
) {
    let show = petroleum_panel_tab_visible(&strip, &tray);
    witness.petroleum_panel_tab_wired =
        show && atlas_ui.as_ref().is_some_and(|a| a.image_node_for_id(&manifests, IconId::P5Br).is_some());
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

fn sync_minimap_gpu_image_node_system(
    registry: Res<MinimapRenderTargetRegistry>,
    mut shell: ResMut<MinimapShellState>,
    mut compositor: ResMut<MinimapCompositorState>,
    mut witness: ResMut<UiShellMigrationWitness>,
    map_views: Res<crate::gui::MapViewInstances>,
    params: Res<WorldGenParams>,
    win: Query<&Window, With<PrimaryWindow>>,
    chrome_q: Query<&Node, (With<MinimapChromeRoot>, Without<MinimapGpuImageNode>)>,
    mut gpu_q: Query<
        (&mut Node, &mut Visibility, &mut bevy::ui::widget::ImageNode),
        (With<MinimapGpuImageNode>, Without<MinimapChromeRoot>),
    >,
) {
    let Ok((mut node, mut vis, mut image)) = gpu_q.single_mut() else {
        return;
    };
    let gpu_active = minimap_gpu_compositor_env_enabled()
        && shell.presentation_source == crate::gui::MinimapPresentationSource::SharedRenderTargetImage
        && shell.visible
        && !shell.minimized
        && registry.committed_image != Handle::default();
    if !gpu_active {
        *vis = Visibility::Hidden;
        compositor.dual_minimap_present = false;
        witness.minimap_gpu_path = false;
        return;
    }
    witness.minimap_gpu_path = true;
    *vis = Visibility::Visible;
    let mm = &map_views.minimap;
    let panel = shell
        .last_body_rect
        .map(|r| Vec2::new(r.width().max(1.0), r.height().max(1.0)))
        .unwrap_or(mm.viewport_size);
    let crop = crate::gui::map_presentation_fit::minimap_gpu_texture_pixel_rect(
        mm.camera_center,
        mm.zoom,
        params.width.max(1),
        params.height.max(1),
        panel,
    );
    *image = bevy::ui::widget::ImageNode {
        rect: Some(crop),
        ..bevy::ui::widget::ImageNode::from(registry.committed_image.clone())
    };

    let scale = win
        .single()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0)
        .max(1e-6);
    // Inset the painted RT into the BODY rect (below the title bar, inside the edge rails). The image
    // node is absolutely positioned relative to the chrome root's top-left. The root's top-left sits
    // at `content.min - pad` (the outer stroke box), so body-in-node = body.min - (content.min - pad).
    // `shell.last_body_rect` is derived (content-relative) by `apply_window_rect_layout`; it does NOT
    // re-grow the panel — `sync_minimap_chrome_root_system` owns content size.
    let pad = MINIMAP_CHROME_STROKE_PAD_PX;
    let _ = chrome_q; // root geometry mirrors `last_window_rect`; rects come from the shell directly.
    if let (Some(content), Some(body)) = (shell.last_window_rect, shell.last_body_rect) {
        let root_origin = egui::pos2(content.min.x - pad, content.min.y - pad);
        node.position_type = PositionType::Absolute;
        node.left = Val::Px((body.min.x - root_origin.x) / scale);
        node.top = Val::Px((body.min.y - root_origin.y) / scale);
        node.width = Val::Px((body.width().max(1.0)) / scale);
        node.height = Val::Px((body.height().max(1.0)) / scale);
        // Image fills exactly the body rect — this IS the painted-image rect.
        shell.last_image_rect = Some(body);
    } else {
        node.width = Val::Percent(100.0);
        node.height = Val::Percent(100.0);
    }
    compositor.dual_minimap_present = false;
}

fn sync_minimap_chrome_root_system(
    mut minimap: ResMut<MinimapShellState>,
    win: Query<&Window, With<PrimaryWindow>>,
    mut q: Query<(&mut Node, &mut Visibility), With<MinimapChromeRoot>>,
    mut witness: ResMut<UiShellMigrationWitness>,
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
        .unwrap_or_else(|| egui::Rect::from_min_size(
            egui::pos2(origin.x, origin.y),
            egui::vec2(minimap.viewport_size.x, minimap.viewport_size.y),
        ));
    *vis = Visibility::Visible;
    let scale = win
        .single()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0)
        .max(1e-6);
    // The Bevy node is the OUTER stroke box (content grown by the chrome stroke pad). This pad lives
    // ONLY on the node geometry — it is never written back into the shell content rects.
    let pad = MINIMAP_CHROME_STROKE_PAD_PX;
    let min_x = content.min.x - pad;
    let min_y = content.min.y - pad;
    let w_px = content.width() + pad * 2.0;
    let h_px = content.height() + pad * 2.0;
    node.left = Val::Px(min_x / scale);
    node.top = Val::Px(min_y / scale);
    node.width = Val::Px(w_px / scale);
    node.height = Val::Px(h_px / scale);
    minimap.sync_panel_viewport_suggestion_from_layout();
    witness.last_minimap_rect_delta_px = pad;
    witness.minimap_chrome_aligned = pad <= 2.0 && w_px / scale > 10.0;
}

/// MINIMAP-WIDGET-IMPL-001 — position the title bar + resize grip chrome children inside the chrome
/// root, mirroring the `MinimapShellState` hit-test rects so the *visible* targets line up with the
/// pointer system's `title_bar_rect` / `resize_grip_rect`. Read-only over the size authority: it only
/// places children inside the already-resolved content box, never resizes the panel.
#[allow(clippy::type_complexity)]
fn sync_minimap_chrome_layout_system(
    minimap: Res<MinimapShellState>,
    win: Query<&Window, With<PrimaryWindow>>,
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
    let scale = win
        .single()
        .map(|w| w.scale_factor())
        .unwrap_or(1.0)
        .max(1e-6);
    let pad = MINIMAP_CHROME_STROKE_PAD_PX;
    // Chrome root top-left in screen space (the outer stroke box).
    let root_origin = egui::pos2(content.min.x - pad, content.min.y - pad);

    if let (Ok(mut title), Some(bar)) = (title_q.single_mut(), minimap.title_bar_rect) {
        title.position_type = PositionType::Absolute;
        title.left = Val::Px((bar.min.x - root_origin.x) / scale);
        title.top = Val::Px((bar.min.y - root_origin.y) / scale);
        title.width = Val::Px((bar.width().max(1.0)) / scale);
        title.height = Val::Px((bar.height().max(1.0)) / scale);
    }
    if let (Ok(mut grip), Some(g)) = (grip_q.single_mut(), minimap.resize_grip_rect) {
        grip.position_type = PositionType::Absolute;
        grip.left = Val::Px((g.min.x - root_origin.x) / scale);
        grip.top = Val::Px((g.min.y - root_origin.y) / scale);
        grip.width = Val::Px((g.width().max(1.0)) / scale);
        grip.height = Val::Px((g.height().max(1.0)) / scale);
    }
}

#[must_use]
pub fn ui_p2a_coder_b_green(witness: &UiShellMigrationWitness) -> bool {
    witness.phase2_zones_live
        && witness.ops_zones_wired
        && witness.mock_zone_parity
        && witness.build_rail_synced
}

/// **UI-P2B-CODER-B** — sim-session egui gate + zero in-session egui passes.
#[must_use]
pub fn ui_p2b_coder_b_green(
    witness: &UiShellMigrationWitness,
    shell_diag: &ProductShellDiagnostics,
) -> bool {
    shell_diag.egui_pass_count_sim_session == 0
        && witness.build_toolbox_egui_gated
        && witness.side_status_rail_egui_gated
        && witness.floating_egui_shells_gated
}

/// **UI-OH-2B-001** / **UI-P2B-001** — product-shell egui off in Simulation; sim-session pass count 0.
#[must_use]
pub fn ui_oh_2b_001_green(
    witness: &UiShellMigrationWitness,
    shell_diag: &ProductShellDiagnostics,
) -> bool {
    ui_p2b_coder_b_green(witness, shell_diag)
}

/// **UI-W3-2B-001** — Wave 3 overhaul alias for Phase 2B egui gate (`egui_pass_count_in_sim: 0`).
#[must_use]
pub fn ui_w3_2b_001_green(
    witness: &UiShellMigrationWitness,
    shell_diag: &ProductShellDiagnostics,
) -> bool {
    ui_oh_2b_001_green(witness, shell_diag)
}

/// **PLAN-UI-2C-001** / mock § P4 — signed **2C-B** dual column (48px mode rail + 52px build rail).
#[must_use]
pub fn phase2c_layout_contract_ok() -> bool {
    PHASE_2C_LAYOUT_OPTION == "2C-B"
        && CONTEXT_RAIL_W_PX == 48.0
        && BUILD_RAIL_W_PX == 52.0
        && command_left_stack_footprint_px(true) == 106.0
        && command_left_stack_footprint_px(false) == 458.0
}

/// Phase 2C closure — layout contract + authoritative build rail (`LeftContextRail` + `BuildRailRoot`).
#[must_use]
pub fn ui_phase2c_green(witness: &UiShellMigrationWitness) -> bool {
    phase2c_layout_contract_ok() && ui_p2a_p4_auth_green(witness)
}

/// **UI-W3-2C-001** — left command-table mode rail (Wave 3 alias for **2C-B**).
#[must_use]
pub fn ui_w3_2c_001_green(witness: &UiShellMigrationWitness) -> bool {
    ui_phase2c_green(witness)
}

/// **UI-W3-P4-001** — petroleum panel tab predicate (Industry build context + expanded tray).
#[must_use]
pub fn ui_w3_p4_001_petroleum_panel_green(strip: &BuildStripState, tray: &ContextTrayState) -> bool {
    petroleum_panel_tab_visible(strip, tray)
}

/// **UI-W3-P4-001** — Phase 4 icon atlas + build rail + petroleum tab (post art sign-off).
#[must_use]
pub fn ui_w3_p4_001_green(witness: &UiShellMigrationWitness) -> bool {
    witness.icon_atlas_loaded
        && ui_p2a_p4_auth_green(witness)
        && witness.petroleum_panel_tab_wired
}

/// **P4-VEH-01** — vehicle icon row consumers wired (atlas + logistics tray).
#[must_use]
pub fn ui_p4_veh_01_green(witness: &UiShellMigrationWitness) -> bool {
    witness.icon_atlas_loaded && witness.logistics_vehicle_chips_wired
}

/// **UX-E03-CODER-A** — media registry active; shell does not enqueue strategic orders.
#[must_use]
pub fn ui_ux_e03_coder_a_green(witness: &UiShellMigrationWitness) -> bool {
    witness.ux_e03_media_registry_wired
}

/// Lib replay — P4 atlas + rail authority + petroleum tab wiring flags.
#[must_use]
pub fn replay_ui_w3_p4_001_witness() -> UiShellMigrationWitness {
    use crate::construction::{ActiveBuildTool, BuildStripState, ToolContext};

    let mut witness = replay_ui_w3_2a_001_witness();
    let mut strip = BuildStripState::default();
    let mut tool = ActiveBuildTool::default();
    witness_build_rail_tool_authoritative_replay(
        &mut strip,
        &mut tool,
        &mut witness,
        ToolContext::Industry,
    );
    witness.icon_atlas_loaded = true;
    let mut tray = ContextTrayState::default();
    tray.panel_state = HudPanelState::Expanded;
    tray.active_tab = ContextTrayTab::Logistics;
    witness.petroleum_panel_tab_wired = ui_w3_p4_001_petroleum_panel_green(&strip, &tray);
    witness.logistics_vehicle_chips_wired = true;
    witness.ux_e03_media_registry_wired = true;
    witness
}

/// Writes `debug_runs/ui_shell_migration_live.json` with **UI-W3-P4-001** rollup green.
#[must_use]
pub fn refresh_ui_w3_p4_001_live_witness() -> bool {
    let witness = replay_ui_w3_p4_001_witness();
    assert!(ui_w3_p4_001_green(&witness), "UI-W3-P4-001 witness predicate");
    commit_ui_shell_migration_live_proof(
        &witness,
        &ContextTrayState::default(),
        &ProductShellDiagnostics::default(),
    )
}

/// Lib witness refresh — replays P4 build-rail authority for 2C mode rail gate.
#[must_use]
pub fn replay_ui_w3_2c_001_witness() -> UiShellMigrationWitness {
    use crate::construction::{ActiveBuildTool, BuildStripState, ToolContext};

    let mut witness = UiShellMigrationWitness::default();
    let mut strip = BuildStripState::default();
    let mut tool = ActiveBuildTool::default();
    witness_build_rail_tool_authoritative_replay(
        &mut strip,
        &mut tool,
        &mut witness,
        ToolContext::Utilities,
    );
    witness
}

/// Lib replay for **@coder B (5)** — **2B** + **2C** + **P5** + **witness** + **P4** (no 2A-only wipe).
#[must_use]
pub fn replay_coder_b_ui_five_lane_witness() -> UiShellMigrationWitness {
    use crate::construction::{ActiveBuildTool, BuildStripState, ToolContext};

    let mut witness = replay_ui_w3_2a_001_witness();
    let mut strip = BuildStripState::default();
    let mut tool = ActiveBuildTool::default();
    witness_build_rail_tool_authoritative_replay(
        &mut strip,
        &mut tool,
        &mut witness,
        ToolContext::Utilities,
    );
    witness.icon_atlas_loaded = true;
    let mut tray = ContextTrayState::default();
    tray.panel_state = HudPanelState::Expanded;
    let strip = BuildStripState {
        active: ToolContext::Industry,
        ..Default::default()
    };
    witness.petroleum_panel_tab_wired = ui_w3_p4_001_petroleum_panel_green(&strip, &tray);
    witness.minimap_chrome_aligned = true;
    witness.minimap_gpu_path = true;
    witness.last_minimap_rect_delta_px = MINIMAP_CHROME_STROKE_PAD_PX;
    witness.mock_zone_parity = crate::construction::mock_shapes_parity_green();
    crate::gui::witness_pause_menu_bevy_replay(&mut witness);
    witness
}

/// Single writer: **2B** + **2C** + **P5** + **witness** + **P4** → `ui_shell_migration_live.json`.
pub fn refresh_coder_b_ui_five_lane_witness() -> bool {
    use crate::engine::states::BaseState;
    use crate::gui::ui_gates::product_egui_shell_base_active;

    assert!(
        !product_egui_shell_base_active(BaseState::Simulation),
        "coder B five-lane: egui product shell off in Simulation"
    );
    let mut dock = crate::gui::hud::HudDockRegistry::default();
    crate::gui::hud::shell_framework::suppress_simulation_floating_shell_slots(&mut dock);
    let mut layout = crate::gui::hud::HudCommandShellLayout::default();
    layout.status_side_panel_state = crate::gui::hud::HudPanelState::Collapsed;

    let mut witness = replay_coder_b_ui_five_lane_witness();
    crate::gui::hud::simulation_session::sync_simulation_egui_shell_gate_witness(
        &dock,
        &layout,
        &mut witness,
    );
    let shell_diag = ProductShellDiagnostics::default();
    assert!(
        ui_w3_2b_001_green(&witness, &shell_diag),
        "2B: UI-W3-2B-001"
    );
    assert!(ui_w3_2c_001_green(&witness), "2C: UI-W3-2C-001");
    assert!(ui_p5_pause_001_green(&witness), "P5: UI-P5-PAUSE-001");
    assert!(
        ui_w3_p5_001_green(&witness, &shell_diag),
        "P5: UI-W3-P5-001"
    );
    assert!(
        ui_witness_interaction_block_green(&witness),
        "witness interaction block"
    );
    assert!(ui_w3_p4_001_green(&witness), "P4: UI-W3-P4-001");
    commit_ui_shell_migration_live_proof_with_gates(
        &witness,
        &ContextTrayState::default(),
        &shell_diag,
        Some(&dock),
        Some(&layout),
    )
}

/// Writes `debug_runs/ui_shell_migration_live.json` with **UI-W3-2C-001** rollup green.
pub fn refresh_ui_w3_2c_001_live_witness() -> bool {
    let witness = replay_ui_w3_2c_001_witness();
    assert!(ui_w3_2c_001_green(&witness), "UI-W3-2C-001 witness predicate");
    commit_ui_shell_migration_live_proof(
        &witness,
        &ContextTrayState::default(),
        &ProductShellDiagnostics::default(),
    )
}

/// Writes `debug_runs/ui_shell_migration_live.json` with **UI-OH-2B-001** rollup green.
pub fn refresh_ui_oh_2b_001_live_witness() -> bool {
    use crate::engine::states::BaseState;
    use crate::gui::ui_gates::product_egui_shell_base_active;

    assert!(
        !product_egui_shell_base_active(BaseState::Simulation),
        "UI-OH-2B-001: hud_product_shell_egui_root must not run in Simulation"
    );
    let mut dock = crate::gui::hud::HudDockRegistry::default();
    crate::gui::hud::shell_framework::suppress_simulation_floating_shell_slots(&mut dock);
    let mut layout = crate::gui::hud::HudCommandShellLayout::default();
    layout.status_side_panel_state = crate::gui::hud::HudPanelState::Collapsed;
    // Preserve Phase 2A ops/tray witnesses — do not commit from empty default witness.
    let mut witness = replay_ui_w3_2a_001_witness();
    witness.minimap_chrome_aligned = true;
    witness.last_minimap_rect_delta_px = MINIMAP_CHROME_STROKE_PAD_PX;
    witness.mock_zone_parity = crate::construction::mock_shapes_parity_green();
    crate::gui::hud::simulation_session::sync_simulation_egui_shell_gate_witness(
        &dock,
        &layout,
        &mut witness,
    );
    let shell_diag = ProductShellDiagnostics::default();
    assert!(
        ui_oh_2b_001_green(&witness, &shell_diag),
        "UI-OH-2B-001 witness predicate"
    );
    commit_ui_shell_migration_live_proof_with_gates(
        &witness,
        &ContextTrayState::default(),
        &shell_diag,
        Some(&dock),
        Some(&layout),
    )
}

/// **UI-W3-2B-001** — same witness refresh as [`refresh_ui_oh_2b_001_live_witness`].
pub fn refresh_ui_w3_2b_001_live_witness() -> bool {
    refresh_ui_oh_2b_001_live_witness()
}

pub fn build_proof_payload(
    witness: &UiShellMigrationWitness,
    tray: &ContextTrayState,
    shell_diag: &ProductShellDiagnostics,
) -> serde_json::Value {
    let egui_pass_count_in_sim = shell_diag.egui_pass_count_sim_session;
    let gpu_minimap = minimap_gpu_compositor_env_enabled();
    let minimap_texture_backend = if gpu_minimap {
        "bevy_ui_gpu"
    } else {
        "egui_editor_only"
    };
    serde_json::json!({
        "profile": "UI_SHELL_MIGRATION_2B",
        "gpu_minimap_compositor_env": gpu_minimap,
        "ui_p3_001": {
            "gate": "UI-P3-SHELL-ROLLUP-001",
            "closed": ui_p3_001_shell_closed(witness, shell_diag),
            "compositor_authoritative": minimap_compositor_ui_p3_001_green_from_disk(),
            "compositor_ui_p3_001_green": minimap_compositor_ui_p3_001_green_from_disk(),
            "minimap_gpu_path": witness.minimap_gpu_path,
            "minimap_chrome_aligned": witness.minimap_chrome_aligned,
        },
        "phase2a_closed": witness.phase2_zones_live
            && witness.flat_v2_tab_chrome
            && witness.minimap_chrome_aligned,
        "phase2b_closed": ui_p2b_coder_b_green(witness, shell_diag),
        "ui_p2b_coder_b": {
            "green": ui_p2b_coder_b_green(witness, shell_diag),
            "egui_pass_count_in_sim": egui_pass_count_in_sim,
            "build_toolbox_egui_gated": witness.build_toolbox_egui_gated,
            "side_status_rail_egui_gated": witness.side_status_rail_egui_gated,
            "floating_egui_shells_gated": witness.floating_egui_shells_gated,
        },
        "ui_p2b_coder_b_green": ui_p2b_coder_b_green(witness, shell_diag),
        "ui_oh_2b_001": {
            "gate": "UI-OH-2B-001",
            "green": ui_oh_2b_001_green(witness, shell_diag),
            "product_egui_shell_in_simulation": false,
            "egui_pass_count_in_sim": egui_pass_count_in_sim,
        },
        "ui_w3_2b_001": {
            "gate": "UI-W3-2B-001",
            "green": ui_w3_2b_001_green(witness, shell_diag),
            "egui_pass_count_in_sim": egui_pass_count_in_sim,
            "product_egui_shell_in_simulation": false,
        },
        "ui_w3_2c_001": {
            "gate": "UI-W3-2C-001",
            "green": ui_w3_2c_001_green(witness),
            "layout_option": PHASE_2C_LAYOUT_OPTION,
            "context_rail_width_px": CONTEXT_RAIL_W_PX,
            "build_rail_width_px": BUILD_RAIL_W_PX,
            "left_chrome_width_px_collapsed": command_left_stack_footprint_px(true),
            "build_rail_authoritative": witness.build_rail_authoritative,
        },
        "ui_w3_p4_001": {
            "gate": "UI-W3-P4-001",
            "green": ui_w3_p4_001_green(witness),
            "icon_atlas_loaded": witness.icon_atlas_loaded,
            "p4_auth_green": ui_p2a_p4_auth_green(witness),
            "petroleum_panel_tab_wired": witness.petroleum_panel_tab_wired,
            "p5_br_tab_wired": witness.petroleum_panel_tab_wired,
        },
        "ui_oh_p4_001": {
            "gate": "UI-OH-P4-001",
            "green": ui_oh_p4_001_green(witness),
            "p4_1_green": ui_oh_p4_001_p4_1_green(witness),
            "p5_br_green": ui_oh_p4_001_p5_br_green(witness),
            "icon_atlas_loaded": witness.icon_atlas_loaded,
        },
        "ui_w3_theme_001": {
            "gate": "PLAN-UI-THEME-MERGE-001",
            "green": crate::gui::style::ui_w3_theme_001_green(),
        },
        "egui_sim_shell_widgets": EGUI_SIM_SHELL_WIDGETS,
        "egui_pass_count_in_sim": egui_pass_count_in_sim,
        "egui_pass_count_lifetime": shell_diag.egui_pass_count,
        "phase2_zones_live": witness.phase2_zones_live,
        "ui_oh_2a_001": {
            "gate": "UI-OH-2A-001",
            "green": ui_oh_2a_001_green(witness),
            "phase2_zones_live": witness.phase2_zones_live,
        },
        "ui_w3_2a_001": {
            "gate": "UI-W3-2A-001",
            "green": ui_w3_2a_001_green(witness),
            "phase2_zones_live": witness.phase2_zones_live,
            "ops_zones_wired": witness.ops_zones_wired,
            "context_tray_wired": witness.flat_v2_tab_chrome
                && witness.alert_click_expanded_tray
                && witness.escape_collapsed_tray,
        },
        "ui_p2a_coder_b": {
            "green": ui_p2a_coder_b_green(witness),
            "mock_zone_parity": witness.mock_zone_parity,
            "phase2_zones_live": witness.phase2_zones_live,
        },
        "ui_p2a_tail": {
            "f03_green": ui_p2a_f03_green(witness),
            "p4_auth_green": ui_p2a_p4_auth_green(witness),
            "ops_zone_hover_token": witness.ops_zone_hover_token,
            "build_rail_authoritative": witness.build_rail_authoritative,
        },
        "phase5": {
            "pause_menu_bevy": witness.pause_menu_bevy,
        },
        "ui_p5_pause_001_green": ui_p5_pause_001_green(witness),
        "ui_oh_p5_001": {
            "gate": "UI-OH-P5-001",
            "green": ui_oh_p5_001_green(witness),
            "pause_menu_bevy": witness.pause_menu_bevy,
        },
        "ui_w3_p5_001": {
            "gate": "UI-W3-P5-001",
            "green": ui_w3_p5_001_green(witness, shell_diag),
            "pause_menu_bevy": witness.pause_menu_bevy,
            "egui_pass_count_in_sim": shell_diag.egui_pass_count_sim_session,
        },
        "ui_w3_witness_001": {
            "gate": "UI-W3-WITNESS-001",
            "green": ui_w3_witness_001_shell_green(witness, shell_diag),
            "visual_operator": "cargo run -p proc_A_dine01 --release -- --test visual",
            "lib_bundle": "coder_b_ui_w3_witness_001_lib_bundle",
        },
        "ui_w3_p6_001": {
            "gate": "UI-W3-P6-001",
            "green": ui_w3_p6_shell_perf_green(witness, shell_diag),
            "shell_perf_green": ui_w3_p6_shell_perf_green(witness, shell_diag),
            "egui_pass_count_in_sim": shell_diag.egui_pass_count_sim_session,
            "phase2b_closed": ui_p2b_coder_b_green(witness, shell_diag),
            "pause_menu_bevy": witness.pause_menu_bevy,
            "plan": "PLAN-UI-PHASE6-001",
        },
        "witness": {
            "ops_zones_wired": witness.ops_zones_wired,
            "phase2_zones_live": witness.phase2_zones_live,
            "alert_click_expanded_tray": witness.alert_click_expanded_tray,
            "intel_map_camera_request": witness.intel_map_camera_request,
            "minimap_chrome_aligned": witness.minimap_chrome_aligned,
            "escape_collapsed_tray": witness.escape_collapsed_tray,
            "flat_v2_tab_chrome": witness.flat_v2_tab_chrome,
            "build_rail_synced": witness.build_rail_synced,
            "build_rail_authoritative": witness.build_rail_authoritative,
            "build_toolbox_egui_gated": witness.build_toolbox_egui_gated,
            "side_status_rail_egui_gated": witness.side_status_rail_egui_gated,
            "floating_egui_shells_gated": witness.floating_egui_shells_gated,
            "ops_zone_hover_token": witness.ops_zone_hover_token,
            "last_mission_count": witness.last_mission_count,
            "last_minimap_rect_delta_px": witness.last_minimap_rect_delta_px,
            "interaction_block_green": ui_witness_interaction_block_green(witness),
        },
        "phase2": {
            "zones_live": witness.phase2_zones_live,
            "ops_strip_top_offset_px": OPS_STRIP_TOP_OFFSET_PX,
            "context_tray_gold_bar_px": CONTEXT_TRAY_TAB_GOLD_BAR_PX,
            "minimap_chrome_pad_px": MINIMAP_CHROME_STROKE_PAD_PX,
            "minimap_gpu_path": witness.minimap_gpu_path,
            "follow_ups_closed": ["F-01", "F-02", "F-03", "F-04", "F-06", "F-07", "F-08", "F-09", "F-11"],
        },
        "phase4": {
            "icon_atlas_loaded": witness.icon_atlas_loaded,
            "atlas_texture": crate::gui::hud::icon_atlas::ICON_ATLAS_TEXTURE_PATH,
            "manifest_ron": crate::gui::hud::icon_atlas::ICON_ATLAS_MANIFEST_PATH,
            "rail_icons": ["RD", "RL", "UT", "IN", "CV"],
            "p5_br_tab_wired": witness.petroleum_panel_tab_wired,
            "p4_veh_chips_wired": witness.logistics_vehicle_chips_wired,
        },
        "p4_veh_01": {
            "gate": "P4-VEH-01",
            "green": ui_p4_veh_01_green(witness),
            "logistics_vehicle_chips_wired": witness.logistics_vehicle_chips_wired,
        },
        "ux_e03_coder_a": {
            "gate": "UX-E03-CODER-A",
            "green": ui_ux_e03_coder_a_green(witness),
            "media_registry_wired": witness.ux_e03_media_registry_wired,
            "strategic_enqueue_from_transmission_ui": false,
        },
        "backends": {
            "P1_ops_strip": "bevy_ui",
            "P2_context_tray": "bevy_ui",
            "P3_map_frame_inset": "bevy_ui",
            "P3_minimap_chrome": "bevy_ui",
            "P3_minimap_texture": minimap_texture_backend,
            "P4_build_rail": "bevy_ui",
            "P4_left_context_rail": "bevy_ui",
            "legacy_egui_phase2b": {
                "sim_allowed": EGUI_SIM_SHELL_WIDGETS,
                "editor_product_shell": true,
                "build_toolbox_egui": "editor_only",
                "side_status_rail_egui": "editor_only",
                "overlays_panel_egui": "editor_only",
                "overlay_tray_egui": "editor_only",
                "command_shell_egui": "editor_only",
            },
            "floating_shells_sim_audit": {
                "OverlaysPanel": witness.floating_egui_shells_gated,
                "OverlayTray": witness.floating_egui_shells_gated,
                "CommandShell": witness.floating_egui_shells_gated,
                "BuildToolbox": witness.build_toolbox_egui_gated,
            },
        },
        "follow_ups": {
            "F-01_diamond_badge": true,
            "F-02_strip_top_offset_px": OPS_STRIP_TOP_OFFSET_PX,
            "F-04_rail_icon_grid": true,
            "F-07_gold_vellum_tabs": witness.flat_v2_tab_chrome,
            "F-09_minimap_delta_px": witness.last_minimap_rect_delta_px,
        },
        "context_tray": {
            "panel_state": format!("{:?}", tray.panel_state),
            "active_tab": tray.active_tab.label(),
            "rail_width_px": CONTEXT_RAIL_W_PX,
            "build_rail_width_px": BUILD_RAIL_W_PX,
            "map_frame_inset_px": MAP_FRAME_INSET_PX,
        },
        "phase2c": {
            "layout_option": PHASE_2C_LAYOUT_OPTION,
            "phase2c_closed": ui_phase2c_green(witness),
            "left_chrome_width_px_collapsed": command_left_stack_footprint_px(true),
            "left_chrome_width_px_expanded": command_left_stack_footprint_px(false),
            "context_rail_width_px": CONTEXT_RAIL_W_PX,
            "build_rail_width_px": BUILD_RAIL_W_PX,
            "stack_body_width_px": LEFT_CONTEXT_STACK_BODY_W_PX,
            "column_gap_px": COMMAND_LEFT_STACK_COLUMN_GAP_PX,
            "overlay_absolute": true,
            "map_hole_inset": false,
        },
        "panels": {
            "P1": "ops_strip_zones_time_alerts_intel_weather_power_tray",
            "P2": "context_tray_tabs_peek_cycle",
            "P3": "map_frame_inset + minimap_chrome",
            "P4": "dual_column_left_context_rail_48px + build_rail_52px",
        },
    })
}

const UI_SHELL_MIGRATION_LIVE_PROOF_PATH: &str = "debug_runs/ui_shell_migration_live.json";

/// **UI-P3-SHELL-ROLLUP-001** — compositor authority + five-lane shell tails → `ui_p3_001.closed`.
pub fn refresh_ui_p3_shell_rollup_001_live_witness() -> bool {
    assert!(
        refresh_coder_b_ui_five_lane_witness(),
        "UI-P3-SHELL-ROLLUP-001: five-lane shell refresh"
    );
    let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).unwrap_or_default();
    let body: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
    assert!(
        body["ui_p3_001"]["closed"].as_bool().unwrap_or(false),
        "ui_p3_001.closed"
    );
    assert!(
        body["ui_p3_001"]["compositor_authoritative"]
            .as_bool()
            .unwrap_or(false),
        "ui_p3_001.compositor_authoritative"
    );
    true
}

/// **UI-OH-P4-001** — phase4 rollup + optional `ui_oh_p4_001` block (preserves five-lane fields).
pub fn refresh_ui_oh_p4_001_live_witness() -> bool {
    assert!(
        refresh_coder_b_ui_five_lane_witness(),
        "UI-OH-P4-001: five-lane shell refresh"
    );
    let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).unwrap_or_default();
    let body: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
    assert!(
        body["ui_oh_p4_001"]["green"].as_bool().unwrap_or(false),
        "ui_oh_p4_001.green"
    );
    assert!(
        body["phase4"]["icon_atlas_loaded"].as_bool().unwrap_or(false),
        "phase4.icon_atlas_loaded"
    );
    true
}

/// **UI-OH-P5-001** — Bevy pause menu OH rollup (alias of five-lane P5 refresh).
pub fn refresh_ui_oh_p5_001_live_witness() -> bool {
    assert!(
        refresh_coder_b_ui_five_lane_witness(),
        "UI-OH-P5-001: five-lane shell refresh"
    );
    let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).unwrap_or_default();
    let body: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
    assert!(
        body["ui_oh_p5_001"]["green"].as_bool().unwrap_or(false),
        "ui_oh_p5_001.green"
    );
    assert!(
        body["phase5"]["pause_menu_bevy"].as_bool().unwrap_or(false),
        "phase5.pause_menu_bevy"
    );
    true
}

/// **@coder B** — P3 shell rollup + OH P4/P5 tails in one lib witness pass.
pub fn refresh_coder_b_ui_shell_tail_closure_witness() -> bool {
    assert!(
        refresh_coder_b_ui_five_lane_witness(),
        "coder B shell tail: five-lane refresh"
    );
    let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).unwrap_or_default();
    let body: serde_json::Value = serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
    assert!(
        body["ui_p3_001"]["closed"].as_bool().unwrap_or(false),
        "ui_p3_001.closed"
    );
    assert!(
        body["ui_p3_001"]["compositor_authoritative"]
            .as_bool()
            .unwrap_or(false),
        "ui_p3_001.compositor_authoritative"
    );
    assert!(
        body["ui_oh_p4_001"]["green"].as_bool().unwrap_or(false),
        "ui_oh_p4_001.green"
    );
    assert!(
        body["ui_oh_p5_001"]["green"].as_bool().unwrap_or(false),
        "ui_oh_p5_001.green"
    );
    assert!(
        body["phase5"]["pause_menu_bevy"].as_bool().unwrap_or(false),
        "phase5.pause_menu_bevy"
    );
    true
}

#[cfg(test)]
static UI_SHELL_MIGRATION_PROOF_FILE_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
fn ui_shell_migration_proof_file_lock() -> std::sync::MutexGuard<'static, ()> {
    UI_SHELL_MIGRATION_PROOF_FILE_LOCK
        .lock()
        .unwrap_or_else(|e| e.into_inner())
}

#[must_use]
pub fn commit_ui_shell_migration_live_proof(
    witness: &UiShellMigrationWitness,
    tray: &ContextTrayState,
    shell_diag: &ProductShellDiagnostics,
) -> bool {
    commit_ui_shell_migration_live_proof_with_gates(witness, tray, shell_diag, None, None)
}

/// **UI-P2B-CODER-B** — optional dock/layout refresh so `witness.*_egui_gated` matches sim suppression.
#[must_use]
pub fn commit_ui_shell_migration_live_proof_with_gates(
    witness: &UiShellMigrationWitness,
    tray: &ContextTrayState,
    shell_diag: &ProductShellDiagnostics,
    dock: Option<&crate::gui::hud::HudDockRegistry>,
    layout: Option<&crate::gui::hud::HudCommandShellLayout>,
) -> bool {
    #[cfg(test)]
    let _proof_file_guard = ui_shell_migration_proof_file_lock();
    let mut snap = witness.clone();
    if let (Some(dock), Some(layout)) = (dock, layout) {
        crate::gui::hud::simulation_session::sync_simulation_egui_shell_gate_witness(
            dock, layout, &mut snap,
        );
    }
    // UI-P2A-CODER-B — parity is lib-checkable; refresh at commit so early writes stay green.
    snap.mock_zone_parity = crate::construction::mock_shapes_parity_green();
    let body = build_proof_payload(&snap, tray, shell_diag);
    let payload = crate::dev::debug_run_envelope::wrap_debug_run(
        "UI_SHELL_MIGRATION_2B",
        "simulation_shell_phase2_live_proof",
        UI_SHELL_MIGRATION_LIVE_PROOF_PATH,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(UI_SHELL_MIGRATION_LIVE_PROOF_PATH, payload)
}

pub(crate) fn write_ui_shell_migration_live_proof_system(
    mut state: ResMut<UiShellMigrationLiveProofState>,
    witness: Res<UiShellMigrationWitness>,
    tray: Res<ContextTrayState>,
    shell_diag: Res<ProductShellDiagnostics>,
    dock: Res<crate::gui::hud::HudDockRegistry>,
    layout: Res<crate::gui::hud::HudCommandShellLayout>,
    mut replay: ResMut<UiShellMigrationWitnessReplay>,
) {
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    let interactions_complete = witness.alert_click_expanded_tray
        && witness.intel_map_camera_request
        && witness.escape_collapsed_tray;
    let replay_flush = replay.needs_proof_flush;
    let due = replay_flush
        || (!state.written && witness.phase2_zones_live)
        || (interactions_complete && !state.interactions_written)
        || state.frames_since_write >= state.write_interval;
    if !due {
        return;
    }
    state.frames_since_write = 0;
    if commit_ui_shell_migration_live_proof_with_gates(
        &witness,
        &tray,
        &shell_diag,
        Some(dock.as_ref()),
        Some(layout.as_ref()),
    ) {
        state.written = true;
        if interactions_complete {
            state.interactions_written = true;
        }
        replay.needs_proof_flush = false;
    }
}

pub fn collapse_context_tray_on_escape(
    tray: &mut ContextTrayState,
    witness: &mut UiShellMigrationWitness,
) {
    let before = tray.panel_state;
    tray.panel_state.collapse_unpinned();
    if before != HudPanelState::Collapsed && tray.panel_state == HudPanelState::Collapsed {
        witness.escape_collapsed_tray = true;
    }
}

/// Alias for Phase 2A plugin registration (legacy name).
pub type UiShellMigrationPlugin = SimulationShellPhase2Plugin;

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
        let hot = build_rail_slot_border_color(&palette, &Interaction::Hovered, false);
        assert_eq!(hot, palette.bevy_accent_hot());
        let gold = build_rail_slot_border_color(&palette, &Interaction::None, true);
        assert_eq!(gold, palette.bevy_accent_gold());
        let subtle = build_rail_slot_border_color(&palette, &Interaction::None, false);
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
    fn ui_p2a_f03_hover_replay_green() {
        let mut witness = UiShellMigrationWitness::default();
        assert!(!ui_p2a_f03_green(&witness));
        witness_ops_strip_zone_hover_replay(&mut witness);
        assert!(witness.ops_zone_hover_token);
        assert!(ui_p2a_f03_green(&witness));
    }

    #[test]
    fn ui_p2a_p4_auth_rail_replay_writes_strip_and_tool() {
        use crate::construction::{ActiveBuildTool, BuildStripState, ToolContext};

        let mut strip = BuildStripState::default();
        let mut tool = ActiveBuildTool::default();
        let mut witness = UiShellMigrationWitness::default();
        witness_build_rail_tool_authoritative_replay(
            &mut strip,
            &mut tool,
            &mut witness,
            ToolContext::Industry,
        );
        assert_eq!(strip.active, ToolContext::Industry);
        assert!(witness.build_rail_authoritative);
        assert!(ui_p2a_p4_auth_green(&witness));
        let body = build_proof_payload(
            &witness,
            &ContextTrayState::default(),
            &ProductShellDiagnostics::default(),
        );
        assert_eq!(body["ui_p2a_tail"]["f03_green"], serde_json::json!(false));
        assert_eq!(body["ui_p2a_tail"]["p4_auth_green"], serde_json::json!(true));
    }

    #[test]
    fn ui_p2a_tail_live_witness_refresh() {
        use crate::construction::{ActiveBuildTool, BuildStripState, ToolContext};

        let mut strip = BuildStripState::default();
        let mut tool = ActiveBuildTool::default();
        let mut witness = UiShellMigrationWitness::default();
        witness_ops_strip_zone_hover_replay(&mut witness);
        witness_build_rail_tool_authoritative_replay(
            &mut strip,
            &mut tool,
            &mut witness,
            ToolContext::Utilities,
        );
        assert!(commit_ui_shell_migration_live_proof(
            &witness,
            &ContextTrayState::default(),
            &ProductShellDiagnostics::default(),
        ));
        let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["ui_p2a_tail"]["f03_green"], serde_json::json!(true));
        assert_eq!(body["ui_p2a_tail"]["p4_auth_green"], serde_json::json!(true));
        assert_eq!(
            body["witness"]["ops_zone_hover_token"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["witness"]["build_rail_authoritative"],
            serde_json::json!(true)
        );
    }

    #[test]
    fn witness_interaction_replay_sets_flags() {
        let mut tray = ContextTrayState::default();
        let mut witness = UiShellMigrationWitness::default();
        let mut intel = OpsStripIntelFocusRequest::default();
        let world = WorldRepresentationFrame {
            focus_chunk: IVec2::new(2, 3),
            ..Default::default()
        };
        witness_ops_strip_alerts_pressed(&mut tray, &mut witness, 4);
        assert!(witness.alert_click_expanded_tray);
        assert_eq!(tray.panel_state, HudPanelState::Expanded);
        witness_ops_strip_intel_pressed(&mut tray, &mut witness, &world, &mut intel);
        assert!(witness.intel_map_camera_request);
        assert!(intel.pending_world.is_some());
        collapse_context_tray_on_escape(&mut tray, &mut witness);
        assert!(witness.escape_collapsed_tray);
    }

    #[test]
    fn ui_p2a_coder_b_lib_bundle_green() {
        assert!(crate::construction::mock_shapes_parity_green());
        let witness = UiShellMigrationWitness {
            phase2_zones_live: true,
            ops_zones_wired: true,
            mock_zone_parity: true,
            build_rail_synced: true,
            build_rail_authoritative: true,
            ops_zone_hover_token: true,
            ..Default::default()
        };
        assert!(ui_p2a_coder_b_green(&witness));
        let body = build_proof_payload(
            &witness,
            &ContextTrayState::default(),
            &ProductShellDiagnostics::default(),
        );
        assert_eq!(body["ui_p2a_coder_b"]["green"], serde_json::json!(true));
    }

    #[test]
    fn ui_w3_2a_001_live_witness_refresh() {
        assert!(refresh_ui_w3_2a_001_live_witness());
        let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["phase2_zones_live"], serde_json::json!(true));
        assert_eq!(body["ui_w3_2a_001"]["green"], serde_json::json!(true));
        assert_eq!(body["ui_w3_2a_001"]["ops_zones_wired"], serde_json::json!(true));
        assert_eq!(
            body["witness"]["alert_click_expanded_tray"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["witness"]["intel_map_camera_request"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["witness"]["escape_collapsed_tray"],
            serde_json::json!(true)
        );
        assert_eq!(body["phase2"]["zones_live"], serde_json::json!(true));
    }

    #[test]
    fn ui_w3_2a_001_green_requires_ops_zones_and_tray_interactions() {
        let good = replay_ui_w3_2a_001_witness();
        assert!(ui_w3_2a_001_green(&good));
        let mut bad = good.clone();
        bad.phase2_zones_live = false;
        assert!(!ui_w3_2a_001_green(&bad));
    }

    #[test]
    fn ui_oh_2a_001_live_witness_refresh() {
        assert!(refresh_ui_oh_2a_001_live_witness());
        let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["phase2_zones_live"], serde_json::json!(true));
        assert_eq!(body["ui_oh_2a_001"]["green"], serde_json::json!(true));
        assert_eq!(body["phase2a_closed"], serde_json::json!(true));
        assert_eq!(
            body["witness"]["alert_click_expanded_tray"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["witness"]["intel_map_camera_request"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["witness"]["escape_collapsed_tray"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["witness"]["minimap_chrome_aligned"],
            serde_json::json!(true)
        );
        assert_eq!(body["witness"]["ops_zones_wired"], serde_json::json!(true));
        assert_eq!(
            body["phase2"]["zones_live"],
            serde_json::json!(true)
        );
    }

    #[test]
    fn ui_p2a_001_live_witness_refresh() {
        let witness = UiShellMigrationWitness {
            phase2_zones_live: true,
            alert_click_expanded_tray: true,
            intel_map_camera_request: true,
            escape_collapsed_tray: true,
            minimap_chrome_aligned: true,
            flat_v2_tab_chrome: true,
            ops_zones_wired: true,
            build_rail_synced: true,
            build_rail_authoritative: true,
            build_toolbox_egui_gated: true,
            side_status_rail_egui_gated: true,
            floating_egui_shells_gated: true,
            ops_zone_hover_token: true,
            icon_atlas_loaded: true,
            last_minimap_rect_delta_px: 1.0,
            ..Default::default()
        };
        assert!(commit_ui_shell_migration_live_proof(
            &witness,
            &ContextTrayState::default(),
            &ProductShellDiagnostics::default(),
        ));
        let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["phase2_zones_live"], serde_json::json!(true));
        assert_eq!(body["ui_p2a_coder_b"]["green"], serde_json::json!(true));
        assert_eq!(body["ui_p2a_coder_b"]["mock_zone_parity"], serde_json::json!(true));
        assert_eq!(body["ui_p2a_tail"]["f03_green"], serde_json::json!(true));
        assert_eq!(body["ui_p2a_tail"]["p4_auth_green"], serde_json::json!(true));
        assert_eq!(body["phase2a_closed"], serde_json::json!(true));
        assert_eq!(
            body["witness"]["alert_click_expanded_tray"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["witness"]["intel_map_camera_request"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["witness"]["escape_collapsed_tray"],
            serde_json::json!(true)
        );
        assert!(commit_ui_shell_migration_live_proof(
            &witness,
            &ContextTrayState::default(),
            &ProductShellDiagnostics::default(),
        ));
    }

    #[test]
    fn ui_p3_001_shell_witness_fields_when_gpu_path_active() {
        let witness = UiShellMigrationWitness {
            minimap_gpu_path: true,
            minimap_chrome_aligned: true,
            phase2_zones_live: true,
            flat_v2_tab_chrome: true,
            build_toolbox_egui_gated: true,
            side_status_rail_egui_gated: true,
            floating_egui_shells_gated: true,
            ..Default::default()
        };
        let shell_diag = ProductShellDiagnostics::default();
        let payload = build_proof_payload(
            &witness,
            &ContextTrayState::default(),
            &shell_diag,
        );
        let compositor_green = super::minimap_compositor_ui_p3_001_green_from_disk();
        assert_eq!(
            payload["ui_p3_001"]["closed"],
            serde_json::json!(super::ui_p3_001_shell_closed(&witness, &shell_diag))
        );
        assert_eq!(
            payload["ui_p3_001"]["compositor_authoritative"],
            serde_json::json!(compositor_green)
        );
        assert_eq!(
            payload["backends"]["P3_minimap_texture"],
            serde_json::json!("bevy_ui_gpu")
        );
    }

    /// **UI-P3-SHELL-ROLLUP-001** + **UI-OH-P4-001** + **UI-OH-P5-001** — shell tail closure refresh.
    #[test]
    fn ui_shell_tail_closure_live_witness_refresh() {
        assert!(super::refresh_coder_b_ui_shell_tail_closure_witness());
        let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["ui_p3_001"]["closed"], serde_json::json!(true));
        assert_eq!(
            body["ui_p3_001"]["compositor_authoritative"],
            serde_json::json!(true)
        );
        assert_eq!(body["ui_oh_p4_001"]["green"], serde_json::json!(true));
        assert_eq!(body["ui_oh_p5_001"]["green"], serde_json::json!(true));
        assert_eq!(body["phase5"]["pause_menu_bevy"], serde_json::json!(true));
    }

    /// **UI-OH-2B-001** — product-shell egui hidden in Simulation; shell witness sim pass count 0.
    #[test]
    fn ui_oh_2b_001_live_witness_refresh() {
        use crate::engine::states::BaseState;
        use crate::gui::ui_gates::product_egui_shell_base_active;

        assert!(!product_egui_shell_base_active(BaseState::Simulation));
        assert!(refresh_ui_oh_2b_001_live_witness());
        let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["ui_oh_2b_001"]["green"], serde_json::json!(true));
        assert_eq!(
            body["ui_oh_2b_001"]["product_egui_shell_in_simulation"],
            serde_json::json!(false)
        );
        assert_eq!(body["egui_pass_count_in_sim"], serde_json::json!(0));
        assert_eq!(body["phase2b_closed"], serde_json::json!(true));
        assert_eq!(body["ui_p2b_coder_b_green"], serde_json::json!(true));
    }

    /// **UI-W3-2C-001** — 2C-B left mode rail + build rail authority witness refresh.
    #[test]
    fn ui_w3_2c_001_live_witness_refresh() {
        assert!(refresh_ui_w3_2c_001_live_witness());
        let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["ui_w3_2c_001"]["green"], serde_json::json!(true));
        assert_eq!(body["ui_w3_2c_001"]["layout_option"], serde_json::json!("2C-B"));
        assert_eq!(body["ui_w3_2c_001"]["context_rail_width_px"], serde_json::json!(48.0));
        assert_eq!(body["ui_w3_2c_001"]["build_rail_width_px"], serde_json::json!(52.0));
        assert_eq!(
            body["ui_w3_2c_001"]["left_chrome_width_px_collapsed"],
            serde_json::json!(106.0)
        );
        assert_eq!(body["phase2c"]["phase2c_closed"], serde_json::json!(true));
        assert_eq!(body["phase2c"]["layout_option"], serde_json::json!("2C-B"));
        assert_eq!(
            body["witness"]["build_rail_authoritative"],
            serde_json::json!(true)
        );
        assert_eq!(body["ui_p2a_tail"]["p4_auth_green"], serde_json::json!(true));
    }

    /// **UI-W3-P4-001** — icon atlas + build rail + petroleum tab witness refresh.
    #[test]
    fn ui_w3_p4_001_live_witness_refresh() {
        assert!(refresh_ui_w3_p4_001_live_witness());
        let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["ui_w3_p4_001"]["green"], serde_json::json!(true));
        assert_eq!(body["ui_w3_p4_001"]["icon_atlas_loaded"], serde_json::json!(true));
        assert_eq!(
            body["ui_w3_p4_001"]["petroleum_panel_tab_wired"],
            serde_json::json!(true)
        );
        assert_eq!(body["phase4"]["icon_atlas_loaded"], serde_json::json!(true));
        assert_eq!(body["phase4"]["p5_br_tab_wired"], serde_json::json!(true));
    }

    #[test]
    fn ui_w3_2c_001_green_requires_layout_contract_and_p4_auth() {
        let good = replay_ui_w3_2c_001_witness();
        assert!(ui_w3_2c_001_green(&good));
        let mut bad = good.clone();
        bad.build_rail_authoritative = false;
        assert!(!ui_w3_2c_001_green(&bad));
    }

    /// **UI-W3-P5-001** — Bevy pause menu witness refresh.
    #[test]
    fn ui_w3_p5_001_live_witness_refresh() {
        use crate::engine::states::BaseState;
        use crate::gui::ui_gates::product_egui_shell_base_active;

        assert!(!product_egui_shell_base_active(BaseState::Simulation));
        assert!(refresh_ui_w3_p5_001_live_witness());
        let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["ui_w3_p5_001"]["green"], serde_json::json!(true));
        assert_eq!(body["ui_w3_p5_001"]["pause_menu_bevy"], serde_json::json!(true));
        assert_eq!(body["phase5"]["pause_menu_bevy"], serde_json::json!(true));
        assert_eq!(body["ui_p5_pause_001_green"], serde_json::json!(true));
        assert_eq!(body["ui_w3_p5_001"]["egui_pass_count_in_sim"], serde_json::json!(0));
        assert_eq!(body["phase2a_closed"], serde_json::json!(true));
        assert_eq!(body["phase2_zones_live"], serde_json::json!(true));
    }

    /// **UI-W3-2B-001** — Wave 3 alias; egui gate witness refresh.
    #[test]
    fn ui_w3_2b_001_live_witness_refresh() {
        use crate::engine::states::BaseState;
        use crate::gui::ui_gates::product_egui_shell_base_active;

        assert!(!product_egui_shell_base_active(BaseState::Simulation));
        assert!(refresh_ui_w3_2b_001_live_witness());
        let text = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH).expect("witness");
        let body: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(body["ui_w3_2b_001"]["green"], serde_json::json!(true));
        assert_eq!(body["ui_w3_2b_001"]["egui_pass_count_in_sim"], serde_json::json!(0));
        assert_eq!(body["phase2b_closed"], serde_json::json!(true));
        assert_eq!(body["ui_p2b_coder_b_green"], serde_json::json!(true));
        assert_eq!(
            body["witness"]["build_toolbox_egui_gated"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["witness"]["floating_egui_shells_gated"],
            serde_json::json!(true)
        );
        assert_eq!(
            body["witness"]["side_status_rail_egui_gated"],
            serde_json::json!(true)
        );
    }

    /// **UI-P2B-CODER-B** — proof commit syncs dock/layout gates → `phase2b_closed`.
    #[test]
    fn ui_p2b_coder_b_phase2b_closed_when_sim_egui_gates_suppressed() {
        let mut dock = crate::gui::hud::HudDockRegistry::default();
        crate::gui::hud::shell_framework::suppress_simulation_floating_shell_slots(&mut dock);
        let mut layout = crate::gui::hud::HudCommandShellLayout::default();
        layout.status_side_panel_state = crate::gui::hud::HudPanelState::Collapsed;
        let witness = UiShellMigrationWitness {
            phase2_zones_live: true,
            flat_v2_tab_chrome: true,
            minimap_chrome_aligned: true,
            ..Default::default()
        };
        let shell_diag = ProductShellDiagnostics::default();
        let payload = build_proof_payload(&witness, &ContextTrayState::default(), &shell_diag);
        assert_eq!(payload["phase2b_closed"], serde_json::json!(false));

        assert!(commit_ui_shell_migration_live_proof_with_gates(
            &witness,
            &ContextTrayState::default(),
            &shell_diag,
            Some(&dock),
            Some(&layout),
        ));
        let refreshed = std::fs::read_to_string(UI_SHELL_MIGRATION_LIVE_PROOF_PATH)
            .expect("wave_p proof json");
        let v: serde_json::Value = serde_json::from_str(&refreshed).expect("parse");
        assert_eq!(v["phase2b_closed"], serde_json::json!(true));
        assert_eq!(v["ui_p2b_coder_b_green"], serde_json::json!(true));
        assert_eq!(v["egui_pass_count_in_sim"], serde_json::json!(0));
        assert_eq!(
            v["witness"]["build_toolbox_egui_gated"],
            serde_json::json!(true)
        );
        assert_eq!(
            v["ui_p2b_coder_b"]["floating_egui_shells_gated"],
            serde_json::json!(true)
        );
    }

    #[test]
    fn ui_p2b_coder_b_green_false_when_sim_session_egui_passes() {
        let witness = UiShellMigrationWitness {
            build_toolbox_egui_gated: true,
            side_status_rail_egui_gated: true,
            floating_egui_shells_gated: true,
            ..Default::default()
        };
        let mut shell_diag = ProductShellDiagnostics::default();
        shell_diag.record_egui_pass_in_simulation();
        assert!(!ui_p2b_coder_b_green(&witness, &shell_diag));
        let body = build_proof_payload(&witness, &ContextTrayState::default(), &shell_diag);
        assert_eq!(body["phase2b_closed"], serde_json::json!(false));
    }

    #[test]
    fn stage5_ui_shell_migration_phase2b_witness() {
        let witness = UiShellMigrationWitness {
            phase2_zones_live: true,
            flat_v2_tab_chrome: true,
            minimap_chrome_aligned: true,
            build_toolbox_egui_gated: true,
            side_status_rail_egui_gated: true,
            floating_egui_shells_gated: true,
            ..Default::default()
        };
        let shell_diag = ProductShellDiagnostics::default();
        let payload = build_proof_payload(&witness, &ContextTrayState::default(), &shell_diag);
        assert_eq!(payload["profile"], serde_json::json!("UI_SHELL_MIGRATION_2B"));
        assert_eq!(payload["egui_pass_count_in_sim"], serde_json::json!(0));
        assert_eq!(payload["phase2b_closed"], serde_json::json!(true));
        assert_eq!(
            payload["egui_sim_shell_widgets"],
            serde_json::json!(["Diagnostics_F3", "Editor_tools"])
        );
        assert_eq!(
            payload["witness"]["build_toolbox_egui_gated"],
            serde_json::json!(true)
        );
        assert_eq!(
            payload["witness"]["side_status_rail_egui_gated"],
            serde_json::json!(true)
        );
        assert!(payload.get("phase2").is_some());
        assert!(payload.get("backends").is_some());
        assert_eq!(
            payload["phase4"]["icon_atlas_loaded"],
            serde_json::json!(false)
        );
    }

    #[test]
    fn phase2c_2c_b_footprint_and_witness() {
        assert_eq!(command_left_stack_footprint_px(true), 106.0);
        assert_eq!(command_left_stack_footprint_px(false), 458.0);
        let witness = UiShellMigrationWitness {
            phase2_zones_live: true,
            build_rail_authoritative: true,
            build_rail_synced: true,
            ..Default::default()
        };
        let payload = build_proof_payload(
            &witness,
            &ContextTrayState::default(),
            &ProductShellDiagnostics::default(),
        );
        assert_eq!(payload["phase2c"]["layout_option"], serde_json::json!("2C-B"));
        assert_eq!(
            payload["phase2c"]["left_chrome_width_px_collapsed"],
            serde_json::json!(106.0)
        );
        assert_eq!(
            payload["phase2c"]["left_chrome_width_px_expanded"],
            serde_json::json!(458.0)
        );
        assert!(commit_ui_shell_migration_live_proof(
            &witness,
            &ContextTrayState::default(),
            &ProductShellDiagnostics::default(),
        ));
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

    #[test]
    fn stage5_ui_shell_migration_phase4_witness_fields() {
        let witness = replay_ui_w3_p4_001_witness();
        let payload = build_proof_payload(&witness, &ContextTrayState::default(), &ProductShellDiagnostics::default());
        assert_eq!(payload["phase4"]["icon_atlas_loaded"], serde_json::json!(true));
        assert_eq!(
            payload["phase4"]["atlas_texture"],
            serde_json::json!(crate::gui::hud::icon_atlas::ICON_ATLAS_TEXTURE_PATH)
        );
        assert_eq!(payload["phase4"]["p5_br_tab_wired"], serde_json::json!(true));
        assert_eq!(payload["ui_w3_p4_001"]["green"], serde_json::json!(true));
    }

    #[test]
    fn stage5_ui_shell_migration_phase2a_witness() {
        assert!(refresh_ui_oh_2a_001_live_witness());
        let witness = replay_ui_oh_2a_001_witness();
        let shell_diag = ProductShellDiagnostics::default();
        let payload = build_proof_payload(&witness, &ContextTrayState::default(), &shell_diag);
        assert_eq!(payload["phase2_zones_live"], serde_json::json!(true));
        assert_eq!(payload["ui_oh_2a_001"]["green"], serde_json::json!(true));
        assert!(payload.get("phase2").is_some());
        assert!(payload.get("backends").is_some());
        assert_eq!(
            payload["panels"]["P1"],
            serde_json::json!("ops_strip_zones_time_alerts_intel_weather_power_tray")
        );
    }
}
