//! Per-frame **coalesced** pointer delta for gameplay systems (`base_visual_dev01` input path).
//!
//! Bevy’s [`AccumulatedMouseMotion`](bevy::input::mouse::AccumulatedMouseMotion) already sums OS
//! motion within the frame; this resource is the single read surface so we never loop raw
//! [`MouseMotion`](bevy::input::mouse::MouseMotion) events in hot paths.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct InputFrame {
    /// Logical pixels (same space as `AccumulatedMouseMotion::delta`).
    pub pointer_delta: Vec2,
}

pub struct InputFramePlugin;

impl Plugin for InputFramePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputFrame>().add_systems(
            PreUpdate,
            capture_input_frame_from_mouse_accumulator,
        );
    }
}

fn capture_input_frame_from_mouse_accumulator(
    mut frame: ResMut<InputFrame>,
    acc: Res<AccumulatedMouseMotion>,
) {
    frame.pointer_delta = acc.delta;
}
