//! Chunk heat **diffusion** compute kernel (CPU v1) with owned ping-pong buffers and GPU registry sync.
//!
//! Reads [`FireVisualFrame`] + [`WorldLodMap`] only; writes [`HeatDiffusionFieldBuffers`] (compute-owned).

use std::collections::HashMap;

use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::render::extract_resource::{ExtractResource, ExtractResourcePlugin};
use bevy::render::render_resource::BufferUsages;
use bevy::render::{Render, RenderApp, RenderSystems};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bytemuck::{Pod, Zeroable};

use crate::gui::{RepresentationResult, WorldLodBand, WorldLodMap, WorldRepresentationFrame};
use crate::render::{
    heat_diffusion_cell_format, packed_byte_size, BufferVisibility, GPUBufferRegistry,
    HEAT_DIFFUSION_FIELD_BUFFER, LodBandBufferPolicy, RegisteredBufferDescriptor,
};
use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualFrame};
use crate::systems::sim_control::SimStepStamp;

use super::frame_snapshots::NavFieldFrame;
use super::ComputeContext;

const DIFFUSION_RATE: f32 = 0.22;
const DECAY_RATE: f32 = 0.02;
const SOURCE_BLEND: f32 = 0.35;

/// One chunk row in the diffusion field (CPU + GPU upload).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HeatDiffusionCell {
    pub chunk: IVec2,
    pub heat: f32,
    pub smoke: f32,
}

/// Packed row for [`HEAT_DIFFUSION_FIELD_BUFFER`].
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct HeatDiffusionGpuCell {
    pub chunk_xy_heat_smoke: [f32; 4],
}

impl From<HeatDiffusionCell> for HeatDiffusionGpuCell {
    fn from(cell: HeatDiffusionCell) -> Self {
        Self {
            chunk_xy_heat_smoke: [
                cell.chunk.x as f32,
                cell.chunk.y as f32,
                cell.heat,
                cell.smoke,
            ],
        }
    }
}

/// Compute-owned ping-pong field; render world reads via extract + registry upload.
#[derive(Resource, Clone, Debug, Default, ExtractResource)]
pub struct HeatDiffusionFieldBuffers {
    pub stamp: SimStepStamp,
    pub source_stamp: SimStepStamp,
    pub generation: u64,
    pub active_cell_count: usize,
    pub gpu_row_capacity: usize,
    front: Vec<HeatDiffusionCell>,
    back: Vec<HeatDiffusionCell>,
    front_is_read: bool,
}

impl HeatDiffusionFieldBuffers {
    #[must_use]
    pub fn read_cells(&self) -> &[HeatDiffusionCell] {
        if self.front_is_read {
            &self.front
        } else {
            &self.back
        }
    }

    fn write_cells_mut(&mut self) -> &mut Vec<HeatDiffusionCell> {
        if self.front_is_read {
            &mut self.back
        } else {
            &mut self.front
        }
    }

    fn swap(&mut self) {
        self.front_is_read = !self.front_is_read;
        self.generation = self.generation.wrapping_add(1);
    }

    #[must_use]
    pub fn gpu_cells(&self) -> Vec<HeatDiffusionGpuCell> {
        self.read_cells().iter().copied().map(Into::into).collect()
    }
}

/// LOD-shaped policy output for the diffusion kernel (mirrors fire influence coverage).
#[derive(Debug, Clone, Default)]
pub struct HeatDiffusionDispatchNode {
    pub active: bool,
    pub influenced_chunk_count: usize,
    pub target_dispatch_hz: f32,
    pub last_generation: u64,
}

impl HeatDiffusionDispatchNode {
    pub fn plan(&mut self, ctx: &ComputeContext<'_>, influence_active: bool) {
        self.target_dispatch_hz = ctx.policy.compute_budget.dispatch_hz;
        self.active = influence_active;
        if !influence_active {
            self.influenced_chunk_count = 0;
            return;
        }
        let fallback = ctx.lod.global_band();
        self.influenced_chunk_count = ctx
            .fire
            .chunk_heat
            .iter()
            .filter(|row| {
                let band = ctx.lod_map.compute_band_at(row.chunk, fallback);
                ctx.policy.pathfinding_active_at_compute_band(band)
            })
            .count();
    }
}

#[inline]
fn chunk_key(chunk: IVec2) -> i64 {
    (chunk.x as i64) << 32 | (chunk.y as i64 & 0xffff_ffff)
}

#[must_use]
pub fn eligible_fire_rows<'a>(
    fire: &'a FireVisualFrame,
    lod_map: &'a WorldLodMap,
    fallback_band: WorldLodBand,
    policy: &RepresentationResult,
) -> Vec<&'a ChunkFireHeat> {
    fire.chunk_heat
        .iter()
        .filter(|row| {
            let band = lod_map.compute_band_at(row.chunk, fallback_band);
            policy.pathfinding_active_at_compute_band(band)
        })
        .collect()
}

