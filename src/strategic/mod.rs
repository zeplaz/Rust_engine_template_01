//! **Operational strategy** — continuous fields, sparse graphs, and derived blobs.
//!
//! # Three spatial systems (do not collapse them)
//!
//! 1. **Static geographic skeleton** — Voronoi [`MacroRegion`](crate::terrain::generation::world_generator_enhanced::MacroRegion),
//!    [`MacroRegionRaster`](crate::terrain::generation::world_generator_enhanced::MacroRegionRaster), terrain semantics
//!    ([`MacroTerrainSemantics`](crate::terrain::generation::polygon_world_semantics::MacroTerrainSemantics)).
//!    Stable IDs, chunk grouping, ecology/climate, saves. Barely moves.
//!
//! 2. **Dynamic operational fields** (this module’s focus) — scalar/vector **heatmaps** per chunk cell: control,
//!    threat, recon certainty, logistics strength, fire/smoke, mobility cost, etc. Updated every tick (or sub-tick).
//!    **Not** polygon ownership. Fronts and contested belts **emerge** from gradients and thresholds on these fields.
//!
//! 3. **Strategic graphs** — roads, rails, pipelines, grids, supply routes, attack axes: sparse, weighted, degradable.
//!    Fields and graphs **couple** (e.g. logistics throughput on edges feeds cell `logistics_strength`).
//!
//! “Regions” in the sense of maneuver warfare are **derived**: flood-fill or level-set on field combinations
//! (`control > 0.65 && threat < τ`), not reassigned provinces.
//!
//! GPU note: layers are dense per chunk; diffusion and updates map cleanly to compute later.
//!
//! **Delivery phases:** `prompts/designer_questions/strategic_platforms/phased_engine_delivery_v1.md` (Phase 1a).
//!
//! **Runbook test rounds:** incremental stubs in [`runbook_rounds`](runbook_rounds.rs) mirror `prompts/guides/*_runbook_v1.md` execution tables.

mod runbook_rounds;
mod logistics_net;
mod plugin;
mod program;
mod sim;
mod transport_bridge;

pub use program::StrategicFieldsAndAiPlugin;
pub use sim::{
    CityPlanningHints, InfrastructureCorridor, LogisticsAiRuntime, OperationalTheaterSummary,
    SettlementSite, StrategicSimulationPlugin, StrategicTransportCorridor,
};

pub use runbook_rounds::city_planning::{
    site_score, utility_redundancy_weight, SettlementArchetype,
};
pub use runbook_rounds::corridor::{
    corridor_capacity_weight, corridor_total_cost, pick_cheaper_corridor_index, CorridorCost,
    CorridorType,
};
pub use runbook_rounds::logistics_ai_policy::{
    demand_forecast, effective_priority_weight, reroute_recommended, LogisticsPriority,
};
pub use runbook_rounds::operational_warfare::{
    doctrine_strike_weight, offensive_commit_score, DroneDoctrine,
    INFRASTRUCTURE_COUPLED_STRIKES_DEFAULT,
};
pub use runbook_rounds::settlement::{
    ecology_hazard_pressure, migration_pull, tier_from_population, SettlementTier,
};

pub use logistics_net::logistics_net_inject_into_overlays;
pub use plugin::StrategicFieldsPlugin;
pub use transport_bridge::StrategicRasterConfig;

use bevy::prelude::{Component, IVec2, Resource, UVec2};

use crate::terrain::ChunkCellKey;

/// Packed faction scalars per cell index (`faction_slot` → value). Runtime maps logical faction id → slot.
pub const MAX_STRATEGIC_FACTION_SLOTS: usize = 16;

/// One simulation cell: **operational** state (many quantities can coexist on the same tile).
///
/// For dense chunk storage prefer SOA via [`ChunkStrategicOverlay`]; this struct documents the logical bundle.
#[derive(Clone, Copy, Debug, Default)]
pub struct StrategicFieldCell {
    pub faction_control: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub threat: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub recon_confidence: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub artillery_danger: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub logistics_strength: [f32; MAX_STRATEGIC_FACTION_SLOTS],
    pub fire_risk: f32,
    pub smoke_density: f32,
    pub mobility_cost: f32,
    pub attrition_rate: f32,
    pub civilian_stability: f32,
    /// Local routing congestion 0..1 (transport + field coupling).
    pub routing_congestion: f32,
    /// EW / comms denial proxy 0..1.
    pub ew_denial: f32,
}

