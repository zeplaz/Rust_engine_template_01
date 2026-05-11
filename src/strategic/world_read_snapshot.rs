//! Compressed **read model** for planners (not simulation truth).

use bevy::prelude::{Local, Query, Res, ResMut, Resource};

use crate::strategic::{ChunkStrategicOverlay, FrontlineState};
use crate::systems::transport::TransportEdgeDirectory;

/// Planner-facing digest: population runs after zone/frontline passes in the strategic pipeline.
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

pub fn world_read_snapshot_refresh_system(
    overlays: Query<&ChunkStrategicOverlay>,
    directory: Res<TransportEdgeDirectory>,
    front: Res<FrontlineState>,
    mut snap: ResMut<WorldReadSnapshot>,
    mut seq: Local<u64>,
) {
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
