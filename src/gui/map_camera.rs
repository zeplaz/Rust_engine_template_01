//! Pan / zoom / edge-scroll / rotate for [`MainWorldCamera`] using [`InputBindings`](crate::gui::InputBindings).
//!
//! **P1-F:** Input writes [`ViewProjectionAuthority`] (WorldMain); [`derive_map_camera_desired_from_view_authority`]
//! mirrors into [`MapCameraDesired`]; smooth + ortho sync follow.
//!
//! **Input priority (tactical map vs egui):**
//! 1. [`ActiveMapViewInput`] — World Preview blocks; minimap chrome uses [`minimap_bevy_scroll_zoom_system`].
//! 2. [`simulation_pointer_gate::SimulationMapPointerGate`] — Bevy chrome rects (pre-egui) + floating HUD (post-egui).
//! 3. Wheel — [`map_camera_wheel_zoom_system`] in Update (OS [`AccumulatedMouseScroll`] only).
//! 4. Pan/keys — [`map_camera_apply_input_to_desired`] blocks when `wants_pointer_input` and cursor outside hole.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseWheel};
use bevy::camera::ClearColorConfig;
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui;

use bevy_egui::EguiContexts;

use crate::engine::states::BaseState;
use crate::engine::{ActiveTestScene, TestScene};
use crate::gui::view_authority::map_camera_desired_from_view_authority;
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::gui::ActiveMapViewInput;
use crate::gui::hud::simulation_pointer_gate;
use crate::gui::{InputBindings, InputFrame, SimulationMapViewport, SimulationViewportSyncSet};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;
use crate::gui::representation_governance::ScaffoldContract;
use crate::render::{trace_camera_sync, DebugRenderTraceConfig, Stage5ReadinessProfile};

/// Hybrid orthographic tilt tuning (VA5 scaffold — visual-only).
#[allow(dead_code)]
pub const HYBRID_ORTO_CAMERA_SCAFFOLD: ScaffoldContract = ScaffoldContract {
    owner: "gui/map_camera",
    intended_replacement: "documented camera runbook final strategic-zoom mode",
    exit_condition: "ortho curve matches experience_layer zoom spec under FULL_APP",
    removal_trigger: "duplicate projection writers for WorldMain",
};

/// **vm-09:** [`crate::gui::ViewAuthoritySystemSet::SyncViewManager`] is ordered **after** this set so
/// [`MapCameraDesired`] is updated before the view-manager bridge reads it each frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapCameraSystemSet {
    ApplyInput,
    /// **TRIAGE-VM-09-v2:** mirror authority → [`MapCameraDesired`] before smooth / bridge.
    DeriveDesired,
    Smooth,
}

/// Marker on the root [`Camera2d`] that carries world-space UI + weather VFX children.
#[derive(Component)]
pub struct MainWorldCamera;

/// Hysteresis for map-hole scissor vs full-window render (reduces flash when UI layout hiccups).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MainWorldCameraViewportLatch {
    pub using_hole: bool,
    pub invalid_streak: u8,
    pub valid_streak: u8,
    /// Steady-state hole toggles after bootstrap enter-hole (PERF-VIS-003 witness).
    pub steady_flip_count: u32,
    bootstrap_hole_committed: bool,
}

// MAP-BLINK-001: require a short adequacy streak before enabling hole scissor.
// This avoids one-frame full-window<->hole mode churn during WorldGen->Simulation handoff.
const CAM_HOLE_VALID_STREAK: u8 = 3;
/// Symmetric release — one inadequate layout frame must not drop hole scissor (ortho/viewport mismatch blink).
const CAM_HOLE_INVALID_STREAK: u8 = 2;

impl MainWorldCameraViewportLatch {
    /// Returns whether the camera should use the sim-map hole scissor this frame.
    pub fn advance(&mut self, sim_adequate: bool) -> bool {
        let was = self.using_hole;
        if sim_adequate {
            self.valid_streak = self.valid_streak.saturating_add(1);
            self.invalid_streak = 0;
            if !self.using_hole && self.valid_streak >= CAM_HOLE_VALID_STREAK {
                self.using_hole = true;
            }
        } else {
            self.invalid_streak = self.invalid_streak.saturating_add(1);
            self.valid_streak = 0;
            if self.using_hole && self.invalid_streak >= CAM_HOLE_INVALID_STREAK {
                self.using_hole = false;
            }
        }
        if self.using_hole != was {
            if !was && self.using_hole && !self.bootstrap_hole_committed {
                self.bootstrap_hole_committed = true;
            } else {
                self.steady_flip_count = self.steady_flip_count.saturating_add(1);
            }
        }
        if TACTICAL_MAP_FULL_WINDOW_RENDER {
            self.using_hole = false;
            return false;
        }
        self.using_hole
    }
}

pub fn reset_main_world_camera_viewport_latch_on_enter_simulation(
    mut latch: ResMut<MainWorldCameraViewportLatch>,
) {
    *latch = MainWorldCameraViewportLatch::default();
}

/// Last orthographic fit written by [`sync_main_world_camera_viewport_and_projection`] (debug).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MainWorldCameraOrthoTrace {
    pub fixed_width: f32,
    pub fixed_height: f32,
    pub view_pixels: Vec2,
    pub using_hole: bool,
    /// True when the camera scissor matches the sim-map hole (not healed to full window).
    pub use_hole_scissor: bool,
    pub authority_zoom: f32,
    pub desired_zoom: f32,
}

/// Per-frame zoom audit — enable `RUST_LOG=map_camera_zoom=debug` (warn on reverts).
#[derive(Resource, Clone, Debug, Default)]
pub struct MapCameraZoomAudit {
    pub frame: u32,
    pub authority_zoom: f32,
    pub desired_zoom: f32,
    pub ortho_fixed_w: f32,
    pub ortho_fixed_h: f32,
    pub view_px: Vec2,
    pub use_hole_scissor: bool,
    pub latch_using_hole: bool,
    pub last_commit_writer: Option<crate::render::view_runtime::ViewAuthorityWriter>,
    pub last_commit_prev_zoom: f32,
    pub last_commit_new_zoom: f32,
    pub revert_warn_count: u32,
}

/// Called from [`ViewProjectionAuthority::commit_pose`] for WorldMain (every pose writer).
pub fn on_world_main_pose_committed(
    writer: crate::render::view_runtime::ViewAuthorityWriter,
    prev_zoom: Option<f32>,
    new_zoom: f32,
) {
    use crate::render::view_runtime::ViewAuthorityWriter;
    let writer_name = match writer {
        ViewAuthorityWriter::MapCameraInput => "MapCameraInput",
        ViewAuthorityWriter::BridgeCompat => "BridgeCompat",
        ViewAuthorityWriter::ViewportPipeline => "ViewportPipeline",
        ViewAuthorityWriter::MinimapFollow => "MinimapFollow",
        ViewAuthorityWriter::MinimapShell => "MinimapShell",
        ViewAuthorityWriter::PreviewPanel => "PreviewPanel",
        ViewAuthorityWriter::Unset => "Unset",
    };
    if let Some(prev) = prev_zoom {
        let ratio = if prev > 1e-6 { new_zoom / prev } else { 1.0 };
        if ratio < 0.65 && (prev - new_zoom).abs() > 0.05 {
            bevy::log::warn!(
                target: "map_camera_zoom",
                "ZOOM_REVERT writer={writer_name} {prev:.4} -> {new_zoom:.4} (ratio={ratio:.3})"
            );
        }
    }
    bevy::log::debug!(
        target: "map_camera_zoom",
        "POSE_COMMIT writer={writer_name} zoom={new_zoom:.4}"
    );
}

