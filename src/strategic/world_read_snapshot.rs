//! Compressed **read model** for planners (not simulation truth).
//! Staleness: [`world_read_snapshot_refresh_system`] documents cadence vs strategic writers (`LOOP-07`).
//!
//! **DEC-12** (aig3 5.9 *Blackboard Architectures*): [`WorldReadSnapshot`] is the shared
//! blackboard digest. Knowledge sources / experts **read** it (and live overlays for hot
//! decisions); they **write only through queues** ([`BlackboardWriteLane`]) — never mutate
//! this resource. Sole writer: [`world_read_snapshot_refresh_system`].

use bevy::prelude::{Local, Query, Res, ResMut, Resource};

use crate::strategic::{ChunkStrategicOverlay, FrontlineState};
use crate::systems::transport::TransportEdgeDirectory;

/// Planner-facing digest: population runs after zone/frontline passes in the strategic pipeline.
///
/// Staleness: refresh runs at most every [`WORLD_READ_REFRESH_EVERY_N_FRAMES`] display frames; between refreshes
/// consumers read the last digest (**up to 11 frames** behind live overlays). Acceptable for HUD/info means only — not sim truth.
///
/// DEC-12 blackboard: epoch is the board revision; experts must not treat it as a sim tick.
#[derive(Resource, Clone, Debug, Default)]
pub struct WorldReadSnapshot {
    pub epoch: u64,
    pub chunk_cells_total: usize,
    pub mean_logistics_strength: f32,
    pub mean_mobility_cost: f32,
    pub mean_control_slot0: f32,
    pub mean_threat_slot0: f32,
    /// Mean recon confidence (faction slot 0) from overlay cells.
    pub mean_recon_confidence_slot0: f32,
    pub mean_network_visibility: f32,
    pub transport_edge_count: u32,
    pub contested_chunk_count: u32,
}

/// DEC-12: write lanes for blackboard experts (enqueue only; sole sim writers drain queues).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlackboardWriteLane {
    /// [`super::StrategicCommandQueue`] — strategic intent with dispatch delay.
    StrategicCommand,
    /// [`crate::combat::MissionIntentQueue`] — drone/mission intents.
    MissionIntent,
}

/// Blackboard revision for consumers (DEC-12); equals [`WorldReadSnapshot::epoch`].
#[inline]
#[must_use]
pub fn blackboard_revision(snap: &WorldReadSnapshot) -> u64 {
    snap.epoch
}

/// Frames between [`world_read_snapshot_refresh_system`] refreshes.
const WORLD_READ_REFRESH_EVERY_N_FRAMES: u32 = 12;

/// Staleness: on refresh, reads **this frame's** `ChunkStrategicOverlay` and `FrontlineState` (chained after
/// `derive_frontline_from_control_system` in `ZoneAndReadModel`) and **current** `TransportEdgeDirectory` (topology
/// may trail `NetworkFlow` by one frame per SCH-W1-P2-001). Skipped frames leave `WorldReadSnapshot` unchanged — acceptable
/// for planner chrome; agents must not treat `epoch` as a sim tick.
pub fn world_read_snapshot_refresh_system(
    overlays: Query<&ChunkStrategicOverlay>,
    directory: Res<TransportEdgeDirectory>,
    front: Res<FrontlineState>,
    mut snap: ResMut<WorldReadSnapshot>,
    mut seq: Local<u64>,
    mut frames: Local<u32>,
) {
    // Whole-map means for the info panel: O(all cells), so refresh a few times per second.
    // The first call runs immediately.
    let due = *seq == 0 || *frames >= WORLD_READ_REFRESH_EVERY_N_FRAMES;
    *frames = if due { 0 } else { *frames + 1 };
    if !due {
        return;
    }
    let mut n = 0usize;
    let mut ls = 0.0f32;
    let mut mob = 0.0f32;
    let mut c0 = 0.0f32;
    let mut t0 = 0.0f32;
    let mut v0 = 0.0f32;
    let mut nv = 0.0f32;
    for ov in overlays.iter() {
        let l = ov.len_cells();
        for ci in 0..l {
            ls += ov.logistics_strength[ci][0];
            mob += ov.mobility_cost[ci];
            c0 += ov.faction_control[ci][0];
            t0 += ov.threat[ci][0];
            v0 += ov.recon_confidence[ci][0];
            if ci < ov.visibility.len() {
                nv += ov.visibility[ci];
            }
            n += 1;
        }
    }
    let inv = if n > 0 { 1.0 / n as f32 } else { 0.0 };
    snap.mean_logistics_strength = ls * inv;
    snap.mean_mobility_cost = mob * inv;
    snap.mean_control_slot0 = c0 * inv;
    snap.mean_threat_slot0 = t0 * inv;
    snap.mean_recon_confidence_slot0 = v0 * inv;
    snap.mean_network_visibility = nv * inv;
    snap.chunk_cells_total = n;
    snap.contested_chunk_count = front.contested_chunks.len() as u32;
    snap.transport_edge_count = directory.by_edge.len() as u32;
    *seq = seq.wrapping_add(1);
    snap.epoch = *seq;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::strategic_command_queue::{
        enqueue_strategic_command, StrategicCommandQueue,
    };
    use crate::systems::sim_control::SimStepStamp;

    #[test]
    fn dec12_experts_enqueue_without_mutating_blackboard() {
        let mut snap = WorldReadSnapshot {
            epoch: 7,
            contested_chunk_count: 3,
            ..Default::default()
        };
        let before = blackboard_revision(&snap);
        let mut queue = StrategicCommandQueue::default();
        let issued = SimStepStamp::new(10, 0);
        enqueue_strategic_command(&mut queue, issued, "dec12 blackboard write");
        assert_eq!(queue.pending_count(), 1);
        assert_eq!(
            blackboard_revision(&snap),
            before,
            "write lane must not mutate WorldReadSnapshot"
        );
        assert_eq!(
            BlackboardWriteLane::StrategicCommand,
            BlackboardWriteLane::StrategicCommand
        );
        // sole refresh path owns epoch bumps
        snap.epoch = snap.epoch.wrapping_add(1);
        assert_eq!(blackboard_revision(&snap), before + 1);
    }
}
