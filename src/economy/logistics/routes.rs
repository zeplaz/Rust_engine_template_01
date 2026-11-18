//! Transport nav reachability for facility pairs (LOG-A-04 / A-05).
//!
//! CQ-53: production paths use a single costed Dijkstra (base traversal × NAV-13 forecast),
//! not unweighted BFS.

use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};

use bevy::prelude::*;

use crate::construction::ConstructionWorldRevision;
use crate::economy::resource_flow::TransportMode;
use crate::strategic::{BuildSiteTile, LogisticsGraph, PlannedSite};
use crate::systems::navigation::LogisticsCostForecastBook;
use crate::systems::transport::{
    TransportCostCache, TransportEdgeDirectory, TransportEdgeId, TransportNavExport,
};

use super::types::{
    CachedRoute, FacilityPortal, LogisticsRouteDirtyTiles, RouteCache, RouteHandle, RoutePathStore,
};

#[must_use]
pub fn tile_node_key(tile: BuildSiteTile) -> String {
    format!("t{}_{}", tile.x, tile.z)
}

#[must_use]
pub fn topology_revision_u32(graph_revision: u64, construction_rev: u64) -> u32 {
    (graph_revision as u32) ^ (construction_rev as u32).rotate_left(7)
}

fn agent_for_mode(mode: TransportMode) -> &'static str {
    match mode {
        TransportMode::Truck => "road_vehicle",
        TransportMode::Rail => "rail_train",
        TransportMode::Pipeline | TransportMode::Conveyor => "road_vehicle",
    }
}

fn edges_at_tile(directory: &TransportEdgeDirectory, key: &str) -> Vec<TransportEdgeId> {
    directory
        .by_edge
        .iter()
        .filter(|(_, m)| m.head_key == key || m.tail_key == key)
        .map(|(id, _)| *id)
        .collect()
}

fn build_edge_adjacency(
    nav: &TransportNavExport,
    _directory: &TransportEdgeDirectory,
) -> HashMap<TransportEdgeId, Vec<TransportEdgeId>> {
    let mut adj: HashMap<TransportEdgeId, Vec<TransportEdgeId>> = HashMap::new();
    let mut preds: HashMap<TransportEdgeId, Vec<TransportEdgeId>> = HashMap::new();

    for e in &nav.edges {
        for &s in &e.successors {
            preds.entry(s).or_default().push(e.id);
            adj.entry(e.id).or_default().push(s);
        }
    }

    for e in &nav.edges {
        for &p in preds.get(&e.id).into_iter().flatten() {
            adj.entry(e.id).or_default().push(p);
        }
    }

    adj
}

#[derive(Clone, Copy)]
struct EdgeDijkstraState {
    cost: f32,
    edge: TransportEdgeId,
}

impl PartialEq for EdgeDijkstraState {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost && self.edge == other.edge
    }
}
impl Eq for EdgeDijkstraState {}
impl PartialOrd for EdgeDijkstraState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for EdgeDijkstraState {
    fn cmp(&self, other: &Self) -> Ordering {
        // Min-heap via reverse float order.
        other
            .cost
            .partial_cmp(&self.cost)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.edge.0.cmp(&other.edge.0))
    }
}

/// CQ-53 single route-cost search on the transport edge graph.
fn dijkstra_path_to_goals(
    adj: &HashMap<TransportEdgeId, Vec<TransportEdgeId>>,
    starts: &[TransportEdgeId],
    goals: &HashSet<TransportEdgeId>,
    mut edge_cost: impl FnMut(TransportEdgeId) -> f32,
) -> Option<Vec<TransportEdgeId>> {
    if starts.is_empty() || goals.is_empty() {
        return None;
    }
    let mut best: HashMap<TransportEdgeId, f32> = HashMap::new();
    let mut parent: HashMap<TransportEdgeId, TransportEdgeId> = HashMap::new();
    let mut heap = BinaryHeap::new();
    for &s in starts {
        let c = edge_cost(s).max(1e-3);
        best.insert(s, c);
        heap.push(EdgeDijkstraState { cost: c, edge: s });
    }
    let mut found: Option<TransportEdgeId> = None;
    while let Some(EdgeDijkstraState { cost, edge: cur }) = heap.pop() {
        if best.get(&cur).is_some_and(|&b| cost > b + 1e-4) {
            continue;
        }
        if goals.contains(&cur) {
            found = Some(cur);
            break;
        }
        for &next in adj.get(&cur).into_iter().flatten() {
            let step = edge_cost(next).max(1e-3);
            let nd = cost + step;
            if best.get(&next).is_none_or(|&b| nd + 1e-4 < b) {
                best.insert(next, nd);
                parent.insert(next, cur);
                heap.push(EdgeDijkstraState {
                    cost: nd,
                    edge: next,
                });
            }
        }
    }
    let goal = found?;
    let mut path = vec![goal];
    let mut cur = goal;
    while let Some(&p) = parent.get(&cur) {
        path.insert(0, p);
        cur = p;
        if starts.contains(&cur) {
            break;
        }
    }
    Some(path)
}

