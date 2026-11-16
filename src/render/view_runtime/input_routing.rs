//! VM-07: map UI input routes through [`ViewProjectionAuthority`] (not direct [`ViewManager`] writes).
//!
//! # Context-sensitive capture (INPUT-04 · ECS-10)
//!
//! Ref-books **INPUT-04** · LEGACY keyword `hid` (GEA 4e I §9.5.8–9.5.9, gea_v1_e4 p564–565 /
//! p538): mode-dependent bindings and **disabling** world input while UI owns the pointer.
//!
//! Ref-books **ECS-10** · Chains of Responsibility (GEA 4e II §17.8.7, gea4v2 p639–641): ordered
//! handlers — **UI chrome → map surface → world** — first claim wins. Esc collapse / pause is the
//! keyboard twin of this chain ([`crate::gui::hud::sim_hud_esc_cascade`]).
//!
//! Product invariant: side rails, ops strip, bottom context tray, and play panels must set
//! [`ViewInputRoutingState::ui_chrome_captures`] so camera / pick / tool commits stay disabled
//! under chrome (prior pain: sidebar / bottom-bar clicks falling through to the map).

use bevy::prelude::*;

use crate::gui::hud::simulation_pointer_gate::SimulationMapPointerGate;
use crate::gui::{ActiveMapViewInput, MapViewInstanceId, MapViewInstances, ViewCameraState};

use super::authority::{ViewAuthorityWriter, ViewProjectionAuthority};
use super::ids::ViewSurfaceId;
use super::trace::ViewRuntimeTrace;

/// Who owns pointer / camera-class input this frame (UI-then-world).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputCaptureOwner {
    /// Play chrome or floating play panels — world map picks / camera must not run.
    UiChrome,
    /// Hovered secondary map surface (e.g. World Preview) steals main-world camera input.
    MapSurface,
    /// Free: main simulation map may receive gameplay input.
    World,
}

/// Per-frame input routing witness (infrastructure JSON + HUD).
#[derive(Resource, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewInputRoutingState {
    pub active_surface: Option<ViewSurfaceId>,
    /// Secondary map surface blocks main-world camera (World Preview hover).
    pub map_surface_blocks: bool,
    /// Simulation chrome / play panels captured the pointer (INPUT-04 disable).
    pub ui_chrome_captures: bool,
    /// Resolved owner after the UI → surface → world chain.
    pub capture_owner: InputCaptureOwner,
    /// Convenience: map surface **or** UI chrome owns the pointer (blocks main-world camera).
    pub blocks_world_main: bool,
}

impl Default for InputCaptureOwner {
    fn default() -> Self {
        Self::World
    }
}

impl ViewInputRoutingState {
    /// True when **pointer** world gameplay (picks, mouse pan, edge scroll, wheel) must stay off.
    /// Keyboard camera (WASD / recenter / zoom keys) is not gated by this (CQ-05).
    #[inline]
    #[must_use]
    pub fn world_gameplay_input_disabled(self) -> bool {
        !world_gameplay_pointer_enabled(self.capture_owner)
    }

    /// Alias for pointer-only disable (CQ-05 naming).
    #[inline]
    #[must_use]
    pub fn world_pointer_input_disabled(self) -> bool {
        self.world_gameplay_input_disabled()
    }
}

/// UI chrome is handler #1 — sidebars, ops strip, bottom tray, play panels.
#[inline]
#[must_use]
pub fn ui_chrome_captures_world_pointer(chrome_blocks: bool, egui_blocks: bool) -> bool {
    chrome_blocks || egui_blocks
}

/// Resolve the capture chain: UI chrome → map surface → world (ECS-10).
#[inline]
#[must_use]
pub fn resolve_pointer_capture_owner(
    ui_chrome_captures: bool,
    map_surface_blocks_world: bool,
) -> InputCaptureOwner {
    if ui_chrome_captures {
        InputCaptureOwner::UiChrome
    } else if map_surface_blocks_world {
        InputCaptureOwner::MapSurface
    } else {
        InputCaptureOwner::World
    }
}

#[inline]
#[must_use]
pub fn world_gameplay_pointer_enabled(owner: InputCaptureOwner) -> bool {
    matches!(owner, InputCaptureOwner::World)
}

/// Compose routing fields from map-surface hover + optional pointer-gate chrome.
#[inline]
#[must_use]
pub fn compose_view_input_routing(
    active_surface: Option<ViewSurfaceId>,
    map_surface_blocks_world: bool,
    ui_chrome_captures: bool,
) -> ViewInputRoutingState {
    let capture_owner =
        resolve_pointer_capture_owner(ui_chrome_captures, map_surface_blocks_world);
    ViewInputRoutingState {
        active_surface,
        map_surface_blocks: map_surface_blocks_world,
        ui_chrome_captures,
        capture_owner,
        blocks_world_main: map_surface_blocks_world || ui_chrome_captures,
    }
}

