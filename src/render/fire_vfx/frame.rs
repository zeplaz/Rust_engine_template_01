//! CPU-side fire particle frame extracted to render world.

use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

use crate::gui::RepresentationBand;

use super::pack::GpuParticleInstance;
use super::witness::FireSparkWitness;

/// CPU-side particle snapshot for one committed sim step (LOD-shaped).
#[derive(Resource, Debug, Clone, ExtractResource)]
pub struct WorldFireParticleFrame {
    pub snapshot_stamp: u64,
    /// Wall clock for shader pulse (avoid using `snapshot_stamp` — it advances with sim, not frames).
    pub anim_time_secs: f32,
    pub active_band: RepresentationBand,
    pub gpu_capacity: usize,
    pub instances: Vec<GpuParticleInstance>,
    pub spark_witness: FireSparkWitness,
}

impl Default for WorldFireParticleFrame {
    fn default() -> Self {
        Self {
            snapshot_stamp: 0,
            anim_time_secs: 0.0,
            active_band: RepresentationBand::Full,
            gpu_capacity: usize::MAX,
            instances: Vec::new(),
            spark_witness: FireSparkWitness::default(),
        }
    }
}

/// Render-world view of the latest particle upload (count only — backend metric).
#[derive(Resource, Default)]
pub struct WorldFireParticleGpuStorage {
    pub instance_count: u32,
    pub expanded_vertex_count: u32,
}