/// Base edge cost from the transport cost cache (1.0 when missing).
#[must_use]
pub fn base_edge_route_cost(cache: Option<&TransportCostCache>, id: TransportEdgeId) -> f32 {
    cache
        .and_then(|c| c.by_edge.get(&id).copied())
        .filter(|c| c.is_finite() && *c > 0.0)
        .unwrap_or(1.0)
}

/// CQ-51/52 soft penalty from nav field (heat + wildlife) at edge endpoints.
#[must_use]
pub fn nav_field_edge_penalty(
    nav: Option<&crate::compute::NavFieldFrame>,
    directory: Option<&TransportEdgeDirectory>,
    id: TransportEdgeId,
) -> f32 {
    let Some(nav) = nav else {
        return 0.0;
    };
    let mut penalty = (nav.mean_heat_cost * 0.25).clamp(0.0, 0.5);
    let Some(meta) = directory.and_then(|d| d.by_edge.get(&id)) else {
        return penalty;
    };
    for key in [&meta.head_key, &meta.tail_key] {
        let Some(coord) = parse_tile_xy(key) else {
            continue;
        };
        for s in &nav.wildlife_soft_samples {
            if s.chunk == coord {
                penalty = penalty.max(s.soft_cost);
            }
        }
        for s in &nav.heat_cost_samples {
            if s.chunk == coord {
                penalty = penalty.max(s.heat_cost);
            }
        }
    }
    penalty.clamp(0.0, 1.0)
}

fn parse_tile_xy(key: &str) -> Option<bevy::math::IVec2> {
    let rest = key.strip_prefix('t')?;
    let (xs, zs) = rest.split_once('_')?;
    Some(bevy::math::IVec2::new(xs.parse().ok()?, zs.parse().ok()?))
}

/// CQ-53/51/52: base × NAV-13 forecast × (1 + heat/wildlife soft).
#[must_use]
pub fn route_edge_cost(
    cache: Option<&TransportCostCache>,
    forecast: Option<&LogisticsCostForecastBook>,
    depart_tick: u64,
    id: TransportEdgeId,
) -> f32 {
    route_edge_cost_with_nav(cache, forecast, None, None, None, depart_tick, id)
}

/// Hard block when head or tail tile is in the flood book (DF-9).
#[must_use]
pub fn flood_blocks_route_edge(
    flood: Option<&crate::systems::water::FloodRouteBlockBook>,
    directory: Option<&TransportEdgeDirectory>,
    id: TransportEdgeId,
) -> bool {
    let Some(flood) = flood else {
        return false;
    };
    let Some(meta) = directory.and_then(|d| d.by_edge.get(&id)) else {
        return false;
    };
    flood.blocks_key(&meta.head_key) || flood.blocks_key(&meta.tail_key)
}

#[must_use]
pub fn route_edge_cost_with_nav(
    cache: Option<&TransportCostCache>,
    forecast: Option<&LogisticsCostForecastBook>,
    nav: Option<&crate::compute::NavFieldFrame>,
    directory: Option<&TransportEdgeDirectory>,
    flood: Option<&crate::systems::water::FloodRouteBlockBook>,
    depart_tick: u64,
    id: TransportEdgeId,
) -> f32 {
    if flood_blocks_route_edge(flood, directory, id) {
        return f32::INFINITY;
    }
    let base = base_edge_route_cost(cache, id);
    let timed = match forecast.filter(|f| f.ready) {
        Some(book) => book.edge_cost(base, depart_tick, 1),
        None => base,
    };
    timed * (1.0 + nav_field_edge_penalty(nav, directory, id))
}

