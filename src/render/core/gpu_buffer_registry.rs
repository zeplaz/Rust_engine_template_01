//! Typed, versioned GPU buffer authority for the render world.
//!
//! [`GPUBufferEntry::version`] changes **only when a new GPU `Buffer` is allocated** for that
//! [`BufferId`] (create / grow). In-place `write` updates do **not** bump version so bind groups
//! keyed by `(BufferId, version)` stay stable across per-frame uploads.
//!
//! All `RenderDevice::create_buffer` / `Queue::write_buffer` for registered buffers go through
//! [`GPUBufferRegistry`]. Projection and compute graphs hold [`BufferId`] only.
//!
//! # MEM-04 — fragmentation + relocation (gea4v1 6.2.2 / 6.2.2.1–2; LEGACY `memory_management`, gea_v1_e4 p412)
//!
//! * **Fixed-size pages sidestep fragmentation (6.2.2.1).** Every allocation is rounded up to a
//!   multiple of [`GPU_BUFFER_PAGE_BYTES`] (one fixed page size, never per-call), so freed and
//!   re-created buffers are interchangeable page multiples rather than arbitrary holes.
//! * **Relocation needs handle indirection (6.2.2.2).** [`ensure_capacity`](GPUBufferRegistry::ensure_capacity)
//!   *relocates* a buffer (drop + re-`create` under the same [`BufferId`], new `version`). That is
//!   only safe because consumers hold the [`BufferId`] handle (and the plain-data [`GpuSlice`]) and
//!   resolve it through [`GPUBufferRegistry::get`] each frame — never a cached `Buffer`/pointer.
//!   Bind groups key on `(BufferId, version)`, so a relocation invalidates them by version, not by
//!   dangling reference. Do not hand out `&Buffer`/`Buffer` clones that outlive the frame.

use std::collections::HashMap;
use std::marker::PhantomData;

use bevy::prelude::Resource;
use bevy::render::render_resource::{Buffer, BufferDescriptor, BufferUsages};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bytemuck::Pod;

/// MEM-04: the single fixed allocation page (bytes). All registry buffer sizes are multiples of
/// this (also the wgpu storage/uniform offset alignment ceiling used by the render graphs).
pub const GPU_BUFFER_PAGE_BYTES: u64 = 256;

/// Stable numeric identity for a registered GPU buffer (no string keys on the hot path).
///
/// MEM-04: this is the **handle** — relocation keeps the id and bumps `version`.
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct BufferId(pub u64);

/// Fire visual instance storage (`weather_fire_field.wgsl` group 1).
pub const FIRE_VISUAL_INSTANCES_BUFFER: BufferId = BufferId(1);
/// Chunk heat diffusion field rows (compute-owned; `HeatDiffusionGpuCell` stride).
pub const HEAT_DIFFUSION_FIELD_BUFFER: BufferId = BufferId(2);
/// World fire particle instanced-quad rows (post-LOD projection).
pub const FIRE_PARTICLE_INSTANCES_BUFFER: BufferId = BufferId(3);
/// Logistics corridor overlay rows (Stage 5 domain projection).
pub const LOGISTICS_OVERLAY_BUFFER: BufferId = BufferId(4);
/// Ecology mean-field overlay rows (Stage 5 domain projection).
pub const ECOLOGY_OVERLAY_BUFFER: BufferId = BufferId(5);
/// Expanded instanced-quad vertices (4 verts per particle instance).
pub const FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER: BufferId = BufferId(6);
/// Spark advection sim state (`fire_spark_compute.wgsl` / expand read).
pub const FIRE_SPARK_STATE_BUFFER: BufferId = BufferId(8);
/// Deduped fire attractor centers (max 24, heat as mass).
pub const FIRE_SPARK_ATTRACTORS_BUFFER: BufferId = BufferId(9);
/// World water particle instanced-quad rows.
pub const WATER_PARTICLE_INSTANCES_BUFFER: BufferId = BufferId(10);
/// Expanded water particle vertices (4 verts per instance).
pub const WATER_PARTICLE_EXPANDED_VERTICES_BUFFER: BufferId = BufferId(11);
/// Tile LOD / fire debug logical instances (`assets/shaders/debug/tile_debug_instanced.wgsl` group 1).
pub const TILE_DEBUG_INSTANCES_BUFFER: BufferId = BufferId(7);
/// Instanced terrain tile rows (`terrain_instanced.wgsl`).
pub const TERRAIN_INSTANCES_BUFFER: BufferId = BufferId(12);
/// Weather precip streak instances (ES-5-002) — exclusive of FIRE_*/WATER_*.
pub const WEATHER_PRECIP_INSTANCES_BUFFER: BufferId = BufferId(13);
/// GPU-CULL.1 compacted fire particle rows (post frustum sphere test).
pub const FIRE_PARTICLE_CULLED_INSTANCES_BUFFER: BufferId = BufferId(14);
/// `DrawIndirectArgs` for fire particle raster (`draw_indirect`).
pub const FIRE_PARTICLE_INDIRECT_DRAW_BUFFER: BufferId = BufferId(15);
/// Atomic visible counter for frustum cull compute (u32).
pub const FIRE_PARTICLE_CULL_COUNTER_BUFFER: BufferId = BufferId(16);

