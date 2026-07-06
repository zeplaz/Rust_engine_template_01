//! Simulation tactical map render-to-texture — camera → GPU image → Bevy UI [`ImageNode`].
//!
//! Replaces the legacy hole/latch pipeline ([`TacticalMapFillRect`] is the sole
//! screen-space rect for picks).

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::RenderLayers;
use bevy::camera::{CameraOutputMode, ClearColorConfig, RenderTarget};
use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, IsDefaultUiCamera, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use crate::gui::style::UiPalette;
use crate::gui::map_camera::{MainWorldCamera, MapCameraDesired};
use crate::gui::TileDebugRenderHost;
use crate::render::TerrainInstancedRenderHost;
use crate::engine::EngineLaunchArgs;
use crate::gui::authoritative_viewport::{
    measure_sim_map_fill_corners_crosscheck, sync_simulation_map_fill_debug_trace,
};
use crate::gui::hud::{ViewportRectSanity, VIEWPORT_SIM_MAP_SAFE_MIN_H, VIEWPORT_SIM_MAP_SAFE_MIN_W};
use crate::gui::in_game_hud::SimulationMapViewportFill;
use crate::gui::SimulationViewportSyncSet;

/// Committed GPU image handle for the tactical map (camera [`RenderTarget::Image`]).
#[derive(Resource, Clone, Debug)]
pub struct SimulationMapTexture(pub Handle<Image>);

/// Deferred RTT bind — avoids same-frame camera↔image swap (Bevy #16159).
#[derive(Debug, Clone)]
pub struct PendingSimulationMapRttBind {
    pub target: Handle<Image>,
    pub scale_factor: f32,
    pub frame_requested: u32,
}

#[derive(Resource, Debug, Default)]
pub struct SimulationMapRttBindBarrier {
    pub pending: Option<PendingSimulationMapRttBind>,
    pub bound: Handle<Image>,
    pub revision: u64,
}

fn pending_rtt_bind_ready(
    pending: &PendingSimulationMapRttBind,
    frame: &FrameCount,
    images: &Assets<Image>,
) -> bool {
    frame.0 > pending.frame_requested && images.get(&pending.target).is_some()
}

/// Window-space AABB of the Bevy UI node that displays [`SimulationMapTexture`].
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct TacticalMapFillRect {
    pub valid: bool,
    pub min: Vec2,
    pub max: Vec2,
    /// Primary window logical size (cursor chrome / legacy pick helpers).
    pub window_logical: Vec2,
    /// Steady-state validity flips after bootstrap (RTT-A1-004 witness).
    pub steady_invalid_flip_count: u32,
    /// Previous-frame validity for flip detection.
    pub(crate) last_valid: bool,
    /// Set once fill becomes valid (bootstrap gate for flip counting).
    pub(crate) had_valid_once: bool,
}

/// Deprecated alias — use [`TacticalMapFillRect`].
pub type SimulationMapFillRect = TacticalMapFillRect;

impl TacticalMapFillRect {
    #[inline]
    #[must_use]
    pub fn logical_size(&self) -> Vec2 {
        (self.max - self.min).max(Vec2::ZERO)
    }

    #[inline]
    #[must_use]
    pub fn is_adequate_for_camera(&self) -> bool {
        let s = self.logical_size();
        s.x >= VIEWPORT_SIM_MAP_SAFE_MIN_W && s.y >= VIEWPORT_SIM_MAP_SAFE_MIN_H
    }

    #[inline]
    #[must_use]
    pub fn contains_cursor(self, cursor: Vec2) -> bool {
        self.valid
            && cursor.x >= self.min.x
            && cursor.x <= self.max.x
            && cursor.y >= self.min.y
            && cursor.y <= self.max.y
    }
}

const RTT_MIN_EXTENT: u32 = 64;
const RTT_MAX_EXTENT: u32 = 4096;

/// World map sprites/meshes use this layer; only [`MainWorldCamera`] (RTT) includes it.
pub const SIMULATION_MAP_RTT_RENDER_LAYER: usize = 1;

/// RTT diagnostic mode — bypass custom Core2d overlay hosts (see `spawn_main_world_rtt_camera`).
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq)]
pub struct RttDiagCameraConfig {
    pub mode: RttDiagCameraMode,
}