/// Logical pose after map input (smoothing target).
///
/// **TRIAGE-VM-09-v2:** RTS input commits [`ViewProjectionAuthority`] first; this resource is updated by
/// [`derive_map_camera_desired_from_view_authority`] only (compatibility read surface for legacy APIs).
/// Prefer
/// [`crate::gui::camera_translation`] / [`crate::gui::camera_zoom`] for consumers that know a [`crate::gui::ViewId`].
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct MapCameraDesired {
    pub translation: Vec3,
    pub scale: Vec3,
    pub rotation: Quat,
}

impl Default for MapCameraDesired {
    fn default() -> Self {
        Self {
            translation: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
        }
    }
}

#[derive(Default)]
struct MapCameraInputLocals {
    middle_tap: Option<f32>,
    before_apply: Option<MapCameraDesired>,
}

/// Live Stage 5 audit (TODO-04): log **mutations** to [`MapCameraDesired`] when profile is [`Stage5ReadinessProfile::FULL_APP`].
/// Enable: `RUST_LOG=map_camera_desired::write=debug`.
pub fn trace_map_camera_desired_write_if_full_app(
    profile: &Stage5ReadinessProfile,
    source: &'static str,
    before: &MapCameraDesired,
    after: &MapCameraDesired,
) {
    if *profile != Stage5ReadinessProfile::FULL_APP {
        return;
    }
    if before == after {
        return;
    }
    debug!(
        target: "map_camera_desired::write",
        "MAP_CAMERA_DESIRED_WRITE source={source} translation=({:.2},{:.2},{:.2}) scale.x={:.4}",
        after.translation.x,
        after.translation.y,
        after.translation.z,
        after.scale.x,
    );
}

/// RTS-style map camera mode (diagnostic label; future behavior hooks).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum MapCameraMode {
    /// Default free pan / zoom.
    #[default]
    Strategic,
    /// Tighter framing semantics (reserved for unit snap, etc.).
    Tactical,
    /// Scripted or smoothed paths (reserved).
    Cinematic,
}

impl MapCameraMode {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Strategic => "STRAT",
            Self::Tactical => "TACT",
            Self::Cinematic => "CINE",
        }
    }

    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Self::Strategic => Self::Tactical,
            Self::Tactical => Self::Cinematic,
            Self::Cinematic => Self::Strategic,
        }
    }
}

/// RTS camera toggles (`base_visual_dev01` theme 1).
#[derive(Resource, Clone, Debug)]
pub struct MapCameraSettings {
    pub edge_scroll_enabled: bool,
    pub mode: MapCameraMode,
}

impl Default for MapCameraSettings {
    fn default() -> Self {
        Self {
            edge_scroll_enabled: true,
            mode: MapCameraMode::default(),
        }
    }
}

pub struct MapCameraPlugin;

impl Plugin for MapCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapCameraDesired>()
            .init_resource::<MapCameraSettings>()
            .add_plugins(crate::gui::InputFramePlugin)
            .configure_sets(
                Update,
                (
                    MapCameraSystemSet::DeriveDesired.after(MapCameraSystemSet::ApplyInput),
                    MapCameraSystemSet::Smooth.after(MapCameraSystemSet::DeriveDesired),
                ),
            )
            .add_systems(
                Update,
                (
                    map_camera_apply_input_to_desired,
                    map_camera_wheel_zoom_system.after(map_camera_apply_input_to_desired),
                )
                    .chain()
                    .in_set(MapCameraSystemSet::ApplyInput)
                    .after(crate::gui::hud::simulation_pointer_gate::sync_simulation_map_pointer_gate_system)
                    .run_if(in_simulation_or_editor_map),
            )
            .add_systems(
                Update,
                derive_map_camera_desired_from_view_authority
                    .in_set(MapCameraSystemSet::DeriveDesired)
                    .run_if(in_simulation_or_editor_map),
            )
            .init_resource::<MainWorldCameraViewportLatch>()
            .init_resource::<MainWorldCameraOrthoTrace>()
            .init_resource::<MapCameraZoomAudit>()
            .add_systems(
                Update,
                map_camera_smooth_toward_desired
                    .in_set(MapCameraSystemSet::Smooth)
                    .run_if(in_simulation_or_editor_map),
            )
            .add_systems(
                Update,
                apply_main_world_camera_ortho_immediate
                    .after(MapCameraSystemSet::Smooth)
                    .run_if(in_simulation_or_editor_map),
            )
            .add_systems(
                PostUpdate,
                (
                    advance_main_world_camera_viewport_latch,
                    sync_main_world_camera_viewport_and_projection
                        .after(advance_main_world_camera_viewport_latch),
                    map_camera_zoom_audit_system
                        .after(sync_main_world_camera_viewport_and_projection),
                )
                    .chain()
                    .in_set(SimulationViewportSyncSet::ApplyCameraScissor)
                    .run_if(in_simulation_or_editor_map),
            );
    }
}

pub fn in_simulation_or_editor_map(state: Res<State<BaseState>>) -> bool {
    matches!(
        state.get(),
        BaseState::Simulation | BaseState::Editor
    )
}

const EDGE_FRACTION: f32 = 0.06;
const KEY_PAN: f32 = 520.0;
const EDGE_PAN: f32 = 340.0;
const GRIP_PAN: f32 = 620.0;
const ZOOM_FACTOR: f32 = 1.20;
/// Fallback logical window size when the primary window is unavailable (camera math only).
pub const MAP_CAMERA_DEFAULT_WINDOW_PX: Vec2 = Vec2::new(1280.0, 720.0);
/// Z-plane for [`MainWorldCamera`] and map pose lift (weather/VFX children share this stack).
pub const MAIN_WORLD_CAMERA_Z: f32 = 999.0;
/// Fallback zoom limits when viewport/world size is unknown (witness / alpha only).
pub const MAP_ZOOM_CLAMP: (f32, f32) = (0.35, 10000.0);

#[inline]
fn sanitize_map_zoom_input(scale: f32) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

/// Primary window logical size for map camera math.
#[inline]
#[must_use]
pub fn primary_window_logical_px(windows: &Query<&Window, With<PrimaryWindow>>) -> Vec2 {
    windows
        .single()
        .map(|w| Vec2::new(w.width().max(1.0), w.height().max(1.0)))
        .unwrap_or(MAP_CAMERA_DEFAULT_WINDOW_PX)
}

/// Logical pixel size for map camera math — match the **active** camera viewport (hole scissor vs full window).
#[must_use]
pub fn map_camera_viewport_pixels(
    window: Vec2,
    map_viewport: Option<&SimulationMapViewport>,
) -> Vec2 {
    map_camera_viewport_pixels_for_scissor(window, map_viewport, false)
}

/// Tactical map renders **full window** — sim chrome is opaque Bevy UI on top.
/// Subrect `Camera.viewport` scissor leaves swapchain margins uncleared (burnt-in ghost frames at prior zoom).
const TACTICAL_MAP_FULL_WINDOW_RENDER: bool = true;

/// Logical view pixels for ortho / clamp — matches [`apply_main_world_camera_ortho`].
#[must_use]
pub fn map_camera_tactical_view_pixels(
    window_px: Vec2,
    sim_viewport: &SimulationMapViewport,
    latch: &MainWorldCameraViewportLatch,
) -> Vec2 {
    if TACTICAL_MAP_FULL_WINDOW_RENDER {
        return map_camera_ortho_view_pixels(window_px, sim_viewport, false);
    }
    let hole = resolve_map_camera_hole_scissor_active(sim_viewport, latch, window_px);
    map_camera_ortho_view_pixels(window_px, sim_viewport, hole)
}

