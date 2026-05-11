//! **Unified spatial network layer** — one graph vocabulary for roads, power, fluids, data, logistics, and supply.
//! Depth (`LayerType`) is a field on nodes, not a second map. See module docs on [`InfrastructureGraph`](super::InfrastructureGraph)
//! for the transport-backed mirror; this module adds typed masks, flow stubs, and per-chunk digests for U7 coupling.
//!
//! **Authoritative ECS enum:** [`NetworkType`]. [`InfrastructureNetworkType`](super::infrastructure_graph::InfrastructureNetworkType)
//! exists only at bake / import / legacy UI boundaries; map **→** `NetworkType` at system edges — never the reverse.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

use bevy::prelude::*;

use super::infrastructure_graph::{InfrastructureGraph, InfrastructureNetworkType};
use super::transport_bridge::StrategicRasterConfig;

// -----------------------------------------------------------------------------
// Network typing (edges + attachments)
// -----------------------------------------------------------------------------

/// Primary network families — every “road / pipe / grid” system maps here.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NetworkType {
    Road = 0,
    Power = 1,
    Fluid = 2,
    Data = 3,
    Logistics = 4,
    MilitarySupply = 5,
}

impl NetworkType {
    pub const COUNT: usize = 6;

    #[inline]
    pub const fn idx(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn mask(self) -> NetworkAttachmentMask {
        match self {
            NetworkType::Road => NetworkAttachmentMask::ROAD,
            NetworkType::Power => NetworkAttachmentMask::POWER,
            NetworkType::Fluid => NetworkAttachmentMask::FLUID,
            NetworkType::Data => NetworkAttachmentMask::DATA,
            NetworkType::Logistics => NetworkAttachmentMask::LOGISTICS,
            NetworkType::MilitarySupply => NetworkAttachmentMask::MILITARY_SUPPLY,
        }
    }

    /// Diffusion / decay knobs for the chunk-local flow solver ([`super::network_flow::network_flow_chunk_local_solver_system`]).
    pub fn flow_rules(self) -> NetworkFlowRules {
        match self {
            NetworkType::Road => NetworkFlowRules {
                diffusion_rate: 0.35,
                decay: 0.04,
                capacity_limit: 1.0,
                layer_penalty: 0.2,
            },
            NetworkType::Power => NetworkFlowRules {
                diffusion_rate: 0.28,
                decay: 0.06,
                capacity_limit: 1.0,
                layer_penalty: 0.35,
            },
            NetworkType::Fluid => NetworkFlowRules {
                diffusion_rate: 0.22,
                decay: 0.05,
                capacity_limit: 1.0,
                layer_penalty: 0.4,
            },
            NetworkType::Data => NetworkFlowRules {
                diffusion_rate: 0.3,
                decay: 0.07,
                capacity_limit: 1.0,
                layer_penalty: 0.15,
            },
            NetworkType::Logistics => NetworkFlowRules {
                diffusion_rate: 0.32,
                decay: 0.045,
                capacity_limit: 1.0,
                layer_penalty: 0.25,
            },
            NetworkType::MilitarySupply => NetworkFlowRules {
                diffusion_rate: 0.3,
                decay: 0.055,
                capacity_limit: 1.0,
                layer_penalty: 0.28,
            },
        }
    }
}

impl From<InfrastructureNetworkType> for NetworkType {
    fn from(v: InfrastructureNetworkType) -> Self {
        match v {
            InfrastructureNetworkType::Roads => NetworkType::Road,
            InfrastructureNetworkType::Rail => NetworkType::Logistics,
            InfrastructureNetworkType::Power => NetworkType::Power,
            InfrastructureNetworkType::Pipelines => NetworkType::Fluid,
            InfrastructureNetworkType::Communications => NetworkType::Data,
        }
    }
}

bitflags::bitflags! {
    /// Which networks a **node** may attach to / consume / emit on.
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
    pub struct NetworkAttachmentMask: u8 {
        const ROAD = 1 << 0;
        const POWER = 1 << 1;
        const FLUID = 1 << 2;
        const DATA = 1 << 3;
        const LOGISTICS = 1 << 4;
        const MILITARY_SUPPLY = 1 << 5;
    }
}

impl NetworkAttachmentMask {
    #[inline]
    pub fn from_network_type(n: NetworkType) -> Self {
        n.mask()
    }
}

/// Flow kernel parameters — solvers write **only** [`ChunkStrategicOverlay`](super::ChunkStrategicOverlay) SOA fields.
#[derive(Clone, Copy, Debug)]
pub struct NetworkFlowRules {
    pub diffusion_rate: f32,
    pub decay: f32,
    pub capacity_limit: f32,
    pub layer_penalty: f32,
}

// -----------------------------------------------------------------------------
// Depth / layer (single world — no underground map asset)
// -----------------------------------------------------------------------------

/// Vertical stratification: **y** in [`SpatialNode`] / [`SpatialNetworkPosition`] is depth index (see docs on spawn).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LayerType {
    #[default]
    Surface,
    Subsurface,
    DeepSubsurface,
}

