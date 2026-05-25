//! **VT-4** visual agreement — stamp + hash validation across fire snapshot consumers.
//!
//! Tactical GPU surfaces agree with [`crate::render::fire_view_extract::tactical_fire_visual`] (view-culled).
//! Shared overlay heat agrees with the full [`FireSimulationSnapshot`] (max-per-chunk), not the culled tactical frame.

use bevy::prelude::*;

use crate::gui::OverlayFieldFrame;
use crate::render::extraction::RenderProjectionGraph;
use crate::render::overlay_field_buffers::SharedOverlayFieldBuffers;
use crate::render::{tactical_fire_visual, FireSimulationSnapshot, FireVisualFramesByView};
use crate::render::sim_visual_extract::{ChunkFireHeat, FireVisualFrame};
use crate::systems::sim_control::SimStepStamp;

/// Latest overlay sample for the world-preview consumer (CPU fallback or GPU target).
#[derive(Resource, Clone, Debug, Default)]
pub struct WorldPreviewVt4Probe {
    pub stamp: SimStepStamp,
    pub overlay_heat_hash: u64,
    pub overlay_revision: u64,
    /// True when the preview window is open and this frame's probe was captured.
    pub consumer_active: bool,
}

impl WorldPreviewVt4Probe {
    #[inline]
    #[must_use]
    pub fn participates_in_vt4(&self) -> bool {
        self.consumer_active
    }
}

/// VT-4 strict agreement debug (minimap / preview / GPU fire field).
#[derive(Resource, Debug, Clone, Default)]
pub struct OverlayAgreementDebug {
    pub stamp: SimStepStamp,
    pub compared_stamp: SimStepStamp,
    pub mismatch_count: u32,
    pub failing_surface_mask: u32,
    pub overlay_revision: u64,
    pub gpu_row_count: u32,
    pub preview_revision: u64,
}

