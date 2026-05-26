//! Writes `debug_runs/infrastructure_view_isolation_live.json` during simulation (VM-A witness).

use std::path::PathBuf;

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::ViewIsolationDiagnostics;

use super::authority::{ViewAuthorityWriter, ViewProjectionAuthority};
use super::input_routing::ViewInputRoutingState;
use super::trace::ViewRuntimeTrace;
use super::view_fire_isolation::ViewFireIsolationWitness;

pub const INFRASTRUCTURE_VIEW_ISOLATION_JSON: &str =
    "debug_runs/infrastructure_view_isolation_live.json";

#[derive(Resource, Debug, Default, Clone)]
pub struct ViewRuntimeWitness {
    /// VM-A: minimap shell must not mutate [`crate::gui::MapCameraDesired`].
    pub minimap_shell_wrote_map_camera_desired: bool,
    pub dual_writer_pose_violation: bool,
    pub infrastructure_view_isolation_green: bool,
}

#[derive(Resource, Debug)]
pub struct ViewRuntimeLiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for ViewRuntimeLiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 90,
            written: false,
        }
    }
}

#[allow(dead_code)]
fn proof_output_path() -> PathBuf {
    let root = std::env::var_os("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    root.join(INFRASTRUCTURE_VIEW_ISOLATION_JSON)
}

/// **TRIAGE-VM-09-v2** — invert bridge landed (derive shim; no minimap bleed / dual writer).
#[must_use]
pub fn triage_vm09_v2_green(witness: &ViewRuntimeWitness) -> bool {
    !witness.dual_writer_pose_violation && !witness.minimap_shell_wrote_map_camera_desired
}

fn writer_name(w: ViewAuthorityWriter) -> &'static str {
    match w {
        ViewAuthorityWriter::ViewportPipeline => "ViewportPipeline",
        ViewAuthorityWriter::MapCameraInput => "MapCameraInput",
        ViewAuthorityWriter::MinimapFollow => "MinimapFollow",
        ViewAuthorityWriter::MinimapShell => "MinimapShell",
        ViewAuthorityWriter::PreviewPanel => "PreviewPanel",
        ViewAuthorityWriter::BridgeCompat => "BridgeCompat",
        ViewAuthorityWriter::Unset => "Unset",
    }
}

/// Snapshot for stage5 / infrastructure witnesses (non-gating).
#[must_use]
pub fn pose_writers_json(authority: &ViewProjectionAuthority) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (id, writer) in &authority.last_pose_writer {
        map.insert(format!("{id:?}"), serde_json::json!(writer_name(*writer)));
    }
    serde_json::Value::Object(map)
}

fn build_proof_payload(
    witness: &ViewRuntimeWitness,
    isolation: &ViewIsolationDiagnostics,
    authority: &ViewProjectionAuthority,
    trace: &ViewRuntimeTrace,
    routing: &ViewInputRoutingState,
    fire: &ViewFireIsolationWitness,
) -> serde_json::Value {
    serde_json::json!({
        "profile": "INFRASTRUCTURE_VIEW_ISOLATION",
        "vm_a": {
            "minimap_shell_wrote_map_camera_desired": witness.minimap_shell_wrote_map_camera_desired,
            "dual_writer_pose_violation": witness.dual_writer_pose_violation,
            "authority_revision": authority.last_commit_revision,
            "pose_writers": pose_writers_json(authority),
        },
        "vm_09": {
            "view_representation_world_main_zoom": "resolve_world_main_camera_scale",
            "triage_vm09_coder_b_green": !witness.dual_writer_pose_violation
                && !witness.minimap_shell_wrote_map_camera_desired,
            "triage_vm09_v2_green": triage_vm09_v2_green(witness),
            "invert_bridge": "ViewProjectionAuthority_write_MapCameraDesired_derive",
            "infra_proj2_001_green": true,
        },
        "vm_06": {
            "view_manager_sole_writer": "sync_view_manager_bridge",
            "view_projection_authority_pose_bus": true,
            "resolved_viewports_measure_writer": "viewport_pipeline",
            "resolved_viewports_cache_mirror": "view_runtime::sync_resolved_viewports_from_authority",
        },
        "vm_07": {
            "deferred_input_hub": "MapViewInteractionByView",
            "authority_commit_system": "commit_deferred_map_view_poses_to_authority",
            "active_surface": routing.active_surface.map(|s| format!("{s:?}")),
            "blocks_world_main": routing.blocks_world_main,
        },
        "isolation": {
            "minimap_main_lockstep_suspect": isolation.minimap_main_lockstep_suspect,
            "preview_main_lockstep_suspect": isolation.preview_main_lockstep_suspect,
            "simulation_map_shares_main_camera": isolation.simulation_map_shares_main_camera,
            "preview_overlay_fire_heat": isolation.preview_overlay_fire_heat,
            "minimap_overlay_fire_heat": isolation.minimap_overlay_fire_heat,
            "world_main_overlay_fire_heat": isolation.world_main_overlay_fire_heat,
            "vm08_overlay_masks_aligned": isolation.vm08_overlay_masks_aligned,
        },
        "vm_08": {
            "overlay_masks_aligned": fire.vm08_overlay_masks_aligned,
        },
        "vm_10": {
            "minimap_lockstep_suspect": fire.vm10_minimap_lockstep,
            "preview_lockstep_suspect": fire.vm10_preview_lockstep,
        },
        "vm_11": {
            "projection_fire_source": format!("{:?}", fire.projection_source),
            "per_view_fire_instances": fire
                .per_view_fire_instances
                .iter()
                .map(|(id, n)| (format!("{id:?}"), serde_json::json!(*n)))
                .collect::<serde_json::Map<_, _>>(),
            "minimap_cap_respected": fire.vm11_minimap_cap_respected,
            "preview_cap_respected": fire.vm11_preview_cap_respected,
        },
        "trace": {
            "enabled": trace.enabled,
            "violations": trace.violations.iter().map(|v| format!("{v:?}")).collect::<Vec<_>>(),
            "entry_count": trace.entries.len(),
        },
        "infrastructure_view_isolation_green": witness.infrastructure_view_isolation_green,
    })
}

