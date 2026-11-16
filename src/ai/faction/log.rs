//! **Faction AI decision log** — the explainability record. Sole writer:
//! [`super::faction_ai_tick_system`]. Readers: witness JSON, tests.
//!
//! **Behaviour changes** ([`FactionBehaviourChangeLog`]): goal switches only — copied into
//! [`crate::strategic::BehaviourChangesSnapshot`] for the Insert explain panel (`DEC-15`).
//!
//! **TAC-07** (aig3 6.2.4 *Learning with Tactical Analyses*): [`TacticalFragMap`] remembers
//! where losses happened (frag heat). Decision scoring may *read* heat; this module owns writes.
//!
//! This is an audit trail of decisions already taken, not a blackboard: nothing in the sim reads it
//! back to decide (`MAI-CUT-BOARD`) except optional frag-heat bias via [`TacticalFragMap::heat_at`].

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::decide::{GoalChoice, ScoreTerm, StrategicGoal};
use super::orders::{AiOrder, OrderOutcome};


/// TAC-07: remembered loss sites (frag map). Sole writers call [`TacticalFragMap::record_loss`].
#[derive(Resource, Clone, Debug, Default)]
pub struct TacticalFragMap {
    /// Tile → accumulated loss weight (deterministic BTree for stable iteration).
    pub losses: BTreeMap<(i32, i32), f32>,
    pub total_recorded: u64,
}

impl TacticalFragMap {
    /// Record a loss event at `tile` (weight > 0). Caps per-cell heat at 8.0.
    pub fn record_loss(&mut self, tile: IVec2, weight: f32) {
        let w = weight.max(0.0);
        if w <= 0.0 {
            return;
        }
        let key = (tile.x, tile.y);
        let entry = self.losses.entry(key).or_insert(0.0);
        *entry = (*entry + w).min(8.0);
        self.total_recorded = self.total_recorded.saturating_add(1);
    }

    #[must_use]
    pub fn heat_at(&self, tile: IVec2) -> f32 {
        self.losses.get(&(tile.x, tile.y)).copied().unwrap_or(0.0)
    }

    /// Uniform decay toward zero; drops near-zero cells.
    pub fn decay(&mut self, factor: f32) {
        let f = factor.clamp(0.0, 1.0);
        self.losses.retain(|_, h| {
            *h *= f;
            *h > 1e-3
        });
    }

    /// Hottest tiles, descending heat then `(y, x)`.
    #[must_use]
    pub fn hottest(&self, n: usize) -> Vec<(IVec2, f32)> {
        let mut v: Vec<(IVec2, f32)> = self
            .losses
            .iter()
            .map(|(&(x, y), &h)| (IVec2::new(x, y), h))
            .collect();
        v.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.y.cmp(&b.0.y)).then(a.0.x.cmp(&b.0.x)));
        v.truncate(n);
        v
    }
}

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

/// Why one brain picked one thing: its scoring terms, in the order they were summed.
///
/// `brain` is `"economy"`, `"military"` or `"logistics"`; `subject` says what was scored
/// (`"reinforce (12, 40)"`, `"build concrete_mixer_plant"`). `order` indexes into
/// [`FactionAiDecisionRecord::orders`] when the choice became an order.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct DecisionTrace {
    pub brain: &'static str,
    pub subject: String,
    pub total: f32,
    pub terms: Vec<ScoreTerm>,
    pub order: Option<usize>,
}

impl DecisionTrace {
    /// The `n` strongest terms by magnitude.
    #[must_use]
    pub fn top_terms(&self, n: usize) -> Vec<&ScoreTerm> {
        let mut terms: Vec<&ScoreTerm> = self.terms.iter().filter(|t| t.value().abs() > 1e-4).collect();
        terms.sort_by(|a, b| b.value().abs().total_cmp(&a.value().abs()));
        terms.truncate(n);
        terms
    }
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
    /// What each brain scored this decision, strongest reasons first within a trace.
    pub traces: Vec<DecisionTrace>,
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
        for trace in &self.traces {
            lines.push(format!("  [{}] {} — {:.2}", trace.brain, trace.subject, trace.total));
            for t in trace.top_terms(3) {
                lines.push(format!("    {:+.2} {} ({:.2} × {:.2})", t.value(), t.label, t.input, t.weight));
            }
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

/// Ring capacity for behaviour-change lines (panel shows the last 8).
pub const BEHAVIOUR_CHANGES_CAP: usize = 32;

/// One perceptible goal switch: `tick · old → new · deciding term`.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct FactionBehaviourChangeRecord {
    pub sim_tick: u64,
    pub slot: u8,
    pub previous_goal: Option<StrategicGoal>,
    pub new_goal: StrategicGoal,
    pub deciding_term: String,
}

impl FactionBehaviourChangeRecord {
    #[must_use]
    pub fn from_choice(
        sim_tick: u64,
        slot: u8,
        previous_goal: Option<StrategicGoal>,
        choice: &GoalChoice,
    ) -> Self {
        Self {
            sim_tick,
            slot,
            previous_goal,
            new_goal: choice.chosen,
            deciding_term: deciding_term_label(choice),
        }
    }

