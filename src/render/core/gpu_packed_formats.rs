//! Packed GPU row layouts + LOD-shaped allocation ceilings for the buffer registry.
//!
//! # Alignment and packing contract (MEM-02)
//!
//! Book: gea4v1 3.3.7.1 *Alignment and Packing* (p162-164) + 3.3.7.2 *Memory Layout of C++
//! Classes* (p164-168). LEGACY keyword `memory_layout` (gea_v1_e4 §3.3 p135; no dedicated
//! `alignment`/`packing` keyword hit). Matrix row **MEM-02**.
//!
//! Every row that crosses into a GPU storage/vertex buffer obeys the std430-friendly rule the
//! engine already ships:
//!
//! 1. `#[repr(C)]` + `Pod` — field order **is** the byte order (Rust's default repr may reorder,
//!    the C++ class-layout hazard in 3.3.7.2).
//! 2. Rows are built only from 16-byte `vec4` lanes (or `[f32; 4]`) — no `vec3` lane (std140/430
//!    pads `vec3` to 16 and a following scalar then lands at an unexpected offset) and no mixed
//!    scalar/vector rows that would need *implicit* padding.
//! 3. Stride is a multiple of [`GPU_ROW_ALIGN`] (16 B) and equals `size_of::<Row>()` — there is
//!    no trailing implicit padding, so the registry stride, the CPU slice stride and the WGSL
//!    `array<T>` stride agree.
//!
//! Rules 1–3 are enforced at compile time by [`ALL_PACKED_FORMATS`] const assertions and by the
//! `mem02_*` tripwire tests (size + per-field offsets). Adding a row type means adding it to the
//! table; a layout drift fails the build, not a frame.

use crate::gui::{WorldLodBand, WorldResolutionPolicy};

pub const LOGISTICS_OVERLAY_ROW_FORMAT: PackedFormatId = PackedFormatId(4);
pub const ECOLOGY_OVERLAY_ROW_FORMAT: PackedFormatId = PackedFormatId(5);

