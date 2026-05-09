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

mod logistics_net;
mod plugin;

pub use logistics_net::logistics_net_inject_into_overlays;
pub use plugin::StrategicFieldsPlugin;

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
        }
    }

    #[inline]
    pub fn len_cells(&self) -> usize {
        self.faction_control.len()
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
