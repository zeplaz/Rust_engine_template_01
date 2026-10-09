//! **Faction strategic AI** (`PLAN-AI-FLEET-v1`) — perceive → decide → plan → act → log.
//!
//! - Perception is a pure fold over [`ChunkStrategicOverlay`] ([`perception::perceive_faction`]).
//! - Goal choice is plain-weight scoring with named terms ([`decide::choose_goal`]).
//! - Orders go through the player's command APIs only ([`AiOrderSinks`]); the AI never writes
//!   overlay fields, graphs, `Transform`, or occupation.
//! - Every decision lands in [`FactionAiDecisionLog`] with its scores and API outcomes.
//! - Cadence is the sim tick (`SimTick`, `SimControlState::should_tick`) — no camera, no LOD gate.

pub mod decide;
pub mod log;
pub mod orders;
pub mod perception;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::strategic::{ChunkStrategicOverlay, StrategicCommandQueue, StrategicFieldPipeline};
use crate::systems::sim_control::{
    SimControlState, SimControlSystemSet, SimStepStamp, SimTick, SimTimeMicros,
};

pub use decide::{choose_goal, FactionDoctrine, GoalChoice, GoalScore, ScoreTerm, StrategicGoal};
pub use log::{FactionAiDecisionLog, FactionAiDecisionRecord, PerceptionDigest};
pub use orders::{plan_orders, AiOrder, OrderOutcome, MAX_ORDERS_PER_DECISION};
pub use perception::{perceive_faction, FactionPerception, FrontCell};

/// Default sim ticks between two decisions of one faction.
pub const DEFAULT_DECISION_INTERVAL_TICKS: u64 = 60;

/// Interim front/logistics surface until `DeploymentOrderQueue` / `LogisticsOrderCommand` land.
pub const API_STRATEGIC_COMMAND_QUEUE: &str = "StrategicCommandQueue";

/// Ordering label for partners that consume AI orders the same frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum FactionAiSet {
    Tick,
}

/// One AI-controlled faction. Lives on the faction kernel entity (`strategic::Faction`) or any
/// entity standing in for the faction slot. Sole writer: [`faction_ai_tick_system`].
#[derive(Component, Clone, Debug)]
pub struct FactionAiController {
    /// Overlay faction slot (`faction_control[slot]`). The player keeps slot 0 without a controller.
    pub slot: u8,
    pub doctrine: FactionDoctrine,
    pub decision_interval_ticks: u64,
    pub next_decision_tick: u64,
    pub goal: Option<StrategicGoal>,
    pub enabled: bool,
}

impl FactionAiController {
    #[must_use]
    pub fn new(slot: u8, doctrine: FactionDoctrine) -> Self {
        Self {
            slot,
            doctrine,
            decision_interval_ticks: DEFAULT_DECISION_INTERVAL_TICKS,
            next_decision_tick: 0,
            goal: None,
            enabled: true,
        }
    }
}

/// Every player-facing enqueue surface the AI may use. Each is optional so the AI degrades to
/// `Deferred` instead of panicking when an API is not in the app.
#[derive(SystemParam)]
pub struct AiOrderSinks<'w> {
    pub command_queue: Option<ResMut<'w, StrategicCommandQueue>>,
}

impl AiOrderSinks<'_> {
    /// Send one order through the API it belongs to (or the interim queue).
    pub fn issue(&mut self, slot: u8, order: &AiOrder, stamp: SimStepStamp) -> OrderOutcome {
        let Some(queue) = self.command_queue.as_deref_mut() else {
            return OrderOutcome::Deferred {
                reason: "no order API in this app",
            };
        };
        let summary = format!(
            "[ai faction {slot}] {}: {}",
            order.corridor_mission(),
            order.describe()
        );
        let msg = queue.enqueue_strategic(stamp, summary);
        OrderOutcome::Fallback {
            api: API_STRATEGIC_COMMAND_QUEUE,
            id: msg.command_id,
        }
    }
}

