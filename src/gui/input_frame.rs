//! Per-frame **coalesced** input snapshot for gameplay systems (`base_visual_dev01` input path).
//!
//! Bevy’s [`AccumulatedMouseMotion`](bevy::input::mouse::AccumulatedMouseMotion) already sums OS
//! motion within the frame; [`AccumulatedMouseScroll`](bevy::input::mouse::AccumulatedMouseScroll)
//! does the same for wheel deltas. This resource is the single read surface so camera code never
//! loops raw [`MouseMotion`](bevy::input::mouse::MouseMotion) events in hot paths.

use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll};
use bevy::prelude::*;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct InputFrame {
    /// Logical pixels (same space as `AccumulatedMouseMotion::delta`).
    pub pointer_delta: Vec2,
    /// Wheel delta coalesced for the frame (matches prior `map_camera` scroll blend).
    pub scroll_delta: f32,
    /// True while a drag-style button is held (middle / left).
    pub drag_active: bool,
    /// Monotonic counter bumped once per `PreUpdate` capture (ordering / debugging).
    pub frame_number: u64,
}

pub struct InputFramePlugin;

impl Plugin for InputFramePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFrame>().add_systems(
            PreUpdate,
            capture_input_frame_from_accumulators,
        );
    }
}

fn capture_input_frame_from_accumulators(
    mut frame: ResMut<InputFrame>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    frame.pointer_delta = motion.delta;
    frame.scroll_delta = scroll.delta.y + scroll.delta.x * 0.25;
    frame.drag_active =
        mouse.pressed(MouseButton::Middle) || mouse.pressed(MouseButton::Left);
    frame.frame_number = frame.frame_number.wrapping_add(1);
}
