//! Fire-related components shared across chunk fire modules.

use bevy::prelude::*;

/// Per-cell heat/fuel (**SoA**) aligned with [`ChunkCellMatrix`] cell count.
#[derive(Component, Debug, Clone)]
pub struct ChunkFireOverlay {
    pub heat: Vec<f32>,
    pub fuel: Vec<f32>,
    /// Smoke density proxy per cell `[0, 1]` — feeds atmosphere / preview (`base_fire_sim.md`).
    pub smoke: Vec<f32>,
    /// Toxic products per cell `[0, 1]`.
    pub toxic: Vec<f32>,
    /// Ticks left that this cell stays at residual heat after fuel drops below spread.
    /// Written only by [`super::chunk_fire_overlay::chunk_fire_overlay_tick`].
    pub burn_hold: Vec<u16>,
}
