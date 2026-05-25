//! Pan / zoom / edge-scroll / rotate for [`MainWorldCamera`] using [`InputBindings`](crate::gui::InputBindings).
//!
//! **P1-F:** Input writes [`MapCameraDesired`]; a follow-up system lerps [`Transform`] toward it (smoothing).
//! Skips input while egui wants the pointer or keyboard. Edge pan respects [`MapCameraSettings::edge_scroll_enabled`].
//!
//! **vm-07:** When [`crate::gui::ActiveMapViewInput`] names [`crate::gui::MapViewInstanceId::WorldPreview`] or
//! [`crate::gui::MapViewInstanceId::Minimap`], keyboard / edge / grip / scroll zoom are skipped so auxiliary
//! map surfaces do not fight the main tactical camera.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui;

use bevy_egui::EguiContexts;

use crate::engine::states::BaseState;
use crate::engine::{ActiveTestScene, TestScene};
use crate::gui::view_authority::commit_map_camera_pose_to_view_authority;
use crate::gui::ActiveMapViewInput;
use crate::gui::{SimulationMapViewport, SimulationViewportSyncSet};
use crate::gui::InputBindings;
use crate::gui::InputFrame;
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
}

const CAM_HOLE_VALID_STREAK: u8 = 1;

impl MainWorldCameraViewportLatch {
    /// Returns whether the camera should use the sim-map hole scissor this frame.
    pub fn advance(&mut self, sim_adequate: bool) -> bool {
        if sim_adequate {
            self.valid_streak = self.valid_streak.saturating_add(1);
            self.invalid_streak = 0;
            if !self.using_hole && self.valid_streak >= CAM_HOLE_VALID_STREAK {
                self.using_hole = true;
            }
        } else {
            self.invalid_streak = self.invalid_streak.saturating_add(1);
            self.valid_streak = 0;
            // Release on first inadequate frame — delayed release caused scissor/ortho mismatch blink.
            self.using_hole = false;
        }
        self.using_hole
    }
}

/// Last orthographic fit written by [`sync_main_world_camera_viewport_and_projection`] (debug).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MainWorldCameraOrthoTrace {
    pub fixed_width: f32,
    pub fixed_height: f32,
    pub view_pixels: Vec2,
    pub using_hole: bool,
}

