//! Pan / zoom / edge-scroll / rotate for the primary [`MainWorldCamera`] using [`InputBindings`](crate::gui::InputBindings).
//!
//! Skips input while egui wants the pointer or keyboard (tooling capture).

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContexts;

use crate::engine::states::BaseState;
use crate::gui::InputBindings;

/// Marker on the root [`Camera2d`] that carries world-space UI + weather VFX children.
#[derive(Component)]
pub struct MainWorldCamera;

pub struct MapCameraPlugin;

impl Plugin for MapCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, map_camera_controls);
    }
}

const EDGE_FRACTION: f32 = 0.06;
const KEY_PAN: f32 = 520.0;
const EDGE_PAN: f32 = 340.0;
const GRIP_PAN: f32 = 620.0;
const ZOOM_FACTOR: f32 = 1.08;
const ZOOM_CLAMP: (f32, f32) = (0.35, 4.5);
const ROTATE_STEP: f32 = 1.35_f32.to_radians();

fn map_camera_controls(
    time: Res<Time>,
    state: Res<State<BaseState>>,
    bindings: Res<InputBindings>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_btn: Res<ButtonInput<MouseButton>>,
    motion_acc: Res<AccumulatedMouseMotion>,
    scroll_acc: Res<AccumulatedMouseScroll>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut contexts: EguiContexts,
    mut q_cam: Query<&mut Transform, With<MainWorldCamera>>,
) {
    if !matches!(state.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let pointer_over_ui = ctx.wants_pointer_input() || ctx.wants_keyboard_input();
    if pointer_over_ui {
        return;
    }

    let Ok(mut transform) = q_cam.single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    if dt <= 0.0 {
        return;
    }

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

    let fast = keys.pressed(bindings.map_pan_fast_modifier);
    let key_speed = KEY_PAN * dt * if fast { 2.2 } else { 1.0 };
    if pan != Vec2::ZERO {
        transform.translation += (pan.normalize() * key_speed).extend(0.0);
    }

    let grip = keys.pressed(bindings.map_mouse_grip) || mouse_btn.pressed(MouseButton::Middle);
    if grip {
        let sum = motion_acc.delta;
        transform.translation +=
            Vec3::new(-sum.x, sum.y, 0.0) * GRIP_PAN * dt * 0.045 * if fast { 1.35 } else { 1.0 };
    }

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
                transform.translation +=
                    (edge.normalize() * EDGE_PAN * dt).extend(0.0) * if fast { 1.4 } else { 1.0 };
            }
        }
    }

    let scroll = scroll_acc.delta.y + scroll_acc.delta.x * 0.25;
    if scroll.abs() >= f32::EPSILON {
        let z = ZOOM_FACTOR.powf(scroll.clamp(-5.0, 5.0));
        let s = (transform.scale.x * z).clamp(ZOOM_CLAMP.0, ZOOM_CLAMP.1);
        transform.scale = Vec3::splat(s);
    }

    let zoom_key = 1.65 * dt;
    if keys.pressed(bindings.map_zoom_in) {
        let s = (transform.scale.x * (1.0 + zoom_key)).clamp(ZOOM_CLAMP.0, ZOOM_CLAMP.1);
        transform.scale = Vec3::splat(s);
    }
    if keys.pressed(bindings.map_zoom_out) {
        let s = (transform.scale.x / (1.0 + zoom_key)).clamp(ZOOM_CLAMP.0, ZOOM_CLAMP.1);
        transform.scale = Vec3::splat(s);
    }

    if keys.just_pressed(bindings.map_rotate_ccw) {
        transform.rotate_z(ROTATE_STEP);
    }
    if keys.just_pressed(bindings.map_rotate_cw) {
        transform.rotate_z(-ROTATE_STEP);
    }
}
