//! Transport nav reachability for facility pairs (LOG-A-04 / A-05).

use std::collections::{HashMap, HashSet, VecDeque};

use bevy::prelude::*;

use crate::construction::ConstructionWorldRevision;
use crate::economy::resource_flow::TransportMode;
use crate::strategic::{BuildSiteTile, LogisticsGraph, PlannedSite};
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId, TransportNavExport};

use super::types::{CachedRoute, FacilityPortal, RouteCache, RouteHandle, RoutePathStore};

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

fn bfs_path_to_goals(
    adj: &HashMap<TransportEdgeId, Vec<TransportEdgeId>>,
    starts: &[TransportEdgeId],
    goals: &HashSet<TransportEdgeId>,
) -> Option<Vec<TransportEdgeId>> {
    if starts.is_empty() || goals.is_empty() {
        return None;
    }
    let mut parent: HashMap<TransportEdgeId, TransportEdgeId> = HashMap::new();
    let mut queue = VecDeque::new();
    let mut seen = HashSet::new();
    for &s in starts {
        seen.insert(s);
        queue.push_back(s);
    }
    let mut found: Option<TransportEdgeId> = None;
    while let Some(cur) = queue.pop_front() {
        if goals.contains(&cur) {
            found = Some(cur);
            break;
        }
        for &next in adj.get(&cur).into_iter().flatten() {
            if seen.insert(next) {
                parent.insert(next, cur);
                queue.push_back(next);
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

#[must_use]
pub fn path_edges_between_tiles(
    nav: &TransportNavExport,
    directory: &TransportEdgeDirectory,
    from: BuildSiteTile,
    to: BuildSiteTile,
    mode: TransportMode,
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
                    || e.profile.contains("road")
            })
            .cloned()
            .collect(),
    };
    let adj = build_edge_adjacency(&filtered_nav, directory);
    let path = bfs_path_to_goals(&adj, &starts, &goal_edges)?;
    if path.is_empty() {
        None
    } else {
        Some(path)
    }
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

pub fn refresh_resource_flow_routes_system(
    graph: Option<Res<LogisticsGraph>>,
    nav: Res<TransportNavExport>,
    directory: Res<TransportEdgeDirectory>,
    construction_rev: Option<Res<ConstructionWorldRevision>>,
    mut route_cache: ResMut<RouteCache>,
    mut path_store: ResMut<RoutePathStore>,
    mut flow: ResMut<crate::economy::resource_flow::ResourceFlowRegistry>,
    mut diagnostics: ResMut<super::types::LogisticsDiagnostics>,
    mut runtime: Option<ResMut<super::types::LogisticsThroughputRuntimeWitness>>,
    portals: Query<(Entity, &FacilityPortal, &PlannedSite)>,
) {
    let graph_rev = graph.as_ref().map(|g| g.revision).unwrap_or(0);
    let topo = topology_revision_u32(
        graph_rev,
        construction_rev.map(|r| r.revision).unwrap_or(0),
    );
    if route_cache.topology_revision != topo {
        if let Some(rt) = runtime.as_mut() {
            rt.saw_route_invalidation = true;
        }
        route_cache.topology_revision = topo;
        route_cache.routes.clear();
        route_cache.next_id = 0;
    }

    let portal_map: HashMap<Entity, (FacilityPortal, BuildSiteTile)> = portals
        .iter()
        .map(|(e, p, site)| (e, (p.clone(), site.origin)))
        .collect();

    diagnostics.routes_open = 0;
    diagnostics.routes_blocked = 0;

    for edge in &mut flow.edges {
        let (Some((_from_p, from_tile)), Some((_to_p, to_tile))) = (
            portal_map.get(&edge.from),
            portal_map.get(&edge.to),
        ) else {
            edge.path_open = false;
            diagnostics.routes_blocked = diagnostics.routes_blocked.saturating_add(1);
            continue;
        };

        let key = (edge.from, edge.to);
        if !route_cache.routes.contains_key(&key) {
            let path_edges = path_edges_between_tiles(
                &nav,
                &directory,
                *from_tile,
                *to_tile,
                edge.transport_mode,
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
                    bottleneck_capacity: if reachable {
                        bottleneck.min(4.0)
                    } else {
                        0.0
                    },
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
        let nav_open = path_edges_between_tiles(
            nav,
            directory,
            from_tile,
            to_tile,
            edge.transport_mode,
        )
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

#[must_use]
pub fn infra_e5_002_graph_only_paths_green(
    flow: &crate::economy::resource_flow::ResourceFlowRegistry,
    nav: &TransportNavExport,
    directory: &TransportEdgeDirectory,
    entity_tiles: &std::collections::HashMap<Entity, BuildSiteTile>,
    diagnostics: &super::types::LogisticsDiagnostics,
) -> bool {
    flow_paths_match_nav_export(flow, nav, directory, entity_tiles)
        && diagnostics.routes_blocked == 0
        && diagnostics.routes_open > 0
}