#[must_use]
pub fn view_surface_from_map_instance(id: MapViewInstanceId) -> ViewSurfaceId {
    match id {
        MapViewInstanceId::WorldPreview => ViewSurfaceId::WorldPreview,
        MapViewInstanceId::Minimap => ViewSurfaceId::Minimap,
        MapViewInstanceId::SimulationMap
        | MapViewInstanceId::TacticalMap
        | MapViewInstanceId::FullscreenMap
        | MapViewInstanceId::CommanderMap
        | MapViewInstanceId::Stage7IntelMap => ViewSurfaceId::SimulationMap,
    }
}

/// Sole writer of [`ViewInputRoutingState`] (CQ-05).
///
/// Reads map-surface hover + published pointer-gate chrome and resolves the
/// UI → surface → world chain in one place. Schedule after
/// [`crate::gui::hud::simulation_pointer_gate::PointerGateSet::Publish`] and
/// before [`crate::gui::MapCameraSystemSet::ApplyInput`].
pub fn sync_view_input_routing(
    active: Res<ActiveMapViewInput>,
    gate: Option<Res<SimulationMapPointerGate>>,
    mut routing: ResMut<ViewInputRoutingState>,
) {
    let map_blocks = active.blocks_main_world_map_camera_input();
    let ui_chrome = gate.is_some_and(|g| {
        ui_chrome_captures_world_pointer(g.chrome_blocks, g.egui_blocks)
    });
    *routing = compose_view_input_routing(
        active.0.map(view_surface_from_map_instance),
        map_blocks,
        ui_chrome,
    );
}

/// Backward-compatible alias — prefer [`sync_view_input_routing`].
pub fn sync_view_input_routing_from_active_map(
    active: Res<ActiveMapViewInput>,
    gate: Option<Res<SimulationMapPointerGate>>,
    routing: ResMut<ViewInputRoutingState>,
) {
    sync_view_input_routing(active, gate, routing);
}

/// Backward-compatible alias — prefer [`sync_view_input_routing`].
pub fn apply_pointer_gate_capture_to_view_input_routing(
    active: Res<ActiveMapViewInput>,
    gate: Option<Res<SimulationMapPointerGate>>,
    routing: ResMut<ViewInputRoutingState>,
) {
    sync_view_input_routing(active, gate, routing);
}

/// After deferred [`MapViewInteractionByView`] applies to [`MapViewInstances`], commit poses to authority.
pub fn commit_deferred_map_view_poses_from_instances(
    views: &MapViewInstances,
    authority: &mut ViewProjectionAuthority,
    trace: &mut ViewRuntimeTrace,
) {
    let preview_cam = ViewCameraState {
        translation: views.world_preview.camera_center,
        zoom: views.world_preview.zoom,
        rotation: 0.0,
    };
    let minimap_cam = ViewCameraState {
        translation: views.minimap.camera_center,
        zoom: views.minimap.zoom,
        rotation: 0.0,
    };

    if trace.enabled {
        authority.commit_pose_traced(
            ViewSurfaceId::WorldPreview,
            preview_cam,
            ViewAuthorityWriter::PreviewPanel,
            Some(trace),
        );
        authority.commit_pose_traced(
            ViewSurfaceId::Minimap,
            minimap_cam,
            ViewAuthorityWriter::MinimapShell,
            Some(trace),
        );
    } else {
        authority.commit_pose(
            ViewSurfaceId::WorldPreview,
            preview_cam,
            ViewAuthorityWriter::PreviewPanel,
        );
        authority.commit_pose(
            ViewSurfaceId::Minimap,
            minimap_cam,
            ViewAuthorityWriter::MinimapShell,
        );
    }
}