impl Default for RttDiagCameraConfig {
    fn default() -> Self {
        Self {
            mode: RttDiagCameraMode::Production,
        }
    }
}

/// How [`MainWorldCamera`] is spawned for RTT draw isolation tests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RttDiagCameraMode {
    /// Production: Camera2d + [`TileDebugRenderHost`] + [`TerrainInstancedRenderHost`].
    Production,
    /// P0 — Bevy `pixel_grid_snap`: Camera2d → Image, no custom render-graph hosts.
    UnhookCamera2d,
    /// P1 — Camera3d ortho → Image (terrain must be [`Mesh2d`], not [`Sprite`]).
    UnhookCamera3d,
}

impl RttDiagCameraMode {
    #[must_use]
    pub fn uses_mesh_terrain(self) -> bool {
        matches!(self, Self::UnhookCamera3d)
    }

    #[must_use]
    pub fn unhooked(self) -> bool {
        !matches!(self, Self::Production)
    }
}

/// Opt-in Core2d overlay hosts on production RTT camera (fire/water/tile-debug passes).
#[must_use]
pub fn rtt_core2d_overlay_hosts_enabled() -> bool {
    std::env::var("RTT_CORE2D_OVERLAY")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Default: **production** camera. Opt-in diagnostics only via env (see table on [`rtt_diag_camera_mode`]).
#[must_use]
pub fn rtt_diag_camera_mode(launch: Option<&EngineLaunchArgs>) -> RttDiagCameraMode {
    let _ = launch;
    if std::env::var("RTT_DIAG_PRODUCTION")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        return RttDiagCameraMode::Production;
    }
    if std::env::var("RTT_DIAG_CAMERA3D")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        return RttDiagCameraMode::UnhookCamera3d;
    }
    if std::env::var("RTT_DIAG_UNHOOK")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    {
        return RttDiagCameraMode::UnhookCamera2d;
    }
    RttDiagCameraMode::Production
}

/// Spawn [`MainWorldCamera`] for tactical RTT (Bevy `pixel_grid_snap` baseline + optional 2.5D spike).
pub fn spawn_main_world_rtt_camera(
    commands: &mut Commands,
    handle: Handle<Image>,
    palette: &UiPalette,
    mode: RttDiagCameraMode,
) {
    commands.insert_resource(RttDiagCameraConfig { mode });
    let rt = RenderTarget::from(handle);
    let mut camera = Camera {
        order: 0,
        ..default()
    };
    apply_simulation_map_camera_clear(&mut camera, palette);

    match mode {
        RttDiagCameraMode::Production => {
            // Bevy pixel_grid_snap baseline: Camera2d → Image, default layer 0, no Core2d overlay hosts.
            // Set RTT_CORE2D_OVERLAY=1 to re-enable fire/water/tile-debug overlay passes on this camera.
            let mut entity = commands.spawn((
                MainWorldCamera,
                Camera2d,
                Msaa::Off,
                rt,
                camera,
                MapCameraDesired::default(),
            ));
            if rtt_core2d_overlay_hosts_enabled() {
                entity.insert((TileDebugRenderHost, TerrainInstancedRenderHost));
            }
        }
        RttDiagCameraMode::UnhookCamera2d => {
            info!(
                target: "sim_map_rtt",
                "RTT_DIAG P0: vanilla Camera2d → Image (Core2d overlay hosts stripped)"
            );
            commands.spawn((
                MainWorldCamera,
                Camera2d,
                Msaa::Off,
                rt,
                camera,
                MapCameraDesired::default(),
            ));
        }
        RttDiagCameraMode::UnhookCamera3d => {
            info!(
                target: "sim_map_rtt",
                "RTT_DIAG P1: Camera3d ortho → Image (Mesh2d terrain; Sprite is Core2d-only)"
            );
            commands.spawn((
                MainWorldCamera,
                Camera3d::default(),
                Projection::Orthographic(OrthographicProjection {
                    near: -1000.0,
                    far: 1000.0,
                    ..OrthographicProjection::default_2d()
                }),
                Msaa::Off,
                rt,
                camera,
                MapCameraDesired::default(),
            ));
        }
    }
}

