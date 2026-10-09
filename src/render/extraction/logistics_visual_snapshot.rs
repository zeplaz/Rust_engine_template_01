//! Stage-5 logistics overlay snapshot (committed sim step; render reads only).

use bevy::prelude::*;

use crate::systems::sim_control::SimStepStamp;

#[derive(Resource, Clone, Debug, Default)]
pub struct LogisticsVisualSnapshot {
    pub stamp: SimStepStamp,
    pub corridor_revision: u64,
    pub active_overlay_rows: u32,
    /// Committed corridor rows (`edge_id`, traffic factor) for GPU projection.
    pub edge_rows: Vec<(u32, f32)>,
    /// Haul position along each `edge_rows` entry, 0 at the origin.
    pub haul_marker_t: Vec<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::haul_marker_t;

    #[test]
    fn over_snapshot_keeps_haul_off_the_tuple() {
        let flow = 1.0_f32;
        let haul = haul_marker_t(flow, 12);
        assert_eq!(haul, 0.75);
        let snap = LogisticsVisualSnapshot {
            edge_rows: vec![(7, flow)],
            haul_marker_t: vec![haul],
            ..Default::default()
        };
        let (edge_id, traffic) = snap.edge_rows[0];
        assert_eq!(edge_id, 7);
        assert_eq!(traffic, flow);
        assert!((snap.haul_marker_t[0] - haul).abs() < 1e-6);
        assert!((traffic - snap.haul_marker_t[0]).abs() > 0.1);
    }
}
