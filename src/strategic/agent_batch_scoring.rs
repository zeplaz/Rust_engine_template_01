//! GPU-first **evaluation** path (stub) with CPU batch scoring — meaning stays on CPU for determinism / ECS.
//!
//! Attach [`AgentCpuBatchScoring`] to opt into the batched score + [`AgentScoreResult`] insert each tick.

use std::cmp::Ordering;

use bevy::prelude::*;

use super::behavior_entities::Agent;
use super::behavior_pressure::PressureField;
use super::hybrid_brain::{HybridAgentEmotions, HybridAgentTraits};
use super::hybrid_fields::WorldFields;

// -----------------------------------------------------------------------------
// IO types (GPU pack layout stub)
// -----------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct AgentScoreInput {
    pub traits: Vec<HybridAgentTraits>,
    pub emotions: Vec<HybridAgentEmotions>,
    pub world_pressure: Vec<WorldPressureSample>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WorldPressureSample {
    pub war_tension: f32,
    pub economic_pressure: f32,
    pub instability: f32,
}

impl WorldPressureSample {
    #[inline]
    pub fn from_resources(world: &WorldFields, pressure: &PressureField) -> Self {
        Self {
            war_tension: world.war_tension,
            economic_pressure: world.economic_pressure,
            instability: world.instability_index.max(pressure.instability),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AgentScoreOutput {
    pub aggression: f32,
    pub cooperation: f32,
    pub fear: f32,
    pub stability: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BatchTacticalChoice {
    #[default]
    Idle,
    Attack,
    Trade,
    Flee,
}

#[inline]
pub fn score_agent_cpu(
    traits: &HybridAgentTraits,
    emotions: &HybridAgentEmotions,
    world: &WorldPressureSample,
) -> AgentScoreOutput {
    AgentScoreOutput {
        aggression: traits.aggression
            + traits.cruelty * 0.5
            + emotions.anger
            + world.war_tension * 0.35,
        cooperation: traits.empathy + traits.rationality * 0.2 + emotions.confidence * 0.25,
        fear: emotions.fear + traits.paranoia * 0.35 + world.instability * 0.4,
        stability: traits.rationality + emotions.confidence * 0.45 - traits.instability * 0.4,
    }
}

#[inline]
pub fn resolve_agent_action(score: AgentScoreOutput) -> BatchTacticalChoice {
    [
        (BatchTacticalChoice::Attack, score.aggression),
        (BatchTacticalChoice::Trade, score.cooperation),
        (BatchTacticalChoice::Flee, score.fear),
        (BatchTacticalChoice::Idle, score.stability),
    ]
    .into_iter()
    .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
    .map(|(c, _)| c)
    .unwrap_or(BatchTacticalChoice::Idle)
}

// -----------------------------------------------------------------------------
// ECS marker + pipeline stub
// -----------------------------------------------------------------------------

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct AgentCpuBatchScoring;

#[derive(Component, Clone, Copy, Debug)]
pub struct AgentScoreResult {
    pub scores: AgentScoreOutput,
    pub last_choice: BatchTacticalChoice,
}

#[derive(Resource, Default)]
pub struct GpuAgentScoringPipeline;

pub fn agent_batch_cpu_score_system(
    world_f: Res<WorldFields>,
    pressure: Res<PressureField>,
    mut commands: Commands,
    q: Query<(Entity, &Agent), With<AgentCpuBatchScoring>>,
) {
    let sample = WorldPressureSample::from_resources(&world_f, &pressure);
    for (e, agent) in q.iter() {
        let scores = score_agent_cpu(&agent.traits, &agent.emotional_state, &sample);
        let last_choice = resolve_agent_action(scores);
        commands
            .entity(e)
            .insert(AgentScoreResult { scores, last_choice });
    }
}

pub struct AgentBatchScoringPlugin;

impl Plugin for AgentBatchScoringPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GpuAgentScoringPipeline>()
            .add_systems(Update, agent_batch_cpu_score_system);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::behavior_entities::{AgentMode, CognitiveState};

    #[test]
    fn resolve_prefers_highest_score_channel() {
        let s = AgentScoreOutput {
            aggression: 0.2,
            cooperation: 0.9,
            fear: 0.1,
            stability: 0.3,
        };
        assert_eq!(resolve_agent_action(s), BatchTacticalChoice::Trade);
    }

    #[test]
    fn batch_system_writes_result_component() {
        let mut world = World::new();
        world.init_resource::<WorldFields>();
        world.init_resource::<PressureField>();
        let mut schedule = Schedule::default();
        schedule.add_systems(agent_batch_cpu_score_system);
        let e = world
            .spawn((
                Agent {
                    id: Entity::PLACEHOLDER,
                    traits: HybridAgentTraits {
                        empathy: 0.8,
                        ..Default::default()
                    },
                    emotional_state: HybridAgentEmotions {
                        confidence: 0.5,
                        ..Default::default()
                    },
                    cognition: CognitiveState::default(),
                    mode: AgentMode::Free,
                },
                AgentCpuBatchScoring,
            ))
            .id();
        schedule.run(&mut world);
        let r = world.entity(e).get::<AgentScoreResult>().expect("result");
        assert_eq!(r.last_choice, BatchTacticalChoice::Trade);
    }
}
