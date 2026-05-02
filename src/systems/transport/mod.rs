//! Transport simulation spine: topology → field integrate → cost cache (logical order).
//!
//! Docs: `prompts/designer_questions/transport/rulebook_drafts.md` §0.2,
//! `transport_code_implementation_plan_v1.md`.
//!
//! This module is **not** the map-editor authoring UI; editor ghost/preview lives in [`crate::gui::editor::map_editor`].

mod bake;
mod persistence;
mod types;
mod snapshot;

pub use bake::*;
pub use persistence::*;
pub use snapshot::*;
pub use types::*;

use crate::systems::sim_control::{SimControlState, SimControlSystemSet};
use bevy::prelude::*;

/// Dependency order per `rulebook_drafts.md` §0.2 (first tranche: steps 1–4 only).
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransportSchedule {
    /// Step 1 — structure / adjacency refresh.
    Topology,
    /// Steps 2–3 — field input + integrate (here: decay only until agents exist).
    FieldIntegrate,
    /// Step 4 — read-only weights for pathfinding.
    CostCache,
}

pub struct TransportSimulationPlugin;

impl Plugin for TransportSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TransportTopology>()
            .init_resource::<TransportFieldStore>()
            .init_resource::<TransportEdgeDirectory>()
            .init_resource::<TransportCostWeights>()
            .init_resource::<TransportCostCache>()
            .init_resource::<TransportNavExport>()
            .init_resource::<TransportLastHydratedSnapshot>()
            .add_plugins(TransportNetworkPersistencePlugin)
            .configure_sets(
                Update,
                (
                    TransportSchedule::Topology.after(SimControlSystemSet::AdvanceSimTick),
                    TransportSchedule::FieldIntegrate.after(TransportSchedule::Topology),
                    TransportSchedule::CostCache.after(TransportSchedule::FieldIntegrate),
                ),
            )
            .add_systems(
                Update,
                transport_topology_tick.in_set(TransportSchedule::Topology),
            )
            .add_systems(
                Update,
                transport_field_integrate.in_set(TransportSchedule::FieldIntegrate),
            )
            .add_systems(
                Update,
                (
                    transport_cost_cache_refresh,
                    transport_nav_export_refresh,
                )
                    .chain()
                    .in_set(TransportSchedule::CostCache),
            );
    }
}

fn transport_topology_tick(_topology: Res<TransportTopology>) {
    // W1: fill from editor bake / procedural generator.
}

fn transport_field_integrate(
    time: Res<Time>,
    sim: Res<SimControlState>,
    mut store: ResMut<TransportFieldStore>,
) {
    let dt = time.delta_secs() * sim.dt_scale();
    if dt <= 0. {
        return;
    }
    for v in store.by_edge.values_mut() {
        let decay = 0.5 * dt;
        v.congestion = (v.congestion - decay).max(0.);
        v.damage = (v.damage - 0.1 * dt).max(0.);
        v.danger = (v.danger - 0.2 * dt).max(0.);
        v.heat = (v.heat - decay).max(0.);
    }
}

fn transport_cost_cache_refresh(
    field: Res<TransportFieldStore>,
    weights: Res<TransportCostWeights>,
    mut cache: ResMut<TransportCostCache>,
) {
    cache.by_edge.clear();
    cache.by_edge.reserve(field.by_edge.len());
    for (id, state) in field.by_edge.iter() {
        let base = state.travel_time_base.max(0.01);
        cache.by_edge.insert(*id, edge_traversal_cost(state, &weights, base));
    }
}

fn transport_nav_export_refresh(
    topology: Res<TransportTopology>,
    cost_cache: Res<TransportCostCache>,
    directory: Res<TransportEdgeDirectory>,
    mut export: ResMut<TransportNavExport>,
) {
    snapshot::refresh_transport_nav_export(&topology, &cost_cache, &directory, &mut export);
}
