//! Upload [`crate::gui::TileDebugInstance`] rows to [`crate::render::gpu_buffer_registry::GPUBufferRegistry`]
//! for `assets/shaders/debug/tile_debug_instanced.wgsl` (storage @group(1) @binding(0)).
//! Instances are drawn in the Core2d graph via [`crate::render::gpu_tile_debug_draw`].

use bevy::prelude::*;
use bevy::render::{
    render_resource::BufferUsages,
    renderer::{RenderDevice, RenderQueue},
    Render, RenderApp, RenderSystems,
};

use crate::gui::{TileDebugInstance, TileDebugInstanceMap, TileDebugViewId};
use crate::render::gpu_buffer_registry::{
    BufferVisibility, GPUBufferRegistry, RegisteredBufferDescriptor, TILE_DEBUG_INSTANCES_BUFFER,
};

pub fn register_tile_debug_instance_storage_upload(app: &mut App) {
    let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
        return;
    };
    render_app.add_systems(
        Render,
        prepare_tile_debug_instance_storage.in_set(RenderSystems::PrepareBindGroups),
    );
}

pub(crate) fn prepare_tile_debug_instance_storage(
    mut local_frame: Local<u64>,
    map: Res<TileDebugInstanceMap>,
    mut registry: ResMut<GPUBufferRegistry>,
    render_device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
) {
    *local_frame = local_frame.wrapping_add(1);
    let rows: &[TileDebugInstance] = map
        .per_view
        .get(&TileDebugViewId::WorldMain)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    let stride = std::mem::size_of::<TileDebugInstance>() as u32;
    let reserve_rows = rows.len().max(1);
    let _ = registry.upload_pod_slice(
        &render_device,
        &queue,
        RegisteredBufferDescriptor {
            id: TILE_DEBUG_INSTANCES_BUFFER,
            size_bytes: 0,
            usage: BufferUsages::COPY_DST | BufferUsages::STORAGE,
            visibility: BufferVisibility::RenderAndCompute,
            stride,
        },
        reserve_rows,
        rows,
        *local_frame,
    );
}