/// Primary-window camera for Bevy UI chrome + map [`ImageNode`] (not world Core2d).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SimulationHudUiCamera;

#[must_use]
pub fn simulation_map_rtt_render_layers() -> RenderLayers {
    // Bevy pixel_grid_snap: offscreen Camera2d + world sprites on default layer 0.
    // SimulationHudUiCamera uses RenderLayers::none() so layer-0 world never hits the window.
    RenderLayers::layer(0)
}

/// RTT + window chrome clear — cool gray-blue tactical void (not paper / not Bevy default gray).
#[must_use]
pub fn simulation_map_rtt_clear_color(palette: &UiPalette) -> Color {
    palette.bevy_sim_map_field_clear()
}

/// One full sim-day cycle length for RTT clear day/night lerp (8 minutes at 1× sim speed).
pub const SIM_MAP_DAY_CYCLE_MICROS: u64 = 480_000_000;

/// Normalized daylight in `[0, 1]` from monotonic sim time (sinusoidal day/night).
#[must_use]
pub fn sim_map_daylight_factor(sim_time_micros: u64) -> f32 {
    let phase = (sim_time_micros % SIM_MAP_DAY_CYCLE_MICROS) as f64 / SIM_MAP_DAY_CYCLE_MICROS as f64;
    (((phase * std::f64::consts::TAU).sin() + 1.0) * 0.5) as f32
}

/// Drive MainWorldCamera + HUD UI camera clear from sim time day/night cycle.
pub fn sync_sim_map_clear_from_day_cycle(
    palette: Res<UiPalette>,
    sim_time: Res<crate::systems::sim_control::SimTimeMicros>,
    mut main_cams: Query<&mut Camera, (With<MainWorldCamera>, Without<SimulationHudUiCamera>)>,
    mut hud_cams: Query<&mut Camera, (With<SimulationHudUiCamera>, Without<MainWorldCamera>)>,
) {
    let color = palette.bevy_sim_map_field_clear_for_daylight(sim_map_daylight_factor(sim_time.0));
    let cfg = ClearColorConfig::Custom(color);
    for mut cam in main_cams.iter_mut().chain(hud_cams.iter_mut()) {
        cam.clear_color = cfg;
        cam.output_mode = CameraOutputMode::Write {
            blend_state: None,
            clear_color: cfg,
        };
    }
}

/// Pre-pass + upscaling write clear for [`MainWorldCamera`] / HUD UI camera.
pub fn apply_simulation_map_camera_clear(camera: &mut Camera, palette: &UiPalette) {
    let color = simulation_map_rtt_clear_color(palette);
    let cfg = ClearColorConfig::Custom(color);
    camera.clear_color = cfg;
    camera.output_mode = CameraOutputMode::Write {
        blend_state: None,
        clear_color: cfg,
    };
}

/// Spawns the window-target UI camera (required once [`MainWorldCamera`] renders to RTT only).
pub fn spawn_simulation_hud_ui_camera(commands: &mut Commands, palette: &UiPalette) {
    let mut camera = Camera {
        order: 1,
        ..default()
    };
    apply_simulation_map_camera_clear(&mut camera, palette);
    commands.spawn((
        SimulationHudUiCamera,
        Camera2d,
        IsDefaultUiCamera,
        RenderLayers::none(),
        camera,
    ));
}

#[must_use]
pub fn simulation_map_rtt_image(width: u32, height: u32) -> Image {
    let w = width.max(RTT_MIN_EXTENT);
    let h = height.max(RTT_MIN_EXTENT);
    // Must match Core2d overlay SDR pipelines (`CORE2D_OVERLAY_SDR_FORMAT` = Rgba8UnormSrgb).
    let mut image = Image::new_target_texture(
        w,
        h,
        crate::render::CORE2D_OVERLAY_SDR_FORMAT,
        None,
    );
    image.texture_descriptor.label = Some("SimulationMapRenderTarget");
    image.asset_usage = RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD;
    image
}

