//! **Faction AI decision** — pure goal scoring over a [`FactionPerception`].
//!
//! Plain weights, no curve asset and no utility crate (`MAI-CUT-UTILITY`). Every score is the sum
//! of named [`ScoreTerm`]s so the HUD and the witness can say *why* a goal won. Deterministic: ties
//! break in [`StrategicGoal::ALL`] order; there is no random roll.

use crate::strategic::HybridAgentTraits;

use super::perception::FactionPerception;

/// Faction-level intent. One is active per faction; it drives the orders the faction issues.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize)]
pub enum StrategicGoal {
    /// Reinforce the most threatened front cell.
    DefendFront,
    /// Push where this faction already leads and threat is low.
    Advance,
    /// Feed under-supplied front cells from the rear.
    SecureSupply,
    /// No pressing front — grow the economy behind the line.
    Develop,
    /// Nothing worth doing; hold.
    #[default]
    Hold,
}

impl StrategicGoal {
    /// Tie-break order: earlier wins on an exact tie.
    pub const ALL: [Self; 5] = [
        Self::DefendFront,
        Self::SecureSupply,
        Self::Advance,
        Self::Develop,
        Self::Hold,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::DefendFront => "defend front",
            Self::Advance => "advance",
            Self::SecureSupply => "secure supply",
            Self::Develop => "develop",
            Self::Hold => "hold",
        }
    }
}

/// Faction personality as plain weights. Defaults are a balanced faction.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FactionDoctrine {
    pub aggression: f32,
    pub caution: f32,
    pub logistics_focus: f32,
    pub development_focus: f32,
    /// Bonus kept by the current goal so the faction does not flip every decision.
    pub commitment: f32,
}

impl Default for FactionDoctrine {
    fn default() -> Self {
        Self {
            aggression: 0.5,
            caution: 0.5,
            logistics_focus: 0.5,
            development_focus: 0.5,
            commitment: 0.1,
        }
    }
}

impl FactionDoctrine {
    /// Map the behavior-brain traits of a faction's lead [`crate::strategic::Agent`] onto doctrine,
    /// so personality authored there shows up in strategic choices.
    #[must_use]
    pub fn from_traits(t: &HybridAgentTraits) -> Self {
        let c = |v: f32| v.clamp(0.0, 1.0);
        Self {
            aggression: c(0.5 * t.ambition + 0.3 * t.aggression.max(t.cruelty) + 0.2 * t.nationalism),
            caution: c(0.6 * t.paranoia + 0.4 * (1.0 - t.risk_tolerance)),
            logistics_focus: c(0.7 * t.rationality + 0.3 * (1.0 - t.instability)),
            development_focus: c(0.5 * t.rationality + 0.5 * t.empathy),
            commitment: 0.1,
        }
    }
}

/// One named contribution: `value = input * weight`.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct ScoreTerm {
    pub label: &'static str,
    pub input: f32,
    pub weight: f32,
}

impl ScoreTerm {
    #[inline]
    #[must_use]
    pub fn value(&self) -> f32 {
        self.input * self.weight
    }
}

/// Score for one goal with its terms.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct GoalScore {
    pub goal: StrategicGoal,
    pub terms: Vec<ScoreTerm>,
}

impl GoalScore {
    #[must_use]
    pub fn total(&self) -> f32 {
        self.terms.iter().map(ScoreTerm::value).sum()
    }
}

/// Result of one decision: every goal's score, the winner, and whether hysteresis kept it.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct GoalChoice {
    pub chosen: StrategicGoal,
    pub scores: Vec<GoalScore>,
    pub kept_by_commitment: bool,
}

impl GoalChoice {
    #[must_use]
    pub fn score_of(&self, goal: StrategicGoal) -> Option<&GoalScore> {
        self.scores.iter().find(|s| s.goal == goal)
    }
}

/// Score every goal for this perception and doctrine.
#[must_use]
pub fn score_goals(p: &FactionPerception, d: &FactionDoctrine, current: Option<StrategicGoal>) -> Vec<GoalScore> {
    let has_front = !p.front.is_empty();
    let front = if has_front { 1.0 } else { 0.0 };
    let threat = p.mean_front_threat().max(p.mean_threat_held);
    let deficit = (-p.mean_front_advantage()).max(0.0);
    let lead = p.mean_front_advantage().max(0.0);
    let shortfall = p.front_supply_shortfall();
    let calm = (1.0 - threat).clamp(0.0, 1.0);
    let commit = |g: StrategicGoal| {
        if current == Some(g) {
            1.0
        } else {
            0.0
        }
    };

    let mut out = Vec::with_capacity(StrategicGoal::ALL.len());
    for goal in StrategicGoal::ALL {
        let mut terms = match goal {
            StrategicGoal::DefendFront => vec![
                ScoreTerm { label: "front threat", input: threat * front, weight: 0.6 + 0.4 * d.caution },
                ScoreTerm { label: "losing ground", input: deficit * front, weight: 1.0 + d.caution },
            ],
            StrategicGoal::Advance => vec![
                ScoreTerm { label: "front lead", input: lead * front, weight: 1.0 + d.aggression },
                ScoreTerm { label: "aggression", input: front, weight: 0.3 * d.aggression },
                ScoreTerm { label: "threat risk", input: threat * front, weight: -0.5 * d.caution },
                ScoreTerm { label: "supply risk", input: shortfall, weight: -0.3 },
            ],
            StrategicGoal::SecureSupply => vec![
                ScoreTerm { label: "front supply shortfall", input: shortfall * front, weight: 0.4 + 0.4 * d.logistics_focus },
                ScoreTerm { label: "rear logistics weak", input: (1.0 - p.mean_logistics_held).clamp(0.0, 1.0) * front, weight: 0.1 },
            ],
            StrategicGoal::Develop => vec![
                ScoreTerm { label: "calm", input: calm, weight: 0.3 * d.development_focus },
                ScoreTerm { label: "no front", input: 1.0 - front, weight: 0.3 },
            ],
            StrategicGoal::Hold => vec![ScoreTerm { label: "baseline", input: 1.0, weight: 0.05 }],
        };
        if commit(goal) > 0.0 && goal != StrategicGoal::Hold {
            terms.push(ScoreTerm { label: "commitment", input: 1.0, weight: d.commitment });
        }
        out.push(GoalScore { goal, terms });
    }
    out
}

