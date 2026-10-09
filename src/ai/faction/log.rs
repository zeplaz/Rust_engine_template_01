//! **Faction AI decision log** — the explainability record. Sole writer:
//! [`super::faction_ai_decide_system`]. Readers: HUD explainability window, witness JSON, tests.
//!
//! This is an audit trail of decisions already taken, not a blackboard: nothing in the sim reads it
//! back to decide (`MAI-CUT-BOARD`).

use bevy::prelude::*;

use super::decide::{GoalChoice, StrategicGoal};
use super::orders::{AiOrder, OrderOutcome};

/// Records kept per faction slot.
pub const DECISION_LOG_PER_FACTION: usize = 32;

/// The few perception scalars that drove one decision (copied once, for the reader).
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize)]
pub struct PerceptionDigest {
    pub cells_held: u32,
    pub front_cells: u32,
    pub mean_front_threat: f32,
    pub mean_front_advantage: f32,
    pub front_supply_shortfall: f32,
    pub mean_logistics_held: f32,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct FactionAiDecisionRecord {
    pub sim_tick: u64,
    pub slot: u8,
    pub goal: StrategicGoal,
    pub previous_goal: Option<StrategicGoal>,
    pub choice: GoalChoice,
    pub digest: PerceptionDigest,
    /// Orders issued this decision and what the player-facing API said about each.
    pub orders: Vec<(AiOrder, OrderOutcome)>,
}

impl FactionAiDecisionRecord {
    /// Plain-language lines: winning goal, its top terms, and the runner-up.
    #[must_use]
    pub fn explain(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let total = self
            .choice
            .score_of(self.goal)
            .map_or(0.0, super::decide::GoalScore::total);
        let switched = match self.previous_goal {
            Some(prev) if prev != self.goal => format!(" (was {})", prev.label()),
            _ => String::new(),
        };
        lines.push(format!(
            "Faction {} → {} — score {:.2}{}{}",
            self.slot,
            self.goal.label(),
            total,
            switched,
            if self.choice.kept_by_commitment {
                " · kept by commitment"
            } else {
                ""
            }
        ));
        if let Some(win) = self.choice.score_of(self.goal) {
            let mut terms: Vec<_> = win.terms.iter().filter(|t| t.value().abs() > 1e-4).collect();
            terms.sort_by(|a, b| b.value().abs().total_cmp(&a.value().abs()));
            for t in terms.into_iter().take(3) {
                lines.push(format!("  {:+.2} {} ({:.2} × {:.2})", t.value(), t.label, t.input, t.weight));
            }
        }
        let mut others: Vec<_> = self.choice.scores.iter().filter(|s| s.goal != self.goal).collect();
        others.sort_by(|a, b| b.total().total_cmp(&a.total()));
        if let Some(runner) = others.first() {
            lines.push(format!("  runner-up: {} {:.2}", runner.goal.label(), runner.total()));
        }
        for (order, outcome) in &self.orders {
            lines.push(format!("  order: {} → {}", order.describe(), outcome.describe()));
        }
        lines
    }
}

/// Ring buffer of recent decisions, per faction slot, ordered oldest → newest.
#[derive(Resource, Clone, Debug, Default)]
pub struct FactionAiDecisionLog {
    pub records: Vec<FactionAiDecisionRecord>,
    /// Decisions ever logged (monotonic; survives ring eviction).
    pub total_decisions: u64,
    /// Goal switches ever logged.
    pub total_goal_switches: u64,
}

impl FactionAiDecisionLog {
    pub fn push(&mut self, record: FactionAiDecisionRecord) {
        self.total_decisions += 1;
        if record.previous_goal.is_some_and(|p| p != record.goal) {
            self.total_goal_switches += 1;
        }
        let slot = record.slot;
        self.records.push(record);
        let count = self.records.iter().filter(|r| r.slot == slot).count();
        if count > DECISION_LOG_PER_FACTION {
            if let Some(i) = self.records.iter().position(|r| r.slot == slot) {
                self.records.remove(i);
            }
        }
    }

    #[must_use]
    pub fn latest_for(&self, slot: u8) -> Option<&FactionAiDecisionRecord> {
        self.records.iter().rev().find(|r| r.slot == slot)
    }

    /// Slots with at least one record, ascending.
    #[must_use]
    pub fn slots(&self) -> Vec<u8> {
        let mut s: Vec<u8> = self.records.iter().map(|r| r.slot).collect();
        s.sort_unstable();
        s.dedup();
        s
    }
}