/// Chunk-aligned **SOA** operational overlay (same cell ordering as [`crate::terrain::generation::ChunkCellMatrix`] when sizes match).
#[derive(Component, Clone, Debug)]
pub struct ChunkStrategicOverlay {
    pub chunk_coord: IVec2,
    pub size: UVec2,
    pub faction_control: Vec<[f32; MAX_STRATEGIC_FACTION_SLOTS]>,
    pub threat: Vec<[f32; MAX_STRATEGIC_FACTION_SLOTS]>,
    pub recon_confidence: Vec<[f32; MAX_STRATEGIC_FACTION_SLOTS]>,
    pub artillery_danger: Vec<[f32; MAX_STRATEGIC_FACTION_SLOTS]>,
    pub logistics_strength: Vec<[f32; MAX_STRATEGIC_FACTION_SLOTS]>,
    pub logistics_throughput: Vec<f32>,
    pub mobility_cost: Vec<f32>,
    pub attrition_rate: Vec<f32>,
    pub fire_risk: Vec<f32>,
    pub smoke_density: Vec<f32>,
    pub civilian_stability: Vec<f32>,
    /// Routing stress from transport endpoints (R5+ channel).
    pub routing_congestion: Vec<f32>,
    /// EW / GNSS denial proxy field.
    pub ew_denial: Vec<f32>,
}

impl ChunkStrategicOverlay {
    pub fn new(chunk_coord: IVec2, size: UVec2) -> Self {
        let n = (size.x as usize).saturating_mul(size.y as usize);
        let z_pack = || vec![[0.0; MAX_STRATEGIC_FACTION_SLOTS]; n];
        let z_scalar = || vec![0.0; n];
        Self {
            chunk_coord,
            size,
            faction_control: z_pack(),
            threat: z_pack(),
            recon_confidence: z_pack(),
            artillery_danger: z_pack(),
            logistics_strength: z_pack(),
            logistics_throughput: z_scalar(),
            mobility_cost: z_scalar(),
            attrition_rate: z_scalar(),
            fire_risk: z_scalar(),
            smoke_density: z_scalar(),
            civilian_stability: z_scalar(),
            routing_congestion: z_scalar(),
            ew_denial: z_scalar(),
        }
    }

    #[inline]
    pub fn len_cells(&self) -> usize {
        self.faction_control.len()
    }

    /// Write **per-faction** threat for one cell (`strategic_overlay` runbook — faction slots / field writers).
    pub fn set_faction_threat(
        &mut self,
        cell: usize,
        faction_slot: usize,
        value: f32,
    ) -> Result<(), ()> {
        if cell >= self.threat.len() || faction_slot >= MAX_STRATEGIC_FACTION_SLOTS {
            return Err(());
        }
        self.threat[cell][faction_slot] = value;
        Ok(())
    }

    /// Recon certainty for a faction slot (overlay writer stub).
    pub fn set_recon_confidence(
        &mut self,
        cell: usize,
        faction_slot: usize,
        value: f32,
    ) -> Result<(), ()> {
        if cell >= self.recon_confidence.len() || faction_slot >= MAX_STRATEGIC_FACTION_SLOTS {
            return Err(());
        }
        self.recon_confidence[cell][faction_slot] = value;
        Ok(())
    }

    /// Artillery danger heat per faction slot.
    pub fn set_artillery_danger(
        &mut self,
        cell: usize,
        faction_slot: usize,
        value: f32,
    ) -> Result<(), ()> {
        if cell >= self.artillery_danger.len() || faction_slot >= MAX_STRATEGIC_FACTION_SLOTS {
            return Err(());
        }
        self.artillery_danger[cell][faction_slot] = value;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct LogisticsNodeId(pub u32);

#[derive(Clone, Debug)]
pub struct LogisticsNode {
    pub id: LogisticsNodeId,
    pub throughput: f32,
    pub stockpile: f32,
    /// Chunk cell for this junction / depot; edges inject flow at anchored endpoints.
    pub anchor: Option<ChunkCellKey>,
}

#[derive(Clone, Debug)]
pub struct LogisticsEdge {
    pub from: LogisticsNodeId,
    pub to: LogisticsNodeId,
    pub capacity: f32,
    pub disruption: f32,
    pub traversal_cost: f32,
}

/// Sparse network: capacity, degradation, and routing live here — not in Voronoi polygons.
#[derive(Resource, Clone, Debug, Default)]
pub struct LogisticsGraph {
    pub nodes: Vec<LogisticsNode>,
    pub edges: Vec<LogisticsEdge>,
}
