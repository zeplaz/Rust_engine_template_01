//! UX-E01 M1 — dedicated minimap GPU compositor RT (separate from world-preview authority).

mod composite;
mod diagnostics;
mod gpu_compute;
mod pass;
mod plugin;
mod render_target;
mod state;

#[cfg(test)]
mod tests;

pub use diagnostics::{
    diagnostics_json_snapshot, minimap_gpu_debug_logging_enabled, MinimapGpuCompositorDiagnostics,
    MinimapGpuDispatchReason, MinimapGpuSkipReason,
};



pub use pass::{
    apply_minimap_gpu_resize_request, bootstrap_minimap_gpu_render_target,
    commit_minimap_render_target_bind_system, queue_minimap_render_target_resize,
    run_minimap_compositor_pass, sync_minimap_presentation_source,
};
pub use state::{
    minimap_gpu_compositor_default_on_unset, minimap_gpu_compositor_env_enabled,
    minimap_gpu_compositor_runtime_enabled, minimap_terrain_source_bind,
    minimap_terrain_source_label, MinimapCompositePath,
    MinimapCompositorState, MinimapTerrainSourceBind,
};
pub use render_target::{
    committed_minimap_render_target_handle, minimap_rgba_image, try_commit_minimap_render_target,
    MinimapGpuResizeQueue, MinimapRenderTargetBindBarrier, MinimapRenderTargetRegistry,
};
pub use plugin::MinimapCompositorPlugin;
