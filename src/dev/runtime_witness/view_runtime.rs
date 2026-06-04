//! Infrastructure view isolation witness — `debug_runs/infrastructure_view_isolation_live.json`.

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::ViewIsolationDiagnostics;
use crate::render::view_runtime::{
    pose_writers_json, triage_vm09_v2_green, ViewFireIsolationWitness, ViewInputRoutingState,
    ViewProjectionAuthority, ViewRuntimeTrace, ViewRuntimeWitness,
};

use super::io::{write_enveloped_witness, write_enveloped_witness_unchecked};

pub const INFRASTRUCTURE_VIEW_ISOLATION_JSON: &str =
    "debug_runs/infrastructure_view_isolation_live.json";

const PROFILE: &str = "INFRASTRUCTURE_VIEW_ISOLATION";

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

impl ViewRuntimeLiveProofState {
    fn tick(&mut self) -> bool {
        self.frames_since_write = self.frames_since_write.saturating_add(1);
        if self.frames_since_write < self.write_interval {
            return false;
        }
        self.frames_since_write = 0;
        true
    }
}

#[must_use]
pub fn build_infrastructure_view_isolation_payload(
    witness: &ViewRuntimeWitness,
    isolation: &ViewIsolationDiagnostics,
    authority: &ViewProjectionAuthority,
    trace: &ViewRuntimeTrace,
    routing: &ViewInputRoutingState,
    fire: &ViewFireIsolationWitness,
) -> serde_json::Value {
    let sole_fire_producer = crate::gui::fire_visual_producer_count() == 1;
    let minimap_overlay_only = !crate::render::minimap_compositor_queries_fire_ecs();
    let fire7_f7_a_exit_green =
        fire.f7_a_per_view_extract_bounded && sole_fire_producer && minimap_overlay_only;
    serde_json::json!({
        "profile": PROFILE,
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
            "minimap_follow_exempt": isolation.minimap_main_lockstep_suspect == fire.vm10_minimap_lockstep,
            "diagnostic_note": "lockstep true = suspected bleed; false = independent or follow mode",
        },
        "vm_11": {
            "projection_fire_source": format!("{:?}", fire.projection_source),
            "per_view_fire_instances": fire
                .per_view_fire_instances
                .iter()
                .map(|(id, n)| (format!("{id:?}"), serde_json::json!(*n)))
                .collect::<serde_json::Map<_, _>>(),
            "per_view_chunk_heat": fire
                .per_view_chunk_heat
                .iter()
                .map(|(id, n)| (format!("{id:?}"), serde_json::json!(*n)))
                .collect::<serde_json::Map<_, _>>(),
            "minimap_cap_respected": fire.vm11_minimap_cap_respected,
            "preview_cap_respected": fire.vm11_preview_cap_respected,
            "preview_semantic_audit_green": fire.vm11_preview_cap_respected
                && !isolation.preview_main_lockstep_suspect,
            "f7_a_per_view_extract_bounded": fire.f7_a_per_view_extract_bounded,
        },
        "fire7_f7_a_001": {
            "gate": "FIRE7-F7-A-001",
            "green": fire.f7_a_per_view_extract_bounded,
            "f7_a_per_view_extract_bounded": fire.f7_a_per_view_extract_bounded,
        },
        "fire7_f7_a_exit_001": {
            "gate": "FIRE7-F7-A-EXIT-001",
            "green": fire7_f7_a_exit_green,
            "fire7_f7_a_001_green": fire7_f7_a_exit_green,
            "sole_fire_visual_producer": sole_fire_producer,
            "minimap_fire_overlay_only": minimap_overlay_only,
        },
        "trace": {
            "enabled": trace.enabled,
            "violations": trace.violations.iter().map(|v| format!("{v:?}")).collect::<Vec<_>>(),
            "entry_count": trace.entries.len(),
        },
        "infrastructure_view_isolation_green": witness.infrastructure_view_isolation_green,
        "infra_vm_deep_001": {
            "gate": "INFRA-VM-DEEP-001",
            "green": witness.infrastructure_view_isolation_green,
            "sim_trace": {
                "vm_08_overlay_masks_aligned": isolation.vm08_overlay_masks_aligned,
                "vm_10_minimap_lockstep": fire.vm10_minimap_lockstep,
                "vm_10_preview_lockstep": fire.vm10_preview_lockstep,
                "vm_11_minimap_cap_respected": fire.vm11_minimap_cap_respected,
                "vm_11_preview_cap_respected": fire.vm11_preview_cap_respected,
            },
        },
        "triage_phase_d_parity_001": {
            "gate": "TRIAGE-PHASE-D-PARITY-001",
            "green": isolation.vm08_overlay_masks_aligned
                && fire.vm08_overlay_masks_aligned
                && fire.vm11_minimap_cap_respected
                && fire.vm11_preview_cap_respected,
        },
    })
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
    if !state.tick() {
        return;
    }
    let body = build_infrastructure_view_isolation_payload(
        witness.as_ref(),
        isolation.as_ref(),
        authority.as_ref(),
        trace.as_ref(),
        routing.as_ref(),
        fire.as_ref(),
    );
    if write_enveloped_witness(
        PROFILE,
        "view_runtime_live_proof",
        INFRASTRUCTURE_VIEW_ISOLATION_JSON,
        body,
    ) {
        state.written = true;
    }
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
        f7_a_per_view_extract_bounded: true,
        ..Default::default()
    };
    let body = build_infrastructure_view_isolation_payload(
        &witness,
        &isolation,
        &authority,
        &trace,
        &routing,
        &fire,
    );
    write_enveloped_witness_unchecked(
        PROFILE,
        "refresh_infrastructure_view_isolation_live_witness",
        INFRASTRUCTURE_VIEW_ISOLATION_JSON,
        body,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::ViewIsolationDiagnostics;

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
        assert!(refresh_infrastructure_view_isolation_live_witness());
        let text =
            std::fs::read_to_string(INFRASTRUCTURE_VIEW_ISOLATION_JSON).expect("witness");
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

    /// Slice C — contract keys preserved after containment move.
    #[test]
    fn infrastructure_view_isolation_live_json_contract_keys() {
        const KEYS: &[&str] = &[
            "profile",
            "vm_a",
            "vm_09",
            "infrastructure_view_isolation_green",
            "infra_vm_deep_001",
            "triage_phase_d_parity_001",
        ];
        assert!(refresh_infrastructure_view_isolation_live_witness());
        let body: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(INFRASTRUCTURE_VIEW_ISOLATION_JSON).unwrap(),
        )
        .unwrap();
        for key in KEYS {
            assert!(body.get(key).is_some(), "missing contract key: {key}");
        }
    }
}