/// Logical pose after map input (smoothing target).
///
/// **vm-09b:** RTS input mutates this resource, then [`commit_map_camera_pose_to_view_authority`]
/// commits pose; [`crate::gui::ViewAuthoritySystemSet::SyncViewManager`] rebuilds [`crate::gui::ViewManager`].
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
    synced_from_cam: bool,
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
                MapCameraSystemSet::Smooth.after(MapCameraSystemSet::ApplyInput),
            )
            .add_systems(
                Update,
                map_camera_apply_input_to_desired
                    .in_set(MapCameraSystemSet::ApplyInput)
                    .run_if(in_simulation_or_editor_map),
            )
            .add_systems(
                Update,
                mirror_world_main_camera_from_map_desired
                    .after(map_camera_apply_input_to_desired)
                    .in_set(MapCameraSystemSet::ApplyInput)
                    .run_if(in_simulation_or_editor_map),
            )
            .init_resource::<MainWorldCameraViewportLatch>()
            .init_resource::<MainWorldCameraOrthoTrace>()
            .add_systems(
                Update,
                map_camera_smooth_toward_desired
                    .in_set(MapCameraSystemSet::Smooth)
                    .run_if(in_simulation_or_editor_map),
            )
            .add_systems(
                PostUpdate,
                sync_main_world_camera_viewport_and_projection
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
const ZOOM_FACTOR: f32 = 1.08;
/// Fallback zoom limits when viewport/world size is unknown.
pub const MAP_ZOOM_CLAMP: (f32, f32) = (0.35, 4.5);

/// Logical pixel size for map camera math — prefer the simulation map viewport hole when valid.
#[must_use]
pub fn map_camera_viewport_pixels(
    window: Vec2,
    map_viewport: Option<&SimulationMapViewport>,
) -> Vec2 {
    if let Some(vp) = map_viewport {
        if vp.is_adequate_for_camera() {
            return vp.logical_size();
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
    let lo = (fit * 0.18).clamp(0.06, fit.max(0.12));
    // Allow zoom until ~8 tiles span the shorter viewport edge (tile inspection on 4k+ worlds).
    let min_span_tiles = 8.0_f32;
    let hi_from_tile_span = vp.x.min(vp.y) / min_span_tiles;
    let hi = hi_from_tile_span
        .max(fit * 14.0)
        .max(8.0)
        .min(2048.0);
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
    mut contexts: EguiContexts,
    params: Res<WorldGenParams>,
    active_map_surface: Res<ActiveMapViewInput>,
    mut settings: ResMut<MapCameraSettings>,
    mut desired: ResMut<MapCameraDesired>,
    q_cam: Query<&Transform, With<MainWorldCamera>>,
    profile: Res<Stage5ReadinessProfile>,
    mut locals: Local<MapCameraInputLocals>,
) {
    locals.before_apply = None;

    if !matches!(state.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }

    if state.is_changed() {
        locals.synced_from_cam = false;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let pointer_over_ui = ctx.wants_pointer_input() || ctx.wants_keyboard_input();
    if pointer_over_ui {
        return;
    }

    if active_map_surface.blocks_main_world_map_camera_input() {
        return;
    }

    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    locals.before_apply = Some(desired.clone());

    if !locals.synced_from_cam {
        if let Ok(t) = q_cam.single() {
            desired.translation = t.translation;
            desired.scale = t.scale;
            desired.rotation = t.rotation;
            locals.synced_from_cam = true;
        }
    }

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

    let window_px = windows
        .single()
        .map(|w| Vec2::new(w.width().max(1.0), w.height().max(1.0)))
        .unwrap_or(Vec2::new(1280.0, 720.0));
    let viewport = map_camera_viewport_pixels(window_px, Some(sim_viewport.as_ref()));
    let world = Vec2::new(world_w, world_h);
    let (zoom_lo, zoom_hi) = map_zoom_limits_for_world(world_w, world_h, viewport);

    if keys.just_pressed(bindings.map_frame_world) {
        desired.translation = center_xy;
        let margin = 0.9;
        let s = margin * (viewport.x / world_w.max(1.0)).min(viewport.y / world_h.max(1.0));
        desired.scale = Vec3::splat(s.clamp(zoom_lo, zoom_hi));
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
    if grip {
        let sum = input_frame.pointer_delta;
        desired.translation +=
            Vec3::new(-sum.x, sum.y, 0.0) * GRIP_PAN * dt * 0.045 * if fast { 1.35 } else { 1.0 };
    }

    if settings.edge_scroll_enabled {
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

    let scroll = input_frame.scroll_delta;
    if scroll.abs() >= f32::EPSILON {
        let z = ZOOM_FACTOR.powf(scroll.clamp(-5.0, 5.0));
        let s = (desired.scale.x * z).clamp(zoom_lo, zoom_hi);
        desired.scale = Vec3::splat(s);
    }

    let zoom_key = 1.65 * dt;
    if keys.pressed(bindings.map_zoom_in) {
        let s = (desired.scale.x * (1.0 + zoom_key)).clamp(zoom_lo, zoom_hi);
        desired.scale = Vec3::splat(s);
    }
    if keys.pressed(bindings.map_zoom_out) {
        let s = (desired.scale.x / (1.0 + zoom_key)).clamp(zoom_lo, zoom_hi);
        desired.scale = Vec3::splat(s);
    }

    if keys.just_pressed(bindings.map_rotate_ccw) {
        desired.rotation *= Quat::from_rotation_z(ROTATE_STEP);
    }
    if keys.just_pressed(bindings.map_rotate_cw) {
        desired.rotation *= Quat::from_rotation_z(-ROTATE_STEP);
    }

    let scale = desired.scale.x.clamp(zoom_lo, zoom_hi);
    desired.scale = Vec3::splat(scale);
    let clamped = clamp_map_camera_translation_xy(
        Vec2::new(desired.translation.x, desired.translation.y),
        scale,
        world,
        viewport,
    );
    desired.translation.x = clamped.x;
    desired.translation.y = clamped.y;
    desired.translation.z = 999.0;

    if let Some(before) = locals.before_apply.take() {
        trace_map_camera_desired_write_if_full_app(
            profile.as_ref(),
            "map_camera_apply_input_to_desired",
            &before,
            desired.as_ref(),
        );
    }
}

/// VM-C C1 / VM-06: commit RTS pose to [`ViewProjectionAuthority`] only.
/// [`ViewManager`] is rebuilt in [`crate::gui::ViewAuthoritySystemSet::SyncViewManager`].
pub fn mirror_world_main_camera_from_map_desired(
    desired: Res<MapCameraDesired>,
    mut authority: ResMut<crate::render::view_runtime::ViewProjectionAuthority>,
    mut trace: ResMut<crate::render::view_runtime::ViewRuntimeTrace>,
) {
    commit_map_camera_pose_to_view_authority(authority.as_mut(), trace.as_mut(), desired.as_ref());
}

/// After Bevy UI layout: apply scissor + orthographic fit from the **same** viewport decision.
pub fn sync_main_world_camera_viewport_and_projection(
    sim: Res<SimulationMapViewport>,
    desired: Res<MapCameraDesired>,
    params: Res<WorldGenParams>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut latch: ResMut<MainWorldCameraViewportLatch>,
    mut ortho_trace: ResMut<MainWorldCameraOrthoTrace>,
    cfg: Option<Res<DebugRenderTraceConfig>>,
    mut q: Query<(&mut Camera, &mut Transform, &mut Projection), With<MainWorldCamera>>,
) {
    let Ok((mut camera, mut tf, mut proj)) = q.single_mut() else {
        return;
    };
    let Ok(win) = windows.single() else {
        camera.viewport = None;
        return;
    };

    let window_px = Vec2::new(win.width().max(1.0), win.height().max(1.0));
    let sim_adequate = sim.is_adequate_for_camera();
    let latch_was = latch.using_hole;
    let render_hole = latch.advance(sim_adequate);
    if latch.using_hole != latch_was {
        crate::gui::hud::trace_viewport_authority(
            crate::gui::hud::ViewportAuthoritySource::CameraLatch,
            sim.min,
            sim.max,
            latch.using_hole,
        );
    }

    if render_hole {
        let scale = win.resolution.scale_factor().max(1e-6);
        let phys_w = win.physical_width().max(1);
        let phys_h = win.physical_height().max(1);
        let w_log = (sim.max.x - sim.min.x).max(1.0);
        let h_log = (sim.max.y - sim.min.y).max(1.0);
        let pos_x = (sim.min.x * scale).round().max(0.0) as u32;
        let pos_y = (sim.min.y * scale).round().max(0.0) as u32;
        let mut size_w = (w_log * scale).round().max(1.0) as u32;
        let mut size_h = (h_log * scale).round().max(1.0) as u32;
        if pos_x >= phys_w || pos_y >= phys_h || size_w == 0 || size_h == 0 {
            camera.viewport = None;
            latch.using_hole = false;
            latch.invalid_streak = u8::MAX;
        } else {
            size_w = size_w.min(phys_w - pos_x);
            size_h = size_h.min(phys_h - pos_y);
            camera.viewport = Some(bevy::camera::Viewport {
                physical_position: UVec2::new(pos_x, pos_y),
                physical_size: UVec2::new(size_w, size_h),
                depth: 0.0..1.0,
            });
        }
    } else {
        camera.viewport = None;
    }

    let view_px = if render_hole {
        sim.logical_size()
    } else {
        window_px
    };

    let zoom = desired.scale.x.max(1e-4);
    let world_w = params.width as f32;
    let world_h = params.height as f32;
    let (fixed_w, fixed_h) = orthographic_fixed_world_span(view_px, zoom, world_w, world_h);
    ortho_trace.fixed_width = fixed_w;
    ortho_trace.fixed_height = fixed_h;
    ortho_trace.view_pixels = view_px;
    ortho_trace.using_hole = render_hole;

    if render_hole {
        crate::gui::hud::trace_viewport_authority(
            crate::gui::hud::ViewportAuthoritySource::CameraApplied,
            sim.min,
            sim.max,
            true,
        );
    } else {
        crate::gui::hud::trace_viewport_authority(
            crate::gui::hud::ViewportAuthoritySource::CameraApplied,
            Vec2::ZERO,
            window_px,
            false,
        );
    }

    let Projection::Orthographic(ref mut ortho) = *proj else {
        return;
    };
    tf.scale = Vec3::ONE;
    ortho.scale = 1.0 / zoom;
    ortho.scaling_mode = bevy::camera::ScalingMode::Fixed {
        width: fixed_w,
        height: fixed_h,
    };
    ortho.viewport_origin = Vec2::new(0.5, 0.5);

    if cfg.as_ref().is_some_and(|c| c.camera_sync_trace) {
        trace_camera_sync(
            cfg.as_ref().unwrap(),
            &format!(
                "hole={} view_px=({:.0},{:.0}) fixed=({:.1},{:.1}) zoom={:.3}",
                latch.using_hole, view_px.x, view_px.y, fixed_w, fixed_h, zoom
            ),
        );
    }
}

fn map_camera_smooth_toward_desired(
    cfg: Res<DebugRenderTraceConfig>,
    time: Res<Time>,
    state: Res<State<BaseState>>,
    desired: Res<MapCameraDesired>,
    params: Res<WorldGenParams>,
    windows: Query<&Window, With<PrimaryWindow>>,
    sim_viewport: Res<SimulationMapViewport>,
    mut q_cam: Query<&mut Transform, With<MainWorldCamera>>,
    mut last_trace: Local<u64>,
) {
    if !matches!(state.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }
    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }
    let k = 1.0 - (-dt * SMOOTH_LAMBDA).exp();
    let Ok(mut xf) = q_cam.single_mut() else {
        return;
    };
    xf.translation = xf.translation.lerp(desired.translation, k);
    xf.rotation = xf.rotation.slerp(desired.rotation, k);
    xf.scale = Vec3::ONE;
    {
        let window_px = windows
            .single()
            .map(|w| Vec2::new(w.width().max(1.0), w.height().max(1.0)))
            .unwrap_or(Vec2::new(1280.0, 720.0));
        let viewport = map_camera_viewport_pixels(window_px, Some(sim_viewport.as_ref()));
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
