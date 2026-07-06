//! VT-5 spatial distribution invariants for projected fire instances.

use bevy::math::IVec2;
use bevy::prelude::Vec4;

use crate::render::extraction::spatial_distribution_stats;
use crate::render::sim_visual_extract::FireVisualGpuInstance;

pub const VT5_MIN_OCCUPIED_CHUNKS: usize = 2;
pub const VT5_MIN_MEAN_DISTANCE: f32 = 1.0;
pub const VT5_MIN_VARIANCE: f32 = 0.1;
/// VR-04 — defer VT-5 readiness until ecology seed spreads past bootstrap burst (`fire_inst` ≈ 2).
pub const VT5_MIN_EVAL_FIRE_INSTANCES: usize = 3;

#[must_use]
pub fn sample_fire_row(chunk: IVec2, heat: f32) -> FireVisualGpuInstance {
    let mut row = FireVisualGpuInstance::default();
    row.chunk_xy_heat_lum = Vec4::new(chunk.x as f32, chunk.y as f32, heat, 1.0);
    row
}

#[must_use]
pub fn passes_vt5_spatial_invariants(rows: &[FireVisualGpuInstance]) -> bool {
    let (occupied, mean, variance) = spatial_distribution_stats(rows);
    occupied >= VT5_MIN_OCCUPIED_CHUNKS
        && mean > VT5_MIN_MEAN_DISTANCE
        && variance > VT5_MIN_VARIANCE
}

/// VR-04 triage — skip VT-5 gate while fire extract is still in bootstrap burst.
#[must_use]
pub fn vt5_spatial_eval_deferred(rows: &[FireVisualGpuInstance]) -> bool {
    rows.len() < VT5_MIN_EVAL_FIRE_INSTANCES
        || spatial_distribution_stats(rows).0 < VT5_MIN_OCCUPIED_CHUNKS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_chunk_rows_pass_vt5() {
        let rows = vec![
            sample_fire_row(IVec2::new(0, 0), 0.8),
            sample_fire_row(IVec2::new(12, 4), 0.7),
            sample_fire_row(IVec2::new(-6, 8), 0.6),
        ];
        assert!(passes_vt5_spatial_invariants(&rows));
    }

    #[test]
    fn collapsed_square_fails_vt5() {
        let rows = vec![
            sample_fire_row(IVec2::new(0, 0), 0.8),
            sample_fire_row(IVec2::new(0, 0), 0.7),
        ];
        assert!(!passes_vt5_spatial_invariants(&rows));
    }

    #[test]
    fn scene_matrix_synthetic_layouts_cover_vt5_thresholds() {
        let layouts = [
            vec![
                sample_fire_row(IVec2::new(0, 0), 0.9),
                sample_fire_row(IVec2::new(24, 0), 0.8),
            ],
            vec![
                sample_fire_row(IVec2::new(-12, 6), 0.7),
                sample_fire_row(IVec2::new(18, -4), 0.75),
                sample_fire_row(IVec2::new(0, 20), 0.65),
            ],
            vec![
                sample_fire_row(IVec2::new(64, 0), 0.6),
                sample_fire_row(IVec2::new(0, 20), 0.6),
            ],
        ];
        for rows in layouts {
            assert!(
                passes_vt5_spatial_invariants(&rows),
                "expected VT-5 pass for spread layout"
            );
        }
    }

    #[test]
    fn vr04_bootstrap_burst_deferred_before_eval_threshold() {
        let rows = vec![
            sample_fire_row(IVec2::new(0, 0), 0.8),
            sample_fire_row(IVec2::new(1, 0), 0.7),
        ];
        assert!(vt5_spatial_eval_deferred(&rows));
        assert!(!passes_vt5_spatial_invariants(&rows));
    }
}
