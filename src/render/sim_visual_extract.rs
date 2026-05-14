//! GPU-ready **snapshots** of sim-owned fire/smoke state (`prompts/guides/base_gui_next.md` Stage 2).
//!
//! [`FireVisualGpuInstance`] is the packed **proxy row** (GPU storage layout); the canonical per-frame **truth** snapshot is
//! [`FireVisualFrame`] (full instances + chunk heat). [`crate::render::extraction::RenderProjectionGraph`] carries the
//! LOD-shaped fire view for GPU upload. [`SimFireEmitterVisualExtract`] mirrors **full** truth instance rows on the
//! **main** world for legacy CPU/debug callers; the render world uploads [`crate::render::gpu_weather_fire_field::FireVisualGpuInstanceStorage`]
//! from extracted [`crate::render::extraction::RenderProjectionGraph`].
//!
//! [`ClimateVisualAggregate`] is the **single** world mean over chunk weather + ecology for GPU / overlay
//! consumers; only [`crate::systems::atmosphere::visual_extract::publish_climate_visual_aggregate`] scans those components.
use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::render::extract_resource::ExtractResource;
use bytemuck::{Pod, Zeroable};

use crate::render::lighting::{FireLightEmission, FireLightType};

/// Packed **std430-friendly** fire visual row (`vec4` lanes) for GPU storage + CPU clustering.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct FireVisualGpuInstance {
    /// Chunk grid `xy` as `f32`, `z` = heat `[0,1]`, `w` = luminosity.
    pub chunk_xy_heat_lum: Vec4,
    /// World sample `xyz`, `w` = influence radius.
    pub world_xyz_radius: Vec4,
    /// `x` smoke density, `y` ember rate, `z` visibility reduction, `w` extract priority.
    pub smoke_ember_vis_priority: Vec4,
    /// Smoke color `rgb`, `w` = toxic density `[0,1]`.
    pub smoke_color_toxic: Vec4,
    /// Fog tint `rgb`, `w` = combustion class ordinal `0..=4` for light typing.
    pub fog_rgb_combust_ord: Vec4,
}

impl FireVisualGpuInstance {
    #[inline]
    pub fn chunk_grid_xy(&self) -> Vec2 {
        self.chunk_xy_heat_lum.xy()
    }

    #[inline]
    pub fn heat(&self) -> f32 {
        self.chunk_xy_heat_lum.z
    }

    #[inline]
    pub fn luminosity(&self) -> f32 {
        self.chunk_xy_heat_lum.w
    }

    #[inline]
    pub fn cluster_emission(&self) -> FireLightEmission {
        FireLightEmission {
            position: self.world_xyz_radius.xyz(),
            heat: self.chunk_xy_heat_lum.z,
            luminosity: self.chunk_xy_heat_lum.w,
            smoke_density: self.smoke_ember_vis_priority.x,
            radius: self.world_xyz_radius.w,
            priority: self.smoke_ember_vis_priority.w,
            fire_type: fire_light_type_from_combustion_ord(self.fog_rgb_combust_ord.w),
        }
    }

    /// Row compatible with the former `FireEmitter`-driven extract (GPU field mean / burst hints).
    #[inline]
    pub fn to_fire_emitter_gpu(&self) -> FireEmitterGpu {
        FireEmitterGpu {
            chunk_xy: Vec4::new(
                self.chunk_xy_heat_lum.x,
                self.chunk_xy_heat_lum.y,
                0.0,
                0.0,
            ),
            params: Vec4::new(
                self.chunk_xy_heat_lum.z,
                self.smoke_ember_vis_priority.x,
                self.smoke_ember_vis_priority.y,
                self.smoke_color_toxic.w,
            ),
        }
    }
}

/// One chunk’s **visual** heat + smoke scalars in the CPU snapshot (`FireVisualFrame::chunk_heat`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ChunkFireHeat {
    pub chunk: IVec2,
    pub heat: f32,
    pub smoke: f32,
}

use crate::systems::sim_control::SimStepStamp;

/// Canonical **CPU** fire visual snapshot for the frame (full detail). Built by [`crate::render::extraction::fire_visual_extract::extract_fire_visual_frame`] only.
#[derive(Resource, Default, Debug, Clone)]
pub struct FireVisualFrame {
    pub stamp: SimStepStamp,
    pub instances: Vec<FireVisualGpuInstance>,
    pub chunk_heat: Vec<ChunkFireHeat>,
}

#[inline]
fn fire_light_type_from_combustion_ord(ord: f32) -> FireLightType {
    match ord.round().clamp(0.0, 4.0) as u8 {
        1 => FireLightType::Fuel,
        2 => FireLightType::Chemical,
        3 => FireLightType::Electrical,
        4 => FireLightType::Structure,
        _ => FireLightType::Forest,
    }
}

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

/// Latest fire emitter snapshot (cleared + refilled each extract tick on the **main** world only).
#[derive(Resource, Default, Clone, Debug)]
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
