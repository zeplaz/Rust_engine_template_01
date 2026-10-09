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

fn transport_topology_tick(
    graph: Option<Res<crate::infrastructure::transport::TransportGraph>>,
    mut topology: ResMut<TransportTopology>,
    mut field_store: ResMut<TransportFieldStore>,
    mut edge_directory: ResMut<TransportEdgeDirectory>,
) {
    let Some(graph) = graph else {
        return;
    };
    if graph.edges.is_empty() {
        return;
    }
    crate::infrastructure::transport::sync_transport_runtime_from_graph(
        graph.as_ref(),
        &mut topology,
        &mut field_store,
        &mut edge_directory,
    );
}

fn transport_field_integrate(
    time: Res<Time>,
    sim: Res<SimControlState>,
    directory: Res<TransportEdgeDirectory>,
    raster: Option<Res<crate::strategic::StrategicRasterConfig>>,
    overlays: Query<&crate::strategic::ChunkStrategicOverlay>,
    mut store: ResMut<TransportFieldStore>,
) {
    let dt = time.delta_secs() * sim.dt_scale();
    if dt <= 0. {
        return;
    }
    let per_chunk = raster
        .as_ref()
        .map(|cfg| cfg.cells_per_chunk)
        .unwrap_or(UVec2::new(32, 32))
        .max(UVec2::ONE);
    let danger_decay = 0.2 * dt;
    for (eid, v) in store.by_edge.iter_mut() {
        let decay = 0.5 * dt;
        v.congestion = (v.congestion - decay).max(0.);
        v.damage = (v.damage - 0.1 * dt).max(0.);
        v.heat = (v.heat - decay).max(0.);
        let decayed = (v.danger - danger_decay).max(0.);
        let sampled = directory
            .by_edge
            .get(eid)
            .map(|meta| {
                anchor_threat_slot0(&meta.head_key, &meta.tail_key, per_chunk, &overlays)
            })
            .unwrap_or(0.);
        v.danger = if sampled.is_finite() {
            sampled.max(decayed)
        } else {
            decayed
        };
    }
}

/// Slot 0 only. Same tile-key parse as [`crate::strategic::rebuild_logistics_graph_from_transport`].
fn anchor_threat_slot0(
    head_key: &str,
    tail_key: &str,
    per_chunk: UVec2,
    overlays: &Query<&crate::strategic::ChunkStrategicOverlay>,
) -> f32 {
    let head = threat_slot0_at_key(head_key, per_chunk, overlays);
    let tail = threat_slot0_at_key(tail_key, per_chunk, overlays);
    head.max(tail)
}

fn threat_slot0_at_key(
    key: &str,
    per_chunk: UVec2,
    overlays: &Query<&crate::strategic::ChunkStrategicOverlay>,
) -> f32 {
    let Some((tx, tz)) = parse_tile_node_key(key) else {
        return 0.;
    };
    let anchor = tile_to_chunk_key(tx, tz, per_chunk);
    for overlay in overlays.iter() {
        if overlay.chunk_coord != anchor.chunk {
            continue;
        }
        let Some(slots) = overlay.threat.get(anchor.cell_index as usize) else {
            return 0.;
        };
        let threat = slots[0];
        return if threat.is_finite() { threat } else { 0. };
    }
    0.
}

fn parse_tile_node_key(key: &str) -> Option<(u32, u32)> {
    let rest = key.strip_prefix('t')?;
    let (xs, zs) = rest.split_once('_')?;
    Some((xs.parse().ok()?, zs.parse().ok()?))
}