pub fn commit_deferred_map_view_poses_to_authority(
    views: Res<MapViewInstances>,
    mut authority: ResMut<ViewProjectionAuthority>,
    mut trace: ResMut<ViewRuntimeTrace>,
) {
    commit_deferred_map_view_poses_from_instances(
        views.as_ref(),
        authority.as_mut(),
        trace.as_mut(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::hud::simulation_pointer_gate::{
        simulation_chrome_blocks_map_pointer, SimulationMapPointerGate,
    };
    use crate::gui::hud::simulation_shell_phase2::ContextTrayState;
    use crate::gui::hud::panel_state::HudPanelState;
    use crate::gui::{CommandLeftStackState, MinimapShellState};
    use bevy::math::Vec2;

    #[test]
    fn ui_chrome_wins_capture_chain_over_map_surface() {
        let owner = resolve_pointer_capture_owner(true, true);
        assert_eq!(owner, InputCaptureOwner::UiChrome);
        assert!(!world_gameplay_pointer_enabled(owner));
    }

    #[test]
    fn map_surface_blocks_world_when_chrome_free() {
        let owner = resolve_pointer_capture_owner(false, true);
        assert_eq!(owner, InputCaptureOwner::MapSurface);
        assert!(!world_gameplay_pointer_enabled(owner));
    }

    #[test]
    fn world_enabled_when_no_upstream_handler() {
        let owner = resolve_pointer_capture_owner(false, false);
        assert_eq!(owner, InputCaptureOwner::World);
        assert!(world_gameplay_pointer_enabled(owner));
    }

    #[test]
    fn left_sidebar_context_disables_world_pointer() {
        let left = CommandLeftStackState { collapsed: true };
        let chrome = simulation_chrome_blocks_map_pointer(
            Vec2::new(20.0, 200.0),
            Vec2::new(1280.0, 720.0),
            &left,
            &MinimapShellState::default(),
            &ContextTrayState::default(),
        );
        assert!(chrome, "left rail is chrome");
        let routing = compose_view_input_routing(None, false, chrome);
        assert!(routing.ui_chrome_captures);
        assert!(routing.world_gameplay_input_disabled());
        assert_eq!(routing.capture_owner, InputCaptureOwner::UiChrome);
    }

    #[test]
    fn bottom_context_tray_disables_world_pointer() {
        let mut tray = ContextTrayState::default();
        tray.panel_state = HudPanelState::Expanded;
        let chrome = simulation_chrome_blocks_map_pointer(
            Vec2::new(640.0, 700.0),
            Vec2::new(1280.0, 720.0),
            &CommandLeftStackState { collapsed: true },
            &MinimapShellState::default(),
            &tray,
        );
        assert!(chrome, "expanded bottom tray is chrome");
        let routing = compose_view_input_routing(None, false, chrome);
        assert!(routing.world_gameplay_input_disabled());
    }

    // ECS-10 tripwires: a pointer event claimed upstream (UI chrome / map surface) never reaches
    // the world handler, regardless of which half of the chain re-composes routing.

    #[test]
    fn ecs10_sole_writer_keeps_chrome_over_map_surface() {
        let mut app = App::new();
        app.init_resource::<ViewInputRoutingState>()
            .insert_resource(ActiveMapViewInput(Some(MapViewInstanceId::WorldPreview)))
            .insert_resource(SimulationMapPointerGate {
                chrome_blocks: true,
                egui_blocks: false,
                ..default()
            });
        app.add_systems(Update, sync_view_input_routing);
        app.update();
        let routing = *app.world().resource::<ViewInputRoutingState>();
        assert!(routing.ui_chrome_captures);
        assert!(routing.map_surface_blocks);
        assert_eq!(routing.capture_owner, InputCaptureOwner::UiChrome, "UI outranks surface");
        assert!(routing.world_pointer_input_disabled());
    }

    #[test]
    fn ecs10_sole_writer_keeps_surface_claim_when_chrome_clear() {
        let mut app = App::new();
        app.init_resource::<ViewInputRoutingState>()
            .insert_resource(ActiveMapViewInput(Some(MapViewInstanceId::WorldPreview)));
        // No published gate resource → chrome declines; surface still claims.
        app.add_systems(Update, sync_view_input_routing);
        app.update();
        let routing = *app.world().resource::<ViewInputRoutingState>();
        assert!(!routing.ui_chrome_captures);
        assert!(routing.map_surface_blocks);
        assert_eq!(routing.capture_owner, InputCaptureOwner::MapSurface);
        assert!(routing.world_pointer_input_disabled());
    }

    #[test]
    fn cq05_pointer_disabled_does_not_mean_keyboard_must_stop() {
        // Documented contract: chrome disables pointer path only; map camera must still
        // accept WASD / recenter when `world_pointer_input_disabled` is true.
        let routing = compose_view_input_routing(None, false, true);
        assert!(routing.world_pointer_input_disabled());
        assert!(routing.world_gameplay_input_disabled());
    }

    #[test]
    fn ecs10_floating_panel_pointer_claim_counts_as_ui_chrome() {
        assert!(ui_chrome_captures_world_pointer(false, true));
        assert!(ui_chrome_captures_world_pointer(true, false));
        assert!(!ui_chrome_captures_world_pointer(false, false));
        let routing = compose_view_input_routing(None, false, true);
        assert_eq!(routing.capture_owner, InputCaptureOwner::UiChrome);
        assert!(routing.blocks_world_main);
    }

    #[test]
    fn map_center_keeps_world_owner_when_chrome_clear() {
        let chrome = simulation_chrome_blocks_map_pointer(
            Vec2::new(640.0, 400.0),
            Vec2::new(1280.0, 720.0),
            &CommandLeftStackState { collapsed: true },
            &MinimapShellState::default(),
            &ContextTrayState::default(),
        );
        assert!(!chrome);
        let routing = compose_view_input_routing(Some(ViewSurfaceId::SimulationMap), false, false);
        assert!(!routing.world_gameplay_input_disabled());
        assert_eq!(routing.capture_owner, InputCaptureOwner::World);
    }
}
