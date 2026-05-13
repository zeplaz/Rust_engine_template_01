//! Pan / zoom / edge-scroll / rotate for [`MainWorldCamera`] using [`InputBindings`](crate::gui::InputBindings).
//!
//! **P1-F:** Input writes [`MapCameraDesired`]; a follow-up system lerps [`Transform`] toward it (smoothing).
//! Skips input while egui wants the pointer or keyboard. Edge pan respects [`MapCameraSettings::edge_scroll_enabled`].

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::engine::states::BaseState;
use crate::engine::{ActiveTestScene, TestScene};
use crate::gui::InputBindings;
use crate::gui::InputFrame;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Marker on the root [`Camera2d`] that carries world-space UI + weather VFX children.
#[derive(Component)]
pub struct MainWorldCamera;

/// Logical pose after map input (smoothing target).
#[derive(Resource, Clone, Debug)]
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
            .add_systems(
                Update,
                (map_camera_apply_input_to_desired, map_camera_smooth_toward_desired)
                    .chain()
                    .run_if(in_simulation_or_editor_map),
            );
    }
}

fn in_simulation_or_editor_map(state: Res<State<BaseState>>) -> bool {
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
pub const MAP_ZOOM_CLAMP: (f32, f32) = (0.35, 4.5);
const ROTATE_STEP: f32 = 1.35_f32.to_radians();
const SMOOTH_LAMBDA: f32 = 12.0;

#[inline]
fn test_scene_zoom(test_scene: Option<Res<ActiveTestScene>>) -> f32 {
    test_scene
        .as_ref()
        .map(|ts| match ts.0 {
            TestScene::Weather => 1.25,
            TestScene::Fire => 1.95,
            TestScene::Atmosphere => 1.75,
            TestScene::Visual => 2.1,
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
    scroll_acc: Res<AccumulatedMouseScroll>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut contexts: EguiContexts,
    params: Res<WorldGenParams>,
    test_scene: Option<Res<ActiveTestScene>>,
    mut settings: ResMut<MapCameraSettings>,
    mut desired: ResMut<MapCameraDesired>,
    q_cam: Query<&Transform, With<MainWorldCamera>>,
    mut middle_tap: Local<Option<f32>>,
    mut synced_from_cam: Local<bool>,
) {
    if !matches!(state.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }

    if state.is_changed() {
        *synced_from_cam = false;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let pointer_over_ui = ctx.wants_pointer_input() || ctx.wants_keyboard_input();
    if pointer_over_ui {
        return;
    }

    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

    if !*synced_from_cam {
        if let Ok(t) = q_cam.single() {
            desired.translation = t.translation;
            desired.scale = t.scale;
            desired.rotation = t.rotation;
            *synced_from_cam = true;
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
        if let Some(prev) = *middle_tap {
            if now - prev < 0.45 {
                recenter_pulse();
                *middle_tap = None;
            } else {
                *middle_tap = Some(now);
            }
        } else {
            *middle_tap = Some(now);
        }
    }

    let default_z = default_map_zoom_for_world(test_scene);
    if keys.just_pressed(bindings.map_reset_zoom) {
        desired.scale = Vec3::splat(default_z);
    }

    if keys.just_pressed(bindings.map_frame_world) {
        desired.translation = center_xy;
        if let Ok(win) = windows.single() {
            let win = Vec2::new(win.width().max(1.0), win.height().max(1.0));
            let margin = 0.9;
            let s = margin * (win.x / world_w.max(1.0)).min(win.y / world_h.max(1.0));
            let s = s.clamp(MAP_ZOOM_CLAMP.0, MAP_ZOOM_CLAMP.1);
            desired.scale = Vec3::splat(s);
        }
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

    let scroll = scroll_acc.delta.y + scroll_acc.delta.x * 0.25;
    if scroll.abs() >= f32::EPSILON {
        let z = ZOOM_FACTOR.powf(scroll.clamp(-5.0, 5.0));
        let s = (desired.scale.x * z).clamp(MAP_ZOOM_CLAMP.0, MAP_ZOOM_CLAMP.1);
        desired.scale = Vec3::splat(s);
    }

    let zoom_key = 1.65 * dt;
    if keys.pressed(bindings.map_zoom_in) {
        let s = (desired.scale.x * (1.0 + zoom_key)).clamp(MAP_ZOOM_CLAMP.0, MAP_ZOOM_CLAMP.1);
        desired.scale = Vec3::splat(s);
    }
    if keys.pressed(bindings.map_zoom_out) {
        let s = (desired.scale.x / (1.0 + zoom_key)).clamp(MAP_ZOOM_CLAMP.0, MAP_ZOOM_CLAMP.1);
        desired.scale = Vec3::splat(s);
    }

    if keys.just_pressed(bindings.map_rotate_ccw) {
        desired.rotation *= Quat::from_rotation_z(ROTATE_STEP);
    }
    if keys.just_pressed(bindings.map_rotate_cw) {
        desired.rotation *= Quat::from_rotation_z(-ROTATE_STEP);
    }
}

fn map_camera_smooth_toward_desired(
    time: Res<Time>,
    state: Res<State<BaseState>>,
    desired: Res<MapCameraDesired>,
    mut q_cam: Query<&mut Transform, With<MainWorldCamera>>,
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
    let s = xf.scale.x.lerp(desired.scale.x, k);
    xf.scale = Vec3::splat(s);
    xf.rotation = xf.rotation.slerp(desired.rotation, k);
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