    /// `tick · old → new · deciding term`
    #[must_use]
    pub fn line(&self) -> String {
        let old = self
            .previous_goal
            .map_or("—".to_string(), |g| g.label().to_string());
        format!(
            "tick {} · {} → {} · {}",
            self.sim_tick, old, self.new_goal.label(), self.deciding_term
        )
    }
}

#[must_use]
fn deciding_term_label(choice: &GoalChoice) -> String {
    let Some(win) = choice.score_of(choice.chosen) else {
        return "—".into();
    };
    let mut terms: Vec<_> = win
        .terms
        .iter()
        .filter(|t| t.label != "commitment" && t.value().abs() > 1e-4)
        .collect();
    terms.sort_by(|a, b| b.value().abs().total_cmp(&a.value().abs()));
    terms
        .first()
        .map(|t| format!("{} ({:+.2})", t.label, t.value()))
        .unwrap_or_else(|| "—".into())
}

/// Goal switches only — for player-facing explainability (`DEC-15`).
#[derive(Resource, Clone, Debug, Default)]
pub struct FactionBehaviourChangeLog {
    pub entries: Vec<FactionBehaviourChangeRecord>,
    pub total: u64,
}

impl FactionBehaviourChangeLog {
    /// Records when the chosen goal differs from the previous one.
    #[must_use]
    pub fn maybe_push(&mut self, record: FactionBehaviourChangeRecord) -> bool {
        if record.previous_goal == Some(record.new_goal) {
            return false;
        }
        self.total += 1;
        self.entries.push(record);
        if self.entries.len() > BEHAVIOUR_CHANGES_CAP {
            let drop = self.entries.len() - BEHAVIOUR_CHANGES_CAP;
            self.entries.drain(0..drop);
        }
        true
    }

    /// Newest-first, capped at `n`.
    #[must_use]
    pub fn recent(&self, n: usize) -> Vec<&FactionBehaviourChangeRecord> {
        self.entries.iter().rev().take(n).collect()
    }
}

#[cfg(test)]
mod behaviour_change_tests {
    use super::*;

    #[test]
    fn unchanged_goal_over_ten_ticks_adds_no_entries() {
        let mut log = FactionBehaviourChangeLog::default();
        let hold = StrategicGoal::Hold;
        for tick in 0..10u64 {
            log.maybe_push(FactionBehaviourChangeRecord {
                sim_tick: tick,
                slot: 1,
                previous_goal: Some(hold),
                new_goal: hold,
                deciding_term: "baseline".into(),
            });
        }
        assert!(log.entries.is_empty());
        assert_eq!(log.total, 0);
    }

    #[test]
    fn one_goal_switch_adds_exactly_one() {
        let mut log = FactionBehaviourChangeLog::default();
        let hold = StrategicGoal::Hold;
        let defend = StrategicGoal::DefendFront;
        for tick in 0..10u64 {
            log.maybe_push(FactionBehaviourChangeRecord {
                sim_tick: tick,
                slot: 1,
                previous_goal: Some(hold),
                new_goal: hold,
                deciding_term: "baseline".into(),
            });
        }
        assert!(log.maybe_push(FactionBehaviourChangeRecord {
            sim_tick: 10,
            slot: 1,
            previous_goal: Some(hold),
            new_goal: defend,
            deciding_term: "front threat (+0.42)".into(),
        }));
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.total, 1);
        assert_eq!(log.entries[0].new_goal, defend);
    }
    #[test]
    fn tac07_frag_map_remembers_and_decays_losses() {
        let mut map = TacticalFragMap::default();
        map.record_loss(IVec2::new(3, 4), 1.0);
        map.record_loss(IVec2::new(3, 4), 1.0);
        map.record_loss(IVec2::new(1, 1), 0.5);
        assert_eq!(map.heat_at(IVec2::new(3, 4)), 2.0);
        assert_eq!(map.total_recorded, 3);
        let hot = map.hottest(1);
        assert_eq!(hot[0].0, IVec2::new(3, 4));
        map.decay(0.5);
        assert!((map.heat_at(IVec2::new(3, 4)) - 1.0).abs() < 1e-5);
    }

}
