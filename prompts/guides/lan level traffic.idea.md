— LANE-LEVEL TRAFFIC SIMULATION (FACTORIO-STYLE RESERVATION)
🧭 PRIMARY DIRECTIVE

You are implementing a deterministic lane-based reservation system.

This is NOT physics traffic. This is:

discrete
graph-based
reservation-driven (Factorio-style)
deadlock-aware
🧱 CORE MODEL (LANE GRAPH — REQUIRED FIRST)
Replace edge-level nav with lane-level graph
#[derive(Component, Clone)]
pub struct LaneNode {
    pub id: u32,
    pub position: Vec2,
}

#[derive(Component, Clone)]
pub struct LaneEdge {
    pub id: u32,
    pub from: Entity,
    pub to: Entity,
    pub length: f32,
    pub speed_limit: f32,
}
From your junction system:

Each LaneConnection → LaneEdge

pub fn junction_to_lane_graph(
    connections: &[LaneConnection],
) -> Vec<LaneEdge> {
    let mut edges = Vec::new();

    for conn in connections {
        edges.push(LaneEdge {
            id: rand::random(),
            from: lane_node_id(conn.from_edge, conn.from_lane),
            to: lane_node_id(conn.to_edge, conn.to_lane),
            length: 1.0,
            speed_limit: 1.0,
        });
    }

    edges
}
🚗 VEHICLE MODEL (ABSTRACT — NOT PHYSICS)
#[derive(Component)]
pub struct Vehicle {
    pub path: Vec<Entity>,     // lane edges
    pub current_edge: usize,
    pub progress: f32,         // 0.0 → 1.0
    pub speed: f32,
}
🔐 RESERVATION SYSTEM (CORE)
Each lane edge has time slots
#[derive(Component, Default)]
pub struct ReservationTable {
    pub slots: Vec<ReservationSlot>,
}

#[derive(Clone)]
pub struct ReservationSlot {
    pub start: f32,
    pub end: f32,
    pub vehicle: Entity,
}
RESERVATION REQUEST
pub fn request_reservation(
    table: &ReservationTable,
    start: f32,
    end: f32,
) -> bool {
    for slot in &table.slots {
        if !(end <= slot.start || start >= slot.end) {
            return false; // conflict
        }
    }
    true
}
COMMIT RESERVATION
pub fn reserve(
    table: &mut ReservationTable,
    vehicle: Entity,
    start: f32,
    end: f32,
) {
    table.slots.push(ReservationSlot {
        start,
        end,
        vehicle,
    });
}
🧠 PATH PLANNING WITH RESERVATION LOOKAHEAD
Vehicles must reserve MULTIPLE edges ahead
const LOOKAHEAD: usize = 3;
pub fn try_reserve_path(
    vehicle: Entity,
    path: &[Entity],
    tables: &mut Query<&mut ReservationTable>,
    current_time: f32,
) -> bool {
    let mut time = current_time;

    for edge in path.iter().take(LOOKAHEAD) {
        let mut table = tables.get_mut(*edge).unwrap();

        let duration = 1.0;

        if !request_reservation(&table, time, time + duration) {
            return false;
        }

        time += duration;
    }

    true
}
🚦 MOVEMENT SYSTEM
pub fn move_vehicles(
    mut vehicles: Query<&mut Vehicle>,
    time: Res<Time>,
) {
    for mut v in &mut vehicles {
        v.progress += v.speed * time.delta_seconds();

        if v.progress >= 1.0 {
            v.progress = 0.0;
            v.current_edge += 1;
        }
    }
}
⚠️ DEADLOCK HANDLING (MINIMAL FIRST PASS)
Detect circular wait
pub fn detect_deadlock(
    vehicles: Query<&Vehicle>,
    tables: Query<&ReservationTable>,
) -> bool {
    // simple heuristic: all vehicles blocked
    vehicles.iter().all(|v| {
        // can't reserve next edge
        true
    })
}
Resolve (basic)
pub fn resolve_deadlock(
    mut vehicles: Query<&mut Vehicle>,
) {
    for mut v in &mut vehicles {
        v.speed = 0.0; // stop all (placeholder)
    }
}
🧪 DEBUG (MANDATORY)
pub fn debug_reservations(
    query: Query<(&LaneEdge, &ReservationTable)>,
) {
    for (edge, table) in &query {
        println!("Edge {} reservations: {}", edge.id, table.slots.len());
    }
}
✅ STEP 1 COMPLETION CRITERIA
✔ lane graph exists
✔ vehicles follow lane edges
✔ reservation prevents overlap
✔ lookahead works
✔ basic deadlock detection exists