/// Whether the map camera uses hole scissor this frame (heal logic shared with PostUpdate sync).
#[must_use]
pub fn resolve_map_camera_hole_scissor_active(
    sim: &SimulationMapViewport,
    latch: &MainWorldCameraViewportLatch,
    window_px: Vec2,
) -> bool {
    if TACTICAL_MAP_FULL_WINDOW_RENDER || !latch.using_hole {
        return false;
    }
    const MIN_HOLE_PHYSICAL_PX: f32 = 32.0;
    let w_log = (sim.max.x - sim.min.x).max(1.0);
    let h_log = (sim.max.y - sim.min.y).max(1.0);
    if !sim.is_adequate_for_camera()
        || w_log < MIN_HOLE_PHYSICAL_PX
        || h_log < MIN_HOLE_PHYSICAL_PX
    {
        return false;
    }
    let _ = window_px;
    true
}

/// Logical pixel size for orthographic fit — shared by wheel, clamp, and PostUpdate sync.
#[must_use]
pub fn map_camera_ortho_view_pixels(
    window_px: Vec2,
    sim_viewport: &SimulationMapViewport,
    hole_scissor_active: bool,
) -> Vec2 {
    let hole_active = hole_scissor_active && sim_viewport.is_adequate_for_camera();
    map_camera_viewport_pixels_for_scissor(window_px, Some(sim_viewport), hole_active)
}

/// Same as [`map_camera_viewport_pixels`] but uses hole dimensions only when hole scissor is active.
#[must_use]
pub fn map_camera_viewport_pixels_for_scissor(
    window: Vec2,
    map_viewport: Option<&SimulationMapViewport>,
    hole_scissor_active: bool,
) -> Vec2 {
    if hole_scissor_active {
        if let Some(vp) = map_viewport {
            if vp.is_adequate_for_camera() {
                return vp.logical_size();
            }
        }
    }
    Vec2::new(window.x.max(1.0), window.y.max(1.0))
}

/// World-space half-extents visible at the map center — matches [`orthographic_fixed_world_span`].
#[must_use]
pub fn map_visible_half_extents(scale: f32, viewport: Vec2, world: Vec2) -> Vec2 {
    let (fw, fh) = orthographic_fixed_world_span(viewport, scale.max(1e-4), world.x, world.y);
    Vec2::new(fw * 0.5, fh * 0.5)
}

/// World span for orthographic [`ScalingMode::Fixed`] — matches viewport aspect (no letterbox stretch).
#[must_use]
pub fn orthographic_fixed_world_span(view_px: Vec2, zoom: f32, _world_w: f32, _world_h: f32) -> (f32, f32) {
    let z = zoom.max(1e-4);
    (view_px.x / z, view_px.y / z)
}

/// Zoom limits derived from world size and window so large maps can zoom in further and small maps can zoom out.
#[must_use]
pub fn map_zoom_limits_for_world(world_w: f32, world_h: f32, viewport: Vec2) -> (f32, f32) {
    let w = world_w.max(1.0);
    let h = world_h.max(1.0);
    let vp = Vec2::new(viewport.x.max(1.0), viewport.y.max(1.0));
    let fit = (vp.x / w).min(vp.y / h) * 0.92;
    // Allow deep strategic zoom-out (~2% of fit-to-world zoom) for full operational context.
    let lo = (fit * 0.02).max(0.02);
    // Allow zoom until ~8 tiles span the shorter viewport edge (tile inspection on 4k+ worlds).
    let min_span_tiles = 8.0_f32;
    let hi_from_tile_span = vp.x.min(vp.y) / min_span_tiles;
    let hi = hi_from_tile_span.max(fit * 14.0).max(8.0).min(1000000.0);
    (lo, hi)
}

/// Keep the map filling the view: camera center clamped so world edges align with screen edges at most.
#[must_use]
pub fn clamp_map_camera_translation_xy(
    center: Vec2,
    scale: f32,
    world: Vec2,
    viewport: Vec2,
) -> Vec2 {
    let half = map_visible_half_extents(scale, viewport, world);
    let min = half;
    let max = world - half;
    let x = if min.x > max.x {
        world.x * 0.5
    } else {
        center.x.clamp(min.x, max.x)
    };
    let y = if min.y > max.y {
        world.y * 0.5
    } else {
        center.y.clamp(min.y, max.y)
    };
    Vec2::new(x, y)
}
const ROTATE_STEP: f32 = 1.35_f32.to_radians();
const SMOOTH_LAMBDA: f32 = 12.0;
/// MAP-ZOOM-001 / Option A — relative scale change above this snaps pan (ortho already instant).
pub const MAP_ZOOM_AXIS_SNAP_EPS: f32 = 0.002;
/// Pan/rotation already matched — skip lerp + clamp work.
const MAP_CAMERA_AT_REST_EPS: f32 = 0.05;
/// Cap smoothing dt so multi-second frame spikes do not overshoot pan/zoom lerp.
const MAX_CAMERA_SMOOTH_DT_SECS: f32 = 0.05;

/// Normalized zoom in `[0, 1]` from [`MAP_ZOOM_CLAMP`] using logical map scale `scale.x`.
#[inline]
pub fn map_zoom_alpha(scale_x: f32) -> f32 {
    map_zoom_alpha_with_limits(scale_x, MAP_ZOOM_CLAMP.0, MAP_ZOOM_CLAMP.1)
}

/// Target zoom alpha for P2-VFX-VISUAL-001 tactical witness (`stage5_full_app_harness`).
pub const TACTICAL_VFX_PROOF_ZOOM_ALPHA: f32 = 0.85;

/// Inverse of [`map_zoom_alpha_with_limits`] — pick map scale for a normalized zoom band.
#[inline]
#[must_use]
pub fn map_scale_for_zoom_alpha(alpha: f32, zoom_lo: f32, zoom_hi: f32) -> f32 {
    let span = (zoom_hi - zoom_lo).max(1e-5);
    zoom_lo + alpha.clamp(0.0, 1.0) * span
}

/// Normalized zoom using per-world limits from [`map_zoom_limits_for_world`].
#[inline]
#[must_use]
pub fn map_zoom_alpha_with_limits(scale_x: f32, zoom_lo: f32, zoom_hi: f32) -> f32 {
    let span = (zoom_hi - zoom_lo).max(1e-5);
    let z = scale_x.clamp(zoom_lo, zoom_hi);
    ((z - zoom_lo) / span).clamp(0.0, 1.0)
}

#[inline]
fn test_scene_zoom(test_scene: Option<Res<ActiveTestScene>>) -> f32 {
    test_scene
        .as_ref()
        .map(|ts| match ts.0 {
            TestScene::Weather => 1.25,
            TestScene::Fire => 1.95,
            TestScene::Atmosphere => 1.75,
            TestScene::Visual | TestScene::VfxSandbox => 2.1,
            TestScene::None => 1.0,
        })
        .unwrap_or(1.0)
}

/// Match [`crate::render::tile_world_fallback::focus_main_camera_on_world_params`] default orthographic scale.
pub fn default_map_zoom_for_world(test_scene: Option<Res<ActiveTestScene>>) -> f32 {
    test_scene_zoom(test_scene)
}