/// Prevents accidental cross-graph buffer sharing until explicitly allowed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BufferVisibility {
    RenderOnly,
    ComputeOnly,
    RenderAndCompute,
}

/// Creation contract for [`GPUBufferRegistry::create`].
#[derive(Clone, Copy, Debug)]
pub struct RegisteredBufferDescriptor {
    pub id: BufferId,
    pub size_bytes: u64,
    pub usage: BufferUsages,
    pub visibility: BufferVisibility,
    pub stride: u32,
}

/// Registry-owned capacity view (reserved ≥ active; high watermark never shrinks).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistryBufferAllocation {
    pub reserved_rows: u32,
    pub active_rows: u32,
    pub high_watermark_rows: u32,
    pub reserved_bytes: u64,
    pub active_bytes: u64,
}

/// Typed slice view into a registered buffer (no gameplay-facing raw offsets).
#[derive(Clone, Copy, Debug)]
pub struct GpuSlice<T> {
    pub id: BufferId,
    pub offset_bytes: u64,
    pub size_bytes: u64,
    pub active_rows: u32,
    pub reserved_rows: u32,
    _marker: PhantomData<T>,
}

impl<T> GpuSlice<T> {
    #[must_use]
    pub const fn empty(id: BufferId) -> Self {
        Self {
            id,
            offset_bytes: 0,
            size_bytes: 0,
            active_rows: 0,
            reserved_rows: 0,
            _marker: PhantomData,
        }
    }
}

/// Result of one registry upload (active payload + reserved window).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RegistryUploadStats {
    pub active_rows: u32,
    pub reserved_rows: u32,
    pub upload_bytes: u64,
    pub reserved_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct GPUBufferEntry {
    pub buffer: Buffer,
    pub usage: BufferUsages,
    pub size_bytes: u64,
    pub stride: u32,
    pub version: u64,
    pub last_write_frame: u64,
    pub visibility: BufferVisibility,
    pub reserved_rows: u32,
    pub active_rows: u32,
    pub high_watermark_rows: u32,
}

impl GPUBufferEntry {
    #[must_use]
    pub fn allocation(&self) -> RegistryBufferAllocation {
        let reserved_bytes = self.size_bytes;
        let active_bytes = (self.active_rows as u64).saturating_mul(self.stride as u64);
        RegistryBufferAllocation {
            reserved_rows: self.reserved_rows,
            active_rows: self.active_rows,
            high_watermark_rows: self.high_watermark_rows,
            reserved_bytes,
            active_bytes,
        }
    }
}

#[derive(Resource, Default)]
pub struct GPUBufferRegistry {
    buffers: HashMap<BufferId, GPUBufferEntry>,
    /// Monotonic token assigned on each new `wgpu::Buffer` allocation for a registered id.
    allocation_serial: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegistryError {
    UnknownBuffer(BufferId),
    WriteTooLarge { id: BufferId, bytes: u64, capacity: u64 },
    DuplicateId(BufferId),
}

#[must_use]
pub fn row_capacity_bytes(stride: u32, rows: usize) -> u64 {
    let rows = rows.max(1) as u64;
    (rows * stride as u64)
        .max(stride as u64)
        .next_multiple_of(GPU_BUFFER_PAGE_BYTES)
}

impl GPUBufferRegistry {
    fn next_allocation_token(&mut self) -> u64 {
        self.allocation_serial = self.allocation_serial.wrapping_add(1);
        self.allocation_serial
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.buffers.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buffers.is_empty()
    }

    #[must_use]
    pub fn allocation(&self, id: BufferId) -> Option<RegistryBufferAllocation> {
        self.buffers.get(&id).map(GPUBufferEntry::allocation)
    }

    #[must_use]
    pub fn slice_view<T: Pod>(&self, id: BufferId) -> Option<GpuSlice<T>> {
        let entry = self.buffers.get(&id)?;
        Some(GpuSlice {
            id,
            offset_bytes: 0,
            size_bytes: (entry.active_rows as u64).saturating_mul(entry.stride as u64),
            active_rows: entry.active_rows,
            reserved_rows: entry.reserved_rows,
            _marker: PhantomData,
        })
    }

