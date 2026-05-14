//! Stage 5 spine governance matrix — CI hook for readiness + VT-4/VT-5 + Phase D/F fixtures.

#[cfg(test)]
mod tests {
    use crate::render::stage5_readiness::{stage5_readiness_passes, AppStage5ReadinessReport};
    use crate::render::vt_ci_matrix::{
        build_deterministic_ci_scenario, run_vt4_ci_matrix, run_vt5_ci_spatial_matrix, Vt4CiReport,
    };
    use crate::render::visual_agreement::VisualAgreementFrame;
    use crate::systems::atmosphere::P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE;

    #[test]
    fn stage5_spine_governance_matrix_passes() {
        let scenario = build_deterministic_ci_scenario();
        let mut agreement = VisualAgreementFrame::default();
        let mut vt4 = Vt4CiReport::default();
        run_vt4_ci_matrix(&scenario, &mut agreement, &mut vt4);
        assert!(vt4.passes());
        assert!(run_vt5_ci_spatial_matrix(&scenario));

        let report = AppStage5ReadinessReport {
            vt4_ok: true,
            vt5_ok: true,
            phase_d_ok: true,
            phase_f_ok: true,
            single_fire_extract: true,
            gpu_field_authoritative: P2H_GPU_PARTIAL_WRITES_AUTHORITATIVE,
            preview_render_target_active: true,
            overlay_from_shared_buffers_only: true,
            particle_lod_scales: true,
            phase_f_lod_proof_ok: true,
            projection_domains: 3,
            registered_producers: 1,
            duplicate_visual_scan_count: 0,
            violations: Vec::new(),
        };
        assert!(stage5_readiness_passes(&report));
    }
}
