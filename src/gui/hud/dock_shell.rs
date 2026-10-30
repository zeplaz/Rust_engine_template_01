//! Detachable HUD docking + overlay tray scaffolding (**BQ-123**).
//!
//! Overlay-tray / command-shell layout state and the dock registry mirror. (The text-dump editor
//! dock panel was dropped: it showed layout flags only and nothing acted on it.)

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::in_simulation_or_editor;
use crate::gui::map_view::MapViewInstances;
use crate::gui::{simulation_minimap_overlay_defaults, MinimapOverlayMask, MinimapShellState, TransmissionShellState};
use crate::strategic::StrategicOverlayDisplayPolicy;

use super::layout_store::HudLayoutStore;
use super::shell_framework::{HudDockRegistry, HudWidgetDockState, HudWidgetId};
use super::panel_state::HudPanelState;
use super::shell_diagnostics::ProductShellDiagnostics;

/// Overlay tray toggles — strategic policy + minimap compositor [`MinimapOverlayMask`] (UI-P3-M2-TRAY-OPT).
#[derive(Resource, Clone, Debug)]
pub struct HudOverlayTrayState {
    pub recon_visible: bool,
    pub logistics_stress_visible: bool,
    pub congestion_visible: bool,
    pub fow_visible: bool,
    pub ew_visible: bool,
    /// GPU minimap compositor heat channels (M2-06 — synced to [`MapViewInstances::minimap`]).
    pub fire_heat: bool,
    pub logistics_heat: bool,
    pub construction_heat: bool,
    pub ecology_heat: bool,
    pub tray_panel_state: HudPanelState,
}

impl HudOverlayTrayState {
    #[must_use]
    pub fn minimap_overlay_mask(&self) -> MinimapOverlayMask {
        let base = simulation_minimap_overlay_defaults();
        MinimapOverlayMask {
            fire_heat: self.fire_heat,
            logistics_heat: self.logistics_heat,
            construction_heat: self.construction_heat,
            ecology_heat: self.ecology_heat,
            fow: self.fow_visible,
            ew: self.ew_visible,
            units: base.units,
            replay_scrub: base.replay_scrub,
        }
    }

    pub fn set_minimap_overlay_mask(&mut self, mask: MinimapOverlayMask) {
        self.fire_heat = mask.fire_heat;
        self.logistics_heat = mask.logistics_heat;
        self.construction_heat = mask.construction_heat;
        self.ecology_heat = mask.ecology_heat;
        self.fow_visible = mask.fow;
        self.ew_visible = mask.ew;
    }
}

impl Default for HudOverlayTrayState {
    fn default() -> Self {
        let mask = simulation_minimap_overlay_defaults();
        Self {
            recon_visible: false,
            logistics_stress_visible: false,
            congestion_visible: false,
            fow_visible: mask.fow,
            ew_visible: mask.ew,
            fire_heat: mask.fire_heat,
            logistics_heat: mask.logistics_heat,
            construction_heat: mask.construction_heat,
            ecology_heat: mask.ecology_heat,
            tray_panel_state: HudPanelState::Collapsed,
        }
    }
}

/// **MINIMAP-ECO-FIRE-TRAY-001** — tray defaults match simulation minimap overlay policy.
#[must_use]
pub fn minimap_eco_fire_tray_defaults_green() -> bool {
    let tray = HudOverlayTrayState::default();
    let mask = tray.minimap_overlay_mask();
    let defaults = simulation_minimap_overlay_defaults();
    !mask.fire_heat && mask.ecology_heat == defaults.ecology_heat
}

/// Command shell layout flags (overlay tray → command tray → intel timeline → command table).
#[derive(Resource, Clone, Debug)]
pub struct HudCommandShellLayout {
    pub overlay_tray_state: HudPanelState,
    /// Floating **Command shell** window (telemetry / layout tools).
    pub command_tray_state: HudPanelState,
    pub intel_timeline_state: HudPanelState,
    pub command_table_state: HudPanelState,
}

impl Default for HudCommandShellLayout {
    fn default() -> Self {
        Self {
            overlay_tray_state: HudPanelState::Collapsed,
            command_tray_state: HudPanelState::Collapsed,
            intel_timeline_state: HudPanelState::Collapsed,
            command_table_state: HudPanelState::Collapsed,
        }
    }
}