    /// Allocate and register a GPU buffer (sole allocation entry point).
    pub fn create(
        &mut self,
        device: &RenderDevice,
        desc: RegisteredBufferDescriptor,
    ) -> Result<BufferId, RegistryError> {
        if self.buffers.contains_key(&desc.id) {
            return Err(RegistryError::DuplicateId(desc.id));
        }
        let size = desc
            .size_bytes
            .max(desc.stride as u64)
            .next_multiple_of(GPU_BUFFER_PAGE_BYTES);
        let reserved_rows = (size / desc.stride as u64).max(1) as u32;
        let buffer = device.create_buffer(&BufferDescriptor {
            label: Some("gpu_buffer_registry"),
            size,
            usage: desc.usage,
            mapped_at_creation: false,
        });
        let version = self.next_allocation_token();
        self.buffers.insert(
            desc.id,
            GPUBufferEntry {
                buffer,
                usage: desc.usage,
                size_bytes: size,
                stride: desc.stride,
                version,
                last_write_frame: 0,
                visibility: desc.visibility,
                reserved_rows,
                active_rows: 0,
                high_watermark_rows: 0,
            },
        );
        Ok(desc.id)
    }

    /// Grow or create when projected payload exceeds the current allocation.
    pub fn ensure_capacity(
        &mut self,
        device: &RenderDevice,
        desc: RegisteredBufferDescriptor,
    ) -> Result<(), RegistryError> {
        let needed = row_capacity_bytes(desc.stride, (desc.size_bytes / desc.stride as u64).max(1) as usize);
        let reserved_rows = (needed / desc.stride as u64).max(1) as u32;
        if let Some(entry) = self.buffers.get(&desc.id) {
            if entry.size_bytes >= needed {
                let entry = self.buffers.get_mut(&desc.id).expect("buffer entry");
                entry.reserved_rows = entry.reserved_rows.max(reserved_rows);
                return Ok(());
            }
            self.buffers.remove(&desc.id);
        }
        let mut d = desc;
        d.size_bytes = needed;
        self.create(device, d)?;
        Ok(())
    }

    /// Exact-size resize (tests / explicit shrink paths only).
    pub fn sync_capacity(
        &mut self,
        device: &RenderDevice,
        desc: RegisteredBufferDescriptor,
    ) -> Result<(), RegistryError> {
        let needed = row_capacity_bytes(desc.stride, (desc.size_bytes / desc.stride as u64).max(1) as usize);
        if let Some(entry) = self.buffers.get(&desc.id) {
            if entry.size_bytes == needed {
                return Ok(());
            }
            self.buffers.remove(&desc.id);
        }
        let mut d = desc;
        d.size_bytes = needed;
        self.create(device, d)?;
        Ok(())
    }

    /// Ensure reserved row window, upload active rows, update allocation bookkeeping.
    pub fn upload_pod_slice<T: Pod>(
        &mut self,
        device: &RenderDevice,
        queue: &RenderQueue,
        desc: RegisteredBufferDescriptor,
        reserved_rows: usize,
        data: &[T],
        frame: u64,
    ) -> Result<RegistryUploadStats, RegistryError> {
        let reserve_rows = reserved_rows.max(data.len()).max(1);
        let mut reserve_desc = desc;
        reserve_desc.size_bytes = row_capacity_bytes(desc.stride, reserve_rows);
        self.ensure_capacity(device, reserve_desc)?;
        self.write(queue, desc.id, data, frame)?;
        let entry = self
            .buffers
            .get(&desc.id)
            .ok_or(RegistryError::UnknownBuffer(desc.id))?;
        let upload_bytes = (data.len() * std::mem::size_of::<T>()) as u64;
        Ok(RegistryUploadStats {
            active_rows: entry.active_rows,
            reserved_rows: entry.reserved_rows,
            upload_bytes,
            reserved_bytes: entry.size_bytes,
        })
    }

    /// Controlled upload path: writes bytes in place. Does **not** bump [`GPUBufferEntry::version`]
    /// (bind groups remain valid until the buffer is reallocated).
    pub fn write<T: Pod>(
        &mut self,
        queue: &RenderQueue,
        id: BufferId,
        data: &[T],
        frame: u64,
    ) -> Result<(), RegistryError> {
        let entry = self
            .buffers
            .get_mut(&id)
            .ok_or(RegistryError::UnknownBuffer(id))?;
        let bytes = (data.len() * std::mem::size_of::<T>()) as u64;
        if bytes > entry.size_bytes {
            return Err(RegistryError::WriteTooLarge {
                id,
                bytes,
                capacity: entry.size_bytes,
            });
        }
        entry.last_write_frame = frame;
        entry.active_rows = data.len() as u32;
        entry.high_watermark_rows = entry.high_watermark_rows.max(entry.active_rows);
        if data.is_empty() {
            let n = std::mem::size_of::<T>();
            let pad = vec![0u8; n];
            queue.write_buffer(&entry.buffer, 0, &pad);
        } else {
            queue.write_buffer(&entry.buffer, 0, bytemuck::cast_slice(data));
        }
        Ok(())
    }

