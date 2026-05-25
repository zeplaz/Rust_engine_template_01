//! Writes `debug_runs/stage6_virtualization_live.json` during simulation (S6-0 witness).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::editor::world_preview::PreviewPathAuthority;
use crate::io::streaming::{gather_wave_c_readiness, wave_c_readiness_passes};

use super::stage6_virtualization::{
    gather_stage6_readiness, stage6_readiness_passes, Stage6ReadinessReport,
    Stage6VirtualizationFrame,
};

pub const STAGE6_VIRTUALIZATION_JSON: &str = "debug_runs/stage6_virtualization_live.json";

#[derive(Resource, Debug, Default, Clone)]
pub struct Stage6VirtualizationWitness {
    pub stage6_virtualization_green: bool,
    pub violations: Vec<&'static str>,
}

#[derive(Resource, Debug)]
pub struct Stage6LiveProofState {
    pub frames_since_write: u32,
    pub write_interval: u32,
    pub written: bool,
}

impl Default for Stage6LiveProofState {
    fn default() -> Self {
        Self {
            frames_since_write: 0,
            write_interval: 90,
            written: false,
        }
    }
}

#[must_use]
pub fn stage6_readiness_violations(report: &Stage6ReadinessReport) -> Vec<&'static str> {
    let mut out = Vec::new();
    if !report.wave_c_ok {
        out.push("wave_c_not_ready");
    }
    if !report.residency_populated {
        out.push("residency_empty");
    }
    if !report.projection_window_populated {
        out.push("consumer_window_empty");
    }
    if !report.atlas_slots_active {
        out.push("gpu_upload_inactive");
    }
    out
}

#[must_use]
pub fn build_stage6_proof_payload(
    witness: &Stage6VirtualizationWitness,
    report: &Stage6ReadinessReport,
    frame: &Stage6VirtualizationFrame,
    wave_c: &crate::io::streaming::WaveCReadinessReport,
    vm_a: Option<serde_json::Value>,
) -> serde_json::Value {
    let passes = stage6_readiness_passes(report) && witness.violations.is_empty();
    serde_json::json!({
        "profile": "STAGE6_VIRTUALIZATION",
        "stage6_readiness": {
            "passes": passes,
            "report": {
                "wave_c_ok": report.wave_c_ok,
                "residency_populated": report.residency_populated,
                "projection_window_populated": report.projection_window_populated,
                "atlas_slots_active": report.atlas_slots_active,
            },
            "violations": witness.violations,
        },
        "frame": {
            "focus_chunk": [frame.focus_chunk.x, frame.focus_chunk.y],
            "residency_chunk_count": frame.residency_chunk_count,
            "core_chunk_count": frame.core_chunk_count,
            "ghost_chunk_count": frame.ghost_chunk_count,
            "consumer_window_len": frame.consumer_window_coords.len(),
            "active_atlas_slots": frame.active_atlas_slots,
            "gpu_upload_bytes_frame": frame.gpu_upload_bytes_frame,
            "per_view_window_count": frame.per_view_window_count,
        },
        "wave_c": {
            "prerequisites_ok": wave_c.prerequisites_ok,
            "open_backlog_items": wave_c.open_backlog_items,
            "passes": wave_c_readiness_passes(wave_c),
        },
        "stage6_virtualization_green": witness.stage6_virtualization_green,
        "vm_a_crosslink": vm_a,
    })
}

pub fn refresh_stage6_virtualization_witness(
    preview: Res<PreviewPathAuthority>,
    frame: Res<Stage6VirtualizationFrame>,
    mut witness: ResMut<Stage6VirtualizationWitness>,
) {
    let report = gather_stage6_readiness(preview.as_ref(), frame.as_ref());
    witness.violations = stage6_readiness_violations(&report);
    witness.stage6_virtualization_green =
        stage6_readiness_passes(&report) && witness.violations.is_empty();
}

pub fn write_stage6_virtualization_live_proof_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<Stage6LiveProofState>,
    witness: Res<Stage6VirtualizationWitness>,
    preview: Res<PreviewPathAuthority>,
    frame: Res<Stage6VirtualizationFrame>,
    view_witness: Option<Res<crate::render::view_runtime::ViewRuntimeWitness>>,
    view_authority: Option<Res<crate::render::view_runtime::ViewProjectionAuthority>>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    state.frames_since_write = state.frames_since_write.saturating_add(1);
    if state.frames_since_write < state.write_interval {
        return;
    }
    state.frames_since_write = 0;

    let report = gather_stage6_readiness(preview.as_ref(), frame.as_ref());
    let wave_c = gather_wave_c_readiness(preview.as_ref());
    let vm_a = view_witness.zip(view_authority).map(|(w, a)| {
        serde_json::json!({
            "dual_writer_pose_violation": w.dual_writer_pose_violation,
            "infrastructure_view_isolation_green": w.infrastructure_view_isolation_green,
            "authority_revision": a.last_commit_revision,
        })
    });
    let body = build_stage6_proof_payload(&witness, &report, frame.as_ref(), &wave_c, vm_a);
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "STAGE6_VIRTUALIZATION",
        "stage6_live_proof",
        STAGE6_VIRTUALIZATION_JSON,
        body,
    );
    if crate::dev::debug_run_envelope::write_debug_run_json(STAGE6_VIRTUALIZATION_JSON, wrapped) {
        state.written = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::editor::world_preview::PreviewPathAuthority;

    #[test]
    fn stage6_live_proof_green_when_readiness_passes() {
        let report = Stage6ReadinessReport {
            wave_c_ok: true,
            residency_populated: true,
            projection_window_populated: true,
            atlas_slots_active: true,
        };
        let witness = Stage6VirtualizationWitness {
            stage6_virtualization_green: true,
            violations: vec![],
        };
        let frame = Stage6VirtualizationFrame {
            residency_chunk_count: 4,
            consumer_window_coords: vec![IVec2::ZERO],
            active_atlas_slots: 1,
            ..Default::default()
        };
        let wave_c = gather_wave_c_readiness(&PreviewPathAuthority::default());
        let body = build_stage6_proof_payload(&witness, &report, &frame, &wave_c, None);
        assert_eq!(
            body.pointer("/stage6_readiness/passes")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("profile").and_then(|v| v.as_str()),
            Some("STAGE6_VIRTUALIZATION")
        );
    }

    #[test]
    fn stage6_readiness_violations_lists_missing_fields() {
        let report = Stage6ReadinessReport::default();
        let v = stage6_readiness_violations(&report);
        assert!(v.contains(&"wave_c_not_ready") || v.contains(&"residency_empty"));
        assert!(!v.is_empty());
    }
}