#[inline]
#[must_use]
pub fn simulation_map_texture_extent(tex: &SimulationMapTexture, images: &Assets<Image>) -> Vec2 {
    images
        .get(&tex.0)
        .map(|img| {
            Vec2::new(img.width() as f32, img.height() as f32).max(Vec2::splat(1.0))
        })
        .unwrap_or(Vec2::new(1920.0, 1080.0))
}

fn clamp_aabb_to_window(min: Vec2, max: Vec2, window: Vec2) -> (Vec2, Vec2) {
    let win = Vec2::new(window.x.max(1.0), window.y.max(1.0));
    let min = Vec2::new(min.x.clamp(0.0, win.x), min.y.clamp(0.0, win.y));
    let mut max = Vec2::new(max.x.clamp(0.0, win.x), max.y.clamp(0.0, win.y));
    if max.x < min.x {
        max.x = min.x;
    }
    if max.y < min.y {
        max.y = min.y;
    }
    (min, max)
}

/// Measure UI fill node → [`TacticalMapFillRect`]; queue RT resize (camera bind deferred).
pub fn sync_simulation_map_fill_rect_system(
    q: Query<(&ComputedNode, &UiGlobalTransform), With<SimulationMapViewportFill>>,
    mut fill: ResMut<TacticalMapFillRect>,
    mut tex: ResMut<SimulationMapTexture>,
    mut images: ResMut<Assets<Image>>,
    mut barrier: ResMut<SimulationMapRttBindBarrier>,
    frame: Res<FrameCount>,
    win: Query<&Window, With<PrimaryWindow>>,
    mut sanity: ResMut<ViewportRectSanity>,
    mut trace: ResMut<crate::gui::SimulationMapViewportTrace>,
    mut sim_dbg: ResMut<crate::gui::SimulationMapViewportDebug>,
) {
    let Ok(w) = win.single() else {
        fill.valid = false;
        return;
    };
    let window_logical = Vec2::new(w.width(), w.height());
    let Ok((node, xf)) = q.single() else {
        fill.valid = false;
        return;
    };
    if node.is_empty() {
        fill.valid = false;
        return;
    }
    let scale = w.scale_factor().max(1e-6);
    let (raw_min, raw_max) = measure_sim_map_fill_corners_crosscheck(node, xf, scale);
    let (clamped_min, clamped_max) = clamp_aabb_to_window(raw_min, raw_max, window_logical);
    let fallback = crate::gui::simulation_map_fallback_logical_extent(window_logical);
    let (min, max, valid) =
        sanity.inspect_simulation_map_aabb(clamped_min, clamped_max, fallback, None);
    fill.min = min;
    fill.max = max;
    if fill.had_valid_once && fill.last_valid != valid {
        fill.steady_invalid_flip_count = fill.steady_invalid_flip_count.saturating_add(1);
    }
    if valid {
        fill.had_valid_once = true;
    }
    fill.last_valid = valid;
    fill.valid = valid;
    fill.window_logical = window_logical;
    sync_simulation_map_fill_debug_trace(fill.as_ref(), trace.as_mut(), sim_dbg.as_mut());

    let size = fill.logical_size();
    if !fill.is_adequate_for_camera() {
        return;
    }
    let want_w = (size.x * scale).round() as u32;
    let want_h = (size.y * scale).round() as u32;
    let want_w = want_w.clamp(RTT_MIN_EXTENT, RTT_MAX_EXTENT);
    let want_h = want_h.clamp(RTT_MIN_EXTENT, RTT_MAX_EXTENT);
    // Compare against committed bind — not tex.0 (Bevy #16159 / preview barrier parity).
    let bound = if barrier.bound != Handle::default() {
        barrier.bound.clone()
    } else {
        tex.0.clone()
    };
    let need_resize = images
        .get(&bound)
        .is_none_or(|img| img.width() != want_w || img.height() != want_h);
    if need_resize {
        if barrier
            .pending
            .as_ref()
            .is_some_and(|p| images.get(&p.target).is_some_and(|img| img.width() == want_w && img.height() == want_h))
        {
            return;
        }
        let new_img = simulation_map_rtt_image(want_w, want_h);
        let handle = images.add(new_img);
        barrier.pending = Some(PendingSimulationMapRttBind {
            target: handle,
            scale_factor: scale,
            frame_requested: frame.0,
        });
    }
}

