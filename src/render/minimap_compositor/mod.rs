//! UX-E01 M1 — dedicated minimap GPU compositor RT (separate from world-preview authority).

mod witness_collectors;
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

pub use witness_collectors::{
    build_minimap_compositor_proof_payload, build_minimap_compositor_proof_payload_with_tray,
    fixture_ui_oh_m2_001_compositor, fixture_ui_w3_m3_001_compositor, witness_harness_tray,
    ui_oh_m2_001_green, ui_oh_m3_001_green, ui_w3_m3_001_green, ui_w3_m3_001_operational_green,
    ui_w3_m3_001_stage7_operational_green, ui_p3_m2_minimap_acceptance_green,
    ui_p3_m2_tray_opt_green, ui_p3_m3_minimap_acceptance_green, ui_p3_m3_replay_001_green,
    ui_p3_m3_units_001_green, ui_p3_m4_minimap_acceptance_green, ui_p3_001_minimap_acceptance_green,
    ui_w3_m2_001_green,
};
pub use crate::dev::runtime_witness::minimap::{
    commit_minimap_compositor_live_proof,
    refresh_perf_vis_p1b_gpu_default_live_witness, refresh_ui_oh_m2_001_live_witness,
    refresh_ui_w3_m2_001_live_witness, refresh_ui_w3_m3_001_live_witness,
    refresh_ui_w3_m3_001_stage7_operational_witness, write_minimap_compositor_live_proof_system,
    MinimapCompositorLiveProofState,
};
pub use pass::{
    apply_minimap_gpu_resize_request, bootstrap_minimap_gpu_render_target,
    commit_minimap_render_target_bind_system, queue_minimap_render_target_resize,
    run_minimap_compositor_pass, sync_minimap_presentation_source,
};
pub use state::{
    minimap_gpu_compositor_default_on_unset, minimap_gpu_compositor_env_enabled,
    minimap_gpu_compositor_runtime_enabled, minimap_terrain_source_bind,
    minimap_terrain_source_label, perf_vis_p1b_gpu_default_001_green, MinimapCompositePath,
    MinimapCompositorState, MinimapTerrainSourceBind,
};
pub use render_target::{
    committed_minimap_render_target_handle, minimap_rgba_image, try_commit_minimap_render_target,
    MinimapGpuResizeQueue, MinimapRenderTargetBindBarrier, MinimapRenderTargetRegistry,
};
pub use plugin::MinimapCompositorPlugin;