#[must_use]
pub fn path_edges_between_tiles(
    nav: &TransportNavExport,
    directory: &TransportEdgeDirectory,
    from: BuildSiteTile,
    to: BuildSiteTile,
    mode: TransportMode,
) -> Option<Vec<TransportEdgeId>> {
    path_edges_between_tiles_costed(nav, directory, from, to, mode, None, None, None, None, 0)
}

/// Costed production path (CQ-53): Dijkstra with [`route_edge_cost_with_nav`].
#[must_use]
pub fn path_edges_between_tiles_costed(
    nav: &TransportNavExport,
    directory: &TransportEdgeDirectory,
    from: BuildSiteTile,
    to: BuildSiteTile,
    mode: TransportMode,
    cache: Option<&TransportCostCache>,
    forecast: Option<&LogisticsCostForecastBook>,
    nav_field: Option<&crate::compute::NavFieldFrame>,
    flood: Option<&crate::systems::water::FloodRouteBlockBook>,
    depart_tick: u64,
) -> Option<Vec<TransportEdgeId>> {
    if from == to {
        return Some(Vec::new());
    }
    let agent = agent_for_mode(mode);
    let from_key = tile_node_key(from);
    let to_key = tile_node_key(to);
    let starts = edges_at_tile(directory, &from_key);
    let goal_edges: HashSet<_> = edges_at_tile(directory, &to_key).into_iter().collect();
    if starts.is_empty() || goal_edges.is_empty() {
        return None;
    }

    let filtered_nav = TransportNavExport {
        edges: nav
            .edges
            .iter()
            .filter(|e| {
                e.allowed_agents.is_empty()
                    || e.allowed_agents.iter().any(|a| a == agent)
                    || (mode != TransportMode::Rail && e.profile.contains("road"))
            })
            .cloned()
            .collect(),
    };
    let adj = build_edge_adjacency(&filtered_nav, directory);
    let path = dijkstra_path_to_goals(&adj, &starts, &goal_edges, |id| {
        route_edge_cost_with_nav(
            cache,
            forecast,
            nav_field,
            Some(directory),
            flood,
            depart_tick,
            id,
        )
    })?;
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
}

/// Sum of stored traversal costs for one already-chosen edge slice.
/// Empty, missing, or non-finite costs return `None`. Does not search.
#[must_use]
pub fn sum_path_traversal_cost(
    edge_ids: &[TransportEdgeId],
    mut cost_of: impl FnMut(TransportEdgeId) -> Option<f32>,
) -> Option<f32> {
    if edge_ids.is_empty() {
        return None;
    }
    let mut sum = 0.0_f32;
    for &id in edge_ids {
        let cost = cost_of(id)?;
        if !cost.is_finite() {
            return None;
        }
        sum += cost;
    }
    if sum.is_finite() { Some(sum) } else { None }
}

#[must_use]
pub fn path_open_between_entities(
    nav: Res<TransportNavExport>,
    directory: Res<TransportEdgeDirectory>,
    from_portal: &FacilityPortal,
    to_portal: &FacilityPortal,
    from_tile: BuildSiteTile,
    to_tile: BuildSiteTile,
    mode: TransportMode,
) -> bool {
    let _ = (from_portal, to_portal);
    path_edges_between_tiles(&nav, &directory, from_tile, to_tile, mode).is_some()
}

