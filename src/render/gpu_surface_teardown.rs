//! Graceful GPU surface release before window teardown (Vulkan `SurfaceAcquireSemaphores` panic).
//!
//! When the OS closes the window while a swapchain texture is still acquired, wgpu can panic on
//! surface drop. Deactivate window cameras and drop preview offscreen targets first.

use bevy::app::AppExit;
use bevy::prelude::*;
use bevy::window::WindowCloseRequested;

use crate::engine::{EngineLaunchArgs, TestScene};
use crate::engine::debug_maneuver::{DebugManeuver, GRACEFUL_EXIT_FRAMES_AFTER_PROOF};
use crate::gui::editor::world_preview::{
    WorldPreviewGpuCamera, WorldPreviewRenderTargetBindBarrier, WorldPreviewRenderTargetRegistry,
};

/// After `--test visual` writes live proof, exit cleanly after a few frames (avoid manual close panic).
#[derive(Resource, Default, Debug)]
pub struct VisualTestGracefulExit {
    pub armed: bool,
    pub frames_remaining: u32,
}

impl VisualTestGracefulExit {
    pub const FRAMES_AFTER_PROOF: u32 = GRACEFUL_EXIT_FRAMES_AFTER_PROOF;
}

#[allow(dead_code)]
pub fn arm_visual_test_graceful_exit(mut gate: ResMut<VisualTestGracefulExit>) {
    gate.armed = true;
    gate.frames_remaining = VisualTestGracefulExit::FRAMES_AFTER_PROOF;
}

fn deactivate_all_cameras(mut cameras: Query<&mut Camera>) {
    for mut camera in &mut cameras {
        camera.is_active = false;
    }
}

pub fn deactivate_cameras_on_window_close(
    mut close: MessageReader<WindowCloseRequested>,
    cameras: Query<&mut Camera>,
) {
    if close.read().next().is_some() {
        deactivate_all_cameras(cameras);
    }
}

pub fn deactivate_cameras_on_app_exit(
    mut exit: MessageReader<AppExit>,
    cameras: Query<&mut Camera>,
) {
    if exit.read().next().is_some() {
        deactivate_all_cameras(cameras);
    }
}

pub fn release_world_preview_gpu_before_teardown(
    mut close: MessageReader<WindowCloseRequested>,
    mut exit: MessageReader<AppExit>,
    mut commands: Commands,
    preview: Query<Entity, With<WorldPreviewGpuCamera>>,
    mut barrier: ResMut<WorldPreviewRenderTargetBindBarrier>,
    mut registry: ResMut<WorldPreviewRenderTargetRegistry>,
) {
    let teardown = close.read().next().is_some() || exit.read().next().is_some();
    if !teardown {
        return;
    }
    for entity in &preview {
        commands.entity(entity).despawn();
    }
    barrier.clear();
    *registry = WorldPreviewRenderTargetRegistry::default();
}

pub fn tick_visual_test_graceful_exit(
    launch: Option<Res<EngineLaunchArgs>>,
    mut gate: ResMut<VisualTestGracefulExit>,
    mut app_exit: MessageWriter<AppExit>,
) {
    let Some(launch) = launch.as_ref() else {
        return;
    };
    if launch.maneuver != DebugManeuver::FullCapture
        && launch.maneuver != DebugManeuver::UnittestWorld
    {
        return;
    }
    if launch.test_scene != TestScene::Visual || !launch.visual_auto_exit {
        return;
    }
    if !gate.armed || gate.frames_remaining == 0 {
        return;
    }
    gate.frames_remaining -= 1;
    if gate.frames_remaining == 0 {
        info!(
            target: "stage5_full_app_harness",
            "visual test proof committed — requesting graceful AppExit"
        );
        app_exit.write(AppExit::Success);
    }
}

pub struct GpuSurfaceTeardownPlugin;

impl Plugin for GpuSurfaceTeardownPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VisualTestGracefulExit>()
            .add_systems(
                First,
                (
                    deactivate_cameras_on_window_close,
                    deactivate_cameras_on_app_exit,
                ),
            )
            .add_systems(
                Last,
                (
                    release_world_preview_gpu_before_teardown,
                    tick_visual_test_graceful_exit,
                )
                    .chain(),
            );
    }
}