/// Pick the highest-scoring goal. Ties break in [`StrategicGoal::ALL`] order.
#[must_use]
pub fn choose_goal(p: &FactionPerception, d: &FactionDoctrine, current: Option<StrategicGoal>) -> GoalChoice {
    let scores = score_goals(p, d, current);
    let mut best: Option<&GoalScore> = None;
    for s in &scores {
        if best.map_or(true, |b| s.total() > b.total()) {
            best = Some(s);
        }
    }
    let chosen = best.map_or(StrategicGoal::Hold, |b| b.goal);
    let kept_by_commitment = current == Some(chosen) && {
        let without = scores
            .iter()
            .filter(|s| s.goal != chosen)
            .map(GoalScore::total)
            .fold(f32::NEG_INFINITY, f32::max);
        let own = scores.iter().find(|s| s.goal == chosen).map_or(0.0, |s| {
            s.terms
                .iter()
                .filter(|t| t.label != "commitment")
                .map(ScoreTerm::value)
                .sum::<f32>()
        });
        own < without
    };
    GoalChoice {
        chosen,
        scores,
        kept_by_commitment,
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::IVec2;

    use super::*;
    use crate::ai::faction::perception::{perceive_faction, test_support::two_faction_strip, FrontCell};

    fn front(threat: f32, own: f32, enemy: f32, logi: f32) -> FactionPerception {
        FactionPerception {
            slot: 1,
            cells_total: 4,
            cells_held: 1,
            mean_logistics_held: 0.8,
            front: vec![FrontCell {
                tile: IVec2::new(1, 0),
                chunk: IVec2::ZERO,
                own_control: own,
                enemy_control: enemy,
                enemy_slot: 0,
                threat,
                logistics_strength: logi,
            }],
            rival_slots: vec![0],
            ..FactionPerception::default()
        }
    }

    #[test]
    fn fai_threat_and_deficit_choose_defend() {
        let p = front(0.8, 0.35, 0.55, 0.6);
        let c = choose_goal(&p, &FactionDoctrine::default(), None);
        assert_eq!(c.chosen, StrategicGoal::DefendFront);
    }

    #[test]
    fn fai_lead_and_calm_choose_advance() {
        let p = front(0.0, 0.55, 0.35, 0.6);
        let c = choose_goal(&p, &FactionDoctrine::default(), None);
        assert_eq!(c.chosen, StrategicGoal::Advance);
    }

    #[test]
    fn fai_starved_even_front_chooses_supply() {
        let p = front(0.05, 0.45, 0.45, 0.0);
        let d = FactionDoctrine {
            logistics_focus: 1.0,
            ..FactionDoctrine::default()
        };
        assert_eq!(choose_goal(&p, &d, None).chosen, StrategicGoal::SecureSupply);
    }

    #[test]
    fn fai_no_front_develops() {
        let p = FactionPerception::default();
        assert_eq!(choose_goal(&p, &FactionDoctrine::default(), None).chosen, StrategicGoal::Develop);
    }

    #[test]
    fn fai_commitment_keeps_a_near_tie() {
        // Advance barely loses without commitment; the faction is already advancing.
        let p = front(0.25, 0.47, 0.43, 0.6);
        let d = FactionDoctrine::default();
        let fresh = choose_goal(&p, &d, None);
        let kept = choose_goal(&p, &d, Some(StrategicGoal::Advance));
        assert_eq!(kept.chosen, StrategicGoal::Advance);
        if fresh.chosen != StrategicGoal::Advance {
            assert!(kept.kept_by_commitment);
        }
    }

    #[test]
    fn fai_scores_explain_the_winner() {
        let ov = two_faction_strip();
        let p = perceive_faction(0, [&ov]);
        let c = choose_goal(&p, &FactionDoctrine::default(), None);
        let win = c.score_of(c.chosen).expect("winner scored");
        assert!(!win.terms.is_empty());
        let sum: f32 = win.terms.iter().map(ScoreTerm::value).sum();
        assert!((sum - win.total()).abs() < 1e-6);
        assert_eq!(c.scores.len(), StrategicGoal::ALL.len());
    }

    #[test]
    fn fai_doctrine_from_traits_tracks_ambition() {
        let bold = FactionDoctrine::from_traits(&HybridAgentTraits {
            ambition: 1.0,
            aggression: 1.0,
            ..HybridAgentTraits::default()
        });
        let meek = FactionDoctrine::from_traits(&HybridAgentTraits::default());
        assert!(bold.aggression > meek.aggression);
    }
}