impl LayerType {
    pub const COUNT: usize = 3;

    #[inline]
    pub fn idx(self) -> usize {
        match self {
            LayerType::Surface => 0,
            LayerType::Subsurface => 1,
            LayerType::DeepSubsurface => 2,
        }
    }

    /// Stubs for pathfinding / visibility: cost to enter `to` when standing on `from` (1.0 = neutral).
    #[inline]
    pub fn transition_cost(from: LayerType, to: LayerType) -> f32 {
        match (from, to) {
            (a, b) if a == b => 1.0,
            (LayerType::Surface, LayerType::Subsurface) => 1.25,
            (LayerType::Surface, LayerType::DeepSubsurface) => 1.6,
            (LayerType::Subsurface, LayerType::Surface) => 1.15,
            (LayerType::Subsurface, LayerType::DeepSubsurface) => 1.2,
            (LayerType::DeepSubsurface, LayerType::Subsurface) => 1.1,
            (LayerType::DeepSubsurface, LayerType::Surface) => 1.5,
            _ => 1.0,
        }
    }

    /// Visibility multiplier for sensors: lower ⇒ harder to detect (bunkers / deep works).
    #[inline]
    pub fn visibility_factor(self) -> f32 {
        match self {
            LayerType::Surface => 1.0,
            LayerType::Subsurface => 0.55,
            LayerType::DeepSubsurface => 0.25,
        }
    }
}

/// Tile column (**x**), depth band (**y**), row (**z**) — aligns with map editor x / z; **y** is logical depth, not render height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct SpatialNetworkPosition {
    pub tile: IVec3,
    pub layer: LayerType,
}

// -----------------------------------------------------------------------------
// ECS — node roles (bunker / trench = same network formalism)
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NetworkPrimitiveKind {
    #[default]
    Generic,
    Bunker,
    Trench,
    CityCore,
    Factory,
    SensorGrid,
}

/// Placeholder component: attach to any entity that sits in the unified spatial network.
#[derive(Component, Clone, Copy, Debug)]
pub struct SpatialNode {
    pub tile: IVec3,
    pub layer: LayerType,
    pub kind: NetworkPrimitiveKind,
}

/// What networks this **node** participates in vs what it **requires** to operate.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct NetworkMembership {
    pub attached: NetworkAttachmentMask,
    pub required: NetworkAttachmentMask,
}

/// Bunker / hardened node — fights visibility + threat fields; pairs with [`NetworkFlowRules::layer_penalty`].
#[derive(Component, Clone, Copy, Debug)]
pub struct NetworkInsulatedNode {
    pub insulation_strength: f32,
    pub layer: LayerType,
    /// Bitmask for exposed interfaces (future: which networks can couple despite insulation).
    pub exposure_mask: u8,
}

// -----------------------------------------------------------------------------
// Logical edge (CPU graph; transport mirror fills from bake)
// -----------------------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct SpatialNetworkEdge {
    pub network: NetworkType,
    pub from_node: u64,
    pub to_node: u64,
    pub capacity: f32,
    pub resistance: f32,
    pub layer_from: LayerType,
    pub layer_to: LayerType,
}

