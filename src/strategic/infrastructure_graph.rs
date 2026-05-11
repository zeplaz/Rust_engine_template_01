//! Stub **infrastructure construction** graph (`infrastructure_construction_runbook_v1` §4–5) mirrored from the
//! active [`LogisticsGraph`](super::LogisticsGraph) so integrity / maintenance consumers can attach without a
//! second bake pass.
//!
//! **`InfrastructureNetworkType` is legacy / import-only.** Authoritative simulation uses [`crate::strategic::spatial_network::NetworkType`];
//! convert at boundaries with `NetworkType::from(infra_enum)` only.

use std::collections::HashMap;

use bevy::prelude::*;

use super::LogisticsGraph;
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId};
use crate::terrain::ChunkCellKey;

/// Network family for future construction-phase authoring (runbook enum).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InfrastructureNetworkType {
    Roads,
    Rail,
    Power,
    Pipelines,
    Communications,
}

#[derive(Clone, Debug)]
pub struct InfrastructureNode {
    pub id: u64,
    pub position: Vec2,
    pub network: InfrastructureNetworkType,
}

#[derive(Clone, Debug)]
pub struct InfrastructureEdge {
    pub from: u64,
    pub to: u64,
    pub throughput: f32,
    pub integrity: f32,
    pub maintenance_cost: f32,
    pub linked_transport_edge: Option<TransportEdgeId>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct InfrastructureGraph {
    pub nodes: Vec<InfrastructureNode>,
    pub edges: Vec<InfrastructureEdge>,
}

fn anchor_to_vec2(cells: UVec2, anchor: Option<ChunkCellKey>) -> Vec2 {
    let Some(k) = anchor else {
        return Vec2::ZERO;
    };
    let sx = cells.x.max(1);
    let sy = cells.y.max(1);
    let lx = k.cell_index % sx;
    let ly = k.cell_index / sx;
    Vec2::new(
        k.chunk.x as f32 * sx as f32 + lx as f32,
        k.chunk.y as f32 * sy as f32 + ly as f32,
    )
}

fn network_from_profile(profile: &str) -> InfrastructureNetworkType {
    let p = profile.to_ascii_lowercase();
    if p.contains("rail") {
        InfrastructureNetworkType::Rail
    } else if p.contains("pipe") {
        InfrastructureNetworkType::Pipelines
    } else if p.contains("power") || p.contains("grid") {
        InfrastructureNetworkType::Power
    } else if p.contains("comm") || p.contains("fiber") {
        InfrastructureNetworkType::Communications
    } else {
        InfrastructureNetworkType::Roads
    }
}

/// Mirror topology + tension scalars from [`LogisticsGraph`]; transport **id** pairing is best-effort (sorted keys).
pub fn sync_infrastructure_graph_from_logistics(
    logistics: Res<LogisticsGraph>,
    directory: Res<TransportEdgeDirectory>,
    cells: Res<super::transport_bridge::StrategicRasterConfig>,
    mut infra: ResMut<InfrastructureGraph>,
) {
    if logistics.nodes.is_empty() {
        infra.nodes.clear();
        infra.edges.clear();
        return;
    }

    let per = cells.cells_per_chunk.max(UVec2::ONE);

    let mut prof_by_chunk: HashMap<IVec2, InfrastructureNetworkType> = HashMap::new();
    for meta in directory.by_edge.values() {
        if let (Some((tx0, tz0)), Some((tx1, tz1))) = (
            parse_tile_key(&meta.head_key),
            parse_tile_key(&meta.tail_key),
        ) {
            for (tx, tz) in [(tx0, tz0), (tx1, tz1)] {
                let ck = tile_chunk_coord(tx, tz, per);
                prof_by_chunk
                    .entry(ck)
                    .or_insert_with(|| network_from_profile(&meta.profile));
            }
        }
    }

    let mut nodes = Vec::with_capacity(logistics.nodes.len());
    for n in &logistics.nodes {
        let network = n
            .anchor
            .map(|a| a.chunk)
            .and_then(|c| prof_by_chunk.get(&c).copied())
            .unwrap_or(InfrastructureNetworkType::Roads);
        nodes.push(InfrastructureNode {
            id: n.id.0 as u64,
            position: anchor_to_vec2(per, n.anchor),
            network,
        });
    }

    let mut transport_ids: Vec<TransportEdgeId> = directory.by_edge.keys().copied().collect();
    transport_ids.sort_by_key(|k| k.0);

    let mut edges = Vec::with_capacity(logistics.edges.len());
    for (i, e) in logistics.edges.iter().enumerate() {
        let disruption = e.disruption.clamp(0.0, 1.0);
        let linked = transport_ids.get(i).copied();
        edges.push(InfrastructureEdge {
            from: e.from.0 as u64,
            to: e.to.0 as u64,
            throughput: e.capacity,
            integrity: 1.0 - disruption,
            maintenance_cost: disruption * 0.5,
            linked_transport_edge: linked,
        });
    }

    infra.nodes = nodes;
    infra.edges = edges;
}

fn parse_tile_key(key: &str) -> Option<(u32, u32)> {
    let rest = key.strip_prefix('t')?;
    let (xs, zs) = rest.split_once('_')?;
    Some((xs.parse().ok()?, zs.parse().ok()?))
}

fn tile_chunk_coord(tx: u32, tz: u32, cells: UVec2) -> IVec2 {
    let sx = cells.x.max(1);
    let sy = cells.y.max(1);
    IVec2::new((tx / sx) as i32, (tz / sy) as i32)
}

pub struct InfrastructureGraphBridgePlugin;

impl Plugin for InfrastructureGraphBridgePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InfrastructureGraph>().add_systems(
            Update,
            sync_infrastructure_graph_from_logistics.after(super::logistics_net::logistics_net_inject_into_overlays),
        );
    }
}
