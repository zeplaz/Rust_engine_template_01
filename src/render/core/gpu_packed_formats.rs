//! Packed GPU row layouts + LOD-shaped allocation ceilings for the buffer registry.

use crate::gui::{WorldLodBand, WorldResolutionPolicy};

pub const LOGISTICS_OVERLAY_ROW_FORMAT: PackedFormatId = PackedFormatId(4);
pub const ECOLOGY_OVERLAY_ROW_FORMAT: PackedFormatId = PackedFormatId(5);

use crate::render::gpu_buffer_registry::{
    BufferId, ECOLOGY_OVERLAY_BUFFER, FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER,
    FIRE_PARTICLE_INSTANCES_BUFFER, FIRE_VISUAL_INSTANCES_BUFFER, HEAT_DIFFUSION_FIELD_BUFFER,
    LOGISTICS_OVERLAY_BUFFER, WATER_PARTICLE_EXPANDED_VERTICES_BUFFER,
    WATER_PARTICLE_INSTANCES_BUFFER,
};
use crate::render::domain_overlay_gpu::{EcologyOverlayGpuRow, LogisticsOverlayGpuRow};
use crate::render::fire_vfx::pack::{GpuParticleInstance, GpuParticleQuadVertex};
use crate::render::gpu_water_particles::{GpuWaterParticleInstance, GpuWaterParticleQuadVertex};
use crate::render::sim_visual_extract::FireVisualGpuInstance;

/// Stable numeric identity for a packed row layout (stride authority).
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PackedFormatId(pub u32);

pub const FIRE_VISUAL_INSTANCE_FORMAT: PackedFormatId = PackedFormatId(1);
pub const HEAT_DIFFUSION_CELL_FORMAT: PackedFormatId = PackedFormatId(2);
/// Fire lane uses [`GpuParticleInstance`] — same 32-byte stride as [`GpuInstancedQuadInstance`].
pub const FIRE_PARTICLE_INSTANCE_FORMAT: PackedFormatId = PackedFormatId(3);
pub const FIRE_PARTICLE_EXPANDED_VERTEX_FORMAT: PackedFormatId = PackedFormatId(6);
pub const WATER_PARTICLE_INSTANCE_FORMAT: PackedFormatId = PackedFormatId(7);
pub const WATER_PARTICLE_EXPANDED_VERTEX_FORMAT: PackedFormatId = PackedFormatId(8);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PackedBufferFormat {
    pub format_id: PackedFormatId,
    pub buffer_id: BufferId,
    pub stride: u32,
}

#[must_use]
pub const fn fire_visual_instance_format() -> PackedBufferFormat {
    PackedBufferFormat {
        format_id: FIRE_VISUAL_INSTANCE_FORMAT,
        buffer_id: FIRE_VISUAL_INSTANCES_BUFFER,
        stride: std::mem::size_of::<FireVisualGpuInstance>() as u32,
    }
}

pub const HEAT_DIFFUSION_CELL_STRIDE: u32 = 16;

#[must_use]
pub const fn fire_particle_instance_format() -> PackedBufferFormat {
    PackedBufferFormat {
        format_id: FIRE_PARTICLE_INSTANCE_FORMAT,
        buffer_id: FIRE_PARTICLE_INSTANCES_BUFFER,
        stride: std::mem::size_of::<GpuParticleInstance>() as u32,
    }
}

#[must_use]
pub const fn fire_particle_expanded_vertex_format() -> PackedBufferFormat {
    PackedBufferFormat {
        format_id: FIRE_PARTICLE_EXPANDED_VERTEX_FORMAT,
        buffer_id: FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER,
        stride: std::mem::size_of::<GpuParticleQuadVertex>() as u32,
    }
}

#[must_use]
pub const fn water_particle_instance_format() -> PackedBufferFormat {
    PackedBufferFormat {
        format_id: WATER_PARTICLE_INSTANCE_FORMAT,
        buffer_id: WATER_PARTICLE_INSTANCES_BUFFER,
        stride: std::mem::size_of::<GpuWaterParticleInstance>() as u32,
    }
}

