//! View-authority sampling — **RGR-V2-005** extraction from `stage5_full_app_harness.rs`.
//!
//! Pure JSON builder for the `viewport_contracts.view_isolation` block of
//! `stage5_full_app_live.json`. Mechanical carve-out — logic verbatim, callable from the
//! harness proof-commit path (thin render/sim probe → this passive builder, no I/O here).
//!
//! Do NOT add disk I/O to this module — it stays a pure sampler; envelope writes remain in
//! [`crate::dev::runtime_witness`] / the harness proof-commit call site.

use crate::gui::{ViewId, ViewIsolationDiagnostics, ViewManager};
use crate::render::view_runtime::{pose_writers_json, ViewProjectionAuthority, ViewRuntimeWitness, ViewSurfaceId};
use crate::render::Stage5FireViewChunkWitness;

/// Build the `view_isolation` JSON sub-block for the stage5 FULL_APP live proof payload.
#[must_use]
pub fn view_authority_sample_json(
    view_isolation: &ViewIsolationDiagnostics,
    view_manager: Option<&ViewManager>,
    fire_witness: Option<&Stage5FireViewChunkWitness>,
    view_projection_authority: Option<&ViewProjectionAuthority>,
    view_runtime_witness: Option<&ViewRuntimeWitness>,
) -> serde_json::Value {
    serde_json::json!({
        "note": "infrastructure_lane_non_gating",
        "minimap_main_lockstep_suspect": view_isolation.minimap_main_lockstep_suspect,
        "preview_main_lockstep_suspect": view_isolation.preview_main_lockstep_suspect,
        "simulation_map_shares_main_camera": view_isolation.simulation_map_shares_main_camera,
        "preview_overlay_fire_heat": view_isolation.preview_overlay_fire_heat,
        "minimap_overlay_fire_heat": view_isolation.minimap_overlay_fire_heat,
        "world_main_viewport": view_manager.and_then(|m| m.view(ViewId::WorldMain)).map(|v| {
            serde_json::json!({
                "width": v.viewport_rect.width(),
                "height": v.viewport_rect.height(),
            })
        }),
        "world_main_visible_fire_orphans": fire_witness.map(|w| w.world_main_visible_orphan_chunks),
        "view_runtime": view_projection_authority.map(|auth| {
            serde_json::json!({
                "authority_revision": auth.last_commit_revision,
                "pose_writers": pose_writers_json(auth),
                "world_preview_logical": auth.surface(ViewSurfaceId::WorldPreview).map(|s| {
                    serde_json::json!({ "x": s.render.logical_size.x, "y": s.render.logical_size.y, "valid": s.render.valid })
                }),
            })
        }),
        "vm_a_witness": view_runtime_witness.map(|w| {
            serde_json::json!({
                "minimap_shell_wrote_map_camera_desired": w.minimap_shell_wrote_map_camera_desired,
                "dual_writer_pose_violation": w.dual_writer_pose_violation,
                "infrastructure_view_isolation_green": w.infrastructure_view_isolation_green,
            })
        }),
    })
}
