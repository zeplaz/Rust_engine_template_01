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

/// Depot/site staging for manufacturable deployables (`dragon_teeth_unit` / `mine_unit`).
///
/// **Sole writer (credit):** [`super::propagation::commit_freight_arrivals_system`] for
/// deployable `FreightLot.buffer_tag`s. UI / HUD / picker must only read.
///
/// **Sole writer (debit):** [`super::propagation::apply_pending_site_staging_debits_system`]
/// in `LogisticsSimulationSet::FreightDispatch` — place/UI only enqueue
/// [`PendingSiteStagingDebits`], never `ResMut` this resource.
#[derive(Resource, Debug, Clone, Default)]
pub struct SiteStagingStock {
    pub amounts: HashMap<String, f32>,
}

impl SiteStagingStock {
    #[must_use]
    pub fn get(&self, tag: &str) -> f32 {
        self.amounts.get(tag).copied().unwrap_or(0.0)
    }

    pub fn credit(&mut self, tag: &str, amount: f32) {
        if amount <= 0.0 {
            return;
        }
        *self.amounts.entry(tag.to_string()).or_insert(0.0) += amount;
    }

    /// Debit for place-commit only — call from FreightDispatch debit applicator, not UI.
    pub fn try_debit(&mut self, tag: &str, amount: f32) -> bool {
        let need = amount.max(0.0);
        if need <= 0.0 {
            return true;
        }
        if self.get(tag) + 1e-6 < need {
            return false;
        }
        let left = self.get(tag) - need;
        if left <= 1e-6 {
            self.amounts.remove(tag);
        } else {
            self.amounts.insert(tag.to_string(), left);
        }
        true
    }
}

/// Place-commit debit request — UI/AI enqueue; FreightDispatch applies via `try_debit`.
#[derive(Clone, Debug)]
pub struct PendingStagingDebit {
    pub buffer_tag: String,
    pub amount: f32,
}

/// Soft-reserved staging debits awaiting FreightDispatch apply (COD-DEPLOYABLE-PLACE-001).
#[derive(Resource, Debug, Clone, Default)]
pub struct PendingSiteStagingDebits {
    pub queue: Vec<PendingStagingDebit>,
}

impl PendingSiteStagingDebits {
    #[must_use]
    pub fn reserved(&self, tag: &str) -> f32 {
        self.queue
            .iter()
            .filter(|d| d.buffer_tag == tag)
            .map(|d| d.amount)
            .sum()
    }

    pub fn enqueue(&mut self, tag: &str, amount: f32) {
        let amount = amount.max(0.0);
        if amount <= f32::EPSILON {
            return;
        }
        self.queue.push(PendingStagingDebit {
            buffer_tag: tag.to_string(),
            amount,
        });
    }
}

#[derive(Clone, Debug)]
pub struct FreightReservation {
    pub edge_index: usize,
    pub amount: f32,
    /// RouteProof / demand request that holds this slice of capacity.
    pub request_id: u64,
}

/// Sparse per-tick reservation ledger (diagnostics). SoA `ThroughputSolverState.reserved`
/// remains the capacity-check authority; this book mirrors the same solve pass.
#[derive(Resource, Default)]
pub struct FreightReservationBook {
    pub entries: Vec<FreightReservation>,
}

impl FreightReservationBook {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn push(&mut self, entry: FreightReservation) {
        self.entries.push(entry);
    }

    /// Sum book amounts by edge index — must match `ThroughputSolverState.reserved`.
    #[must_use]
    pub fn reserved_sums(&self, edge_count: usize) -> Vec<f32> {
        let mut sums = vec![0.0; edge_count];
        for e in &self.entries {
            if e.edge_index < edge_count {
                sums[e.edge_index] += e.amount;
            }
        }
        sums
    }
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

/// One saturated corridor sample for LOG-D-05 diagnostics panel.
#[derive(Clone, Debug, Default)]
pub struct SaturatedEdgeSample {
    pub edge_id: u64,
    pub pressure: f32,
    pub load: f32,
    pub capacity: f32,
}

#[derive(Resource, Default)]
pub struct LogisticsDiagnostics {
    pub routes_open: u32,
    pub routes_blocked: u32,
    pub proofs: Vec<RouteProof>,
    pub request_id_seq: u64,
    /// Top saturated transport edges by solver pressure (LOG-D-05).
    pub top_saturated_edges: Vec<SaturatedEdgeSample>,
    /// Catalog ids of facilities with [`crate::economy::resource_flow::FacilityFlowState::starved`].
    pub starved_facilities: Vec<String>,
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
    /// Sticky: district-scoped solve applied (LOG-D-02) — empty skip or active mask.
    pub saw_district_scoped_solve: bool,
    /// Sticky: async district job applied on main thread (LOG-D-04).
    pub saw_async_district_solve: bool,
    /// Sticky: panel collector wrote top-edge or starved samples (LOG-D-05).
    pub saw_diagnostics_panel_detail: bool,
}

/// Active industrial-district edge mask for throughput solve (LOG-D-02).
#[derive(Resource, Clone, Debug, Default)]
pub struct DistrictSolveScope {
    pub active_district_count: u32,
    pub empty_districts_skipped: u32,
    pub edges_skipped_outside_district: u32,
    /// Sorted unique transport-edge indices that remain solvable.
    pub active_edge_indices: Vec<usize>,
}

impl DistrictSolveScope {
    pub fn clear(&mut self) {
        self.active_district_count = 0;
        self.empty_districts_skipped = 0;
        self.edges_skipped_outside_district = 0;
        self.active_edge_indices.clear();
    }
}
