//! App-level VT integration harness (VT-4 / VT-5 smoke).

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::MinimalPlugins;

    use crate::gui::OverlayFieldFrame;
    use crate::render::extraction::RenderProjectionGraph;
    use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualFrame};
    use crate::render::visual_agreement::{
        hash_shared_overlay_heat, update_visual_agreement_frame, VisualAgreementFrame,
    };
    use crate::render::OverlayAgreementDebug;
    use crate::render::SharedOverlayFieldBuffers;
    use crate::render::WorldPreviewVt4Probe;
    use crate::systems::sim_control::SimStepStamp;

    #[test]
    fn vt4_app_surfaces_agree_on_committed_stamp() {
        let stamp = SimStepStamp::new(9, 3);
        let rows = vec![ChunkFireHeat {
            chunk: IVec2::new(2, 2),
            heat: 0.5,
            smoke: 0.0,
        }];
        let frame = FireVisualFrame {
            stamp,
            instances: Vec::new(),
            chunk_heat: rows.clone(),
        };
        let mut shared = SharedOverlayFieldBuffers::default();
        shared.stamp = stamp;
        shared.chunk_fire_heat.insert(IVec2::new(2, 2), 0.5);
        let overlay = OverlayFieldFrame {
            stamp,
            fields: std::collections::HashMap::new(),
            fire_heat_overlay_revision: 2,
        };
        let mut graph = RenderProjectionGraph::default();
        graph.fire.chunk_heat = rows;
        let probe = WorldPreviewVt4Probe {
            stamp,
            overlay_heat_hash: hash_shared_overlay_heat(&shared.chunk_fire_heat),
            overlay_revision: 2,
            consumer_active: true,
        };
        let mut agreement = VisualAgreementFrame::default();
        let mut overlay_debug = OverlayAgreementDebug::default();
        update_visual_agreement_frame(
            &frame,
            &shared,
            &overlay,
            Some(&graph),
            Some(&probe),
            &mut agreement,
        );
        overlay_debug.stamp = agreement.stamp;
        overlay_debug.overlay_revision = agreement.overlay_revision;
        overlay_debug.gpu_row_count = agreement.projected_instance_count as u32;
        overlay_debug.preview_revision = agreement.preview_overlay_revision;
        assert_eq!(overlay_debug.stamp, stamp);
        assert_eq!(agreement.mismatch_count, 0);
    }

    #[test]
    fn vt5_app_spatial_rows_pass_invariants() {
        use crate::render::vt_spatial_invariants::{passes_vt5_spatial_invariants, sample_fire_row};
        let rows = vec![
            sample_fire_row(IVec2::new(0, 0), 0.8),
            sample_fire_row(IVec2::new(16, 4), 0.7),
        ];
        assert!(passes_vt5_spatial_invariants(&rows));
    }

    #[test]
    fn vt_harness_boots_minimal_app_with_agreement_resources() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<VisualAgreementFrame>();
        app.init_resource::<OverlayAgreementDebug>();
        app.update();
        assert!(app.world().contains_resource::<VisualAgreementFrame>());
    }
}