fn map_camera_apply_input_to_desired(
    time: Res<Time>,
    state: Res<State<BaseState>>,
    bindings: Res<InputBindings>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    input_frame: Res<InputFrame>,
    windows: Query<&Window, With<PrimaryWindow>>,
    sim_viewport: Res<SimulationMapViewport>,
    latch: Res<MainWorldCameraViewportLatch>,
    mut contexts: EguiContexts,
    params: Res<WorldGenParams>,
    active_map_surface: Res<ActiveMapViewInput>,
    mut settings: ResMut<MapCameraSettings>,
    mut authority: ResMut<ViewProjectionAuthority>,
    mut locals: Local<MapCameraInputLocals>,
) {
    // PERF-INSTR-VFX-001: name this system inside the `map_cam` wall bracket (STALL/PERF only).
    let _perf = crate::render::PerfScope::new("upd_map_camera_apply_input");
    locals.before_apply = None;

    if !matches!(state.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }

    let pointer_over_ui = contexts
        .ctx_mut()
        .ok()
        .map(|ctx| ctx.wants_pointer_input())
        .unwrap_or(false);
    let window_px = primary_window_logical_px(&windows);
    let cursor_over_sim_map = windows.single().ok().and_then(|window| {
        window.cursor_position().map(|cursor| {
            if sim_viewport.is_adequate_for_camera() {
                cursor.x >= sim_viewport.min.x
                    && cursor.x <= sim_viewport.max.x
                    && cursor.y >= sim_viewport.min.y
                    && cursor.y <= sim_viewport.max.y
            } else {
                cursor.x >= 0.0
                    && cursor.y >= 0.0
                    && cursor.x <= window_px.x
                    && cursor.y <= window_px.y
            }
        })
    }).unwrap_or(false);

    if active_map_surface
        .0
        .is_some_and(|id| id == crate::gui::MapViewInstanceId::Minimap)
    {
        let grip = keys.pressed(bindings.map_mouse_grip) || mouse_btn.pressed(MouseButton::Middle);
        if grip {
            return;
        }
        if input_frame.scroll_delta.abs() >= f32::EPSILON {
            return;
        }
    }

    if active_map_surface.blocks_main_world_map_camera_input() {
        return;
    }

    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    let pointer_blocks_mouse = pointer_over_ui && !cursor_over_sim_map;

    let mut desired = map_camera_desired_from_view_authority(authority.as_ref());

    locals.before_apply = Some(desired.clone());

    if keys.just_pressed(bindings.map_toggle_edge_scroll) {
        settings.edge_scroll_enabled = !settings.edge_scroll_enabled;
    }

    if keys.just_pressed(bindings.map_cycle_camera_mode) {
        settings.mode = settings.mode.next();
    }

    let world_w = params.width as f32;
    let world_h = params.height as f32;
    let center_xy = if params.width > 0 && params.height > 0 {
        Vec3::new(world_w * 0.5, world_h * 0.5, 0.0)
    } else {
        desired.translation
    };

    let mut recenter_pulse = || {
        desired.translation = center_xy;
    };

    if keys.just_pressed(bindings.map_recenter_world) {
        recenter_pulse();
    }
    if mouse_btn.just_pressed(MouseButton::Middle) {
        let now = time.elapsed_secs();
        if let Some(prev) = locals.middle_tap {
            if now - prev < 0.45 {
                recenter_pulse();
                locals.middle_tap = None;
            } else {
                locals.middle_tap = Some(now);
            }
        } else {
            locals.middle_tap = Some(now);
        }
    }

    let default_z = default_map_zoom_for_world(None);
    if keys.just_pressed(bindings.map_reset_zoom) {
        desired.scale = Vec3::splat(default_z);
    }

    let viewport = map_camera_tactical_view_pixels(window_px, sim_viewport.as_ref(), latch.as_ref());
    let world = Vec2::new(world_w, world_h);

    if keys.just_pressed(bindings.map_frame_world) {
        desired.translation = center_xy;
        let margin = 0.9;
        let s = margin * (viewport.x / world_w.max(1.0)).min(viewport.y / world_h.max(1.0));
        desired.scale = Vec3::splat(sanitize_map_zoom_input(s));
    }

    let fast = keys.pressed(bindings.map_pan_fast_modifier);
    let key_speed = KEY_PAN * dt * if fast { 2.2 } else { 1.0 };

    let mut pan = Vec2::ZERO;
    if keys.pressed(bindings.map_pan_west) {
        pan.x -= 1.0;
    }
    if keys.pressed(bindings.map_pan_east) {
        pan.x += 1.0;
    }
    if keys.pressed(bindings.map_pan_north) {
        pan.y += 1.0;
    }
    if keys.pressed(bindings.map_pan_south) {
        pan.y -= 1.0;
    }
    if pan != Vec2::ZERO {
        desired.translation += (pan.normalize() * key_speed).extend(0.0);
    }

    let grip = keys.pressed(bindings.map_mouse_grip) || mouse_btn.pressed(MouseButton::Middle);
    if grip && !pointer_blocks_mouse {
        let sum = input_frame.pointer_delta;
        desired.translation +=
            Vec3::new(-sum.x, sum.y, 0.0) * GRIP_PAN * dt * 0.045 * if fast { 1.35 } else { 1.0 };
    }

    if settings.edge_scroll_enabled && !pointer_blocks_mouse {
        if let Ok(win) = windows.single() {
            let size = Vec2::new(win.width(), win.height());
            if let Some(cursor) = win.cursor_position() {
                let nx = (cursor.x / size.x).clamp(0.0, 1.0);
                let ny = (cursor.y / size.y).clamp(0.0, 1.0);
                let mut edge = Vec2::ZERO;
                if nx < EDGE_FRACTION {
                    edge.x -= 1.0;
                } else if nx > 1.0 - EDGE_FRACTION {
                    edge.x += 1.0;
                }
                if ny < EDGE_FRACTION {
                    edge.y += 1.0;
                } else if ny > 1.0 - EDGE_FRACTION {
                    edge.y -= 1.0;
                }
                if edge != Vec2::ZERO {
                    desired.translation +=
                        (edge.normalize() * EDGE_PAN * dt).extend(0.0) * if fast { 1.4 } else { 1.0 };
                }
            }
        }
    }

    let zoom_key = 4.0 * dt;
    if keys.pressed(bindings.map_zoom_in) {
        let s = sanitize_map_zoom_input(desired.scale.x * (1.0 + zoom_key));
        desired.scale = Vec3::splat(s);
    }
    if keys.pressed(bindings.map_zoom_out) {
        let s = sanitize_map_zoom_input(desired.scale.x / (1.0 + zoom_key));
        desired.scale = Vec3::splat(s);
    }

    if keys.just_pressed(bindings.map_rotate_ccw) {
        desired.rotation *= Quat::from_rotation_z(ROTATE_STEP);
    }
    if keys.just_pressed(bindings.map_rotate_cw) {
        desired.rotation *= Quat::from_rotation_z(-ROTATE_STEP);
    }

    let scale = sanitize_map_zoom_input(desired.scale.x);
    desired.scale = Vec3::splat(scale);
    let clamped = clamp_map_camera_translation_xy(
        Vec2::new(desired.translation.x, desired.translation.y),
        scale,
        world,
        viewport,
    );
    desired.translation.x = clamped.x;
    desired.translation.y = clamped.y;
    desired.translation.z = MAIN_WORLD_CAMERA_Z;

    if locals.before_apply.as_ref() == Some(&desired) {
        return;
    }

    ensure_world_main_authority_bootstrapped(authority.as_mut(), &desired);
    if locals
        .before_apply
        .as_ref()
        .is_some_and(|b| (b.scale.x - desired.scale.x).abs() > 1e-6)
    {
        bevy::log::info!(
            target: "map_camera_zoom",
            "KEY_ZOOM scale {:.4} -> {:.4}",
            locals.before_apply.as_ref().map(|b| b.scale.x).unwrap_or(0.0),
            desired.scale.x
        );
    }
    commit_map_camera_pose_to_view_authority_simple(authority.as_mut(), &desired);
}

