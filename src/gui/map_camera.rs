//! Pan / zoom / edge-scroll / rotate for [`MainWorldCamera`] (RTT path).
//!
//! Input mutates the [`MapCameraDesired`] **component** on [`MainWorldCamera`]; [`mirror_map_camera_component_to_resource`]
//! mirrors into [`Res<MapCameraDesiredRes>`]; [`sync_map_camera_pose_to_view_authority`] publishes WorldMain.
//! Schedule: **ApplyInput → DeriveDesired → Smooth** (matches engine spine + stall probes).

use bevy::input::mouse::{AccumulatedMouseScroll, MouseWheel};
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::egui;

use crate::engine::states::BaseState;
use crate::engine::{ActiveTestScene, TestScene};
use crate::render::view_runtime::ViewProjectionAuthority;
use crate::gui::ActiveMapViewInput;
use crate::gui::in_game_hud::SimulationMapViewportFill;
use crate::gui::sim_map_rtt::{apply_simulation_map_camera_clear, simulation_map_texture_extent};
use crate::gui::SimulationMapTexture;
use crate::gui::style::UiPalette;
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
    Smooth,
    DeriveDesired,
}

/// Marker on the root [`Camera2d`] that renders the tactical map RTT.
#[derive(Component)]
pub struct MainWorldCamera;

/// Deprecated — RTT path removed hole latch (witness/debug compat only).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MainWorldCameraViewportLatch {
    pub using_hole: bool,
    pub invalid_streak: u32,
    pub valid_streak: u32,
    pub steady_flip_count: u32,
}

#[allow(clippy::missing_const_for_fn)]
pub fn reset_main_world_camera_viewport_latch_on_enter_simulation(_: Commands) {}

/// Last orthographic fit written by [`sync_main_world_camera_projection`] (debug).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct MainWorldCameraOrthoTrace {
    pub fixed_width: f32,
    pub fixed_height: f32,
    pub view_pixels: Vec2,
    pub authority_zoom: f32,
    pub desired_zoom: f32,
    /// RTT path — always false (legacy hole scissor witness).
    pub using_hole: bool,
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

/// Logical pose on [`MainWorldCamera`] — ECS authority; [`Res<MapCameraDesiredRes>`] mirrors the component each frame.
#[derive(Component, Clone, Debug, PartialEq)]
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

/// Resource mirror of [`MapCameraDesired`] on [`MainWorldCamera`] (Bevy 0.19: no dual Component+Resource derive).
#[derive(Resource, Clone, Debug, PartialEq, Default)]
pub struct MapCameraDesiredRes(pub MapCameraDesired);