fn nav_and_directory_without_closed_edges(
    graph: Option<&LogisticsGraph>,
    nav: &TransportNavExport,
    directory: &TransportEdgeDirectory,
) -> (TransportNavExport, TransportEdgeDirectory) {
    let mut closed = HashSet::new();
    if let Some(graph) = graph {
        for edge in &graph.edges {
            if edge.disruption >= 1.0 {
                if let Some(id) = edge.transport_edge {
                    closed.insert(id);
                }
            }
        }
    }
    let mut directory_open = TransportEdgeDirectory::default();
    for (id, meta) in &directory.by_edge {
        if !closed.contains(id) {
            directory_open.by_edge.insert(*id, meta.clone());
        }
    }
    if closed.is_empty() {
        return (nav.clone(), directory_open);
    }
    let nav_open = TransportNavExport {
        edges: nav
            .edges
            .iter()
            .filter(|edge| !closed.contains(&edge.id))
            .map(|edge| {
                let mut edge = edge.clone();
                edge.successors.retain(|id| !closed.contains(id));
                edge
            })
            .collect(),
    };
    (nav_open, directory_open)
}

fn mark_dirty_tiles_for_edge(dirty: &mut LogisticsRouteDirtyTiles, head: &str, tail: &str) {
    dirty.keys.insert(head.to_string());
    dirty.keys.insert(tail.to_string());
}

/// NAV-10 — mark tile keys for newly closed corridors and removed directory edges (per tick).
pub fn accumulate_logistics_route_dirty_tiles_system(
    graph: Option<Res<LogisticsGraph>>,
    directory: Res<TransportEdgeDirectory>,
    construction_rev: Option<Res<ConstructionWorldRevision>>,
    mut dirty: ResMut<LogisticsRouteDirtyTiles>,
    mut prior_disrupted: Local<HashSet<TransportEdgeId>>,
    mut prior_endpoints: Local<HashMap<TransportEdgeId, (String, String)>>,
    mut prior_sig: Local<u64>,
    mut prior_construction: Local<u64>,
) {
    let sig = crate::strategic::transport_directory_edge_signature(&directory);
    let cons = construction_rev.map(|r| r.revision).unwrap_or(0);
    if sig != *prior_sig || cons != *prior_construction {
        let live: HashSet<_> = directory.by_edge.keys().copied().collect();
        for (id, (head, tail)) in prior_endpoints.iter() {
            if !live.contains(id) {
                mark_dirty_tiles_for_edge(&mut dirty, head, tail);
            }
        }
        *prior_sig = sig;
        *prior_construction = cons;
    }

    let mut now_disrupted = HashSet::new();
    if let Some(graph) = graph.as_ref() {
        for edge in &graph.edges {
            let Some(tid) = edge.transport_edge else {
                continue;
            };
            if edge.disruption >= 1.0 {
                now_disrupted.insert(tid);
                if !prior_disrupted.contains(&tid) {
                    if let Some(meta) = directory.by_edge.get(&tid) {
                        mark_dirty_tiles_for_edge(&mut dirty, &meta.head_key, &meta.tail_key);
                    }
                }
            }
        }
    }
    *prior_disrupted = now_disrupted;

    prior_endpoints.clear();
    for (&id, meta) in &directory.by_edge {
        prior_endpoints.insert(
            id,
            (meta.head_key.clone(), meta.tail_key.clone()),
        );
    }
}

#[must_use]
pub fn cached_route_intersects_dirty_tiles(
    cached: &CachedRoute,
    path_store: &RoutePathStore,
    directory: &TransportEdgeDirectory,
    dirty: &HashSet<String>,
) -> bool {
    if dirty.is_empty() {
        return false;
    }
    for &eid in path_store.edge_slice(cached.path) {
        let Some(meta) = directory.by_edge.get(&eid) else {
            return true;
        };
        if dirty.contains(&meta.head_key) || dirty.contains(&meta.tail_key) {
            return true;
        }
    }
    false
}

pub fn invalidate_routes_touching_dirty_tiles(
    route_cache: &mut RouteCache,
    path_store: &RoutePathStore,
    directory: &TransportEdgeDirectory,
    dirty: &LogisticsRouteDirtyTiles,
) -> u32 {
    if dirty.keys.is_empty() {
        return 0;
    }
    let before = route_cache.routes.len() as u32;
    route_cache.routes.retain(|_, cached| {
        !cached_route_intersects_dirty_tiles(cached, path_store, directory, &dirty.keys)
    });
    before.saturating_sub(route_cache.routes.len() as u32)
}

