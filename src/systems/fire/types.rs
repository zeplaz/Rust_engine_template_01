//! Fire-related components shared across chunk fire modules.

use bevy::prelude::*;

/// Per-cell heat/fuel (**SoA**) aligned with [`ChunkCellMatrix`] cell count.
#[derive(Component, Debug, Clone)]
pub struct ChunkFireOverlay {
    pub heat: Vec<f32>,
    pub fuel: Vec<f32>,
}
