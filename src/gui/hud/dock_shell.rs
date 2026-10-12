//! Detachable HUD docking + overlay tray scaffolding (**BQ-123**).
//!
//! Editor dock MVP: [`EditorDockBevyRoot`] hosts Command shell + Overlay tray +
//! minimized strip + side status rail (visibility from [`HudDockRegistry`] /
//! [`HudCommandShellLayout`]).

use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::engine::states::BaseState;
use crate::gui::input_bindings::InputBindings;
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::{in_simulation_or_editor, product_shell_active, PanelPresence};
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
    /// Docked left **status rail** (egui `SidePanel`) — not the floating command window.
    pub status_side_panel_state: HudPanelState,
    /// Floating **Command shell** window (telemetry / layout tools).
    pub command_tray_state: HudPanelState,
    pub intel_timeline_state: HudPanelState,
    pub command_table_state: HudPanelState,
}

impl Default for HudCommandShellLayout {
    fn default() -> Self {
        Self {
            overlay_tray_state: HudPanelState::Collapsed,
            status_side_panel_state: HudPanelState::Collapsed,
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

/// Editor product-shell dock root (Bevy UI — replaces retired egui dock bodies).
#[derive(Component)]
pub struct EditorDockBevyRoot;

#[derive(Component)]
struct EditorDockBodyText;

#[derive(Default)]
struct EditorDockStamp {
    key: u64,
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
            .add_plugins(super::hud_dev_overlay::HudDevOverlayPlugin)
            .add_plugins(super::hud_root_tick::HudRootTickPlugin)
            .add_systems(
                Update,
                (
                    sync_hud_overlay_tray_to_policy,
                    sync_hud_overlay_tray_to_minimap_overlays,
                    sync_minimap_overlays_to_hud_tray,
                    mirror_hud_dock_registry_from_widgets,
                    sync_hud_shell_interaction_router,
                    hud_dock_shell_keyboard_toggle.run_if(product_shell_active),
                    sync_editor_dock_bevy_shell.run_if(product_shell_active),
                    despawn_editor_dock_bevy_when_inactive.run_if(not(product_shell_active)),
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

fn hud_dock_shell_keyboard_toggle(
    keys: Res<ButtonInput<KeyCode>>,
    bindings: Res<InputBindings>,
    mut layout: ResMut<HudCommandShellLayout>,
    mut dock: ResMut<HudDockRegistry>,
) {
    if keys.just_pressed(bindings.toggle_command_left_stack) {
        layout.command_tray_state = if layout.command_tray_state.shows_content() {
            HudPanelState::Collapsed
        } else {
            HudPanelState::Expanded
        };
        dock.slot_mut(HudWidgetId::CommandShell).visible =
            HudCommandShellLayout::panel_open(layout.command_tray_state);
    }
}

fn editor_dock_body_text(
    dock: &HudDockRegistry,
    layout: &HudCommandShellLayout,
    tray: &HudOverlayTrayState,
) -> String {
    let mut lines = Vec::new();
    lines.push("Editor dock".into());
    lines.push(format!(
        "Command shell: {}",
        if dock.slot(HudWidgetId::CommandShell).visible {
            "open"
        } else {
            "closed"
        }
    ));
    lines.push(format!(
        "  tray={:?}  status_rail={:?}  intel={:?}  table={:?}",
        layout.command_tray_state,
        layout.status_side_panel_state,
        layout.intel_timeline_state,
        layout.command_table_state
    ));
    lines.push(format!(
        "Overlay tray: {}  fire={} logistics={} construction={} ecology={}",
        if dock.slot(HudWidgetId::OverlayTray).visible {
            "open"
        } else {
            "closed"
        },
        tray.fire_heat,
        tray.logistics_heat,
        tray.construction_heat,
        tray.ecology_heat
    ));
    lines.push("── Minimized strip ──".into());
    let mut any_min = false;
    for id in HudWidgetId::ALL {
        let slot = dock.slot(id);
        if slot.minimized && slot.visible {
            any_min = true;
            lines.push(format!("  · {}", id.label()));
        }
    }
    if !any_min {
        lines.push("  (none)".into());
    }
    lines.push("── Side status rail ──".into());
    if HudCommandShellLayout::panel_open(layout.status_side_panel_state) {
        lines.push("  status rail expanded (Bevy MVP)".into());
    } else {
        lines.push("  status rail collapsed".into());
    }
    lines.join("\n")
}

fn despawn_editor_dock_bevy_when_inactive(
    mut commands: Commands,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<EditorDockBevyRoot>>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    presence.editor_dock_root = false;
}

/// Bevy sync for editor dock chrome (Command shell + Overlay tray + strip + status rail).
pub fn sync_editor_dock_bevy_shell(
    mut commands: Commands,
    dock: Res<HudDockRegistry>,
    layout: Res<HudCommandShellLayout>,
    tray: Res<HudOverlayTrayState>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<EditorDockBevyRoot>>,
    mut body: Query<&mut Text, With<EditorDockBodyText>>,
    mut stamp: Local<EditorDockStamp>,
) {
    presence.editor_dock_root = !roots.is_empty();
    // Product-shell gate already scopes this system to Editor (¬WorldGen).
    // Always host the dock MVP summary; child slot lines follow HudDockRegistry.
    let text = editor_dock_body_text(dock.as_ref(), layout.as_ref(), tray.as_ref());
    let mut key = text.len() as u64;
    for b in text.bytes() {
        key = key.wrapping_mul(0x9E37_79B9).wrapping_add(b as u64);
    }

    if roots.is_empty() {
        let tf = TextFont::from_font_size(12.0).with_font(mono.0.clone());
        commands
            .spawn((
                EditorDockBevyRoot,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(40.0),
                    left: Val::Px(8.0),
                    width: Val::Px(280.0),
                    max_height: Val::Px(360.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(1.0)),
                    border_radius: BorderRadius::ZERO,
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(palette.bevy_hud_panel_fill()),
                BorderColor::all(palette.bevy_wire_magenta()),
                FocusPolicy::Block,
                ZIndex(930),
            ))
            .with_children(|p| {
                p.spawn((
                    EditorDockBodyText,
                    Text::new(text.clone()),
                    tf,
                    TextColor(palette.bevy_primary_text()),
                ));
            });
        stamp.key = key;
        presence.editor_dock_root = true;
        return;
    }

    if stamp.key != key {
        for mut t in &mut body {
            *t = Text::new(text.clone());
        }
        stamp.key = key;
    }
}



fn panel_state_checkbox() {}


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