pub fn refresh_resource_flow_routes_system(
    graph: Option<Res<LogisticsGraph>>,
    nav: Res<TransportNavExport>,
    directory: Res<TransportEdgeDirectory>,
    cost_cache: Option<Res<TransportCostCache>>,
    construction_rev: Option<Res<ConstructionWorldRevision>>,
    mut dirty_tiles: ResMut<LogisticsRouteDirtyTiles>,
    forecast: Option<Res<LogisticsCostForecastBook>>,
    nav_field: Option<Res<crate::compute::NavFieldFrame>>,
    flood: Option<Res<crate::systems::water::FloodRouteBlockBook>>,
    clock: Option<Res<crate::systems::sim_control::SimStepClock>>,
    mut route_cache: ResMut<RouteCache>,
    mut path_store: ResMut<RoutePathStore>,
    mut flow: ResMut<crate::economy::resource_flow::ResourceFlowRegistry>,
    mut diagnostics: ResMut<super::types::LogisticsDiagnostics>,
    mut runtime: Option<ResMut<super::types::LogisticsThroughputStats>>,
    portals: Query<(Entity, &FacilityPortal, &PlannedSite)>,
) {
    let graph_rev = graph.as_ref().map(|g| g.revision).unwrap_or(0);
    let topo = topology_revision_u32(graph_rev, construction_rev.map(|r| r.revision).unwrap_or(0));
    let topo_changed = route_cache.topology_revision != topo;
    if topo_changed {
        route_cache.topology_revision = topo;
    }
    let path_read = path_store.as_ref();
    let purged = invalidate_routes_touching_dirty_tiles(
        &mut route_cache,
        path_read,
        &directory,
        &dirty_tiles,
    );
    let mut forecast_purged = 0_u32;
    if let Some(book) = forecast.as_ref() {
        if book.ready && book.revision != route_cache.forecast_consumed_revision {
            forecast_purged = route_cache.routes.len() as u32;
            route_cache.routes.clear();
            route_cache.forecast_consumed_revision = book.revision;
        }
    }
    if purged > 0 || topo_changed || forecast_purged > 0 {
        if let Some(rt) = runtime.as_mut() {
            rt.saw_route_invalidation = true;
        }
    }
    for cached in route_cache.routes.values_mut() {
        if cached.handle.topology_revision != topo {
            cached.handle.topology_revision = topo;
        }
    }
    dirty_tiles.keys.clear();

    let portal_map: HashMap<Entity, (FacilityPortal, BuildSiteTile)> = portals
        .iter()
        .map(|(e, p, site)| (e, (p.clone(), site.origin)))
        .collect();

    diagnostics.routes_open = 0;
    diagnostics.routes_blocked = 0;

    let (nav_open, directory_open) =
        nav_and_directory_without_closed_edges(graph.as_deref(), &nav, &directory);

    for edge in &mut flow.edges {
        let (Some((_from_p, from_tile)), Some((_to_p, to_tile))) =
            (portal_map.get(&edge.from), portal_map.get(&edge.to))
        else {
            edge.path_open = false;
            diagnostics.routes_blocked = diagnostics.routes_blocked.saturating_add(1);
            continue;
        };

        let key = (edge.from, edge.to);
        if !route_cache.routes.contains_key(&key) {
            let depart_tick = clock.as_ref().map(|c| c.steps).unwrap_or(0);
            let path_edges = path_edges_between_tiles_costed(
                &nav_open,
                &directory_open,
                *from_tile,
                *to_tile,
                edge.transport_mode,
                cost_cache.as_deref(),
                forecast.as_deref(),
                nav_field.as_deref(),
                flood.as_deref(),
                depart_tick,
            );
            let reachable = path_edges.is_some();
            let path = path_edges
                .as_ref()
                .map(|p| path_store.insert_path(p))
                .unwrap_or(super::types::RoutePath {
                    first_edge: 0,
                    edge_count: 0,
                });
            let handle = RouteHandle {
                id: route_cache.next_id,
                topology_revision: topo,
            };
            route_cache.next_id = route_cache.next_id.saturating_add(1);
            let bottleneck = graph
                .as_ref()
                .map(|g| {
                    g.edges
                        .iter()
                        .map(|e| e.capacity * (1.0 - e.disruption.clamp(0.0, 1.0)))
                        .fold(f32::MAX, f32::min)
                })
                .unwrap_or(4.0);
            route_cache.routes.insert(
                key,
                CachedRoute {
                    handle,
                    path,
                    reachable,
                    bottleneck_capacity: if reachable { bottleneck.min(4.0) } else { 0.0 },
                },
            );
        }

        if let Some(route) = route_cache.routes.get(&key) {
            edge.path_open = route.reachable && route.handle.topology_revision == topo;
            edge.route_handle = Some(route.handle);
            if edge.path_open {
                diagnostics.routes_open = diagnostics.routes_open.saturating_add(1);
            } else {
                diagnostics.routes_blocked = diagnostics.routes_blocked.saturating_add(1);
            }
        }
    }
}