/// Authoritative sparse graph for planners (optional; filled from infrastructure + manual spawns later).
#[derive(Resource, Clone, Debug, Default)]
pub struct SpatialNetworkGraph {
    pub next_node_id: u64,
    pub edges: Vec<SpatialNetworkEdge>,
}

// -----------------------------------------------------------------------------
// Chunk digest — density / stress hooks for U7 + GPU (Step 5)
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, Default)]
pub struct ChunkNetworkSummary {
    pub edge_crossings_by_type: [u32; NetworkType::COUNT],
    pub node_count_by_layer: [u32; LayerType::COUNT],
    pub insulated_node_hits: u32,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct ChunkNetworkDigest {
    pub epoch: u64,
    pub road_signature: u64,
    pub power_signature: u64,
    pub pipe_signature: u64,
    pub connectivity_hash: u64,
    pub flow_hash: u64,
    pub by_chunk: HashMap<IVec2, ChunkNetworkSummary>,
}

fn hash_infra_edges_filtered(
    infra: &InfrastructureGraph,
    include: impl Fn(NetworkType) -> bool,
) -> u64 {
    let mut h = DefaultHasher::new();
    for e in &infra.edges {
        let Some(na) = infra.nodes.iter().find(|n| n.id == e.from) else {
            continue;
        };
        let nt = NetworkType::from(na.network);
        if include(nt) {
            e.from.hash(&mut h);
            e.to.hash(&mut h);
            e.throughput.to_bits().hash(&mut h);
            e.integrity.to_bits().hash(&mut h);
        }
    }
    h.finish()
}

fn hash_connectivity(infra: &InfrastructureGraph) -> u64 {
    let mut pairs: Vec<(u64, u64, u8)> = Vec::new();
    for e in &infra.edges {
        let Some(na) = infra.nodes.iter().find(|n| n.id == e.from) else {
            continue;
        };
        let nt = NetworkType::from(na.network) as u8;
        pairs.push((e.from, e.to, nt));
    }
    pairs.sort();
    let mut h = DefaultHasher::new();
    for p in pairs {
        p.hash(&mut h);
    }
    h.finish()
}

fn hash_flow_graph(graph: &SpatialNetworkGraph) -> u64 {
    let mut h = DefaultHasher::new();
    for e in &graph.edges {
        e.from_node.hash(&mut h);
        e.to_node.hash(&mut h);
        e.network.hash(&mut h);
        e.capacity.to_bits().hash(&mut h);
        e.resistance.to_bits().hash(&mut h);
    }
    h.finish()
}

fn chunk_from_xy(position_xy: Vec2, cells: UVec2) -> IVec2 {
    let sx = cells.x.max(1);
    let sy = cells.y.max(1);
    let tx = position_xy.x.floor().max(0.0) as u32;
    let tz = position_xy.y.floor().max(0.0) as u32;
    IVec2::new((tx / sx) as i32, (tz / sy) as i32)
}

/// Rebuild digest from mirrored [`InfrastructureGraph`] (surface-oriented until subsurface nodes register).
pub fn rebuild_chunk_network_digest_system(
    infra: Res<InfrastructureGraph>,
    raster: Res<StrategicRasterConfig>,
    insulated_spatial: Query<(&NetworkInsulatedNode, &SpatialNode)>,
    spatial_nodes: Query<&SpatialNode>,
    mut digest: ResMut<ChunkNetworkDigest>,
    mut graph: ResMut<SpatialNetworkGraph>,
) {
    let cells = raster.cells_per_chunk.max(UVec2::ONE);
    let mut by_chunk: HashMap<IVec2, ChunkNetworkSummary> = HashMap::new();

    let bump_edge = |by: &mut HashMap<IVec2, ChunkNetworkSummary>, ck: IVec2, net: NetworkType| {
        let e = by.entry(ck).or_default();
        e.edge_crossings_by_type[net.idx()] = e.edge_crossings_by_type[net.idx()].saturating_add(1);
    };

    for e in &infra.edges {
        let from_n = infra.nodes.iter().find(|n| n.id == e.from);
        let to_n = infra.nodes.iter().find(|n| n.id == e.to);
        let (Some(a), Some(b)) = (from_n, to_n) else {
            continue;
        };
        let net = NetworkType::from(a.network);
        let ck_a = chunk_from_xy(a.position, cells);
        let ck_b = chunk_from_xy(b.position, cells);
        bump_edge(&mut by_chunk, ck_a, net);
        if ck_a != ck_b {
            bump_edge(&mut by_chunk, ck_b, net);
        }
    }

    for n in &infra.nodes {
        let ck = chunk_from_xy(n.position, cells);
        let entry = by_chunk.entry(ck).or_default();
        entry.node_count_by_layer[LayerType::Surface.idx()] =
            entry.node_count_by_layer[LayerType::Surface.idx()].saturating_add(1);
    }

    for sn in spatial_nodes.iter() {
        let ck = IVec2::new(sn.tile.x, sn.tile.z);
        let e = by_chunk.entry(ck).or_default();
        let li = sn.layer.idx();
        if li < LayerType::COUNT {
            e.node_count_by_layer[li] = e.node_count_by_layer[li].saturating_add(1);
        }
    }

    for (_, sn) in insulated_spatial.iter() {
        let ck = IVec2::new(sn.tile.x, sn.tile.z);
        let e = by_chunk.entry(ck).or_default();
        e.insulated_node_hits = e.insulated_node_hits.saturating_add(1);
    }

    digest.by_chunk = by_chunk;
    digest.road_signature = hash_infra_edges_filtered(&infra, |t| t == NetworkType::Road);
    digest.power_signature = hash_infra_edges_filtered(&infra, |t| t == NetworkType::Power);
    digest.pipe_signature = hash_infra_edges_filtered(&infra, |t| t == NetworkType::Fluid);
    digest.connectivity_hash = hash_connectivity(&infra);
    digest.epoch = digest.epoch.wrapping_add(1);

    graph.edges.clear();
    for e in &infra.edges {
        let from_n = infra.nodes.iter().find(|n| n.id == e.from);
        let to_n = infra.nodes.iter().find(|n| n.id == e.to);
        let (Some(a), Some(b)) = (from_n, to_n) else {
            continue;
        };
        graph.edges.push(SpatialNetworkEdge {
            network: NetworkType::from(a.network),
            from_node: e.from,
            to_node: e.to,
            capacity: e.throughput,
            resistance: 1.0 - e.integrity.clamp(0.0, 1.0),
            layer_from: LayerType::Surface,
            layer_to: LayerType::Surface,
        });
        let _ = b;
    }
    digest.flow_hash = hash_flow_graph(&graph);
}

pub struct SpatialNetworkPlugin;

impl Plugin for SpatialNetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkNetworkDigest>()
            .init_resource::<SpatialNetworkGraph>()
            .add_systems(
                Update,
                rebuild_chunk_network_digest_system
                    .after(super::infrastructure_graph::sync_infrastructure_graph_from_logistics),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_mask_roundtrip() {
        let m = NetworkType::Power.mask() | NetworkType::Logistics.mask();
        assert!(m.contains(NetworkType::Power.mask()));
        assert!(!m.contains(NetworkType::Fluid.mask()));
    }

    #[test]
    fn layer_visibility_monotone_deeper() {
        assert!(LayerType::Surface.visibility_factor() > LayerType::Subsurface.visibility_factor());
        assert!(LayerType::Subsurface.visibility_factor() > LayerType::DeepSubsurface.visibility_factor());
    }

    #[test]
    fn infrastructure_to_network_mapping() {
        assert_eq!(
            NetworkType::from(InfrastructureNetworkType::Pipelines),
            NetworkType::Fluid
        );
        assert_eq!(
            NetworkType::from(InfrastructureNetworkType::Rail),
            NetworkType::Logistics
        );
    }
}