impl HudCommandShellLayout {
    #[inline]
    fn panel_open(state: HudPanelState) -> bool {
        state.shows_content()
    }
}

pub struct HudDockShellPlugin;

impl Plugin for HudDockShellPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(super::panel_state::HudPanelStatePlugin)
            .init_resource::<HudDockRegistry>()
            .init_resource::<HudOverlayTrayState>()
            .init_resource::<HudCommandShellLayout>()
            .init_resource::<HudLayoutStore>()
            .init_resource::<ProductShellDiagnostics>()
            .init_resource::<super::viewport_rect_sanity::ViewportRectSanity>()
            .init_resource::<super::stage6_telemetry::Stage6HudTelemetry>()
            .init_resource::<super::frame_budget_diagnostics::FrameBudgetDiagnostics>()
            .init_resource::<super::shell_update_budget::ProductShellUpdateBudget>()
            .init_resource::<super::shell_widget_timing::ShellWidgetDiagnostics>()
            .init_resource::<super::widget_presentation::WidgetPresentationPolicy>()
            .init_resource::<super::hud_async_task_queue::HudAsyncTaskQueue>()
            .init_resource::<super::pending_hud_layout_commit::PendingHudLayoutCommit>()
            .init_resource::<super::world_interaction_diagnostics::WorldInteractionDiagnostics>()
            .init_resource::<super::retained_widget_cache::RetainedWidgetCache>()
            .init_resource::<super::hud_interaction_budget::HudFrameBudget>()
            .init_resource::<super::interaction_latency::InteractionLatencyMetrics>()
            .init_resource::<super::shell_surface::ShellSurfacePolicy>()
            .init_resource::<super::widget_presentation::HudShellInteractionRouter>()
            .init_resource::<super::hud_shell_stress_harness::HudShellStressHarness>()
            .init_resource::<super::player_intent_panel::PlayerIntentPanelState>()
            .add_plugins(super::hud_ui_scale::HudUiScalePlugin)
            .add_plugins(super::hud_root_tick::HudRootTickPlugin)
            .add_systems(
                Update,
                (
                    sync_hud_overlay_tray_to_policy,
                    sync_hud_overlay_tray_to_minimap_overlays,
                    sync_minimap_overlays_to_hud_tray,
                    mirror_hud_dock_registry_from_widgets,
                    sync_hud_shell_interaction_router,
                    super::shell_diagnostics::product_shell_diagnostics_tick,
                    super::shell_update_budget::sync_product_shell_update_budget,
                    super::shell_update_budget::advance_product_shell_update_budget,
                    super::hud_async_task_queue::drain_hud_async_task_queue,
                    super::world_interaction_diagnostics::refresh_world_interaction_diagnostics,
                    super::interaction_latency::refresh_interaction_latency_metrics,
                )
                    .run_if(in_simulation_or_editor),
            )
            .add_systems(
                PostUpdate,
                (
                    super::hud_interaction_budget::apply_hud_interaction_frame_budget,
                    super::pending_hud_layout_commit::finalize_pending_hud_layout_commits,
                    super::pending_hud_layout_commit::flush_pending_hud_layout_on_pointer_release,
                )
                    .run_if(in_simulation_or_editor),
            );
    }
}

fn sync_hud_overlay_tray_to_policy(
    tray: Res<HudOverlayTrayState>,
    mut policy: ResMut<StrategicOverlayDisplayPolicy>,
) {
    if !tray.is_changed() {
        return;
    }
    policy.apply_routing_congestion = tray.congestion_visible;
    policy.apply_ew_denial = tray.ew_visible;
}

/// UI-P3-M2-TRAY-OPT — overlay tray checkboxes drive [`MapViewInstances::minimap`] compositor mask.
fn sync_hud_overlay_tray_to_minimap_overlays(
    base: Res<State<BaseState>>,
    tray: Res<HudOverlayTrayState>,
    mut map_views: ResMut<MapViewInstances>,
) {
    if !matches!(*base.get(), BaseState::Simulation) || !tray.is_changed() {
        return;
    }
    let mask = tray.minimap_overlay_mask();
    if map_views.minimap.overlays == mask {
        return;
    }
    map_views.minimap.overlays = mask;
    map_views.minimap.bump_revision();
}

