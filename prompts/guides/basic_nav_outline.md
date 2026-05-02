NAV INTEGRATION (A* OVER LANE GRAPH + FIELD COSTS)
🧭 PRIMARY DIRECTIVE

You are implementing a deterministic, lane-level pathfinding system that:

operates on LaneEdge graph
uses field state (congestion, danger, damage) as cost
respects junction lane connectivity
integrates with reservation system
supports dynamic re-routing
🧱 1. NAV GRAPH (LANE-LEVEL)
Required adjacency (build once or on change)
// nav_graph.rs
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct LaneNavGraph {
    pub neighbors: HashMap<Entity, Vec<Entity>>, // edge -> next edges
}
Build from junction topology
pub fn build_nav_graph(
    mut graph: ResMut<LaneNavGraph>,
    junctions: Query<&JunctionTopology>,
) {
    graph.neighbors.clear();

    for topo in &junctions {
        for conn in &topo.connections {
            let from_edge = conn.from_edge;
            let to_edge = conn.to_edge;

            graph
                .neighbors
                .entry(from_edge)
                .or_default()
                .push(to_edge);
        }
    }
}
💰 2. COST FUNCTION (FIELD-DRIVEN)
THIS is the heart of hybrid behavior
// cost.rs
pub fn edge_cost(e: &EdgeFieldState) -> f32 {
    let base = e.travel_time;

    base
        * (1.0 + e.congestion * 3.0)
        * (1.0 + e.damage * 5.0)
        * (1.0 + e.danger * 10.0)
}
Optional: agent-specific bias
pub fn edge_cost_for_agent(
    e: &EdgeFieldState,
    cautious: f32,
) -> f32 {
    e.travel_time
        * (1.0 + e.congestion * 3.0)
        * (1.0 + e.damage * 5.0)
        * (1.0 + e.danger * 10.0 * cautious)
}
🧠 3. A* IMPLEMENTATION (LANE GRAPH)
// astar.rs
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Copy, Clone)]
struct Node {
    cost: f32,
    edge: Entity,
}

impl Eq for Node {}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.cost == other.cost
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.partial_cmp(&self.cost).unwrap()
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
Core search
pub fn astar_lane(
    start: Entity,
    goal: Entity,
    graph: &LaneNavGraph,
    field: &Query<&EdgeFieldState>,
) -> Option<Vec<Entity>> {
    let mut open = BinaryHeap::new();
    let mut came_from: HashMap<Entity, Entity> = HashMap::new();
    let mut g_score: HashMap<Entity, f32> = HashMap::new();

    g_score.insert(start, 0.0);

    open.push(Node {
        cost: 0.0,
        edge: start,
    });

    while let Some(Node { edge, .. }) = open.pop() {
        if edge == goal {
            // reconstruct
            let mut path = vec![edge];
            let mut current = edge;

            while let Some(prev) = came_from.get(&current) {
                path.push(*prev);
                current = *prev;
            }

            path.reverse();
            return Some(path);
        }

        let neighbors = graph.neighbors.get(&edge).unwrap_or(&vec![]);

        for next in neighbors {
            let e = field.get(*next).unwrap();

            let tentative = g_score[&edge] + edge_cost(e);

            if tentative < *g_score.get(next).unwrap_or(&f32::INFINITY) {
                came_from.insert(*next, edge);
                g_score.insert(*next, tentative);

                open.push(Node {
                    cost: tentative,
                    edge: *next,
                });
            }
        }
    }

    None
}
🔄 4. DYNAMIC RE-ROUTING
Trigger when conditions change
#[derive(Component)]
pub struct NeedsRepath;
pub fn trigger_repath(
    mut commands: Commands,
    vehicles: Query<(Entity, &Vehicle)>,
    edges: Query<&EdgeFieldState>,
) {
    for (e, v) in &vehicles {
        let current_edge = v.path[v.current_edge];

        let field = edges.get(current_edge).unwrap();

        if field.danger > 0.5 || field.congestion > 0.8 {
            commands.entity(e).insert(NeedsRepath);
        }
    }
}
Recompute path
pub fn repath_system(
    mut commands: Commands,
    mut vehicles: Query<(Entity, &mut Vehicle), With<NeedsRepath>>,
    graph: Res<LaneNavGraph>,
    field: Query<&EdgeFieldState>,
) {
    for (e, mut v) in &mut vehicles {
        if let Some(new_path) = astar_lane(
            v.path[v.current_edge],
            *v.path.last().unwrap(),
            &graph,
            &field,
        ) {
            v.path = new_path;
            v.current_edge = 0;
        }

        commands.entity(e).remove::<NeedsRepath>();
    }
}
🔐 5. RESERVATION INTEGRATION
Cost penalty for reserved edges
pub fn edge_cost_with_reservation(
    e: &EdgeFieldState,
    reservation: &ReservationTable,
) -> f32 {
    let mut cost = edge_cost(e);

    if !reservation.slots.is_empty() {
        cost *= 2.0; // discourage
    }

    cost
}
🧪 6. DEBUG PATHS
pub fn debug_paths(
    mut gizmos: Gizmos,
    vehicles: Query<&Vehicle>,
    edges: Query<&LaneEdge>,
) {
    for v in &vehicles {
        for edge_e in &v.path {
            if let Ok(edge) = edges.get(*edge_e) {
                gizmos.line_2d(
                    edge_start(edge),
                    edge_end(edge),
                    Color::BLUE,
                );
            }
        }
    }
}
⚠️ CRITICAL DESIGN RULES
- pathfinding operates ONLY on lane graph (never raw terrain)
- all costs come from field state (single source of truth)
- agents never “ignore” congestion/danger
- re-routing must be limited (avoid thrashing)
🧠 OPTIONAL (BUT STRONGLY RECOMMENDED)
Hierarchical pathfinding (HPA*)
chunk graph → lane graph
Flow-aware heuristic
h(n) = distance / avg_speed * congestion_bias
Path caching
(Hash(start, goal) → path)
invalidate on major field change
✅ COMPLETION CRITERIA
✔ A* finds valid lane paths
✔ respects junction topology
✔ avoids congested/damaged/danger edges
✔ vehicles re-route dynamically
✔ integrates with reservation system
✔ debug visualization works
🧩 WHAT YOU NOW HAVE (FULL STACK)

You’ve built:

spline editor ✔
junction system ✔
lane graph ✔
hybrid traffic model ✔
reservation system ✔
mesh generation ✔
nav system ✔