/// Cross-surface agreement record for one committed sim step.
#[derive(Resource, Debug, Clone, Default)]
pub struct VisualAgreementFrame {
    pub stamp: SimStepStamp,
    pub fire_instance_count: usize,
    pub chunk_heat_count: usize,
    pub fire_heat_hash: u64,
    pub overlay_revision: u64,
    pub projected_fire_heat_hash: u64,
    pub projected_instance_count: usize,
    pub preview_overlay_heat_hash: u64,
    pub preview_overlay_revision: u64,
    /// Max-per-chunk hash from full [`FireSimulationSnapshot`] (overlay spine truth).
    pub sim_overlay_heat_hash: u64,
    pub mismatch_count: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualAgreementError {
    StampMismatch {
        label: &'static str,
        expected: SimStepStamp,
        actual: SimStepStamp,
    },
}

/// FNV-1a over chunk heat rows (stable for VT-4 harnesses).
#[must_use]
pub fn hash_chunk_fire_heat(rows: &[ChunkFireHeat]) -> u64 {
    let mut sorted: Vec<ChunkFireHeat> = rows.to_vec();
    sorted.sort_by_key(|row| (row.chunk.x, row.chunk.y));
    let mut hash = 0xcbf29ce484222325u64;
    for row in &sorted {
        for byte in row.chunk.x.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        for byte in row.chunk.y.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        let heat = (row.heat.clamp(0.0, 1.0) * 10_000.0).round() as u32;
        for byte in heat.to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}

/// Same aggregation as [`sync_shared_overlay_from_simulation`] (max heat per chunk).
#[must_use]
pub fn hash_simulation_chunk_heat_for_overlay(sim: &FireSimulationSnapshot) -> u64 {
    use std::collections::HashMap;
    let mut map: HashMap<IVec2, f32> = HashMap::new();
    for h in &sim.chunk_heat {
        map.entry(h.chunk)
            .and_modify(|e| *e = f32::max(*e, h.heat))
            .or_insert(h.heat);
    }
    hash_shared_overlay_heat(&map)
}

#[must_use]
pub fn hash_shared_overlay_heat(map: &std::collections::HashMap<IVec2, f32>) -> u64 {
    let rows: Vec<ChunkFireHeat> = map
        .iter()
        .map(|(chunk, heat)| ChunkFireHeat {
            chunk: *chunk,
            heat: *heat,
            smoke: 0.0,
        })
        .collect();
    hash_chunk_fire_heat(&rows)
}

#[inline]
pub fn assert_snapshot_stamp(
    label: &'static str,
    expected: SimStepStamp,
    actual: SimStepStamp,
) -> Result<(), VisualAgreementError> {
    if expected == actual {
        Ok(())
    } else {
        Err(VisualAgreementError::StampMismatch {
            label,
            expected,
            actual,
        })
    }
}

pub fn record_visual_agreement_frame(
    fire_by_view: Res<FireVisualFramesByView>,
    sim: Res<FireSimulationSnapshot>,
    shared: Res<SharedOverlayFieldBuffers>,
    overlay: Res<OverlayFieldFrame>,
    projection: Option<Res<RenderProjectionGraph>>,
    preview_probe: Option<Res<WorldPreviewVt4Probe>>,
    mut agreement: ResMut<VisualAgreementFrame>,
    mut overlay_debug: ResMut<OverlayAgreementDebug>,
) {
    let fire = tactical_fire_visual(fire_by_view.as_ref());
    update_visual_agreement_frame(
        fire,
        sim.as_ref(),
        shared.as_ref(),
        overlay.as_ref(),
        projection.as_deref(),
        preview_probe.as_deref(),
        agreement.as_mut(),
    );
    overlay_debug.stamp = agreement.stamp;
    overlay_debug.compared_stamp = agreement.stamp;
    overlay_debug.mismatch_count = agreement.mismatch_count.min(u32::MAX as u64) as u32;
    overlay_debug.overlay_revision = agreement.overlay_revision;
    overlay_debug.gpu_row_count = agreement.projected_instance_count as u32;
    overlay_debug.preview_revision = agreement.preview_overlay_revision;
}

pub fn update_visual_agreement_frame(
    fire: &FireVisualFrame,
    sim: &FireSimulationSnapshot,
    shared: &SharedOverlayFieldBuffers,
    overlay: &OverlayFieldFrame,
    projection: Option<&RenderProjectionGraph>,
    preview_probe: Option<&WorldPreviewVt4Probe>,
    agreement: &mut VisualAgreementFrame,
) {
    agreement.mismatch_count = 0;
    agreement.stamp = fire.stamp;
    agreement.fire_instance_count = fire.instances.len();
    agreement.chunk_heat_count = fire.chunk_heat.len();
    agreement.fire_heat_hash = hash_chunk_fire_heat(&fire.chunk_heat);
    agreement.sim_overlay_heat_hash = hash_simulation_chunk_heat_for_overlay(sim);
    agreement.overlay_revision = overlay.fire_heat_overlay_revision;

    if let Some(graph) = projection.as_deref() {
        agreement.projected_fire_heat_hash = hash_chunk_fire_heat(&graph.fire.chunk_heat);
        agreement.projected_instance_count = graph.fire.instance_buffer.len();
    } else {
        agreement.projected_fire_heat_hash = 0;
        agreement.projected_instance_count = 0;
    }

    if let Some(probe) = preview_probe.as_deref() {
        agreement.preview_overlay_heat_hash = probe.overlay_heat_hash;
        agreement.preview_overlay_revision = probe.overlay_revision;
    } else {
        agreement.preview_overlay_heat_hash = 0;
        agreement.preview_overlay_revision = 0;
    }

    let checks = [
        assert_snapshot_stamp("overlay_field_frame", fire.stamp, overlay.stamp),
        assert_snapshot_stamp("shared_overlay_buffers", fire.stamp, shared.stamp),
    ];
    for check in checks {
        if let Err(err) = check {
            agreement.mismatch_count = agreement.mismatch_count.wrapping_add(1);
            match err {
                VisualAgreementError::StampMismatch {
                    label,
                    expected,
                    actual,
                } => {
                    warn!(
                        "VT-4 stamp mismatch on {label}: expected tick {} (t={}µs), got tick {} (t={}µs)",
                        expected.tick,
                        expected.sim_time_micros,
                        actual.tick,
                        actual.sim_time_micros
                    );
                }
            }
        }
    }

    let overlay_hash = hash_shared_overlay_heat(&shared.chunk_fire_heat);
    // PLAY-06c: empty sim snapshot + retained shared overlay is intentional — not a VT-4 failure.
    let held_overlay_persist =
        sim.chunk_heat.is_empty() && !shared.chunk_fire_heat.is_empty();
    if !held_overlay_persist && overlay_hash != agreement.sim_overlay_heat_hash {
        agreement.mismatch_count = agreement.mismatch_count.wrapping_add(1);
        warn!(
            "VT-4 overlay vs sim heat hash mismatch: sim={} overlay={} tactical_frame={}",
            agreement.sim_overlay_heat_hash, overlay_hash, agreement.fire_heat_hash
        );
    }

    if projection.is_some() {
        if agreement.projected_fire_heat_hash != agreement.fire_heat_hash {
            agreement.mismatch_count = agreement.mismatch_count.wrapping_add(1);
            warn!(
                "VT-4 projected fire heat hash mismatch: frame={} projection={}",
                agreement.fire_heat_hash, agreement.projected_fire_heat_hash
            );
        }
    }

    if let Some(probe) = preview_probe.as_deref() {
        if probe.participates_in_vt4() && probe.overlay_heat_hash != agreement.sim_overlay_heat_hash {
            agreement.mismatch_count = agreement.mismatch_count.wrapping_add(1);
            warn!(
                "VT-4 world preview overlay hash mismatch: sim={} preview={} tactical_frame={}",
                agreement.sim_overlay_heat_hash, probe.overlay_heat_hash, agreement.fire_heat_hash
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn sim_from_frame(frame: &FireVisualFrame) -> FireSimulationSnapshot {
        FireSimulationSnapshot {
            stamp: frame.stamp,
            instances: frame.instances.clone(),
            chunk_heat: frame.chunk_heat.clone(),
        }
    }

    #[test]
    fn projected_and_preview_hashes_match_frame_rows() {
        let rows = vec![ChunkFireHeat {
            chunk: IVec2::new(1, 2),
            heat: 0.5,
            smoke: 0.0,
        }];
        let frame = FireVisualFrame {
            stamp: SimStepStamp::new(1, 2),
            instances: Vec::new(),
            chunk_heat: rows.clone(),
        };
        let mut shared = SharedOverlayFieldBuffers::default();
        shared.stamp = frame.stamp;
        shared.chunk_fire_heat.insert(IVec2::new(1, 2), 0.5);
        let overlay = OverlayFieldFrame {
            stamp: frame.stamp,
            fields: HashMap::new(),
            fire_heat_overlay_revision: 1,
        };
        let mut graph = RenderProjectionGraph::default();
        graph.fire.chunk_heat = rows;
        let probe = WorldPreviewVt4Probe {
            stamp: frame.stamp,
            overlay_heat_hash: hash_shared_overlay_heat(&shared.chunk_fire_heat),
            overlay_revision: 1,
            consumer_active: true,
        };
        let sim = sim_from_frame(&frame);
        let mut agreement = VisualAgreementFrame::default();
        update_visual_agreement_frame(
            &frame,
            &sim,
            &shared,
            &overlay,
            Some(&graph),
            Some(&probe),
            &mut agreement,
        );
        assert_eq!(agreement.mismatch_count, 0);
        assert_eq!(agreement.fire_heat_hash, agreement.projected_fire_heat_hash);
        assert_eq!(agreement.sim_overlay_heat_hash, agreement.preview_overlay_heat_hash);
    }

    #[test]
    fn vt4_strict_overlay_agreement_debug_matches_projection() {
        let rows = vec![ChunkFireHeat {
            chunk: IVec2::new(2, 1),
            heat: 0.75,
            smoke: 0.0,
        }];
        let frame = FireVisualFrame {
            stamp: SimStepStamp::new(3, 9),
            instances: Vec::new(),
            chunk_heat: rows.clone(),
        };
        let mut shared = SharedOverlayFieldBuffers::default();
        shared.stamp = frame.stamp;
        shared.chunk_fire_heat.insert(IVec2::new(2, 1), 0.75);
        let overlay = OverlayFieldFrame {
            stamp: frame.stamp,
            fields: HashMap::new(),
            fire_heat_overlay_revision: 4,
        };
        let mut graph = RenderProjectionGraph::default();
        graph.fire.chunk_heat = rows;
        graph.fire.instance_buffer = Vec::new();
        let probe = WorldPreviewVt4Probe {
            stamp: frame.stamp,
            overlay_heat_hash: hash_shared_overlay_heat(&shared.chunk_fire_heat),
            overlay_revision: 4,
            consumer_active: true,
        };
        let sim = sim_from_frame(&frame);
        let mut agreement = VisualAgreementFrame::default();
        let mut overlay_debug = OverlayAgreementDebug::default();
        update_visual_agreement_frame(
            &frame,
            &sim,
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
        assert_eq!(overlay_debug.stamp, frame.stamp);
        assert_eq!(overlay_debug.overlay_revision, 4);
        assert_eq!(overlay_debug.preview_revision, 4);
        assert_eq!(agreement.mismatch_count, 0);
    }

    #[test]
    fn vt4_three_surface_stamp_agreement() {
        let stamp = SimStepStamp::new(11, 5);
        let rows = vec![ChunkFireHeat {
            chunk: IVec2::new(3, 4),
            heat: 0.6,
            smoke: 0.0,
        }];
        let frame = FireVisualFrame {
            stamp,
            instances: Vec::new(),
            chunk_heat: rows.clone(),
        };
        let mut shared = SharedOverlayFieldBuffers::default();
        shared.stamp = stamp;
        shared.chunk_fire_heat.insert(IVec2::new(3, 4), 0.6);
        let overlay = OverlayFieldFrame {
            stamp,
            fields: HashMap::new(),
            fire_heat_overlay_revision: 9,
        };
        let mut graph = RenderProjectionGraph::default();
        graph.fire.chunk_heat = rows;
        let probe = WorldPreviewVt4Probe {
            stamp,
            overlay_heat_hash: hash_shared_overlay_heat(&shared.chunk_fire_heat),
            overlay_revision: 9,
            consumer_active: true,
        };
        let sim = sim_from_frame(&frame);
        let mut agreement = VisualAgreementFrame::default();
        let mut overlay_debug = OverlayAgreementDebug::default();
        update_visual_agreement_frame(
            &frame,
            &sim,
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
        assert_eq!(overlay_debug.stamp, shared.stamp);
        assert_eq!(overlay_debug.stamp, overlay.stamp);
        assert_eq!(overlay_debug.preview_revision, overlay_debug.overlay_revision);
        assert_eq!(agreement.mismatch_count, 0);
    }

    #[test]
    fn overlay_agrees_when_tactical_frame_view_culled() {
        let stamp = SimStepStamp::new(340, 76_083_967);
        let sim_rows = vec![
            ChunkFireHeat {
                chunk: IVec2::new(10, 5),
                heat: 0.8,
                smoke: 0.0,
            },
            ChunkFireHeat {
                chunk: IVec2::new(11, 6),
                heat: 0.6,
                smoke: 0.0,
            },
            ChunkFireHeat {
                chunk: IVec2::new(12, 7),
                heat: 0.4,
                smoke: 0.0,
            },
        ];
        let sim = FireSimulationSnapshot {
            stamp,
            instances: Vec::new(),
            chunk_heat: sim_rows,
        };
        let tactical = FireVisualFrame {
            stamp,
            instances: Vec::new(),
            chunk_heat: Vec::new(),
        };
        let mut shared = SharedOverlayFieldBuffers::default();
        shared.stamp = stamp;
        for h in &sim.chunk_heat {
            shared
                .chunk_fire_heat
                .entry(h.chunk)
                .and_modify(|e| *e = f32::max(*e, h.heat))
                .or_insert(h.heat);
        }
        let overlay = OverlayFieldFrame {
            stamp,
            fields: HashMap::new(),
            fire_heat_overlay_revision: 61,
        };
        let mut agreement = VisualAgreementFrame::default();
        update_visual_agreement_frame(
            &tactical,
            &sim,
            &shared,
            &overlay,
            None,
            None,
            &mut agreement,
        );
        assert_eq!(agreement.mismatch_count, 0);
        assert_ne!(agreement.fire_heat_hash, agreement.sim_overlay_heat_hash);
    }

    #[test]
    fn held_overlay_persist_skips_vt4_overlay_sim_mismatch() {
        let stamp = SimStepStamp::new(99, 1_000);
        let sim = FireSimulationSnapshot {
            stamp,
            instances: Vec::new(),
            chunk_heat: Vec::new(),
        };
        let tactical = FireVisualFrame {
            stamp,
            instances: Vec::new(),
            chunk_heat: Vec::new(),
        };
        let mut shared = SharedOverlayFieldBuffers::default();
        shared.stamp = stamp;
        shared
            .chunk_fire_heat
            .insert(IVec2::new(3, 4), 0.42);
        let overlay = OverlayFieldFrame {
            stamp,
            fields: HashMap::new(),
            fire_heat_overlay_revision: 1,
        };
        let mut agreement = VisualAgreementFrame::default();
        update_visual_agreement_frame(
            &tactical,
            &sim,
            &shared,
            &overlay,
            None,
            None,
            &mut agreement,
        );
        assert_eq!(agreement.mismatch_count, 0);
        assert_ne!(agreement.sim_overlay_heat_hash, hash_shared_overlay_heat(&shared.chunk_fire_heat));
    }

    #[test]
    fn shared_overlay_hash_matches_frame_rows() {
        let rows = vec![
            ChunkFireHeat {
                chunk: IVec2::new(1, 2),
                heat: 0.5,
                smoke: 0.0,
            },
            ChunkFireHeat {
                chunk: IVec2::new(0, 0),
                heat: 0.25,
                smoke: 0.0,
            },
        ];
        let mut map = std::collections::HashMap::new();
        map.insert(IVec2::new(1, 2), 0.5);
        map.insert(IVec2::new(0, 0), 0.25);
        assert_eq!(hash_chunk_fire_heat(&rows), hash_shared_overlay_heat(&map));
    }
}