fn tile_to_chunk_key(tx: u32, tz: u32, cells: UVec2) -> crate::terrain::ChunkCellKey {
    let sx = cells.x.max(1);
    let sy = cells.y.max(1);
    let cx = (tx / sx) as i32;
    let cz = (tz / sy) as i32;
    let lx = tx % sx;
    let lz = tz % sy;
    let cell = lz * sx + lx;
    crate::terrain::ChunkCellKey::new(IVec2::new(cx, cz), cell)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::{
        refresh_resource_flow_routes_system, FacilityPortal, RouteCache, RoutePathStore,
        TransportNodeAnchor,
    };
    use crate::economy::resource_flow::{ResourceFlowEdge, ResourceFlowRegistry, TransportMode};
    use crate::strategic::{
        rebuild_logistics_graph_from_transport, BuildSiteTile, ChunkStrategicOverlay,
        CorridorConstructionBook, FootprintTiles, LayerType, PlannedSite, SiteArchetype, SiteId,
        StrategicRasterConfig,
    };
    use crate::systems::sim_control::SimControlState;
    use crate::terrain::ChunkCellKey;
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    fn road(head: &str, tail: &str) -> TransportEdgeMeta {
        TransportEdgeMeta {
            profile: "default_road".into(),
            head_key: head.into(),
            tail_key: tail.into(),
            ..Default::default()
        }
    }

    #[test]
    fn threat_drops_the_open_edge() {
        let mut app = App::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_secs(1));
        app.insert_resource(time)
            .init_resource::<SimControlState>()
            .init_resource::<TransportCostWeights>()
            .init_resource::<TransportTopology>()
            .init_resource::<TransportCostCache>()
            .init_resource::<TransportNavExport>()
            .insert_resource(StrategicRasterConfig::default())
            .insert_resource(CorridorConstructionBook::default());

        let mut directory = TransportEdgeDirectory::default();
        directory
            .by_edge
            .insert(TransportEdgeId(1), road("t0_0", "t4_0"));
        directory
            .by_edge
            .insert(TransportEdgeId(4), road("t6_0", "t7_0"));
        app.insert_resource(directory);

        let mut store = TransportFieldStore::default();
        store.by_edge.insert(
            TransportEdgeId(1),
            EdgeFieldState {
                congestion: 1.0,
                damage: 0.2,
                danger: 0.5,
                heat: 1.0,
                travel_time_base: 1.0,
            },
        );
        store.by_edge.insert(
            TransportEdgeId(4),
            EdgeFieldState {
                danger: 0.4,
                travel_time_base: 1.0,
                ..Default::default()
            },
        );
        app.insert_resource(store);

        let mut overlay = ChunkStrategicOverlay::new(IVec2::ZERO, UVec2::new(32, 32));
        overlay.threat[0][0] = 1.0;
        overlay.threat[6][0] = 0.0;
        overlay.threat[6][1] = 1.0;
        app.world_mut().spawn(overlay);

        app.world_mut()
            .run_system_once(transport_field_integrate)
            .unwrap();

        {
            let store = app.world().resource::<TransportFieldStore>();
            let closed = store.by_edge.get(&TransportEdgeId(1)).unwrap();
            let slot1 = store.by_edge.get(&TransportEdgeId(4)).unwrap();
            assert!(
                (closed.danger - 1.0).abs() < 1e-5,
                "danger {} congestion {}",
                closed.danger,
                closed.congestion
            );
            assert!((closed.congestion - 0.5).abs() < 1e-5);
            assert!((closed.damage - 0.1).abs() < 1e-5);
            assert!((closed.heat - 0.5).abs() < 1e-5);
            assert!((slot1.danger - 0.2).abs() < 1e-5);
        }

        {
            let mut directory = app.world_mut().resource_mut::<TransportEdgeDirectory>();
            directory
                .by_edge
                .insert(TransportEdgeId(2), road("t0_0", "t4_0"));
            directory
                .by_edge
                .insert(TransportEdgeId(9), road("t8_0", "t9_0"));
        }
        {
            let mut store = app.world_mut().resource_mut::<TransportFieldStore>();
            store.by_edge.insert(
                TransportEdgeId(2),
                EdgeFieldState {
                    travel_time_base: 1.0,
                    ..Default::default()
                },
            );
            store.by_edge.insert(
                TransportEdgeId(9),
                EdgeFieldState {
                    danger: f32::NAN,
                    travel_time_base: 1.0,
                    ..Default::default()
                },
            );
        }

        let graph = {
            let directory = app.world().resource::<TransportEdgeDirectory>();
            let fields = app.world().resource::<TransportFieldStore>();
            let weights = app.world().resource::<TransportCostWeights>();
            let cells = app.world().resource::<StrategicRasterConfig>();
            let book = app.world().resource::<CorridorConstructionBook>();
            rebuild_logistics_graph_from_transport(directory, fields, weights, cells, book, 3)
        };
        let closed_edge = graph
            .edges
            .iter()
            .find(|e| e.transport_edge == Some(TransportEdgeId(1)))
            .unwrap();
        let open_edge = graph
            .edges
            .iter()
            .find(|e| e.transport_edge == Some(TransportEdgeId(2)))
            .unwrap();
        let nan_edge = graph
            .edges
            .iter()
            .find(|e| e.transport_edge == Some(TransportEdgeId(9)))
            .unwrap();
        assert!(closed_edge.disruption >= 1.0);
        assert!(open_edge.disruption < 1.0);
        assert!(nan_edge.disruption < 1.0);
        assert_eq!(
            (closed_edge.capacity * (1.0 - closed_edge.disruption.clamp(0.0, 1.0))).max(0.0),
            0.0
        );
        let weights = app.world().resource::<TransportCostWeights>().clone();
        let closed_field = app
            .world()
            .resource::<TransportFieldStore>()
            .by_edge
            .get(&TransportEdgeId(1))
            .cloned()
            .unwrap();
        assert_eq!(
            closed_edge.traversal_cost,
            edge_traversal_cost(&closed_field, &weights, closed_field.travel_time_base)
        );

        {
            let mut topo = app.world_mut().resource_mut::<TransportTopology>();
            topo.neighbors.insert(TransportEdgeId(1), Vec::new());
            topo.neighbors.insert(TransportEdgeId(2), Vec::new());
        }
        app.world_mut()
            .run_system_once(transport_cost_cache_refresh)
            .unwrap();
        app.world_mut()
            .run_system_once(transport_nav_export_refresh)
            .unwrap();
        app.insert_resource(graph);

        let from = BuildSiteTile { x: 0, z: 0 };
        let to = BuildSiteTile { x: 4, z: 0 };
        let from_e = app
            .world_mut()
            .spawn((
                FacilityPortal {
                    anchor: ChunkCellKey::new(IVec2::ZERO, 0),
                    transport_anchor: TransportNodeAnchor("t0_0".into()),
                },
                PlannedSite {
                    site_id: SiteId(1),
                    origin: from,
                    footprint: FootprintTiles { width: 1, depth: 1 },
                    archetype: SiteArchetype::Factory,
                    layer: LayerType::Surface,
                    catalog_id: None,
                    placement: None,
                },
            ))
            .id();
        let to_e = app
            .world_mut()
            .spawn((
                FacilityPortal {
                    anchor: ChunkCellKey::new(IVec2::ZERO, 4),
                    transport_anchor: TransportNodeAnchor("t4_0".into()),
                },
                PlannedSite {
                    site_id: SiteId(2),
                    origin: to,
                    footprint: FootprintTiles { width: 1, depth: 1 },
                    archetype: SiteArchetype::Factory,
                    layer: LayerType::Surface,
                    catalog_id: None,
                    placement: None,
                },
            ))
            .id();
        app.insert_resource(ResourceFlowRegistry {
            edges: vec![ResourceFlowEdge {
                from: from_e,
                to: to_e,
                transport_mode: TransportMode::Truck,
                max_rate: 1.0,
                latency_ticks: 1.0,
                path_open: false,
                route_handle: None,
                buffer_tag: None,
            }],
        });
        app.init_resource::<RouteCache>();
        app.init_resource::<RoutePathStore>();
        app.init_resource::<crate::economy::logistics::LogisticsDiagnostics>();

        app.world_mut()
            .run_system_once(refresh_resource_flow_routes_system)
            .unwrap();

        let flow = app.world().resource::<ResourceFlowRegistry>();
        assert!(flow.edges[0].path_open);
        let cache = app.world().resource::<RouteCache>();
        let path = cache.routes.get(&(from_e, to_e)).unwrap().path;
        let slice = app.world().resource::<RoutePathStore>().edge_slice(path);
        assert_eq!(slice, &[TransportEdgeId(2)]);
    }
}
