//! Logistics throughput types (LOG-A…D).

use bevy::prelude::*;
use std::collections::HashMap;

use crate::systems::transport::TransportEdgeId;
use crate::terrain::ChunkCellKey;

/// Tile node key aligned with transport bake (`t{x}_{z}`).
#[derive(Component, Clone, Debug, PartialEq, Eq, Hash)]
pub struct TransportNodeAnchor(pub String);

#[derive(Component, Clone, Debug)]
pub struct FacilityPortal {
    pub anchor: ChunkCellKey,
    pub transport_anchor: TransportNodeAnchor,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct FacilityPortalRegistered;

#[derive(Resource, Clone, Debug, Default)]
pub struct PortalAttachmentMap {
    pub revision: u64,
    pub facility_to_graph: HashMap<Entity, crate::strategic::LogisticsNodeId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RouteHandle {
    pub id: u32,
    pub topology_revision: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct RoutePath {
    pub first_edge: u32,
    pub edge_count: u16,
}

#[derive(Resource, Default)]
pub struct RoutePathStore {
    pub edges: Vec<TransportEdgeId>,
    pub paths: Vec<RoutePath>,
}

impl RoutePathStore {
    pub fn insert_path(&mut self, edge_ids: &[TransportEdgeId]) -> RoutePath {
        let first = self.edges.len() as u32;
        self.edges.extend_from_slice(edge_ids);
        let path = RoutePath {
            first_edge: first,
            edge_count: edge_ids.len().min(u16::MAX as usize) as u16,
        };
        self.paths.push(path);
        path
    }

    #[must_use]
    pub fn edge_slice(&self, path: RoutePath) -> &[TransportEdgeId] {
        let start = path.first_edge as usize;
        let end = start + path.edge_count as usize;
        &self.edges[start..end.min(self.edges.len())]
    }
}

#[derive(Resource, Default)]
pub struct RouteCache {
    pub topology_revision: u32,
    pub routes: HashMap<(Entity, Entity), CachedRoute>,
    pub next_id: u32,
}

#[derive(Clone, Debug)]
pub struct CachedRoute {
    pub handle: RouteHandle,
    pub path: RoutePath,
    pub reachable: bool,
    pub bottleneck_capacity: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FreightMovementModel {
    Continuous,
    Batched,
}

#[derive(Clone, Debug)]
pub struct FreightLot {
    pub destination: Entity,
    pub buffer_tag: String,
    pub amount: f32,
    pub route: RouteHandle,
    pub path: RoutePath,
    pub progress_edge: u16,
    pub remaining_ticks: u16,
    pub movement: FreightMovementModel,
}

#[derive(Resource, Default)]
pub struct InTransitLedger {
    pub lots: Vec<FreightLot>,
}

#[derive(Clone, Debug)]
pub struct FreightReservation {
    pub edge_index: usize,
    pub amount: f32,
}

#[derive(Resource, Default)]
pub struct ThroughputSolverState {
    pub topology_revision: u32,
    pub load: Vec<f32>,
    pub capacity: Vec<f32>,
    pub reserved: Vec<f32>,
    pub edge_pressure: Vec<f32>,
}

impl ThroughputSolverState {
    pub fn ensure_len(&mut self, edge_count: usize) {
        if self.load.len() < edge_count {
            self.load.resize(edge_count, 0.0);
            self.capacity.resize(edge_count, 0.0);
            self.reserved.resize(edge_count, 0.0);
            self.edge_pressure.resize(edge_count, 0.0);
        }
    }

    pub fn clear_tick(&mut self) {
        for v in &mut self.load {
            *v = 0.0;
        }
        for v in &mut self.reserved {
            *v = 0.0;
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RouteProof {
    pub request_id: u64,
    pub from_catalog: String,
    pub to_catalog: String,
    pub requested: f32,
    pub delivered: f32,
    pub blocked_at: Option<TransportEdgeId>,
    pub bottleneck_capacity: f32,
}

#[derive(Resource, Default)]
pub struct LogisticsDiagnostics {
    pub routes_open: u32,
    pub routes_blocked: u32,
    pub proofs: Vec<RouteProof>,
    pub request_id_seq: u64,
}

#[derive(Resource, Default)]
pub struct LogisticsThroughputRuntimeWitness {
    pub routes_open: u32,
    pub routes_blocked: u32,
    pub topology_revision: u32,
    pub edge_saturation_max: f32,
    /// Sticky: ledger held freight at least once this session (LOG-B-02).
    pub saw_in_transit_lot: bool,
    /// Sticky: route paths were cached at least once (LOG-B-01).
    pub saw_route_path: bool,
    /// Sticky: transport congestion rose from solver load (LOG-C-03).
    pub saw_congestion_feedback: bool,
    /// Sticky: corridor pressure diffused to a successor edge (LOG-C-04).
    pub saw_corridor_pressure: bool,
    /// Sticky: overlay injection used solver load (LOG-C-06).
    pub saw_overlay_solver_load: bool,
    /// Sticky: route cache rebuilt after topology revision bump (LOG-D-03).
    pub saw_route_invalidation: bool,
}