/// Returns true when scroll changed the committed map zoom.
#[must_use]
pub fn apply_map_camera_wheel_zoom(
    scroll: f32,
    params: &WorldGenParams,
    sim_viewport: &SimulationMapViewport,
    window_px: Vec2,
    latch: &MainWorldCameraViewportLatch,
    authority: &mut ViewProjectionAuthority,
) -> bool {
    if scroll.abs() < f32::EPSILON {
        return false;
    }
    let world_w = params.width as f32;
    let world_h = params.height as f32;
    if world_w <= 0.0 || world_h <= 0.0 {
        return false;
    }
    let viewport = map_camera_tactical_view_pixels(window_px, sim_viewport, latch);

    let mut desired = map_camera_desired_from_view_authority(authority);
    let before_scale = desired.scale.x;
    let z = ZOOM_FACTOR.powf(scroll.clamp(-24.0, 24.0));
    let scale = sanitize_map_zoom_input(desired.scale.x * z);
    if (scale - before_scale).abs() < 1e-6 {
        return false;
    }
    desired.scale = Vec3::splat(scale);
    let clamped = clamp_map_camera_translation_xy(
        Vec2::new(desired.translation.x, desired.translation.y),
        scale,
        Vec2::new(world_w, world_h),
        viewport,
    );
    desired.translation.x = clamped.x;
    desired.translation.y = clamped.y;
    desired.translation.z = MAIN_WORLD_CAMERA_Z;
    commit_map_camera_pose_to_view_authority_simple(authority, &desired);
    true
}

/// Cursor inside the measured sim-map hole (ignores [`SimulationMapViewport::valid`] — wheel must not freeze while latch settles).
#[inline]
#[must_use]
pub fn cursor_in_sim_map_adequate_aabb(cursor: Vec2, map_vp: &SimulationMapViewport) -> bool {
    if !map_vp.is_adequate_for_camera() {
        return true;
    }
    cursor.x >= map_vp.min.x
        && cursor.x <= map_vp.max.x
        && cursor.y >= map_vp.min.y
        && cursor.y <= map_vp.max.y
}

/// True when tactical map wheel zoom must not run (world preview only).
#[inline]
fn tactical_map_wheel_zoom_blocked(active: &ActiveMapViewInput) -> bool {
    active.blocks_main_world_map_camera_input()
}

/// Single scroll source for tactical wheel zoom (Update — OS accumulators + InputFrame mirror).
#[inline]
fn resolve_tactical_map_wheel_scroll(
    scroll_acc: &AccumulatedMouseScroll,
    input_frame: &InputFrame,
    wheel_events: &mut MessageReader<MouseWheel>,
) -> f32 {
    let os = scroll_acc.delta.y + scroll_acc.delta.x * 0.25;
    if os.abs() >= f32::EPSILON {
        return os;
    }
    if input_frame.scroll_delta.abs() >= f32::EPSILON {
        return input_frame.scroll_delta;
    }
    let mut sum = 0.0f32;
    for ev in wheel_events.read() {
        sum += ev.y + ev.x * 0.25;
    }
    sum
}

/// Sole tactical map wheel zoom — runs in Update after pan/keys, before derive.
fn map_camera_wheel_zoom_system(
    state: Res<State<BaseState>>,
    active_map_surface: Res<ActiveMapViewInput>,
    latch: Res<MainWorldCameraViewportLatch>,
    params: Res<WorldGenParams>,
    sim_viewport: Res<SimulationMapViewport>,
    desired: Res<MapCameraDesired>,
    windows: Query<&Window, With<PrimaryWindow>>,
    scroll_acc: Res<AccumulatedMouseScroll>,
    input_frame: Res<InputFrame>,
    mut wheel_events: MessageReader<MouseWheel>,
    mut authority: ResMut<ViewProjectionAuthority>,
) {
    let _perf = crate::render::PerfScope::new("map_camera_wheel");
    if !matches!(state.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }
    let scroll = resolve_tactical_map_wheel_scroll(
        scroll_acc.as_ref(),
        input_frame.as_ref(),
        &mut wheel_events,
    );
    let blocked = tactical_map_wheel_zoom_blocked(active_map_surface.as_ref());
    if blocked {
        if scroll.abs() > f32::EPSILON {
            bevy::log::warn!(
                target: "map_camera_zoom",
                "WHEEL_BLOCKED scroll={scroll:.3} active={:?}",
                active_map_surface.0,
            );
        }
        return;
    }
    if scroll.abs() < f32::EPSILON {
        return;
    }

    ensure_world_main_authority_bootstrapped(authority.as_mut(), desired.as_ref());
    let window_px = primary_window_logical_px(&windows);
    let before = authority
        .surface(crate::render::view_runtime::ViewSurfaceId::WorldMain)
        .map(|s| s.camera.zoom)
        .unwrap_or(0.0);
    if apply_map_camera_wheel_zoom(
        scroll,
        params.as_ref(),
        sim_viewport.as_ref(),
        window_px,
        latch.as_ref(),
        authority.as_mut(),
    ) {
        let after = authority
            .surface(crate::render::view_runtime::ViewSurfaceId::WorldMain)
            .map(|s| s.camera.zoom)
            .unwrap_or(0.0);
        bevy::log::info!(
            target: "map_camera_zoom",
            "WHEEL_APPLIED scroll={scroll:.3} zoom {before:.4} -> {after:.4}"
        );
    }
}

/// Advance hole scissor latch once per frame ([`sync_main_world_camera_viewport_and_projection`] may run twice).
pub fn advance_main_world_camera_viewport_latch(
    sim: Res<SimulationMapViewport>,
    mut latch: ResMut<MainWorldCameraViewportLatch>,
    frame: Res<FrameCount>,
    mut last_advanced: Local<u32>,
) {
    let f = frame.0;
    if *last_advanced == f {
        return;
    }
    *last_advanced = f;
    let was = latch.using_hole;
    latch.advance(sim.is_adequate_for_camera());
    if TACTICAL_MAP_FULL_WINDOW_RENDER {
        latch.using_hole = false;
    }
    if latch.using_hole != was {
        crate::gui::hud::trace_viewport_authority(
            crate::gui::hud::ViewportAuthoritySource::CameraLatch,
            sim.min,
            sim.max,
            latch.using_hole,
        );
    }
}

/// RTS input → authority without trace param (keeps system under Bevy param limit).
fn commit_map_camera_pose_to_view_authority_simple(
    authority: &mut ViewProjectionAuthority,
    desired: &MapCameraDesired,
) {
    use crate::render::view_runtime::ViewAuthorityWriter;
    use crate::render::view_runtime::ViewSurfaceId;

    let cam = crate::gui::view_authority::view_camera_state_from_map_camera_desired(desired);
    authority.commit_pose(ViewSurfaceId::WorldMain, cam, ViewAuthorityWriter::MapCameraInput);
    authority.commit_pose(
        ViewSurfaceId::SimulationMap,
        cam,
        ViewAuthorityWriter::MapCameraInput,
    );
}

/// Seed WorldMain when RTS input runs before the view bridge has published a surface.
fn ensure_world_main_authority_bootstrapped(
    authority: &mut ViewProjectionAuthority,
    fallback: &MapCameraDesired,
) {
    use crate::render::view_runtime::{ViewAuthorityWriter, ViewSurfaceId};
    if authority.surface(ViewSurfaceId::WorldMain).is_some() {
        return;
    }
    let cam = crate::gui::view_authority::view_camera_state_from_map_camera_desired(fallback);
    authority.commit_pose(ViewSurfaceId::WorldMain, cam, ViewAuthorityWriter::MapCameraInput);
    authority.commit_pose(
        ViewSurfaceId::SimulationMap,
        cam,
        ViewAuthorityWriter::MapCameraInput,
    );
    bevy::log::warn!(
        target: "map_camera_zoom",
        "BOOTSTRAP WorldMain from MapCameraDesired zoom={:.4}",
        fallback.scale.x
    );
}