/// One explicit diffusion step: seed from fire, relax toward 4-neighbor mean, decay.
pub fn run_heat_diffusion_step(
    fire_rows: &[&ChunkFireHeat],
    previous: &[HeatDiffusionCell],
    out: &mut Vec<HeatDiffusionCell>,
) {
    out.clear();
    let mut values: HashMap<i64, HeatDiffusionCell> = HashMap::new();
    for cell in previous {
        values.insert(chunk_key(cell.chunk), *cell);
    }
    for row in fire_rows {
        let key = chunk_key(row.chunk);
        let entry = values.entry(key).or_insert(HeatDiffusionCell {
            chunk: row.chunk,
            heat: 0.0,
            smoke: 0.0,
        });
        entry.heat = entry.heat * (1.0 - SOURCE_BLEND) + row.heat * SOURCE_BLEND;
        entry.smoke = entry.smoke * (1.0 - SOURCE_BLEND) + row.smoke * SOURCE_BLEND;
    }

    let mut keys: Vec<i64> = values.keys().copied().collect();
    for row in fire_rows {
        let chunk = row.chunk;
        for neighbor in [
            chunk + IVec2::new(1, 0),
            chunk + IVec2::new(-1, 0),
            chunk + IVec2::new(0, 1),
            chunk + IVec2::new(0, -1),
        ] {
            let key = chunk_key(neighbor);
            if !keys.contains(&key) {
                keys.push(key);
            }
        }
    }

    let fire_by_chunk: HashMap<i64, &ChunkFireHeat> = fire_rows
        .iter()
        .map(|row| (chunk_key(row.chunk), *row))
        .collect();

    let mut next_values = HashMap::new();
    for key in keys {
        let chunk = IVec2::new((key >> 32) as i32, key as i32);
        let center = values.get(&key).copied().unwrap_or(HeatDiffusionCell {
            chunk,
            heat: 0.0,
            smoke: 0.0,
        });
        let mut neighbor_heat = 0.0f32;
        let mut neighbor_smoke = 0.0f32;
        let mut count = 0.0f32;
        for neighbor in [
            chunk + IVec2::new(1, 0),
            chunk + IVec2::new(-1, 0),
            chunk + IVec2::new(0, 1),
            chunk + IVec2::new(0, -1),
        ] {
            if let Some(cell) = values.get(&chunk_key(neighbor)) {
                neighbor_heat += cell.heat;
                neighbor_smoke += cell.smoke;
                count += 1.0;
            }
        }
        let mean_heat = if count > 0.0 {
            neighbor_heat / count
        } else {
            0.0
        };
        let mean_smoke = if count > 0.0 {
            neighbor_smoke / count
        } else {
            0.0
        };
        let mut heat = center.heat * (1.0 - DECAY_RATE) + DIFFUSION_RATE * mean_heat;
        let mut smoke = center.smoke * (1.0 - DECAY_RATE) + DIFFUSION_RATE * mean_smoke;
        if let Some(src) = fire_by_chunk.get(&key) {
            heat = heat * (1.0 - SOURCE_BLEND) + src.heat * SOURCE_BLEND;
            smoke = smoke * (1.0 - SOURCE_BLEND) + src.smoke * SOURCE_BLEND;
        }
        next_values.insert(
            key,
            HeatDiffusionCell {
                chunk,
                heat: heat.clamp(0.0, 1.5),
                smoke: smoke.clamp(0.0, 1.0),
            },
        );
    }

    let mut rows: Vec<HeatDiffusionCell> = next_values.into_values().collect();
    rows.retain(|cell| cell.heat > 1e-4 || cell.smoke > 1e-4);
    rows.sort_by_key(|cell| chunk_key(cell.chunk));
    *out = rows;
}

pub fn advance_heat_diffusion_field(
    field: &mut HeatDiffusionFieldBuffers,
    ctx: &ComputeContext<'_>,
    influence_active: bool,
) {
    if !influence_active {
        field.active_cell_count = 0;
        return;
    }
    let fallback = ctx.lod.global_band();
    let eligible = eligible_fire_rows(ctx.fire, ctx.lod_map, fallback, ctx.policy);
    if eligible.is_empty() && field.read_cells().is_empty() {
        field.active_cell_count = 0;
        field.source_stamp = ctx.fire.stamp;
        field.stamp = ctx.fire.stamp;
        return;
    }
    let read = field.read_cells().to_vec();
    let write = field.write_cells_mut();
    run_heat_diffusion_step(&eligible, &read, write);
    field.swap();
    field.active_cell_count = field.read_cells().len();
    field.gpu_row_capacity = LodBandBufferPolicy::heat_diffusion_rows(
        ctx.policy.world_lod_band,
        field.active_cell_count,
    );
    field.source_stamp = ctx.fire.stamp;
    field.stamp = ctx.fire.stamp;
}