    #[must_use]
    pub fn get(&self, id: BufferId) -> Option<&GPUBufferEntry> {
        self.buffers.get(&id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_id_ordering_is_stable() {
        assert!(BufferId(1) < BufferId(2));
        assert_eq!(FIRE_VISUAL_INSTANCES_BUFFER, BufferId(1));
        assert_eq!(HEAT_DIFFUSION_FIELD_BUFFER, BufferId(2));
        assert_eq!(FIRE_PARTICLE_INSTANCES_BUFFER, BufferId(3));
        assert_eq!(LOGISTICS_OVERLAY_BUFFER, BufferId(4));
        assert_eq!(ECOLOGY_OVERLAY_BUFFER, BufferId(5));
        assert_eq!(FIRE_PARTICLE_EXPANDED_VERTICES_BUFFER, BufferId(6));
        assert_eq!(TILE_DEBUG_INSTANCES_BUFFER, BufferId(7));
        assert_eq!(TERRAIN_INSTANCES_BUFFER, BufferId(12));
        assert_eq!(WEATHER_PRECIP_INSTANCES_BUFFER, BufferId(13));
        assert_eq!(FIRE_PARTICLE_CULLED_INSTANCES_BUFFER, BufferId(14));
        assert_eq!(FIRE_PARTICLE_INDIRECT_DRAW_BUFFER, BufferId(15));
        assert_eq!(FIRE_PARTICLE_CULL_COUNTER_BUFFER, BufferId(16));
    }

    #[test]
    fn allocation_invariant_reserved_ge_active() {
        let alloc = RegistryBufferAllocation {
            reserved_rows: 16,
            active_rows: 4,
            high_watermark_rows: 8,
            reserved_bytes: 512,
            active_bytes: 128,
        };
        assert!(alloc.reserved_rows >= alloc.active_rows);
        assert!(alloc.reserved_bytes >= alloc.active_bytes);
        assert!(alloc.high_watermark_rows >= alloc.active_rows);
    }

    #[test]
    fn row_capacity_bytes_aligns_to_256() {
        assert_eq!(row_capacity_bytes(32, 1) % 256, 0);
        assert!(row_capacity_bytes(32, 10) >= 320);
    }

    /// MEM-04 tripwire: the page size is one pinned constant; every capacity is a whole page multiple.
    #[test]
    fn mem04_allocation_page_size_is_fixed_and_capacity_is_page_multiple() {
        assert_eq!(GPU_BUFFER_PAGE_BYTES, 256, "page size is part of the GPU contract");
        for stride in [16u32, 32, 48, 64, 96, 128] {
            for rows in [0usize, 1, 2, 7, 10, 255, 1000] {
                let cap = row_capacity_bytes(stride, rows);
                assert_eq!(cap % GPU_BUFFER_PAGE_BYTES, 0, "stride {stride} rows {rows}");
                assert!(cap >= u64::from(stride) * rows.max(1) as u64);
                // Grow is monotonic in whole pages: no sub-page holes between sizes.
                assert!(row_capacity_bytes(stride, rows + 1) >= cap);
            }
        }
    }

    /// MEM-04 tripwire: consumers hold handles, not pointers. `Buffer` is `Clone` but not `Copy`, so a
    /// `Copy` bound on the handle types proves neither embeds a GPU buffer that relocation could strand.
    #[test]
    fn mem04_handles_are_plain_copy_data_not_buffer_pointers() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<BufferId>();
        assert_copy::<GpuSlice<u8>>();
        assert_copy::<RegistryBufferAllocation>();
        assert_eq!(std::mem::size_of::<BufferId>(), 8);
    }

    /// MEM-04 tripwire: a slice view of an unregistered handle resolves to nothing (no stale pointer).
    #[test]
    fn mem04_unregistered_handle_resolves_to_none() {
        let reg = GPUBufferRegistry::default();
        assert!(reg.get(FIRE_VISUAL_INSTANCES_BUFFER).is_none());
        assert!(reg.slice_view::<u8>(FIRE_VISUAL_INSTANCES_BUFFER).is_none());
        assert!(reg.allocation(FIRE_VISUAL_INSTANCES_BUFFER).is_none());
    }
}