/// Keep tray toggles aligned when minimap toolbar edits the same mask.
fn sync_minimap_overlays_to_hud_tray(
    base: Res<State<BaseState>>,
    map_views: Res<MapViewInstances>,
    mut tray: ResMut<HudOverlayTrayState>,
) {
    if !matches!(*base.get(), BaseState::Simulation) || !map_views.is_changed() {
        return;
    }
    let mask = map_views.minimap.overlays;
    if tray.minimap_overlay_mask() == mask {
        return;
    }
    tray.set_minimap_overlay_mask(mask);
}

fn sync_hud_shell_interaction_router(
    dock: Res<HudDockRegistry>,
    mut router: ResMut<super::widget_presentation::HudShellInteractionRouter>,
) {
    if dock.is_changed() {
        router.sync_from_dock(&dock);
    }
}

fn mirror_hud_dock_registry_from_widgets(
    base: Res<State<BaseState>>,
    minimap: Res<MinimapShellState>,
    transmission: Res<TransmissionShellState>,
    layout: Res<HudCommandShellLayout>,
    tray: Res<HudOverlayTrayState>,
    mut dock: ResMut<HudDockRegistry>,
) {
    if !minimap.is_changed()
        && !transmission.is_changed()
        && !layout.is_changed()
        && !tray.is_changed()
        && !base.is_changed()
    {
        return;
    }
    let sim = matches!(*base.get(), BaseState::Simulation);
    let next_minimap = HudWidgetDockState {
        visible: minimap.visible,
        minimized: minimap.minimized,
        detached: minimap.detached,
        z_order: dock.slot(HudWidgetId::Minimap).z_order,
    };
    let next_transmission = HudWidgetDockState {
        visible: transmission.active,
        minimized: transmission.minimized,
        detached: false,
        z_order: dock.slot(HudWidgetId::Transmission).z_order,
    };
    let next_overlay = HudWidgetDockState {
        visible: if sim {
            false
        } else {
            HudCommandShellLayout::panel_open(tray.tray_panel_state)
        },
        minimized: dock.slot(HudWidgetId::OverlayTray).minimized,
        detached: false,
        z_order: dock.slot(HudWidgetId::OverlayTray).z_order,
    };
    let next_command = HudWidgetDockState {
        visible: if sim {
            false
        } else {
            HudCommandShellLayout::panel_open(layout.command_tray_state)
        },
        minimized: dock.slot(HudWidgetId::CommandShell).minimized,
        detached: false,
        z_order: dock.slot(HudWidgetId::CommandShell).z_order,
    };
    if dock.slot(HudWidgetId::Minimap) != next_minimap {
        dock.set_state(HudWidgetId::Minimap, next_minimap);
    }
    if dock.slot(HudWidgetId::Transmission) != next_transmission {
        dock.set_state(HudWidgetId::Transmission, next_transmission);
    }
    if dock.slot(HudWidgetId::OverlayTray).visible != next_overlay.visible {
        dock.slot_mut(HudWidgetId::OverlayTray).visible = next_overlay.visible;
    }
    if dock.slot(HudWidgetId::CommandShell).visible != next_command.visible {
        dock.slot_mut(HudWidgetId::CommandShell).visible = next_command.visible;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_shell_layout_defaults_closed_on_load() {
        let layout = HudCommandShellLayout::default();
        assert_eq!(layout.overlay_tray_state, HudPanelState::Collapsed);
        assert_eq!(layout.command_tray_state, HudPanelState::Collapsed);
        assert_eq!(layout.command_table_state, HudPanelState::Collapsed);
    }

    #[test]
    fn overlay_tray_default_matches_simulation_minimap_mask() {
        let tray = HudOverlayTrayState::default();
        let expected = simulation_minimap_overlay_defaults();
        assert_eq!(tray.minimap_overlay_mask(), expected);
    }

    #[test]
    fn overlay_tray_minimap_mask_roundtrip() {
        let mut tray = HudOverlayTrayState::default();
        tray.logistics_heat = false;
        tray.construction_heat = true;
        let mask = tray.minimap_overlay_mask();
        assert!(!mask.logistics_heat);
        assert!(mask.construction_heat);
        tray.set_minimap_overlay_mask(simulation_minimap_overlay_defaults());
        assert!(!tray.ecology_heat);
    }

    #[test]
    fn minimap_eco_fire_tray_defaults_match_sim_policy() {
        assert!(super::minimap_eco_fire_tray_defaults_green());
    }
}