use crate::render::core::gpu_buffer_registry::{
    BufferId, ECOLOGY_OVERLAY_BUFFER, FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER,
    FIRE_PARTICLE_INSTANCES_BUFFER, FIRE_VISUAL_INSTANCES_BUFFER, HEAT_DIFFUSION_FIELD_BUFFER,
    LOGISTICS_OVERLAY_BUFFER, WATER_PARTICLE_EXPANDED_VERTICES_BUFFER,
    WATER_PARTICLE_INSTANCES_BUFFER, WEATHER_PRECIP_INSTANCES_BUFFER,
};
use crate::render::pipelines::domain_overlay_gpu::{EcologyOverlayGpuRow, LogisticsOverlayGpuRow};
use crate::render::fire_vfx::pack::{GpuParticleInstance, GpuParticleQuadVertex};
use crate::render::pipelines::gpu_instanced_quad::GpuInstancedQuadInstance;
use crate::render::pipelines::gpu_water_particles::{GpuWaterParticleInstance, GpuWaterParticleQuadVertex};
use crate::render::extraction::sim_visual_extract::FireVisualGpuInstance;

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
/// Weather precip streaks — same 32-byte stride as [`GpuInstancedQuadInstance`].
pub const WEATHER_PRECIP_INSTANCE_FORMAT: PackedFormatId = PackedFormatId(9);

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
pub const fn weather_precip_instance_format() -> PackedBufferFormat {
    PackedBufferFormat {
        format_id: WEATHER_PRECIP_INSTANCE_FORMAT,
        buffer_id: WEATHER_PRECIP_INSTANCES_BUFFER,
        stride: std::mem::size_of::<GpuInstancedQuadInstance>() as u32,
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

/// Required stride granularity for every packed GPU row (one `vec4<f32>` lane, std430 `vec4` align).
pub const GPU_ROW_ALIGN: u32 = 16;

/// `true` when `stride` is a legal packed-row stride: non-zero and a whole number of `vec4` lanes.
#[must_use]
pub const fn is_aligned_row_stride(stride: u32) -> bool {
    stride != 0 && stride % GPU_ROW_ALIGN == 0
}

/// Every packed format the buffer registry allocates (single table; stride/id authority).
pub const ALL_PACKED_FORMATS: [PackedBufferFormat; 9] = [
    fire_visual_instance_format(),
    heat_diffusion_cell_format(),
    fire_particle_instance_format(),
    logistics_overlay_row_format(),
    ecology_overlay_row_format(),
    fire_particle_expanded_vertex_format(),
    water_particle_instance_format(),
    water_particle_expanded_vertex_format(),
    weather_precip_instance_format(),
];

const fn all_formats_aligned() -> bool {
    let mut i = 0;
    while i < ALL_PACKED_FORMATS.len() {
        if !is_aligned_row_stride(ALL_PACKED_FORMATS[i].stride) {
            return false;
        }
        i += 1;
    }
    true
}

const _: () = assert!(
    all_formats_aligned(),
    "MEM-02: a packed GPU row stride is not a multiple of GPU_ROW_ALIGN (16 B)"
);

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

    use std::mem::{offset_of, size_of};

    /// MEM-02 tripwire: byte size of each Pod row (a change = WGSL/registry stride break).
    #[test]
    fn mem02_row_sizes_are_pinned() {
        assert_eq!(size_of::<FireVisualGpuInstance>(), 80);
        assert_eq!(size_of::<GpuParticleInstance>(), 32);
        assert_eq!(size_of::<GpuInstancedQuadInstance>(), 32);
        assert_eq!(size_of::<GpuParticleQuadVertex>(), 16);
        assert_eq!(size_of::<GpuWaterParticleInstance>(), 32);
        assert_eq!(size_of::<GpuWaterParticleQuadVertex>(), 32);
        assert_eq!(size_of::<LogisticsOverlayGpuRow>(), 16);
        assert_eq!(size_of::<EcologyOverlayGpuRow>(), 16);
        assert_eq!(size_of::<crate::compute::HeatDiffusionGpuCell>(), 16);
        assert_eq!(HEAT_DIFFUSION_CELL_STRIDE, 16);
    }

    /// MEM-02 tripwire: every field sits on a 16 B `vec4` lane boundary, in declaration order
    /// (`repr(C)` — no reordering, no implicit interior padding).
    #[test]
    fn mem02_field_offsets_are_vec4_lanes() {
        assert_eq!(offset_of!(FireVisualGpuInstance, chunk_xy_heat_lum), 0);
        assert_eq!(offset_of!(FireVisualGpuInstance, world_xyz_radius), 16);
        assert_eq!(offset_of!(FireVisualGpuInstance, smoke_ember_vis_priority), 32);
        assert_eq!(offset_of!(FireVisualGpuInstance, smoke_color_toxic), 48);
        assert_eq!(offset_of!(FireVisualGpuInstance, fog_rgb_combust_ord), 64);

        assert_eq!(offset_of!(GpuParticleInstance, world_xyz_heat), 0);
        assert_eq!(offset_of!(GpuParticleInstance, ember_class_radius_smoke), 16);

        assert_eq!(offset_of!(GpuInstancedQuadInstance, position_xyz_custom0), 0);
        assert_eq!(offset_of!(GpuInstancedQuadInstance, custom1_custom2_custom3_custom4), 16);

        assert_eq!(offset_of!(GpuParticleQuadVertex, world_xy_custom01), 0);

        assert_eq!(offset_of!(GpuWaterParticleInstance, world_xyz_profile), 0);
        assert_eq!(offset_of!(GpuWaterParticleInstance, flow_half_twinkle), 16);
        assert_eq!(offset_of!(GpuWaterParticleQuadVertex, world_xy_profile_phase), 0);
        assert_eq!(offset_of!(GpuWaterParticleQuadVertex, uv_stretch_twinkle), 16);

        assert_eq!(offset_of!(LogisticsOverlayGpuRow, corridor_xy_revision), 0);
        assert_eq!(offset_of!(EcologyOverlayGpuRow, ecology_xy_means), 0);
        assert_eq!(offset_of!(crate::compute::HeatDiffusionGpuCell, chunk_xy_heat_smoke), 0);
    }

    /// MEM-02: table is the stride authority — aligned, unique ids, byte size is whole lanes.
    #[test]
    fn mem02_format_table_is_aligned_and_unique() {
        for (i, f) in ALL_PACKED_FORMATS.iter().enumerate() {
            assert!(is_aligned_row_stride(f.stride), "format {:?} stride {}", f.format_id, f.stride);
            for g in &ALL_PACKED_FORMATS[i + 1..] {
                assert_ne!(f.format_id, g.format_id, "duplicate PackedFormatId");
            }
            for rows in [0usize, 1, 3, 1000] {
                let bytes = packed_byte_size(*f, rows);
                assert_eq!(bytes % u64::from(GPU_ROW_ALIGN), 0);
                assert!(bytes >= u64::from(f.stride), "zero-row floor is one row");
            }
        }
        assert!(!is_aligned_row_stride(0));
        assert!(!is_aligned_row_stride(12));
        assert!(!is_aligned_row_stride(24));
        assert!(is_aligned_row_stride(16));
    }

    #[test]
    fn fire_particle_stride_matches_pod_layout() {
        assert_eq!(
            fire_particle_instance_format().stride,
            std::mem::size_of::<GpuParticleInstance>() as u32
        );
    }

    #[test]
    fn weather_precip_stride_matches_instanced_quad() {
        assert_eq!(
            weather_precip_instance_format().stride,
            std::mem::size_of::<GpuInstancedQuadInstance>() as u32
        );
        assert_eq!(weather_precip_instance_format().buffer_id, WEATHER_PRECIP_INSTANCES_BUFFER);
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