/// **TRIAGE-VM-09-v2:** sole production writer to [`MapCameraDesired`] — mirror from WorldMain authority.
pub fn derive_map_camera_desired_from_view_authority(
    authority: Res<ViewProjectionAuthority>,
    mut desired: ResMut<MapCameraDesired>,
    profile: Res<Stage5ReadinessProfile>,
) {
    // PERF-INSTR-VFX-001: name this system inside the `map_cam` wall bracket (STALL/PERF only).
    let _perf = crate::render::PerfScope::new("upd_map_camera_derive");
    use crate::render::view_runtime::ViewSurfaceId;
    let Some(cam) = authority
        .surface(ViewSurfaceId::WorldMain)
        .map(|s| s.camera)
    else {
        // Never wipe desired with Default when authority has not bootstrapped WorldMain yet.
        return;
    };
    let before = desired.clone();
    let next = MapCameraDesired {
        translation: Vec3::new(cam.translation.x, cam.translation.y, MAIN_WORLD_CAMERA_Z),
        scale: Vec3::splat(cam.zoom.max(1e-4)),
        rotation: Quat::from_rotation_z(cam.rotation),
    };
    if *desired == next {
        return;
    }
    *desired = next;
    trace_map_camera_desired_write_if_full_app(
        profile.as_ref(),
        "derive_map_camera_desired_from_view_authority",
        &before,
        desired.as_ref(),
    );
}

/// VM-09-v2 compat alias — authority → desired (no longer desired → authority).
pub fn mirror_world_main_camera_from_map_desired(
    authority: Res<ViewProjectionAuthority>,
    desired: ResMut<MapCameraDesired>,
    profile: Res<Stage5ReadinessProfile>,
) {
    derive_map_camera_desired_from_view_authority(authority, desired, profile);
}

/// Apply authority zoom → orthographic projection + pan (shared Update + PostUpdate).
fn apply_main_world_camera_ortho_core(
    camera: &mut Camera,
    tf: &mut Transform,
    proj: &mut Projection,
    window_px: Vec2,
    sim: &SimulationMapViewport,
    latch: &MainWorldCameraViewportLatch,
    authority: &ViewProjectionAuthority,
    desired: &MapCameraDesired,
    params: &WorldGenParams,
    ortho_trace: &mut MainWorldCameraOrthoTrace,
    zoom_audit: &mut MapCameraZoomAudit,
) {
    use crate::render::view_runtime::ViewSurfaceId;

    // Full-window render — subrect scissor leaves burnt-in prior frames in swapchain margins.
    camera.viewport = None;

    let view_px = map_camera_tactical_view_pixels(window_px, sim, latch);
    let auth_cam = authority.surface(ViewSurfaceId::WorldMain).map(|s| s.camera);
    let auth_zoom = auth_cam.map(|c| c.zoom).unwrap_or(desired.scale.x);
    let zoom = auth_zoom.max(1e-4);

    if let Some(cam) = auth_cam {
        tf.translation = Vec3::new(cam.translation.x, cam.translation.y, MAIN_WORLD_CAMERA_Z);
    } else {
        tf.translation = desired.translation;
    }

    let world_w = params.width as f32;
    let world_h = params.height as f32;
    let (fixed_w, fixed_h) = orthographic_fixed_world_span(view_px, zoom, world_w, world_h);

    ortho_trace.fixed_width = fixed_w;
    ortho_trace.fixed_height = fixed_h;
    ortho_trace.view_pixels = view_px;
    ortho_trace.using_hole = latch.using_hole && sim.is_adequate_for_camera();
    ortho_trace.use_hole_scissor = false;
    ortho_trace.authority_zoom = auth_zoom;
    ortho_trace.desired_zoom = desired.scale.x;

    zoom_audit.ortho_fixed_w = fixed_w;
    zoom_audit.ortho_fixed_h = fixed_h;
    zoom_audit.view_px = view_px;
    zoom_audit.use_hole_scissor = false;
    zoom_audit.latch_using_hole = latch.using_hole;
    zoom_audit.authority_zoom = auth_zoom;
    zoom_audit.desired_zoom = desired.scale.x;

    camera.clear_color = ClearColorConfig::Custom(Color::srgba(0.04, 0.05, 0.07, 1.0));

    let Projection::Orthographic(ref mut ortho) = *proj else {
        return;
    };
    tf.scale = Vec3::ONE;
    ortho.scale = 1.0;
    ortho.scaling_mode = bevy::camera::ScalingMode::Fixed {
        width: fixed_w,
        height: fixed_h,
    };
    ortho.viewport_origin = Vec2::new(0.5, 0.5);
}

/// Update: apply ortho immediately after input/smooth (same frame as wheel/keys).
fn apply_main_world_camera_ortho_immediate(
    sim: Res<SimulationMapViewport>,
    authority: Res<ViewProjectionAuthority>,
    desired: Res<MapCameraDesired>,
    params: Res<WorldGenParams>,
    windows: Query<&Window, With<PrimaryWindow>>,
    latch: Res<MainWorldCameraViewportLatch>,
    mut ortho_trace: ResMut<MainWorldCameraOrthoTrace>,
    mut zoom_audit: ResMut<MapCameraZoomAudit>,
    mut q: Query<(&mut Camera, &mut Transform, &mut Projection), With<MainWorldCamera>>,
) {
    let Ok((mut camera, mut tf, mut proj)) = q.single_mut() else {
        return;
    };
    let Ok(win) = windows.single() else {
        camera.is_active = false;
        return;
    };
    if !crate::render::primary_window_logical_presentable(win.width(), win.height()) {
        camera.is_active = false;
        return;
    }
    camera.is_active = true;
    let window_px = Vec2::new(win.width().max(1.0), win.height().max(1.0));
    apply_main_world_camera_ortho_core(
        &mut camera,
        &mut tf,
        &mut proj,
        window_px,
        sim.as_ref(),
        latch.as_ref(),
        authority.as_ref(),
        desired.as_ref(),
        params.as_ref(),
        ortho_trace.as_mut(),
        zoom_audit.as_mut(),
    );
}

/// After Bevy UI layout: apply scissor + orthographic fit from the **same** viewport decision.
pub fn sync_main_world_camera_viewport_and_projection(
    sim: Res<SimulationMapViewport>,
    authority: Res<ViewProjectionAuthority>,
    desired: Res<MapCameraDesired>,
    params: Res<WorldGenParams>,
    windows: Query<&Window, With<PrimaryWindow>>,
    latch: Res<MainWorldCameraViewportLatch>,
    mut ortho_trace: ResMut<MainWorldCameraOrthoTrace>,
    mut zoom_audit: ResMut<MapCameraZoomAudit>,
    cfg: Option<Res<DebugRenderTraceConfig>>,
    mut q: Query<(&mut Camera, &mut Transform, &mut Projection), With<MainWorldCamera>>,
) {
    let Ok((mut camera, mut tf, mut proj)) = q.single_mut() else {
        return;
    };
    let Ok(win) = windows.single() else {
        camera.viewport = None;
        camera.is_active = false;
        return;
    };

    if !crate::render::primary_window_logical_presentable(win.width(), win.height()) {
        camera.viewport = None;
        camera.is_active = false;
        return;
    }
    camera.is_active = true;

    let window_px = Vec2::new(win.width().max(1.0), win.height().max(1.0));
    apply_main_world_camera_ortho_core(
        &mut camera,
        &mut tf,
        &mut proj,
        window_px,
        sim.as_ref(),
        latch.as_ref(),
        authority.as_ref(),
        desired.as_ref(),
        params.as_ref(),
        ortho_trace.as_mut(),
        zoom_audit.as_mut(),
    );

    crate::gui::hud::trace_viewport_authority(
        crate::gui::hud::ViewportAuthoritySource::CameraApplied,
        Vec2::ZERO,
        window_px,
        false,
    );

    if cfg.as_ref().is_some_and(|c| c.camera_sync_trace) {
        trace_camera_sync(
            cfg.as_ref().unwrap(),
            &format!(
                "full_window view_px=({:.0},{:.0}) fixed=({:.1},{:.1}) zoom={:.3} auth_z={:.3} des_z={:.3}",
                ortho_trace.view_pixels.x,
                ortho_trace.view_pixels.y,
                ortho_trace.fixed_width,
                ortho_trace.fixed_height,
                ortho_trace.authority_zoom,
                ortho_trace.authority_zoom,
                ortho_trace.desired_zoom
            ),
        );
    }
}