/// Perceive, decide, plan, and act for every faction whose interval elapsed.
pub fn faction_ai_tick_system(
    tick: Option<Res<SimTick>>,
    time: Option<Res<SimTimeMicros>>,
    ctrl: Option<Res<SimControlState>>,
    overlays: Query<&ChunkStrategicOverlay>,
    mut controllers: Query<&mut FactionAiController>,
    mut log: ResMut<FactionAiDecisionLog>,
    mut sinks: AiOrderSinks,
) {
    if ctrl.as_deref().is_some_and(|c| !c.should_tick()) {
        return;
    }
    let now = tick.map_or(0, |t| t.0);
    let stamp = SimStepStamp::new(now, time.map_or(0, |t| t.0));

    let mut due: Vec<Mut<FactionAiController>> = controllers
        .iter_mut()
        .filter(|c| c.enabled && now >= c.next_decision_tick)
        .collect();
    due.sort_by_key(|c| c.slot);

    for mut controller in due {
        let perception = perceive_faction(controller.slot, overlays.iter());
        let previous = controller.goal;
        let choice = choose_goal(&perception, &controller.doctrine, previous);
        let goal = choice.chosen;
        let orders: Vec<(AiOrder, OrderOutcome)> = plan_orders(goal, &perception)
            .into_iter()
            .map(|order| {
                let outcome = sinks.issue(controller.slot, &order, stamp);
                (order, outcome)
            })
            .collect();

        controller.goal = Some(goal);
        controller.next_decision_tick = now.saturating_add(controller.decision_interval_ticks.max(1));
        log.push(FactionAiDecisionRecord {
            sim_tick: now,
            slot: controller.slot,
            goal,
            previous_goal: previous,
            choice,
            digest: PerceptionDigest {
                cells_held: perception.cells_held,
                front_cells: perception.front.len() as u32,
                mean_front_threat: perception.mean_front_threat(),
                mean_front_advantage: perception.mean_front_advantage(),
                front_supply_shortfall: perception.front_supply_shortfall(),
                mean_logistics_held: perception.mean_logistics_held,
            },
            orders,
        });
    }
}

pub struct FactionAiPlugin;

impl Plugin for FactionAiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FactionAiDecisionLog>()
            .configure_sets(
                Update,
                FactionAiSet::Tick
                    .after(SimControlSystemSet::AdvanceSimTick)
                    .after(StrategicFieldPipeline::ZoneAndReadModel),
            )
            .add_systems(Update, faction_ai_tick_system.in_set(FactionAiSet::Tick));
    }
}

#[cfg(test)]
mod tests {
    use super::perception::test_support::two_faction_strip;
    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimTick>()
            .init_resource::<SimTimeMicros>()
            .init_resource::<SimControlState>()
            .init_resource::<StrategicCommandQueue>()
            .add_plugins(FactionAiPlugin);
        app.world_mut().spawn(two_faction_strip());
        app
    }

    #[test]
    fn fai_tick_decides_on_interval_and_queues_orders() {
        let mut app = app();
        let mut c = FactionAiController::new(1, FactionDoctrine::default());
        c.decision_interval_ticks = 10;
        app.world_mut().spawn(c);
        for t in 0..25u64 {
            app.world_mut().resource_mut::<SimTick>().0 = t;
            app.update();
        }
        let log = app.world().resource::<FactionAiDecisionLog>();
        assert_eq!(log.total_decisions, 3, "ticks 0, 10, 20");
        let rec = log.latest_for(1).expect("record");
        assert!(!rec.explain().is_empty());
        let reached = rec.orders.iter().filter(|(_, o)| o.reached_api()).count();
        assert_eq!(reached, rec.orders.len());
        let queue = app.world().resource::<StrategicCommandQueue>();
        assert!(queue.pending_count() > 0 || !queue.delivered.is_empty() || rec.orders.is_empty());
    }

    #[test]
    fn fai_paused_sim_does_not_decide() {
        let mut app = app();
        app.world_mut().resource_mut::<SimControlState>().paused = true;
        app.world_mut().spawn(FactionAiController::new(1, FactionDoctrine::default()));
        app.update();
        assert_eq!(app.world().resource::<FactionAiDecisionLog>().total_decisions, 0);
    }

    #[test]
    fn fai_missing_api_defers_instead_of_panicking() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins).add_plugins(FactionAiPlugin);
        let mut ov = two_faction_strip();
        ov.threat[1][1] = 0.9;
        app.world_mut().spawn(ov);
        app.world_mut().spawn(FactionAiController::new(1, FactionDoctrine::default()));
        app.update();
        let log = app.world().resource::<FactionAiDecisionLog>();
        let rec = log.latest_for(1).expect("record");
        assert!(rec
            .orders
            .iter()
            .all(|(_, o)| matches!(o, OrderOutcome::Deferred { .. })));
    }
}
