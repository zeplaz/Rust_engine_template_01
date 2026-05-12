//! GPU-ready **snapshots** of sim-owned fire/smoke state (`prompts/guides/base_gui_next.md` Stage 2).
//!
//! Simulation systems **write** these resources; render passes read only them (no gameplay queries in render).
//! `ExtractResource` copies the whole resource to the render world each frame (`gfx-extract-2` baseline);
//! dedicated GPU buffer uploads from extracted `Vec`s remain future work.
//!
//! [`ClimateVisualAggregate`] is the **single** world mean over chunk weather + ecology for GPU / overlay
//! consumers; only [`crate::systems::atmosphere::visual_extract::publish_climate_visual_aggregate`] scans those components.
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;

/// One chunk-aligned fire emitter row for instancing / compute (`base_gui_next.md`).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FireEmitterGpu {
    /// `xy` = chunk grid index, `zw` = unused (world XY later).
    pub chunk_xy: Vec4,
    /// `x` intensity, `y` smoke_rate, `z` ember_rate, `w` fuel toxic scalar `[0,1]`.
    pub params: Vec4,
}

/// Chunk smoke scalars for volumetric / fog passes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChunkSmokeGpu {
    pub chunk_xy: Vec4,
    /// `x` density, `y` toxicity, `z` visibility_penalty, `w` unused.
    pub density_tox_vis: Vec4,
}

/// Latest fire emitter snapshot (cleared + refilled each extract tick).
#[derive(Resource, Default, Clone, Debug, ExtractResource)]
pub struct SimFireEmitterVisualExtract {
    pub instances: Vec<FireEmitterGpu>,
}

/// Non-empty chunk smoke cells for fog / volume sampling.
#[derive(Resource, Default, Clone, Debug, ExtractResource)]
pub struct SimChunkSmokeVisualExtract {
    pub instances: Vec<ChunkSmokeGpu>,
}

/// World-aggregated chunk weather + ecology for **visual** paths (GPU uniforms, precip overlay).
///
/// Filled in [`crate::systems::atmosphere::AtmospherePipelineSet::VisualExtract`]; consumers must not query
/// [`crate::systems::weather::ChunkWeather`] / [`crate::systems::ecology::ChunkEcology`] again for the same means.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ClimateVisualAggregate {
    pub mean_rain: f32,
    pub mean_snow: f32,
    pub mean_fog_density: f32,
    pub mean_wind_speed: f32,
    pub mean_lightning_risk: f32,
    pub mean_biomass: f32,
    pub mean_fire_risk: f32,
    pub weather_chunk_count: u32,
    pub ecology_chunk_count: u32,
}