/// End-of-frame zoom audit — logs drift between authority, desired, and ortho fit.
pub fn map_camera_zoom_audit_system(
    frame: Res<FrameCount>,
    authority: Res<ViewProjectionAuthority>,
    desired: Res<MapCameraDesired>,
    ortho: Res<MainWorldCameraOrthoTrace>,
    latch: Res<MainWorldCameraViewportLatch>,
    mut audit: ResMut<MapCameraZoomAudit>,
) {
    use crate::render::view_runtime::ViewSurfaceId;
    audit.frame = frame.0;
    audit.authority_zoom = authority
        .surface(ViewSurfaceId::WorldMain)
        .map(|s| s.camera.zoom)
        .unwrap_or(0.0);
    audit.desired_zoom = desired.scale.x;
    audit.ortho_fixed_w = ortho.fixed_width;
    audit.ortho_fixed_h = ortho.fixed_height;
    audit.view_px = ortho.view_pixels;
    audit.use_hole_scissor = ortho.use_hole_scissor;
    audit.latch_using_hole = latch.using_hole;

    let auth_des_drift = (audit.authority_zoom - audit.desired_zoom).abs();
    if auth_des_drift > 0.02 {
        bevy::log::warn!(
            target: "map_camera_zoom",
            "AUTH_DESIRED_DRIFT frame={} auth={:.4} desired={:.4}",
            frame.0,
            audit.authority_zoom,
            audit.desired_zoom
        );
    }

    if latch.using_hole && !ortho.use_hole_scissor {
        bevy::log::warn!(
            target: "map_camera_zoom",
            "SCISSOR_HEAL frame={} latch_hole=true but rendering full window (ghost risk)",
            frame.0
        );
    }

    bevy::log::debug!(
        target: "map_camera_zoom",
        "FRAME frame={} auth_z={:.4} des_z={:.4} fixed=({:.1},{:.1}) view_px=({:.0},{:.0}) latch_hole={} scissor={}",
        frame.0,
        audit.authority_zoom,
        audit.desired_zoom,
        audit.ortho_fixed_w,
        audit.ortho_fixed_h,
        audit.view_px.x,
        audit.view_px.y,
        latch.using_hole,
        ortho.use_hole_scissor
    );
}

fn map_camera_smooth_toward_desired(
    cfg: Res<DebugRenderTraceConfig>,
    time: Res<Time>,
    state: Res<State<BaseState>>,
    desired: Res<MapCameraDesired>,
    params: Res<WorldGenParams>,
    windows: Query<&Window, With<PrimaryWindow>>,
    sim_viewport: Res<SimulationMapViewport>,
    latch: Res<MainWorldCameraViewportLatch>,
    mut q_cam: Query<&mut Transform, With<MainWorldCamera>>,
    mut last_desired_scale: Local<f32>,
    mut last_trace: Local<u64>,
) {
    // PERF-INSTR-VFX-001: name this system inside the `map_cam` wall bracket (STALL/PERF only).
    let _perf = crate::render::PerfScope::new("upd_map_camera_smooth");
    if !matches!(state.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }
    let dt = time.delta_secs().clamp(0.0, MAX_CAMERA_SMOOTH_DT_SECS);
    if dt <= 0.0 {
        return;
    }
    let zoom_axis_changed =
        (*last_desired_scale - desired.scale.x).abs() > MAP_ZOOM_AXIS_SNAP_EPS;
    *last_desired_scale = desired.scale.x;
    let Ok(mut xf) = q_cam.single_mut() else {
        return;
    };
    if !zoom_axis_changed {
        let pan_rest = xf
            .translation
            .truncate()
            .distance_squared(desired.translation.truncate())
            <= MAP_CAMERA_AT_REST_EPS * MAP_CAMERA_AT_REST_EPS;
        let rot_rest = xf.rotation.dot(desired.rotation).abs() >= 1.0 - 1.0e-5;
        if pan_rest && rot_rest {
            xf.scale = Vec3::ONE;
            return;
        }
    }
    let k = 1.0 - (-dt * SMOOTH_LAMBDA).exp();
    if zoom_axis_changed {
        xf.translation = desired.translation;
        xf.rotation = desired.rotation;
    } else {
        xf.translation = xf.translation.lerp(desired.translation, k);
        xf.rotation = xf.rotation.slerp(desired.rotation, k);
    }
    xf.scale = Vec3::ONE;
    {
        let window_px = primary_window_logical_px(&windows);
        let viewport = map_camera_tactical_view_pixels(
            window_px,
            sim_viewport.as_ref(),
            latch.as_ref(),
        );
        let world_size = Vec2::new(params.width as f32, params.height as f32);
        if world_size.x > 0.0 && world_size.y > 0.0 {
            let scale = desired.scale.x;
            let c = clamp_map_camera_translation_xy(
                Vec2::new(xf.translation.x, xf.translation.y),
                scale,
                world_size,
                viewport,
            );
            xf.translation.x = c.x;
            xf.translation.y = c.y;
        }
    }
    if cfg.camera_sync_trace {
        *last_trace = last_trace.wrapping_add(1);
        if *last_trace % 30 == 0 {
            let win = windows
                .single()
                .map(|w| format!("{:.0}x{:.0}", w.width(), w.height()))
                .unwrap_or_else(|_| "unknown".into());
            trace_camera_sync(
                &cfg,
                &format!(
                    "main_world_camera desired=({:.1},{:.1}) scale={:.3} transform=({:.1},{:.1}) scale={:.3} window={win}",
                    desired.translation.x,
                    desired.translation.y,
                    desired.scale.x,
                    xf.translation.x,
                    xf.translation.y,
                    xf.scale.x,
                ),
            );
        }
    }
}

/// Primary-window cursor (**logical** px) → world **XY** on the `z = 0` plane for orthographic [`Camera2d`].
#[must_use]
pub fn primary_cursor_world_xy(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    _window: &Window,
    cursor_logical: Vec2,
) -> Option<Vec2> {
    let ray = camera
        .viewport_to_world(camera_transform, cursor_logical)
        .ok()?;
    let o = ray.origin;
    let d = ray.direction;
    if d.z.abs() < 1e-5 {
        return None;
    }
    let t = -o.z / d.z;
    if !t.is_finite() {
        return None;
    }
    let p = o + *d * t;
    Some(Vec2::new(p.x, p.y))
}

#[must_use]
pub fn sim_map_image_rect(map_vp: &SimulationMapViewport) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2(map_vp.min.x, map_vp.min.y),
        egui::pos2(map_vp.max.x, map_vp.max.y),
    )
}

/// Visible world span for the map hole — matches [`orthographic_fixed_world_span`] on [`MainWorldCamera`].
#[must_use]
pub fn sim_map_visible_world_span(
    map_vp: &SimulationMapViewport,
    zoom: f32,
    world_w: f32,
    world_h: f32,
) -> (f32, f32) {
    orthographic_fixed_world_span(map_vp.logical_size(), zoom.max(1e-6), world_w, world_h)
}