/// **INFRA-E5-002** — every `path_open` flag matches `TransportNavExport` reachability (no tile paint fallback).
#[must_use]
pub fn flow_paths_match_nav_export(
    flow: &crate::economy::resource_flow::ResourceFlowRegistry,
    nav: &TransportNavExport,
    directory: &TransportEdgeDirectory,
    entity_tiles: &std::collections::HashMap<Entity, BuildSiteTile>,
) -> bool {
    for edge in &flow.edges {
        let (Some(&from_tile), Some(&to_tile)) =
            (entity_tiles.get(&edge.from), entity_tiles.get(&edge.to))
        else {
            if edge.path_open {
                return false;
            }
            continue;
        };
        let nav_open =
            path_edges_between_tiles(nav, directory, from_tile, to_tile, edge.transport_mode)
                .is_some();
        if edge.path_open != nav_open {
            return false;
        }
    }
    true
}

#[must_use]
pub fn collect_portal_entity_tiles(
    portals: &Query<(Entity, &FacilityPortal, &PlannedSite)>,
) -> std::collections::HashMap<Entity, BuildSiteTile> {
    portals
        .iter()
        .map(|(entity, _, site)| (entity, site.origin))
        .collect()
}

#[must_use]
pub fn collect_portal_entity_tiles_from_world(
    world: &mut World,
) -> std::collections::HashMap<Entity, BuildSiteTile> {
    let mut query = world.query::<(Entity, &FacilityPortal, &PlannedSite)>();
    query
        .iter(world)
        .map(|(entity, _, site)| (entity, site.origin))
        .collect()
}

#[cfg(test)]
mod nav10_tests {
    use super::*;
    use crate::construction::ConstructionWorldRevision;
    use crate::economy::logistics::LogisticsDiagnostics;
    use crate::economy::resource_flow::{ResourceFlowEdge, ResourceFlowRegistry, TransportMode};
    use crate::strategic::{
        rebuild_logistics_graph_from_transport, CorridorConstructionBook, FootprintTiles, LayerType,
        PlannedSite, SiteArchetype, SiteId, StrategicRasterConfig,
    };
    use crate::systems::transport::{
        refresh_transport_nav_export, EdgeFieldState, TransportCostCache, TransportCostWeights,
        TransportEdgeDirectory, TransportEdgeMeta, TransportFieldStore, TransportNavExport,
        TransportTopology,
    };
    use super::super::types::TransportNodeAnchor;
    use crate::terrain::ChunkCellKey;
    use bevy::ecs::system::RunSystemOnce;

    fn road(head: &str, tail: &str) -> TransportEdgeMeta {
        TransportEdgeMeta {
            profile: "default_road".into(),
            head_key: head.into(),
            tail_key: tail.into(),
            ..Default::default()
        }
    }

    fn portal(app: &mut World, site_id: u64, tile: BuildSiteTile, key: &str) -> Entity {
        app.spawn((
            FacilityPortal {
                anchor: ChunkCellKey::new(IVec2::ZERO, 0),
                transport_anchor: TransportNodeAnchor(key.into()),
            },
            PlannedSite {
                site_id: SiteId(site_id),
                origin: tile,
                footprint: FootprintTiles {
                    width: 1,
                    depth: 1,
                },
                archetype: SiteArchetype::Factory,
                layer: LayerType::Surface,
                catalog_id: None,
                placement: None,
            },
        ))
        .id()
    }

