//! Scalar **field layers** per cell (economy / control / sensing) — parallel to [`super::ChunkStrategicOverlay`]
//! but oriented toward unified planner reads ([`super::WorldReadSnapshot`]).
//!
//! Nodes and zones write **deltas** into these layers; systems blend, decay, and expose summaries.

use bevy::prelude::*;

/// Packed cell — matches the “world as layered fields” mental model (single grid cell).
#[derive(Clone, Copy, Debug, Default)]
pub struct ChunkFieldCell {
    pub supply: f32,
    pub demand: f32,
    pub power: f32,
    pub control: f32,
    pub visibility: f32,
    pub threat: f32,
    pub morale: f32,
}

impl ChunkFieldCell {
    #[inline]
    pub fn add_scaled(&mut self, o: &Self, scale: f32) {
        self.supply += o.supply * scale;
        self.demand += o.demand * scale;
        self.power += o.power * scale;
        self.control += o.control * scale;
        self.visibility += o.visibility * scale;
        self.threat += o.threat * scale;
        self.morale += o.morale * scale;
    }
}

/// Epoch bumped whenever field layers are fully recomputed (GPU / planner invalidation).
#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct WorldFieldLayerEpoch(pub u64);

/// Config: authoritative chunk slab shape for field layers (mirrors first seen [`crate::terrain::generation::ChunkCellMatrix::size`]).
#[derive(Resource, Clone, Copy, Debug)]
pub struct WorldFieldLayerConfig {
    pub cells_per_chunk: UVec2,
}

impl Default for WorldFieldLayerConfig {
    fn default() -> Self {
        Self {
            cells_per_chunk: UVec2::new(32, 32),
        }
    }
}