/// World XY → egui screen inside the simulation map hole (aspect matches camera ortho, not isotropic zoom).
#[must_use]
pub fn sim_map_world_xy_to_egui(
    world_xy: Vec2,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    world_w: f32,
    world_h: f32,
) -> Option<egui::Pos2> {
    if !map_vp.is_adequate_for_camera() {
        return None;
    }
    let rect = sim_map_image_rect(map_vp);
    let zoom = desired.scale.x.abs().max(1e-6);
    let cam = desired.translation.truncate();
    let (fw, fh) = sim_map_visible_world_span(map_vp, zoom, world_w, world_h);
    let sx = rect.center().x + (world_xy.x - cam.x) * (rect.width() / fw.max(1e-6));
    let sy = rect.center().y + (world_xy.y - cam.y) * (rect.height() / fh.max(1e-6));
    Some(egui::pos2(sx, sy))
}

/// Inverse of [`sim_map_world_xy_to_egui`] — logical cursor in the map hole → world XY.
#[must_use]
pub fn sim_map_screen_to_world_xy(
    screen_logical: Vec2,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    world_w: f32,
    world_h: f32,
) -> Option<Vec2> {
    if !map_vp.is_adequate_for_camera() {
        return None;
    }
    let rect = sim_map_image_rect(map_vp);
    let zoom = desired.scale.x.abs().max(1e-6);
    let cam = desired.translation.truncate();
    let (fw, fh) = sim_map_visible_world_span(map_vp, zoom, world_w, world_h);
    let dx = (screen_logical.x - rect.center().x) * fw / rect.width().max(1.0);
    let dy = (screen_logical.y - rect.center().y) * fh / rect.height().max(1.0);
    Some(Vec2::new(cam.x + dx, cam.y + dy))
}

/// Primary-window cursor → world XY on the tactical map (prefer over [`primary_cursor_world_xy`] when the camera uses a viewport hole).
#[must_use]
pub fn sim_map_cursor_world_xy(
    cursor_logical: Vec2,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    world_w: f32,
    world_h: f32,
) -> Option<Vec2> {
    if map_vp.valid && !map_vp.contains_cursor(cursor_logical) {
        return None;
    }
    sim_map_screen_to_world_xy(cursor_logical, desired, map_vp, world_w, world_h)
}

/// Horizontal XY on the tactical map plane from a world [`Vec3`].
///
/// Construction path tools use `Vec3(x, 0, row)` (grid row in **Z**); the live map camera and
/// fallback raster use **XY** (`row` ↔ world **Y**). When `y ≈ 0`, row is read from `z`.
#[inline]
#[must_use]
pub fn map_plane_horizontal_xy(world: Vec3) -> Vec2 {
    if world.y.abs() < 1e-4 {
        Vec2::new(world.x, world.z)
    } else {
        Vec2::new(world.x, world.y)
    }
}

/// [`sim_map_world_xy_to_egui`] for world-space [`Vec3`] via [`map_plane_horizontal_xy`].
#[must_use]
pub fn sim_map_world_vec3_to_egui(
    world: Vec3,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    world_w: f32,
    world_h: f32,
) -> Option<egui::Pos2> {
    sim_map_world_xy_to_egui(map_plane_horizontal_xy(world), desired, map_vp, world_w, world_h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_map_camera_wheel_zoom_commits_authority() {
        use crate::render::view_runtime::{ViewAuthorityWriter, ViewProjectionAuthority, ViewSurfaceId};

        let mut authority = ViewProjectionAuthority::default();
        authority.commit_pose(
            ViewSurfaceId::WorldMain,
            crate::gui::ViewCameraState {
                translation: Vec2::new(100.0, 200.0),
                zoom: 1.0,
                rotation: 0.0,
            },
            ViewAuthorityWriter::MapCameraInput,
        );
        let mut vp = SimulationMapViewport::default();
        vp.valid = true;
        vp.min = Vec2::ZERO;
        vp.max = Vec2::new(800.0, 600.0);
        let params = WorldGenParams {
            width: 512,
            height: 512,
            ..Default::default()
        };
        let latch = MainWorldCameraViewportLatch::default();
        assert!(apply_map_camera_wheel_zoom(
            1.0,
            &params,
            &vp,
            Vec2::new(800.0, 600.0),
            &latch,
            &mut authority,
        ));
        let z = authority
            .surface(ViewSurfaceId::WorldMain)
            .expect("WorldMain")
            .camera
            .zoom;
        assert!(z > 1.0, "wheel must increase committed zoom, got {z}");
    }

    #[test]
    fn sim_map_projection_matches_ortho_aspect() {
        let mut vp = SimulationMapViewport::default();
        vp.valid = true;
        vp.min = Vec2::new(100.0, 50.0);
        vp.max = Vec2::new(900.0, 550.0);
        let desired = MapCameraDesired {
            translation: Vec3::new(2048.0, 2048.0, 0.0),
            scale: Vec3::splat(2.0),
            ..Default::default()
        };
        let world = Vec2::new(2048.0, 2048.0);
        let screen = sim_map_world_xy_to_egui(world, &desired, &vp, 4096.0, 4096.0).unwrap();
        let rect = sim_map_image_rect(&vp);
        assert!((screen.x - rect.center().x).abs() < 2.0);
        assert!((screen.y - rect.center().y).abs() < 2.0);
        let back = sim_map_screen_to_world_xy(Vec2::new(screen.x, screen.y), &desired, &vp, 4096.0, 4096.0)
            .unwrap();
        assert!((back.x - world.x).abs() < 0.5);
        assert!((back.y - world.y).abs() < 0.5);
    }

    #[test]
    fn clamp_keeps_visible_rect_inside_world() {
        let world = Vec2::new(200.0, 100.0);
        let viewport = Vec2::new(400.0, 150.0);
        let scale = 2.0;
        let half = map_visible_half_extents(scale, viewport, world);
        let c = clamp_map_camera_translation_xy(Vec2::new(-50.0, 500.0), scale, world, viewport);
        assert!((c.x - (world.x - half.x)).abs() < 1e-3);
        assert!((c.y - (world.y - half.y)).abs() < 1e-3);
    }

    #[test]
    fn zoom_limits_allow_deep_zoom_on_large_world() {
        let world = Vec2::new(4096.0, 4096.0);
        let viewport = Vec2::new(1280.0, 720.0);
        let (lo, hi) = map_zoom_limits_for_world(world.x, world.y, viewport);
        assert!(hi > MAP_ZOOM_CLAMP.1);
        assert!(hi > 80.0, "4096² worlds must allow tile-level zoom, got hi={hi}");
        assert!(lo < hi);
    }

    #[test]
    fn clamp_allows_pan_when_zoomed_in() {
        let world = Vec2::new(512.0, 512.0);
        let viewport = Vec2::new(1280.0, 720.0);
        let scale = 40.0;
        let half = map_visible_half_extents(scale, viewport, world);
        assert!(half.x < world.x * 0.5);
        let c = clamp_map_camera_translation_xy(Vec2::new(100.0, 400.0), scale, world, viewport);
        assert!((c.x - 100.0).abs() < 1e-3);
        assert!((c.y - 400.0).abs() < 1e-3);
    }

    #[test]
    fn orthographic_fixed_world_span_matches_viewport_aspect() {
        let view = Vec2::new(800.0, 400.0);
        let (w, h) = orthographic_fixed_world_span(view, 2.0, 4096.0, 4096.0);
        assert!((w - 400.0).abs() < 1e-3);
        assert!((h - 200.0).abs() < 1e-3);
        assert!((w / h - 2.0).abs() < 1e-3);
    }

    #[test]
    fn map_plane_horizontal_xy_reads_row_from_z_for_path_tools() {
        let p = Vec3::new(10.0, 0.0, 42.0);
        let xy = map_plane_horizontal_xy(p);
        assert!((xy.x - 10.0).abs() < 1e-5);
        assert!((xy.y - 42.0).abs() < 1e-5);
    }
}
