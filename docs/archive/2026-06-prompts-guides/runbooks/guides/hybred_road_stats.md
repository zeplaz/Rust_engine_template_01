HYBRID MODEL: FIELD (STATISTICAL) + AGENT (DISCRETE)
🧭 PRINCIPLE
Total Flow = FieldFlow (baseline) + AgentDelta (events/perturbations)

FieldFlow:
  deterministic, continuous, cheap, always-on

AgentDelta:
  sparse, discrete, high-impact, event-driven
🧱 1) FIELD MODEL (EDGE-LEVEL STATE)

Each LaneEdge carries continuous state (updated every tick).

#[derive(Component, Clone)]
pub struct EdgeFieldState {
    pub capacity: f32,        // max throughput
    pub demand: f32,          // requested flow
    pub flow: f32,            // resolved flow
    pub congestion: f32,      // 0..1
    pub damage: f32,          // 0..1
    pub danger: f32,          // 0..1 (military / accidents)
    pub travel_time: f32,     // derived
}
FIELD UPDATE (MACRO SOLVE)

Cheap, local relaxation (no global solver needed to start):

pub fn update_edge_field(
    mut q: Query<&mut EdgeFieldState>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();

    for mut e in &mut q {
        // capacity reduced by damage
        let effective_capacity = e.capacity * (1.0 - e.damage);

        // simple congestion curve
        let utilization = (e.demand / effective_capacity).clamp(0.0, 2.0);
        e.congestion = (utilization - 1.0).max(0.0).clamp(0.0, 1.0);

        // flow saturates at capacity
        e.flow = e.demand.min(effective_capacity);

        // travel time grows with congestion
        e.travel_time = 1.0 * (1.0 + 3.0 * e.congestion);

        // passive decay (self-heal / normalize)
        e.damage = (e.damage - 0.02 * dt).max(0.0);
        e.danger = (e.danger - 0.05 * dt).max(0.0);
    }
}
📦 2) DEMAND PROPAGATION (SUPPLY → SINK)

Demand is injected from economy (G5), then pushed along graph.

#[derive(Component)]
pub struct Source {
    pub rate: f32,
}

#[derive(Component)]
pub struct Sink {
    pub need: f32,
}
pub fn propagate_demand(
    mut edges: Query<&mut EdgeFieldState>,
    sources: Query<(&Source, &ConnectedEdges)>,
) {
    // reset
    for mut e in &mut edges {
        e.demand = 0.0;
    }

    // push from sources
    for (src, conns) in &sources {
        let share = src.rate / conns.edges.len() as f32;
        for edge in &conns.edges {
            if let Ok(mut e) = edges.get_mut(*edge) {
                e.demand += share;
            }
        }
    }
}
🚗 3) AGENT LAYER (SPARSE, EVENTFUL)

Agents exist only when needed:

#[derive(Component)]
pub struct FlowAgent {
    pub cargo: ResourceType,
    pub path: Vec<Entity>,   // LaneEdges
    pub impact: f32,         // weight vs field
    pub state: AgentState,
}

pub enum AgentState {
    Moving,
    Waiting,
    Destroyed,
}
AGENTS MODIFY THE FIELD (NOT REPLACE IT)
pub fn apply_agent_delta(
    agents: Query<&FlowAgent>,
    mut edges: Query<&mut EdgeFieldState>,
) {
    for agent in &agents {
        for edge in agent.path.iter().take(2) {
            if let Ok(mut e) = edges.get_mut(*edge) {
                // agents increase congestion locally
                e.congestion = (e.congestion + 0.05 * agent.impact).clamp(0.0, 1.0);
            }
        }
    }
}
💥 4) EVENTS (HIGH IMPACT)

Example: truck destroyed

pub fn on_agent_destroyed(
    agent: &FlowAgent,
    mut edges: Query<&mut EdgeFieldState>,
) {
    for edge in &agent.path {
        if let Ok(mut e) = edges.get_mut(*edge) {
            e.damage += 0.3;
            e.danger += 0.5;
        }
    }
}
🧠 5) AGENT SPAWNING (CONTROLLED)

Agents represent fraction of flow, not all of it.

pub fn spawn_agents(
    edges: Query<(Entity, &EdgeFieldState)>,
    mut commands: Commands,
) {
    for (edge_e, e) in &edges {
        // spawn only if congestion or danger is interesting
        if e.congestion > 0.5 || e.danger > 0.3 {
            commands.spawn(FlowAgent {
                cargo: ResourceType::Metal,
                path: vec![edge_e],
                impact: 1.0,
                state: AgentState::Moving,
            });
        }
    }
}
🧭 6) ROUTING (FIELD-INFLUENCED)

Agents route using field values:

pub fn edge_cost(e: &EdgeFieldState) -> f32 {
    1.0
    + e.congestion * 5.0
    + e.damage * 10.0
    + e.danger * 20.0
}
🔄 7) FEEDBACK LOOP
Field updates →
    affects routing →
        affects agent paths →
            agents modify field →
                new equilibrium
⚖️ 8) CONTROL KNOBS (CRITICAL FOR DESIGNERS)

Expose via RON / LLM tuning:

HybridConfig(
  agent_spawn_threshold_congestion: 0.5,
  agent_spawn_threshold_danger: 0.3,
  agent_impact_scale: 0.05,
  damage_decay: 0.02,
  danger_decay: 0.05,
  congestion_multiplier: 3.0
)
🧪 9) DEBUG OVERLAYS
pub fn debug_field(
    mut gizmos: Gizmos,
    edges: Query<(&LaneEdge, &EdgeFieldState)>,
) {
    for (edge, e) in &edges {
        let color = Color::rgb(e.congestion, 1.0 - e.congestion, 0.0);
        gizmos.line_2d(
            edge_start(edge),
            edge_end(edge),
            color,
        );
    }
}
🚫 HARD RULES
- agents must NOT carry full economy
- field must NOT be bypassed
- all routing uses field-informed cost
- agents only amplify, never replace baseline flow
✅ COMPLETION CRITERIA
✔ field produces stable flows without agents
✔ agents create visible local disruptions
✔ destroying agents affects macro flow
✔ routing adapts to danger/damage
✔ agent count stays low under load
🔥 WHAT THIS UNLOCKS

Now your system can:

scale to massive worlds (field does heavy lifting)
still feel reactive (agents create spikes/events)
support warfare/logistics (damage + danger fields)
avoid Cities-style agent explosion