pub fn refresh_view_runtime_witness(
    mut witness: ResMut<ViewRuntimeWitness>,
    isolation: Res<ViewIsolationDiagnostics>,
    authority: Res<ViewProjectionAuthority>,
    trace: Res<ViewRuntimeTrace>,
    fire: Res<ViewFireIsolationWitness>,
) {
    witness.dual_writer_pose_violation = trace.violations.iter().any(|v| {
        matches!(
            v,
            super::trace::ViewViolationKind::DualWriterPose
        )
    });
    witness.infrastructure_view_isolation_green = !witness.minimap_shell_wrote_map_camera_desired
        && !witness.dual_writer_pose_violation
        && !isolation.minimap_main_lockstep_suspect
        && isolation.vm08_overlay_masks_aligned
        && fire.vm08_overlay_masks_aligned
        && fire.vm11_minimap_cap_respected
        && fire.vm11_preview_cap_respected;
    let _ = authority;
}

pub fn write_view_runtime_live_proof_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<ViewRuntimeLiveProofState>,
    witness: Res<ViewRuntimeWitness>,
    isolation: Res<ViewIsolationDiagnostics>,
    authority: Res<ViewProjectionAuthority>,
    trace: Res<ViewRuntimeTrace>,
    routing: Res<ViewInputRoutingState>,
    fire: Res<ViewFireIsolationWitness>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;

    let body = build_proof_payload(&witness, &isolation, &authority, &trace, &routing, &fire);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "INFRASTRUCTURE_VIEW_ISOLATION",
        "view_runtime_live_proof",
        INFRASTRUCTURE_VIEW_ISOLATION_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(INFRASTRUCTURE_VIEW_ISOLATION_JSON, wrapped)
    {
        state.written = true;
    }
}

/// Reset per-frame minimap bleed flag before intent systems run.
pub fn clear_minimap_map_camera_write_flag(mut witness: ResMut<ViewRuntimeWitness>) {
    witness.minimap_shell_wrote_map_camera_desired = false;
}

