//! Projection-owned burst hints for instanced FX (`base_gui_next.md`).
//! **Must not** become a second GPU upload path — hints are filled from committed
//! [`crate::render::sim_visual_extract::FireVisualFrame`] during projection only.

use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FxParticleBurstRequest {
    pub chunk_ix: i32,
    pub chunk_iy: i32,
    pub intensity: f32,
}

/// Collect burst hints from committed fire visual rows (no live ECS scan).
#[must_use]
pub fn collect_burst_hints_from_fire_visual(
    rows: &[crate::render::sim_visual_extract::FireVisualGpuInstance],
    heat_threshold: f32,
) -> Vec<FxParticleBurstRequest> {
    let mut out = Vec::new();
    for row in rows {
        let heat = row.heat();
        if heat < heat_threshold {
            continue;
        }
        let xy = row.chunk_grid_xy();
        out.push(FxParticleBurstRequest {
            chunk_ix: xy.x.floor() as i32,
            chunk_iy: xy.y.floor() as i32,
            intensity: heat,
        });
    }
    out
}
