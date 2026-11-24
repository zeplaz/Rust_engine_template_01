//! **Decision pipeline (composed score)** — documents the locked formula; no hard scripted outcomes.
//!
//! `Final ≈ traits + emotion + faction + script + environment` (each channel stays fuzzy).
//!
//! **DEC-05** (aig3 5.5 *Fuzzy Logic*): [`fuzzy_degree`] / [`fuzzy_and`] / [`fuzzy_or`] keep
//! channels as soft memberships — no hard thresholds. [`compose_decision_score_fuzzy`] is the
//! fuzzy state composition; linear [`compose_decision_score`] remains the HUD sum.

use bevy::prelude::*;

use super::behavior_entities::{Agent, AgentFactionLink, AgentMode, Faction};
use super::behavior_pressure::PressureField;
use super::behavior_script::ScriptInfluence;
use super::hybrid_fields::WorldFields;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DecisionScoreComponents {
    pub base_traits: f32,
    pub emotional: f32,
    pub faction_pressure: f32,
    pub script_influence: f32,
    pub environmental: f32,
}


/// DEC-05: triangular membership of `x` in [lo, peak, hi] → degree in [0, 1].
#[inline]
#[must_use]
#[cfg(test)] // only tests use it today
pub fn fuzzy_degree(x: f32, lo: f32, peak: f32, hi: f32) -> f32 {
    if !(lo < peak && peak < hi) {
        return 0.0;
    }
    if x <= lo || x >= hi {
        return 0.0;
    }
    if x == peak {
        return 1.0;
    }
    if x < peak {
        (x - lo) / (peak - lo)
    } else {
        (hi - x) / (hi - peak)
    }
}

/// Soft AND (product t-norm).
#[inline]
#[must_use]
#[cfg(test)] // only tests use it today
pub fn fuzzy_and(a: f32, b: f32) -> f32 {
    (a.clamp(0.0, 1.0) * b.clamp(0.0, 1.0)).clamp(0.0, 1.0)
}

/// Soft OR (probabilistic sum).
#[inline]
#[must_use]
#[cfg(test)] // only tests use it today
pub fn fuzzy_or(a: f32, b: f32) -> f32 {
    let a = a.clamp(0.0, 1.0);
    let b = b.clamp(0.0, 1.0);
    (a + b - a * b).clamp(0.0, 1.0)
}

/// DEC-05: compose channels as fuzzy degrees then soft-OR (no hard cutoffs).
#[must_use]
#[cfg(test)] // only tests use it today
pub fn compose_decision_score_fuzzy(c: &DecisionScoreComponents) -> f32 {
    let channels = [
        fuzzy_degree(c.base_traits, -0.05, 0.15, 0.45),
        fuzzy_degree(c.emotional, -0.15, 0.05, 0.35),
        fuzzy_degree(c.faction_pressure, -0.05, 0.1, 0.4),
        fuzzy_degree(c.script_influence, -0.05, 0.12, 0.5),
        fuzzy_degree(c.environmental, -0.05, 0.15, 0.55),
    ];
    channels.into_iter().fold(0.0_f32, fuzzy_or)
}

#[inline]
pub fn compose_decision_score(c: &DecisionScoreComponents) -> f32 {
    c.base_traits + c.emotional + c.faction_pressure + c.script_influence + c.environmental
}

/// Sample components for one agent (simulation-native scoring hooks).
pub fn sample_decision_components(
    agent: &Agent,
    world: &WorldFields,
    pressure_field: &PressureField,
    faction_opt: Option<&Faction>,
    script: Option<&ScriptInfluence>,
) -> DecisionScoreComponents {
    let blended_traits = match agent.mode {
        AgentMode::Free => agent.traits,
        AgentMode::Scripted => match script {
            Some(s) => ScriptInfluence::blend_traits(&agent.traits, 1.0, s.priority, &s.bias_vector),
            None => agent.traits,
        },
        AgentMode::Hybrid { script_weight } => match script {
            Some(s) => ScriptInfluence::blend_traits(&agent.traits, script_weight, s.priority, &s.bias_vector),
            None => agent.traits,
        },
    };

    let base_traits = blended_traits.ambition * 0.12
        + blended_traits.rationality * 0.08
        + blended_traits.cruelty * 0.04;
    let emotional = agent.emotional_state.confidence * 0.1
        - agent.emotional_state.fear * 0.06
        + agent.emotional_state.anger * 0.04
        + agent.cognition.narrative_salience * 0.05;

    let faction_pressure = faction_opt
        .map(|f| {
            (1.0 - f.cohesion) * 0.08 + f.control_strength * 0.05 + f.resources.clamp(0.0, 200.0) * 0.0002
        })
        .unwrap_or(0.0);

    let script_influence = script
        .map(|s| {
            s.priority * 0.15
                + s.forced_intents.iter().map(|i| i.weight).sum::<f32>() * 0.02
        })
        .unwrap_or(0.0);

    let card_pressure = script
        .map(|s| {
            let p = &s.pressure_profile;
            p.paranoia * 0.02 + p.aggression * 0.02 + p.instability * 0.025
        })
        .unwrap_or(0.0);

    let environmental = world.economic_pressure * 0.06
        + world.instability_index * 0.05
        + pressure_field.paranoia * 0.04
        + pressure_field.aggression * 0.04
        + pressure_field.instability * 0.05
        + card_pressure;

    DecisionScoreComponents {
        base_traits,
        emotional,
        faction_pressure,
        script_influence,
        environmental,
    }
}

/// Per-agent pass: mean score published for HUD / debugger (full game may branch per action type).
pub fn decision_pipeline_composition_system(
    world: Res<WorldFields>,
    pressure_field: Res<PressureField>,
    agents: Query<(Entity, &Agent, Option<&ScriptInfluence>), With<Agent>>,
    links: Query<&AgentFactionLink>,
    factions: Query<&Faction>,
    mut sink: ResMut<super::behavior_interface::DecisionPipelineSink>,
) {
    let mut sum = 0.0_f32;
    let mut n = 0usize;
    for (entity, agent, script) in agents.iter() {
        let fac = links
            .iter()
            .find(|l| l.agent == entity)
            .and_then(|l| factions.get(l.faction).ok());
        let c = sample_decision_components(agent, world.as_ref(), pressure_field.as_ref(), fac, script);
        sum += compose_decision_score(&c);
        n += 1;
    }
    sink.last_mean_composed_score = if n > 0 { sum / n as f32 } else { 0.0 };
    sink.last_agent_samples = n;
}

#[cfg(test)]
mod dec05_tests {
    use super::*;

    #[test]
    fn dec05_fuzzy_membership_and_soft_compose() {
        assert!((fuzzy_degree(0.15, -0.05, 0.15, 0.45) - 1.0).abs() < 1e-5);
        assert_eq!(fuzzy_degree(-1.0, 0.0, 0.5, 1.0), 0.0);
        assert!((fuzzy_and(0.5, 0.5) - 0.25).abs() < 1e-5);
        assert!((fuzzy_or(0.5, 0.5) - 0.75).abs() < 1e-5);
        let c = DecisionScoreComponents {
            base_traits: 0.15,
            emotional: 0.05,
            faction_pressure: 0.1,
            script_influence: 0.12,
            environmental: 0.15,
        };
        let soft = compose_decision_score_fuzzy(&c);
        assert!(soft > 0.5 && soft <= 1.0);
        assert_ne!(soft, compose_decision_score(&c));
    }
}