pub fn sync_nav_field_from_heat_diffusion(nav: &mut NavFieldFrame, field: &HeatDiffusionFieldBuffers) {
    nav.stamp = field.stamp;
    nav.cell_count = field.active_cell_count as u32;
}

pub fn register_heat_diffusion_gpu_sync(app: &mut App) {
    app.add_plugins(ExtractResourcePlugin::<HeatDiffusionFieldBuffers>::default());
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app.add_systems(
        Render,
        prepare_heat_diffusion_gpu_buffer.in_set(RenderSystems::PrepareResources),
    );
}

fn prepare_heat_diffusion_gpu_buffer(
    field: Option<Res<HeatDiffusionFieldBuffers>>,
    mut registry: ResMut<GPUBufferRegistry>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    let Some(field) = field else {
        return;
    };
    let format = heat_diffusion_cell_format();
    let alloc_rows = field.gpu_row_capacity.max(1);
    let size_bytes = packed_byte_size(format, alloc_rows);
    if registry
        .ensure_capacity(
            &render_device,
            RegisteredBufferDescriptor {
                id: HEAT_DIFFUSION_FIELD_BUFFER,
                size_bytes,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                visibility: BufferVisibility::ComputeOnly,
                stride: format.stride,
            },
        )
        .is_err()
    {
        return;
    }
    let frame = field.generation;
    let payload = field.gpu_cells();
    if payload.is_empty() {
        let pad = [HeatDiffusionGpuCell::default()];
        let _ = registry.write(&queue, HEAT_DIFFUSION_FIELD_BUFFER, &pad, frame);
    } else {
        let _ = registry.write(&queue, HEAT_DIFFUSION_FIELD_BUFFER, &payload, frame);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::LodCell;

    #[test]
    fn diffusion_spreads_heat_to_neighbor_chunk() {
        let fire = [ChunkFireHeat {
            chunk: IVec2::ZERO,
            heat: 1.0,
            smoke: 0.0,
        }];
        let fire_refs: Vec<&ChunkFireHeat> = fire.iter().collect();
        let mut out = Vec::new();
        run_heat_diffusion_step(&fire_refs, &[], &mut out);
        assert!(out.iter().any(|c| c.chunk == IVec2::ZERO && c.heat > 0.3));
        let mut second = Vec::new();
        run_heat_diffusion_step(&fire_refs, &out, &mut second);
        assert!(second.iter().any(|c| c.chunk == IVec2::new(1, 0) && c.heat > 0.0));
    }

    #[test]
    fn eligible_fire_rows_skip_macro_compute_band() {
        let mut fire = FireVisualFrame::default();
        fire.chunk_heat.push(ChunkFireHeat {
            chunk: IVec2::ZERO,
            heat: 0.8,
            smoke: 0.0,
        });
        fire.chunk_heat.push(ChunkFireHeat {
            chunk: IVec2::new(4, 0),
            heat: 0.8,
            smoke: 0.0,
        });
        let lod_map = WorldLodMap {
            cells: vec![
                LodCell {
                    coord: IVec2::ZERO,
                    render_band: WorldLodBand::LocalTactical,
                    compute_band: WorldLodBand::LocalTactical,
                    importance: 1.0,
                },
                LodCell {
                    coord: IVec2::new(4, 0),
                    render_band: WorldLodBand::Macro,
                    compute_band: WorldLodBand::Macro,
                    importance: 0.0,
                },
            ],
            ..Default::default()
        };
        let rows = eligible_fire_rows(
            &fire,
            &lod_map,
            WorldLodBand::LocalTactical,
            &RepresentationResult::default(),
        );
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].chunk, IVec2::ZERO);
    }

    #[test]
    fn advance_bumps_generation_and_stamp() {
        let mut field = HeatDiffusionFieldBuffers::default();
        let mut fire = FireVisualFrame::default();
        fire.stamp = SimStepStamp::new(7, 42);
        fire.chunk_heat.push(ChunkFireHeat {
            chunk: IVec2::ZERO,
            heat: 0.8,
            smoke: 0.0,
        });
        let lod = WorldRepresentationFrame::default();
        let lod_map = WorldLodMap::default();
        let policy = crate::gui::RepresentationResult::default();
        let ctx = ComputeContext {
            policy: &policy,
            lod: &lod,
            lod_map: &lod_map,
            agents: &crate::compute::AgentFrame::default(),
            navigation: &crate::compute::NavFieldFrame::default(),
            fire: &fire,
        };
        advance_heat_diffusion_field(&mut field, &ctx, true);
        assert_eq!(field.generation, 1);
        assert_eq!(field.stamp, fire.stamp);
        assert!(field.active_cell_count > 0);
    }
}