    fn refresh_routes(app: &mut World) {
        app.run_system_once(accumulate_logistics_route_dirty_tiles_system)
            .unwrap();
        app.run_system_once(refresh_resource_flow_routes_system)
            .unwrap();
    }

    /// NAV-10 / L3 — one closed edge re-routes only the flow that used it.
    #[test]
    fn closure_replans_affected_flow_and_keeps_unrelated_route() {
        let mut app = App::new();
        app.init_resource::<TransportCostWeights>()
            .init_resource::<TransportTopology>()
            .init_resource::<TransportCostCache>()
            .init_resource::<TransportNavExport>()
            .init_resource::<StrategicRasterConfig>()
            .insert_resource(ConstructionWorldRevision { revision: 1 })
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
        for id in [TransportEdgeId(1), TransportEdgeId(4)] {
            store.by_edge.insert(
                id,
                EdgeFieldState {
                    travel_time_base: 1.0,
                    ..Default::default()
                },
            );
        }
        app.insert_resource(store);

        let graph_open = rebuild_logistics_graph_from_transport(
            app.world().resource::<TransportEdgeDirectory>(),
            app.world().resource::<TransportFieldStore>(),
            app.world().resource::<TransportCostWeights>(),
            app.world().resource::<StrategicRasterConfig>(),
            app.world().resource::<CorridorConstructionBook>(),
            1,
        );
        app.insert_resource(graph_open);

        let topo = TransportTopology::default();
        app.insert_resource(topo);
        let mut nav = TransportNavExport::default();
        let mut cost_cache = TransportCostCache::default();
        {
            let directory = app.world().resource::<TransportEdgeDirectory>();
            let field = app.world().resource::<TransportFieldStore>();
            let weights = app.world().resource::<TransportCostWeights>();
            for (id, st) in &field.by_edge {
                cost_cache.by_edge.insert(
                    *id,
                    crate::systems::transport::edge_traversal_cost(st, weights, st.travel_time_base),
                );
            }
            refresh_transport_nav_export(
                app.world().resource::<TransportTopology>(),
                &cost_cache,
                directory,
                &mut nav,
            );
        }
        app.insert_resource(cost_cache);
        app.insert_resource(nav);

        let from_a = portal(
            app.world_mut(),
            1,
            BuildSiteTile { x: 0, z: 0 },
            "t0_0",
        );
        let to_a = portal(
            app.world_mut(),
            2,
            BuildSiteTile { x: 4, z: 0 },
            "t4_0",
        );
        let from_b = portal(
            app.world_mut(),
            3,
            BuildSiteTile { x: 6, z: 0 },
            "t6_0",
        );
        let to_b = portal(
            app.world_mut(),
            4,
            BuildSiteTile { x: 7, z: 0 },
            "t7_0",
        );

        app.insert_resource(ResourceFlowRegistry {
            edges: vec![
                ResourceFlowEdge {
                    from: from_a,
                    to: to_a,
                    transport_mode: TransportMode::Truck,
                    max_rate: 1.0,
                    latency_ticks: 1.0,
                    path_open: false,
                    route_handle: None,
                    buffer_tag: Some("a".into()),
                },
                ResourceFlowEdge {
                    from: from_b,
                    to: to_b,
                    transport_mode: TransportMode::Truck,
                    max_rate: 1.0,
                    latency_ticks: 1.0,
                    path_open: false,
                    route_handle: None,
                    buffer_tag: Some("b".into()),
                },
            ],
        });
        app.init_resource::<RouteCache>();
        app.init_resource::<RoutePathStore>();
        app.init_resource::<LogisticsRouteDirtyTiles>();
        app.init_resource::<LogisticsDiagnostics>();

        refresh_routes(app.world_mut());
        let path_store = app.world().resource::<RoutePathStore>().clone();
        let cache = app.world().resource::<RouteCache>();
        let path_a_before = cache.routes.get(&(from_a, to_a)).unwrap().path;
        let path_b_before = cache.routes.get(&(from_b, to_b)).unwrap().path;
        let slice_a_before: Vec<_> = path_store.edge_slice(path_a_before).to_vec();
        let slice_b_before: Vec<_> = path_store.edge_slice(path_b_before).to_vec();
        assert_eq!(slice_a_before, vec![TransportEdgeId(1)]);
        assert_eq!(slice_b_before, vec![TransportEdgeId(4)]);

        {
            let mut directory = app.world_mut().resource_mut::<TransportEdgeDirectory>();
            directory
                .by_edge
                .insert(TransportEdgeId(2), road("t0_0", "t4_0"));
            let mut store = app.world_mut().resource_mut::<TransportFieldStore>();
            store.by_edge.insert(
                TransportEdgeId(2),
                EdgeFieldState {
                    travel_time_base: 1.0,
                    ..Default::default()
                },
            );
            let closed = store.by_edge.get_mut(&TransportEdgeId(1)).unwrap();
            closed.danger = 1.0;
            closed.congestion = 1.0;
            closed.damage = 0.2;
            closed.heat = 1.0;
        }
        {
            let weights = app.world().resource::<TransportCostWeights>().clone();
            let edge2 = app
                .world()
                .resource::<TransportFieldStore>()
                .by_edge
                .get(&TransportEdgeId(2))
                .cloned()
                .unwrap();
            app.world_mut()
                .resource_mut::<TransportCostCache>()
                .by_edge
                .insert(
                    TransportEdgeId(2),
                    crate::systems::transport::edge_traversal_cost(&edge2, &weights, 1.0),
                );
        }
        let nav = {
            let mut nav = TransportNavExport::default();
            refresh_transport_nav_export(
                app.world().resource::<TransportTopology>(),
                app.world().resource::<TransportCostCache>(),
                app.world().resource::<TransportEdgeDirectory>(),
                &mut nav,
            );
            nav
        };
        app.insert_resource(nav);
        let graph_closed = rebuild_logistics_graph_from_transport(
            app.world().resource::<TransportEdgeDirectory>(),
            app.world().resource::<TransportFieldStore>(),
            app.world().resource::<TransportCostWeights>(),
            app.world().resource::<StrategicRasterConfig>(),
            app.world().resource::<CorridorConstructionBook>(),
            2,
        );
        assert!(
            graph_closed
                .edges
                .iter()
                .any(|e| e.transport_edge == Some(TransportEdgeId(1)) && e.disruption >= 1.0)
        );
        app.insert_resource(graph_closed);
        refresh_routes(app.world_mut());

        let cache = app.world().resource::<RouteCache>();
        let path_store = app.world().resource::<RoutePathStore>();
        let path_a_after = cache.routes.get(&(from_a, to_a)).unwrap().path;
        let path_b_after = cache.routes.get(&(from_b, to_b)).unwrap().path;
        let slice_a_after: Vec<_> = path_store.edge_slice(path_a_after).to_vec();
        let slice_b_after: Vec<_> = path_store.edge_slice(path_b_after).to_vec();

        assert_eq!(slice_a_after, vec![TransportEdgeId(2)], "affected flow must re-route");
        assert_ne!(slice_a_before, slice_a_after);
        assert_eq!(
            (path_b_before.first_edge, path_b_before.edge_count),
            (path_b_after.first_edge, path_b_after.edge_count),
            "unrelated flow keeps cached path"
        );
        assert_eq!(slice_b_after, slice_b_before);
    }

    #[test]
    fn flooded_road_blocks_route_cost() {
        let mut directory = TransportEdgeDirectory::default();
        directory
            .by_edge
            .insert(TransportEdgeId(9), road("t1_0", "t2_0"));
        let mut flood = crate::systems::water::FloodRouteBlockBook::default();
        assert!(
            !flood_blocks_route_edge(Some(&flood), Some(&directory), TransportEdgeId(9)),
            "dry edge open"
        );
        flood.blocked.insert("t2_0".into());
        assert!(flood_blocks_route_edge(
            Some(&flood),
            Some(&directory),
            TransportEdgeId(9)
        ));
        let cost = route_edge_cost_with_nav(
            None,
            None,
            None,
            Some(&directory),
            Some(&flood),
            0,
            TransportEdgeId(9),
        );
        assert!(cost.is_infinite(), "flooded edge must hard-block");
    }
}