impl std::ops::Deref for MapCameraDesiredRes {
    type Target = MapCameraDesired;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for MapCameraDesiredRes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct MapCameraPlugin;

impl Plugin for MapCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapCameraDesiredRes>()
            .init_resource::<MapCameraSettings>()
            .init_resource::<MainWorldCameraViewportLatch>()
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
                derive_map_camera_desired_from_view_authority
                    .in_set(MapCameraSystemSet::DeriveDesired)
                    .run_if(in_simulation_or_editor_map),
            )
            .add_systems(
                Update,
                (
                    map_camera_apply_input,
                    map_camera_wheel_zoom_system.after(map_camera_apply_input),
                    mirror_map_camera_component_to_resource.after(map_camera_wheel_zoom_system),
                    sync_map_camera_pose_to_view_authority.after(mirror_map_camera_component_to_resource),
                )
                    .chain()
                    .in_set(MapCameraSystemSet::ApplyInput)
                    .run_if(in_simulation_or_editor_map),
            )
            .init_resource::<MainWorldCameraOrthoTrace>()
            .init_resource::<MapCameraZoomAudit>()
            .add_systems(
                Update,
                (
                    map_camera_smooth_toward_desired.in_set(MapCameraSystemSet::Smooth),
                    apply_main_world_camera_ortho_immediate.after(MapCameraSystemSet::Smooth),
                )
                    .chain()
                    .run_if(in_simulation_or_editor_map),
            )
            .add_systems(
                PostUpdate,
                (
                    sync_main_world_camera_projection,
                    map_camera_zoom_audit_system.after(sync_main_world_camera_projection),
                )
                    .chain()
                    .in_set(SimulationViewportSyncSet::ApplyCameraProjection)
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

/// RTT path — ortho aspect follows fill rect matched to GPU texture aspect (no letterbox ghost).
#[must_use]
pub fn map_camera_rtt_view_pixels(fill: &SimulationMapViewport, tex_extent: Vec2) -> Vec2 {
    if !fill.is_adequate_for_camera() {
        return tex_extent.max(Vec2::splat(1.0));
    }
    let logical = fill.logical_size();
    let tex = tex_extent.max(Vec2::splat(1.0));
    let aspect = (tex.x / tex.y).max(1e-6);
    let logical_aspect = logical.x / logical.y.max(1.0);
    if (logical_aspect - aspect).abs() < 1e-4 {
        logical
    } else if logical_aspect > aspect {
        Vec2::new(logical.y * aspect, logical.y)
    } else {
        Vec2::new(logical.x, logical.x / aspect)
    }
}

/// Legacy alias — tactical map uses RTT fill rect, not full-window render.
#[inline]
#[must_use]
pub fn tactical_map_full_window_render() -> bool {
    false
}

#[must_use]
pub fn map_camera_viewport_pixels(
    _window: Vec2,
    map_viewport: Option<&SimulationMapViewport>,
) -> Vec2 {
    map_viewport
        .filter(|v| v.is_adequate_for_camera())
        .map(|v| v.logical_size())
        .unwrap_or(Vec2::splat(1.0))
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

/// Pan/zoom the tactical camera to frame a world-space tile AABB (fire test / intel focus).
#[must_use]
pub fn map_camera_desired_fit_tile_aabb(
    min_tile: Vec2,
    max_tile: Vec2,
    map_vp: &SimulationMapViewport,
    window_px: Vec2,
    tex_extent: Vec2,
    world_w: f32,
    world_h: f32,
    margin: f32,
) -> MapCameraDesired {
    let center = (min_tile + max_tile) * 0.5;
    let span = (max_tile - min_tile).max(Vec2::splat(32.0));
    let viewport = map_camera_viewport_pixels(window_px, Some(map_vp));
    let view_px = map_camera_rtt_view_pixels(map_vp, tex_extent);
    let (zoom_lo, zoom_hi) = map_zoom_limits_for_world(world_w, world_h, viewport);
    let m = margin.max(1.05);
    let zoom = (view_px.x / (span.x * m))
        .min(view_px.y / (span.y * m))
        .clamp(zoom_lo, zoom_hi);
    MapCameraDesired {
        translation: Vec3::new(center.x, center.y, MAIN_WORLD_CAMERA_Z),
        scale: Vec3::splat(zoom),
        ..Default::default()
    }
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

fn map_fill_accepts_pointer(interaction: &Interaction) -> bool {
    matches!(*interaction, Interaction::Hovered | Interaction::Pressed)
}

fn mirror_map_camera_component_to_resource(
    q: Query<&MapCameraDesired, With<MainWorldCamera>>,
    mut res: ResMut<MapCameraDesiredRes>,
) {
    if let Ok(d) = q.single() {
        if res.0 != *d {
            res.0 = d.clone();
        }
    }
}

fn sync_map_camera_pose_to_view_authority(
    q: Query<&MapCameraDesired, With<MainWorldCamera>>,
    mut authority: ResMut<ViewProjectionAuthority>,
) {
    let Ok(desired) = q.single() else {
        return;
    };
    commit_map_camera_pose_to_view_authority_simple(authority.as_mut(), desired);
}

fn map_camera_apply_input(
    time: Res<Time>,
    state: Res<State<BaseState>>,
    bindings: Res<InputBindings>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    input_frame: Res<InputFrame>,
    windows: Query<&Window, With<PrimaryWindow>>,
    fill: Res<SimulationMapViewport>,
    tex: Res<SimulationMapTexture>,
    images: Res<Assets<Image>>,
    map_fill_interaction: Query<&Interaction, With<SimulationMapViewportFill>>,
    params: Res<WorldGenParams>,
    active_map_surface: Res<ActiveMapViewInput>,
    mut settings: ResMut<MapCameraSettings>,
    mut q_cam: Query<(&mut Transform, &mut MapCameraDesired), With<MainWorldCamera>>,
    mut locals: Local<MapCameraInputLocals>,
) {
    // PERF-INSTR-VFX-001: name this system inside the `map_cam` wall bracket (STALL/PERF only).
    let _perf = crate::render::PerfScope::new("upd_map_camera_apply_input");
    locals.before_apply = None;

    if !matches!(state.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }

    let Ok((mut tf, mut desired)) = q_cam.single_mut() else {
        return;
    };

    let tex_extent = simulation_map_texture_extent(tex.as_ref(), images.as_ref());
    let viewport = map_camera_rtt_view_pixels(fill.as_ref(), tex_extent);
    let pointer_on_map = map_fill_interaction
        .single()
        .map(map_fill_accepts_pointer)
        .unwrap_or(false);
    let pointer_blocks_mouse = !pointer_on_map;

    locals.before_apply = Some(desired.clone());

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

    if keys.just_pressed(bindings.map_toggle_edge_scroll) {
        settings.edge_scroll_enabled = !settings.edge_scroll_enabled;
    }

    if keys.just_pressed(bindings.map_cycle_camera_mode) {
        settings.mode = settings.mode.next();
    }

    let world_w = params.width as f32;
    let world_h = params.height as f32;
    let world = Vec2::new(world_w, world_h);
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
    tf.translation = desired.translation;
    tf.rotation = desired.rotation;

    if locals.before_apply.as_ref() == Some(&*desired) {
        return;
    }

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
}

/// Returns true when scroll changed the committed map zoom.
#[must_use]
pub fn apply_map_camera_wheel_zoom(
    scroll: f32,
    params: &WorldGenParams,
    fill: &SimulationMapViewport,
    tex_extent: Vec2,
    desired: &mut MapCameraDesired,
) -> bool {
    if scroll.abs() < f32::EPSILON {
        return false;
    }
    let world_w = params.width as f32;
    let world_h = params.height as f32;
    if world_w <= 0.0 || world_h <= 0.0 {
        return false;
    }
    let viewport = map_camera_rtt_view_pixels(fill, tex_extent);

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
    true
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
    params: Res<WorldGenParams>,
    fill: Res<SimulationMapViewport>,
    tex: Res<SimulationMapTexture>,
    images: Res<Assets<Image>>,
    map_fill_interaction: Query<&Interaction, With<SimulationMapViewportFill>>,
    scroll_acc: Res<AccumulatedMouseScroll>,
    input_frame: Res<InputFrame>,
    mut wheel_events: MessageReader<MouseWheel>,
    mut q_cam: Query<(&mut MapCameraDesired, &mut Transform), With<MainWorldCamera>>,
) {
    let _perf = crate::render::PerfScope::new("map_camera_wheel");
    if !matches!(state.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }
    if tactical_map_wheel_zoom_blocked(active_map_surface.as_ref()) {
        return;
    }
    if !map_fill_interaction
        .single()
        .map(map_fill_accepts_pointer)
        .unwrap_or(false)
    {
        return;
    }
    let scroll = resolve_tactical_map_wheel_scroll(
        scroll_acc.as_ref(),
        input_frame.as_ref(),
        &mut wheel_events,
    );
    if scroll.abs() < f32::EPSILON {
        return;
    }

    let Ok((mut desired, mut tf)) = q_cam.single_mut() else {
        return;
    };
    let before = desired.scale.x;
    let tex_extent = simulation_map_texture_extent(tex.as_ref(), images.as_ref());
    if apply_map_camera_wheel_zoom(
        scroll,
        params.as_ref(),
        fill.as_ref(),
        tex_extent,
        desired.as_mut(),
    ) {
        tf.translation = desired.translation;
        bevy::log::info!(
            target: "map_camera_zoom",
            "WHEEL_APPLIED scroll={scroll:.3} zoom {before:.4} -> {:.4}",
            desired.scale.x
        );
    }
}

fn apply_main_world_camera_ortho_core(
    camera: &mut Camera,
    tf: &mut Transform,
    proj: &mut Projection,
    fill: &SimulationMapViewport,
    tex_extent: Vec2,
    desired: &MapCameraDesired,
    params: &WorldGenParams,
    palette: &UiPalette,
    ortho_trace: &mut MainWorldCameraOrthoTrace,
    zoom_audit: &mut MapCameraZoomAudit,
) {
    camera.viewport = None;

    let view_px = map_camera_rtt_view_pixels(fill, tex_extent);
    let zoom = desired.scale.x.max(1e-4);

    tf.translation = desired.translation;
    tf.rotation = desired.rotation;

    let world_w = params.width as f32;
    let world_h = params.height as f32;
    let (fixed_w, fixed_h) = orthographic_fixed_world_span(view_px, zoom, world_w, world_h);

    ortho_trace.fixed_width = fixed_w;
    ortho_trace.fixed_height = fixed_h;
    ortho_trace.view_pixels = view_px;
    ortho_trace.authority_zoom = zoom;
    ortho_trace.desired_zoom = zoom;

    zoom_audit.ortho_fixed_w = fixed_w;
    zoom_audit.ortho_fixed_h = fixed_h;
    zoom_audit.view_px = view_px;
    zoom_audit.authority_zoom = zoom;
    zoom_audit.desired_zoom = zoom;

    apply_simulation_map_camera_clear(camera, palette);

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

fn apply_main_world_camera_ortho_immediate(
    fill: Res<SimulationMapViewport>,
    desired: Res<MapCameraDesiredRes>,
    tex: Res<SimulationMapTexture>,
    images: Res<Assets<Image>>,
    params: Res<WorldGenParams>,
    palette: Res<UiPalette>,
    windows: Query<&Window, With<PrimaryWindow>>,
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
    let tex_extent = simulation_map_texture_extent(tex.as_ref(), images.as_ref());
    apply_main_world_camera_ortho_core(
        &mut camera,
        &mut tf,
        &mut proj,
        fill.as_ref(),
        tex_extent,
        desired.as_ref(),
        params.as_ref(),
        palette.as_ref(),
        ortho_trace.as_mut(),
        zoom_audit.as_mut(),
    );
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

/// Mirror WorldMain authority onto the camera component + resource (minimap / preview writers).
pub fn derive_map_camera_desired_from_view_authority(
    authority: Res<ViewProjectionAuthority>,
    mut desired_res: ResMut<MapCameraDesiredRes>,
    mut q_cam: Query<&mut MapCameraDesired, With<MainWorldCamera>>,
    profile: Res<Stage5ReadinessProfile>,
) {
    // PERF-INSTR-VFX-001: name this system inside the `map_cam` wall bracket (STALL/PERF only).
    let _perf = crate::render::PerfScope::new("upd_map_camera_derive");
    use crate::render::view_runtime::ViewSurfaceId;
    let Some(cam) = authority
        .surface(ViewSurfaceId::WorldMain)
        .map(|s| s.camera)
    else {
        return;
    };
    let before = desired_res.0.clone();
    let next = MapCameraDesired {
        translation: Vec3::new(cam.translation.x, cam.translation.y, MAIN_WORLD_CAMERA_Z),
        scale: Vec3::splat(cam.zoom.max(1e-4)),
        rotation: Quat::from_rotation_z(cam.rotation),
    };
    if desired_res.0 == next {
        return;
    }
    desired_res.0 = next.clone();
    if let Ok(mut d) = q_cam.single_mut() {
        *d = next;
    }
    trace_map_camera_desired_write_if_full_app(
        profile.as_ref(),
        "derive_map_camera_desired_from_view_authority",
        &before,
        &desired_res.0,
    );
}

/// VM-09-v2 compat alias — [`derive_map_camera_desired_from_view_authority`].
pub fn mirror_world_main_camera_from_map_desired(
    authority: Res<ViewProjectionAuthority>,
    desired: ResMut<MapCameraDesiredRes>,
    q_cam: Query<&mut MapCameraDesired, With<MainWorldCamera>>,
    profile: Res<Stage5ReadinessProfile>,
) {
    derive_map_camera_desired_from_view_authority(authority, desired, q_cam, profile);
}

/// PostUpdate ortho refresh after UI fill measure.
pub fn sync_main_world_camera_projection(
    fill: Res<SimulationMapViewport>,
    desired: Res<MapCameraDesiredRes>,
    tex: Res<SimulationMapTexture>,
    images: Res<Assets<Image>>,
    params: Res<WorldGenParams>,
    palette: Res<UiPalette>,
    windows: Query<&Window, With<PrimaryWindow>>,
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
    let tex_extent = simulation_map_texture_extent(tex.as_ref(), images.as_ref());
    apply_main_world_camera_ortho_core(
        &mut camera,
        &mut tf,
        &mut proj,
        fill.as_ref(),
        tex_extent,
        desired.as_ref(),
        params.as_ref(),
        palette.as_ref(),
        ortho_trace.as_mut(),
        zoom_audit.as_mut(),
    );

    if cfg.as_ref().is_some_and(|c| c.camera_sync_trace) {
        trace_camera_sync(
            cfg.as_ref().unwrap(),
            &format!(
                "rtt view_px=({:.0},{:.0}) fixed=({:.1},{:.1}) zoom={:.3}",
                ortho_trace.view_pixels.x,
                ortho_trace.view_pixels.y,
                ortho_trace.fixed_width,
                ortho_trace.fixed_height,
                ortho_trace.desired_zoom
            ),
        );
    }
}

/// End-of-frame zoom audit — logs drift between authority, desired, and ortho fit.
pub fn map_camera_zoom_audit_system(
    frame: Res<FrameCount>,
    authority: Res<ViewProjectionAuthority>,
    desired: Res<MapCameraDesiredRes>,
    ortho: Res<MainWorldCameraOrthoTrace>,
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

    bevy::log::debug!(
        target: "map_camera_zoom",
        "FRAME frame={} auth_z={:.4} des_z={:.4} fixed=({:.1},{:.1}) view_px=({:.0},{:.0})",
        frame.0,
        audit.authority_zoom,
        audit.desired_zoom,
        audit.ortho_fixed_w,
        audit.ortho_fixed_h,
        audit.view_px.x,
        audit.view_px.y,
    );
}
fn map_camera_smooth_toward_desired(
    cfg: Res<DebugRenderTraceConfig>,
    time: Res<Time>,
    state: Res<State<BaseState>>,
    desired: Res<MapCameraDesiredRes>,
    params: Res<WorldGenParams>,
    fill: Res<SimulationMapViewport>,
    tex: Res<SimulationMapTexture>,
    images: Res<Assets<Image>>,
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
        let tex_extent = simulation_map_texture_extent(tex.as_ref(), images.as_ref());
        let viewport = map_camera_rtt_view_pixels(fill.as_ref(), tex_extent);
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
            let win = format!(
                "{:.0}x{:.0}",
                fill.window_logical.x,
                fill.window_logical.y
            );
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

/// Visible world span for the live tactical camera (RTT fill rect).
#[must_use]
pub fn tactical_visible_world_span(
    _window_px: Vec2,
    map_vp: &SimulationMapViewport,
    tex_extent: Vec2,
    zoom: f32,
    world_w: f32,
    world_h: f32,
) -> (f32, f32) {
    let view_px = map_camera_rtt_view_pixels(map_vp, tex_extent);
    orthographic_fixed_world_span(view_px, zoom.max(1e-6), world_w, world_h)
}

/// World XY → egui screen (matches [`MainWorldCamera`] ortho over fill rect).
#[must_use]
pub fn sim_map_world_xy_to_egui(
    world_xy: Vec2,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    world_w: f32,
    world_h: f32,
) -> Option<egui::Pos2> {
    sim_map_world_xy_to_egui_with_window(world_xy, desired, map_vp, world_w, world_h, None, None)
}

/// Like [`sim_map_world_xy_to_egui`] with optional texture extent override.
#[must_use]
pub fn sim_map_world_xy_to_egui_with_window(
    world_xy: Vec2,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    world_w: f32,
    world_h: f32,
    tex_extent: Option<Vec2>,
    _latch: Option<&MainWorldCameraViewportLatch>,
) -> Option<egui::Pos2> {
    if !map_vp.is_adequate_for_camera() {
        return None;
    }
    let zoom = desired.scale.x.abs().max(1e-6);
    let cam = desired.translation.truncate();
    let rect = sim_map_image_rect(map_vp);
    let extent = tex_extent.unwrap_or(map_vp.logical_size());
    let (fw, fh) = tactical_visible_world_span(Vec2::ZERO, map_vp, extent, zoom, world_w, world_h);
    let sx = rect.center().x + (world_xy.x - cam.x) * (rect.width() / fw.max(1e-6));
    let sy = rect.center().y + (world_xy.y - cam.y) * (rect.height() / fh.max(1e-6));
    Some(egui::pos2(sx, sy))
}

/// Inverse of [`sim_map_world_xy_to_egui`] — logical cursor → world XY.
#[must_use]
pub fn sim_map_screen_to_world_xy(
    screen_logical: Vec2,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    world_w: f32,
    world_h: f32,
) -> Option<Vec2> {
    sim_map_screen_to_world_xy_with_window(
        screen_logical,
        desired,
        map_vp,
        world_w,
        world_h,
        None,
        None,
    )
}

/// Like [`sim_map_screen_to_world_xy`] with optional texture extent override.
#[must_use]
pub fn sim_map_screen_to_world_xy_with_window(
    screen_logical: Vec2,
    desired: &MapCameraDesired,
    map_vp: &SimulationMapViewport,
    world_w: f32,
    world_h: f32,
    tex_extent: Option<Vec2>,
    _latch: Option<&MainWorldCameraViewportLatch>,
) -> Option<Vec2> {
    if !map_vp.is_adequate_for_camera() {
        return None;
    }
    let zoom = desired.scale.x.abs().max(1e-6);
    let cam = desired.translation.truncate();
    let rect = sim_map_image_rect(map_vp);
    let extent = tex_extent.unwrap_or(map_vp.logical_size());
    let (fw, fh) = tactical_visible_world_span(Vec2::ZERO, map_vp, extent, zoom, world_w, world_h);
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

/// Legacy alias — [`sync_main_world_camera_projection`].
pub use sync_main_world_camera_projection as sync_main_world_camera_viewport_and_projection;

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
        let mut desired = MapCameraDesired::default();
        assert!(apply_map_camera_wheel_zoom(
            1.0,
            &params,
            &vp,
            Vec2::new(800.0, 600.0),
            &mut desired,
        ));
        let _ = latch;
        assert!(desired.scale.x > 1.0, "wheel must increase zoom");
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