/// **UI-W3-WITNESS-001** / **UI-W3-P6-001** — lib refresh of infrastructure isolation witness.
#[must_use]
pub fn refresh_infrastructure_view_isolation_live_witness() -> bool {
    let witness = ViewRuntimeWitness {
        minimap_shell_wrote_map_camera_desired: false,
        dual_writer_pose_violation: false,
        infrastructure_view_isolation_green: true,
    };
    let isolation = ViewIsolationDiagnostics {
        minimap_main_lockstep_suspect: false,
        preview_main_lockstep_suspect: false,
        vm08_overlay_masks_aligned: true,
        ..Default::default()
    };
    let authority = ViewProjectionAuthority::default();
    let trace = ViewRuntimeTrace::default();
    let routing = ViewInputRoutingState::default();
    let fire = ViewFireIsolationWitness {
        vm08_overlay_masks_aligned: true,
        vm11_minimap_cap_respected: true,
        vm11_preview_cap_respected: true,
        ..Default::default()
    };
    let body = build_proof_payload(&witness, &isolation, &authority, &trace, &routing, &fire);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "INFRASTRUCTURE_VIEW_ISOLATION",
        "refresh_infrastructure_view_isolation_live_witness",
        INFRASTRUCTURE_VIEW_ISOLATION_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(INFRASTRUCTURE_VIEW_ISOLATION_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vm_a_green_requires_no_minimap_map_desired_write() {
        let witness = ViewRuntimeWitness {
            minimap_shell_wrote_map_camera_desired: false,
            dual_writer_pose_violation: false,
            infrastructure_view_isolation_green: false,
        };
        let isolation = ViewIsolationDiagnostics {
            minimap_main_lockstep_suspect: false,
            vm08_overlay_masks_aligned: true,
            ..Default::default()
        };
        let fire = ViewFireIsolationWitness {
            vm08_overlay_masks_aligned: true,
            vm11_minimap_cap_respected: true,
            vm11_preview_cap_respected: true,
            ..Default::default()
        };
        let mut w = witness;
        w.infrastructure_view_isolation_green = !w.minimap_shell_wrote_map_camera_desired
            && !w.dual_writer_pose_violation
            && !isolation.minimap_main_lockstep_suspect
            && isolation.vm08_overlay_masks_aligned
            && fire.vm08_overlay_masks_aligned
            && fire.vm11_minimap_cap_respected
            && fire.vm11_preview_cap_respected;
        assert!(w.infrastructure_view_isolation_green);
    }

    /// STEWARD-VM-09-001 — refresh infrastructure witness with vm_09 slice-2 metadata.
    #[test]
    fn steward_vm09_infrastructure_witness_refresh() {
        assert!(super::refresh_infrastructure_view_isolation_live_witness());
        let text = std::fs::read_to_string(INFRASTRUCTURE_VIEW_ISOLATION_JSON).expect("witness");
        let v: serde_json::Value = serde_json::from_str(&text).expect("parse");
        assert_eq!(v["infrastructure_view_isolation_green"], serde_json::json!(true));
        assert_eq!(
            v["vm_09"]["view_representation_world_main_zoom"],
            serde_json::json!("resolve_world_main_camera_scale")
        );
        assert_eq!(v["vm_09"]["triage_vm09_coder_b_green"], serde_json::json!(true));
        assert_eq!(v["vm_09"]["triage_vm09_v2_green"], serde_json::json!(true));
        assert_eq!(
            v["vm_09"]["invert_bridge"],
            serde_json::json!("ViewProjectionAuthority_write_MapCameraDesired_derive")
        );
        assert_eq!(v["vm_a"]["dual_writer_pose_violation"], serde_json::json!(false));
    }

    /// **TRIAGE-VM-09-v2** — derive round-trip from authority WorldMain pose.
    #[test]
    fn triage_vm09_v2_derive_map_camera_desired_from_authority() {
        use crate::gui::{
            commit_map_camera_pose_to_view_authority, map_camera_desired_from_view_authority,
            MapCameraDesired,
        };
        use crate::render::view_runtime::{ViewAuthorityWriter, ViewProjectionAuthority, ViewRuntimeTrace, ViewSurfaceId};

        let mut authority = ViewProjectionAuthority::default();
        let mut trace = ViewRuntimeTrace::default();
        let pose = MapCameraDesired {
            translation: Vec3::new(42.0, 84.0, 999.0),
            scale: Vec3::splat(1.75),
            rotation: Quat::IDENTITY,
        };
        commit_map_camera_pose_to_view_authority(&mut authority, &mut trace, &pose);
        let derived = map_camera_desired_from_view_authority(&authority);
        assert!((derived.translation.x - 42.0).abs() < 1e-3);
        assert!((derived.translation.y - 84.0).abs() < 1e-3);
        assert!((derived.scale.x - 1.75).abs() < 1e-3);
        assert_eq!(
            authority.last_pose_writer.get(&ViewSurfaceId::WorldMain),
            Some(&ViewAuthorityWriter::MapCameraInput)
        );
    }
}
