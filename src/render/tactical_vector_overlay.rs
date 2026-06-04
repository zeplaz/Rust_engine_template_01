//! Tactical wire overlay (**VFX-VECTOR-SHAPES-001**) — `bevy_vector_shapes` on projection spine.
//!
//! Reads [`RenderProjectionGraph::fire`] instance rows only; no duplicate fire extract.

use bevy::prelude::*;
use bevy_vector_shapes::prelude::*;

use crate::engine::states::BaseState;
use crate::render::extraction::RenderProjectionGraph;

/// Max wire primitives per frame (tactical zoom; not a sim authority cap).
const MAX_WIRE_SHAPES_PER_FRAME: usize = 48;

/// Wire ring radius in world units at each projected fire instance.
const WIRE_RING_RADIUS: f32 = 3.5;

/// Spine-attached tactical overlay state.
#[derive(Resource, Debug, Clone, Default)]
pub struct TacticalVectorOverlayState {
    pub wired: bool,
    pub overlay_revision: u64,
    pub projection_stamp: u64,
    pub drawn_shapes: u32,
}

#[must_use]
pub fn tactical_vector_overlay_witness_json(state: &TacticalVectorOverlayState) -> serde_json::Value {
    serde_json::json!({
        "gate": "VFX-VECTOR-SHAPES-001",
        "wired": state.wired,
        "overlay_revision": state.overlay_revision,
        "projection_stamp": state.projection_stamp,
        "drawn_shapes": state.drawn_shapes,
        "green": state.wired && state.drawn_shapes > 0 && state.projection_stamp > 0,
        "backend": "bevy_vector_shapes",
    })
}

pub fn sync_tactical_vector_overlay_from_projection(
    graph: Res<RenderProjectionGraph>,
    mut state: ResMut<TacticalVectorOverlayState>,
) {
    state.wired = true;
    state.projection_stamp = graph.fire.snapshot_stamp;
}

/// Draw magenta wire rings at tactical fire instance world samples (L3 only).
pub fn draw_tactical_vector_wire_overlay(
    graph: Res<RenderProjectionGraph>,
    mut painter: ShapePainter,
    mut state: ResMut<TacticalVectorOverlayState>,
) {
    let buffer = &graph.fire.instance_buffer;
    if buffer.is_empty() {
        state.drawn_shapes = 0;
        return;
    }

    state.wired = true;
    state.projection_stamp = graph.fire.snapshot_stamp;
    state.drawn_shapes = 0;

    painter.set_3d();
    painter.set_color(Color::srgb(0.95, 0.15, 0.85));
    painter.hollow = true;
    painter.thickness = 1.25;

    for row in buffer.iter().take(MAX_WIRE_SHAPES_PER_FRAME) {
        let world = row.world_xyz_radius;
        painter.set_translation(Vec3::new(world.x, world.y, world.z + 0.05));
        painter.circle(WIRE_RING_RADIUS);
        state.drawn_shapes = state.drawn_shapes.saturating_add(1);
    }

    state.overlay_revision = state.overlay_revision.saturating_add(1);
}

pub struct TacticalVectorOverlayPlugin;

impl Plugin for TacticalVectorOverlayPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TacticalVectorOverlayState>()
            .add_plugins(ShapePlugin::default())
            .add_systems(
                PostUpdate,
                (
                    sync_tactical_vector_overlay_from_projection,
                    draw_tactical_vector_wire_overlay,
                )
                    .chain()
                    .after(crate::render::FireVisualFrameSet::ProjectGpu)
                    .run_if(in_state(BaseState::Simulation)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_green_when_shapes_drawn() {
        let state = TacticalVectorOverlayState {
            wired: true,
            overlay_revision: 1,
            projection_stamp: 10,
            drawn_shapes: 2,
        };
        let j = tactical_vector_overlay_witness_json(&state);
        assert_eq!(j["backend"], "bevy_vector_shapes");
        assert_eq!(j["green"], true);
    }
}