/// Commit pending RTT handle to [`MainWorldCamera`] one frame after allocation.
pub fn commit_simulation_map_rtt_bind_system(
    frame: Res<FrameCount>,
    images: Res<Assets<Image>>,
    mut barrier: ResMut<SimulationMapRttBindBarrier>,
    mut tex: ResMut<SimulationMapTexture>,
    mut cam_rt: Query<&mut RenderTarget, With<MainWorldCamera>>,
    mut cam: Query<&mut Camera, With<MainWorldCamera>>,
) {
    let Some(pending) = barrier.pending.as_ref() else {
        return;
    };
    if !pending_rtt_bind_ready(pending, &frame, &images) {
        return;
    }
    let pending = barrier.pending.take().expect("checked above");
    barrier.bound = pending.target.clone();
    barrier.revision = barrier.revision.wrapping_add(1);
    tex.0 = pending.target.clone();
    let rt = RenderTarget::from(pending.target);
    for mut target in cam_rt.iter_mut() {
        *target = rt.clone();
    }
    for mut camera in cam.iter_mut() {
        camera.is_active = true;
    }
}

pub struct SimulationMapRttPlugin;

/// Bind committed RT handle to the tactical map [`ImageNode`].
pub fn sync_simulation_map_image_node_system(
    tex: Res<SimulationMapTexture>,
    mut q: Query<&mut bevy::ui::widget::ImageNode, With<SimulationMapViewportFill>>,
) {
    if tex.0 == Handle::default() {
        return;
    }
    for mut node in q.iter_mut() {
        if node.image != tex.0 {
            *node = bevy::ui::widget::ImageNode::new(tex.0.clone())
                .with_mode(bevy::ui::widget::NodeImageMode::Stretch);
        } else if !matches!(node.image_mode, bevy::ui::widget::NodeImageMode::Stretch) {
            node.image_mode = bevy::ui::widget::NodeImageMode::Stretch;
        }
    }
}

/// Reset RTT fill validity flip counter on Simulation bootstrap (RTT-A1-004).
pub fn reset_tactical_map_fill_streak_on_enter_simulation(mut fill: ResMut<TacticalMapFillRect>) {
    fill.steady_invalid_flip_count = 0;
    fill.last_valid = false;
    fill.had_valid_once = false;
}

impl Plugin for SimulationMapRttPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<crate::gui::sim_map_rtt::TacticalMapFillRect>()
            .add_systems(
                OnEnter(crate::engine::states::BaseState::Simulation),
                reset_tactical_map_fill_streak_on_enter_simulation,
            )
            .add_systems(
                Update,
                sync_sim_map_clear_from_day_cycle
                    .after(crate::systems::sim_control::SimControlSystemSet::AdvanceSimTick),
            )
            .add_systems(
                PostUpdate,
                (
                    sync_simulation_map_fill_rect_system,
                    commit_simulation_map_rtt_bind_system,
                    sync_simulation_map_image_node_system,
                )
                    .chain()
                    .run_if(crate::gui::ui_gates::in_simulation_or_editor)
                    .after(bevy::ui::UiSystems::Stack)
                    .in_set(SimulationViewportSyncSet::MeasureUiHole),
            );
    }
}

/// Bootstrap RT handle before first layout measure (camera spawn path).
pub fn insert_simulation_map_texture(
    images: &mut Assets<Image>,
    commands: &mut Commands,
    width: u32,
    height: u32,
) -> Handle<Image> {
    let image = simulation_map_rtt_image(width, height);
    let handle = images.add(image);
    commands.insert_resource(SimulationMapTexture(handle.clone()));
    commands.insert_resource(SimulationMapRttBindBarrier {
        bound: handle.clone(),
        ..Default::default()
    });
    handle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtt_image_data_matches_descriptor_extent() {
        let img = simulation_map_rtt_image(1920, 1080);
        let bytes = img.data.as_ref().expect("target texture data");
        assert_eq!(bytes.len(), 4 * 1920 as usize * 1080 as usize);
        assert_eq!(img.width(), 1920);
        assert_eq!(img.height(), 1080);
    }
}