#[must_use]
pub const fn water_particle_expanded_vertex_format() -> PackedBufferFormat {
    PackedBufferFormat {
        format_id: WATER_PARTICLE_EXPANDED_VERTEX_FORMAT,
        buffer_id: WATER_PARTICLE_EXPANDED_VERTICES_BUFFER,
        stride: std::mem::size_of::<GpuWaterParticleQuadVertex>() as u32,
    }
}

#[must_use]
pub const fn heat_diffusion_cell_format() -> PackedBufferFormat {
    PackedBufferFormat {
        format_id: HEAT_DIFFUSION_CELL_FORMAT,
        buffer_id: HEAT_DIFFUSION_FIELD_BUFFER,
        stride: HEAT_DIFFUSION_CELL_STRIDE,
    }
}

#[must_use]
pub const fn logistics_overlay_row_format() -> PackedBufferFormat {
    PackedBufferFormat {
        format_id: LOGISTICS_OVERLAY_ROW_FORMAT,
        buffer_id: LOGISTICS_OVERLAY_BUFFER,
        stride: std::mem::size_of::<LogisticsOverlayGpuRow>() as u32,
    }
}

#[must_use]
pub const fn ecology_overlay_row_format() -> PackedBufferFormat {
    PackedBufferFormat {
        format_id: ECOLOGY_OVERLAY_ROW_FORMAT,
        buffer_id: ECOLOGY_OVERLAY_BUFFER,
        stride: std::mem::size_of::<EcologyOverlayGpuRow>() as u32,
    }
}

#[must_use]
pub fn packed_byte_size(format: PackedBufferFormat, row_count: usize) -> u64 {
    let rows = row_count.max(1) as u64;
    (rows * format.stride as u64).max(format.stride as u64)
}

/// LOD-shaped row ceilings for registry allocation (shrink-on-band policy input).
#[derive(Clone, Copy, Debug, Default)]
pub struct LodBandBufferPolicy;

impl LodBandBufferPolicy {
    #[must_use]
    pub fn fire_storage_rows(resolution: &WorldResolutionPolicy) -> usize {
        let cap = resolution.fire_instance_cap;
        if cap == usize::MAX {
            cap
        } else {
            cap.max(1)
        }
    }

    #[must_use]
    pub fn heat_diffusion_rows(band: WorldLodBand, active_rows: usize) -> usize {
        let active = active_rows.max(1);
        match band {
            WorldLodBand::LocalTactical => active,
            WorldLodBand::Operational => active.min(512).max(1),
            WorldLodBand::Strategic => active.min(128).max(1),
            WorldLodBand::Macro => active.min(32).max(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::WorldResolutionPolicy;

    #[test]
    fn fire_particle_stride_matches_pod_layout() {
        assert_eq!(
            fire_particle_instance_format().stride,
            std::mem::size_of::<GpuParticleInstance>() as u32
        );
    }

    #[test]
    fn fire_stride_matches_pod_layout() {
        assert_eq!(
            fire_visual_instance_format().stride as usize,
            std::mem::size_of::<FireVisualGpuInstance>()
        );
    }

    #[test]
    fn heat_diffusion_stride_matches_compute_pod() {
        assert_eq!(
            heat_diffusion_cell_format().stride as usize,
            std::mem::size_of::<crate::compute::HeatDiffusionGpuCell>()
        );
    }

    #[test]
    fn heat_diffusion_rows_shrink_on_macro_band() {
        assert_eq!(
            LodBandBufferPolicy::heat_diffusion_rows(WorldLodBand::Macro, 10_000),
            32
        );
        assert_eq!(
            LodBandBufferPolicy::heat_diffusion_rows(WorldLodBand::LocalTactical, 10_000),
            10_000
        );
    }

    #[test]
    fn operational_fire_cap_respects_resolution_policy() {
        let mut resolution = WorldResolutionPolicy::default();
        resolution.fire_instance_cap = 48;
        assert_eq!(LodBandBufferPolicy::fire_storage_rows(&resolution), 48);
    }
}
