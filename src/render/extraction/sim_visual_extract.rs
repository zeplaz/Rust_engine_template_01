//! GPU-ready **snapshots** of sim-owned fire/smoke state (`prompts/guides/base_gui_next.md` Stage 2).
//!
//! [`FireVisualGpuInstance`] is the packed **proxy row** (GPU storage layout); the canonical per-frame **truth** snapshot is
//! [`FireVisualFrame`] (full instances + chunk heat). [`crate::render::extraction::RenderProjectionGraph`] carries the
//! LOD-shaped fire view for GPU upload. [`SimFireEmitterVisualExtract`] mirrors **full** truth instance rows on the
//! **main** world for legacy CPU/debug callers; the render world uploads [`crate::render::pipelines::gpu_weather_fire_field::FireVisualGpuInstanceStorage`]
//! from extracted [`crate::render::extraction::RenderProjectionGraph`].
//!
//! [`ClimateVisualAggregate`] is the **single** world mean over chunk weather + ecology for GPU / overlay
//! consumers; only [`crate::systems::atmosphere::visual_extract::publish_climate_visual_aggregate`] scans those components.
//!
//! Extract snapshots below document **Staleness** per [`LOOP-07`](../../dev/plan_ref_books_upgrade_v2.md) (GEA 17.6.3.3):
//! writers name which sim step they sample and why visual lag is bounded.
use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::extract_resource::ExtractResource;
use bytemuck::{Pod, Zeroable};

use crate::render::lighting::{FireLightEmission, FireLightType};
use crate::systems::sim_control::SimStepStamp;

/// Packed **std430-friendly** fire visual row (`vec4` lanes) for GPU storage + CPU clustering.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct FireVisualGpuInstance {
    /// Chunk grid `xy` as `f32`, `z` = heat `[0,1]`, `w` = luminosity.
    pub chunk_xy_heat_lum: Vec4,
    /// World sample `xyz`, `w` = influence radius (**light falloff**, not billboard size — see Phase-F remap).
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

/// Heat above this counts as “active fire” for debug overlays / cheap visibility gates.
/// Aligns with per-cell extinction in [`crate::systems::fire::chunk_fire_overlay`] / surface fire
/// (`< 0.02` clamps to zero) so smolder near the floor is still visible to representation.
pub const FIRE_VISUAL_ACTIVE_HEAT_EPS: f32 = 0.02;

/// Canonical **CPU** fire visual snapshot for the frame (full detail). Filled by
/// [`crate::render::extraction::fire_visual_extract::build_fire_visual_frame_from_simulation`] from
/// [`crate::render::fx_spine::fire_chunk_runtime::FireSimulationSnapshot`] (simulation is ECS-only in `extract_fire_simulation_snapshot`).
///
/// Staleness: **this sim tick** when `stamp` matches [`crate::systems::sim_control::SimTick`] after
/// [`crate::render::extraction::fire_visual_extract::extract_fire_simulation_snapshot`] runs; else retained
/// **previous tick** (cadence gate / `snapshot_unchanged`) — acceptable because
/// [`crate::render::extraction::render_projection_graph::fire_projection_stamp_aligned`] allows ≤1 tick vs
/// [`crate::render::extraction::visual_snapshot_commit::CommittedVisualSnapshotFence`] and GPU paths never write sim.
#[derive(Resource, Default, Debug, Clone)]
pub struct FireVisualFrame {
    pub stamp: SimStepStamp,
    pub instances: Vec<FireVisualGpuInstance>,
    pub chunk_heat: Vec<ChunkFireHeat>,
}

impl FireVisualFrame {
    /// Chunk indices with visual heat above [`FIRE_VISUAL_ACTIVE_HEAT_EPS`] in either instances or `chunk_heat`.
    #[must_use]
    pub fn chunk_coords_with_active_heat(&self) -> std::collections::HashSet<IVec2> {
        use std::collections::HashSet;
        let mut out = HashSet::default();
        for row in &self.instances {
            if row.heat() <= FIRE_VISUAL_ACTIVE_HEAT_EPS {
                continue;
            }
            let xy = row.chunk_grid_xy();
            out.insert(IVec2::new(xy.x as i32, xy.y as i32));
        }
        for h in &self.chunk_heat {
            if h.heat > FIRE_VISUAL_ACTIVE_HEAT_EPS {
                out.insert(h.chunk);
            }
        }
        out
    }
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
///
/// Staleness: mirrors tactical [`FireVisualFrame`] from [`crate::render::extraction::fire_view_extract::build_fire_visual_frames_by_view`]
/// on the same frame (**this tick** when extract ran; **previous tick** when cadence skipped rebuild) — acceptable for
/// legacy CPU/debug callers only; authoritative GPU path uses [`crate::render::extraction::RenderProjectionGraph`].
#[derive(Resource, Default, Clone, Debug)]
pub struct SimFireEmitterVisualExtract {
    pub instances: Vec<FireEmitterGpu>,
}

/// Non-empty chunk smoke cells for fog / volume sampling.
///
/// Staleness: **this sim tick** — [`crate::systems::atmosphere::visual_extract::publish_sim_visual_extract`] reads
/// `ChunkSmokeField` / substrate contamination after `ChunkEnvironmentSet::Fire` for the current step; render-world
/// extract may trail main world by one render frame (Bevy extract), not an extra sim step.
#[derive(Resource, Default, Clone, Debug, ExtractResource)]
#[extract_app(RenderApp)]
pub struct SimChunkSmokeVisualExtract {
    pub instances: Vec<ChunkSmokeGpu>,
}

/// World-aggregated chunk weather + ecology for **visual** paths (GPU uniforms, precip overlay).
///
/// Filled in [`crate::systems::atmosphere::AtmospherePipelineSet::VisualExtract`]; consumers must not query
/// [`crate::systems::weather::ChunkWeather`] / [`crate::systems::ecology::ChunkEcology`] again for the same means.
///
/// Staleness: **this sim tick** — [`crate::systems::atmosphere::visual_extract::publish_climate_visual_aggregate`]
/// scans live chunk weather/ecology after environment fire/weather for the step; presentation-only means, so no gameplay coupling.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct ClimateVisualAggregate {
    pub mean_rain: f32,
    pub mean_snow: f32,
    pub mean_fog_density: f32,
    pub mean_wind_speed: f32,
    /// Mean map-plane wind (m/s) from chunk weather projections (W2 local VFX).
    pub mean_wind_mps: Vec2,
    pub mean_lightning_risk: f32,
    pub mean_biomass: f32,
    pub mean_fire_risk: f32,
    pub weather_chunk_count: u32,
    pub ecology_chunk_count: u32,
}
