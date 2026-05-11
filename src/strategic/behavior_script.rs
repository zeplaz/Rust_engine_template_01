//! **Authority layer (pressure only)** — scripts/missions **never** move agents, spawn wars, or force outcomes.
//! They attach [`ScriptInfluence`] (bias, rare intent weights, filters) consumed by the decision pipeline.

use bevy::prelude::*;

use super::behavior_pressure::PressureProfile;
use super::hybrid_brain::HybridAgentTraits;

/// Rare narrative nudge mapped into the same **weight** vocabulary as [`super::hybrid_brain::WorldIntentField`]
/// (simulation still resolves — no guaranteed outcome).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum IntentChannel {
    WarPressure,
    TradePressure,
    RevoltPressure,
    CooperatePressure,
}

/// Weighted channel — **not** a discrete “do this action” order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScriptedIntentWeight {
    pub channel: IntentChannel,
    pub weight: f32,
}

/// Filter stub — DSL / scenario conditions attach here later.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Condition {
    pub tag: &'static str,
}

/// Narrative / designer pressure bundle (attach to agent or copy from [`super::behavior_mission::Mission`]).
#[derive(Component, Clone, Debug, Default)]
pub struct ScriptInfluence {
    /// 0…1 — scales how strongly bias + intents apply vs pure simulation.
    pub priority: f32,
    /// Added into scoring bias (scaled by [`AgentMode`](super::behavior_entities::AgentMode)).
    pub bias_vector: HybridAgentTraits,
    /// Per-agent / tooling “pressure card” (feeds decision pipeline — not a quest trigger).
    pub pressure_profile: PressureProfile,
    pub forced_intents: Vec<ScriptedIntentWeight>,
    pub context_filters: Vec<Condition>,
}

impl ScriptInfluence {
    /// Effective bias for [`AgentMode::Hybrid`]: `sim + (script - sim) * script_weight * priority`.
    #[inline]
    pub fn blend_traits(
        sim_traits: &HybridAgentTraits,
        script_weight: f32,
        priority: f32,
        bias: &HybridAgentTraits,
    ) -> HybridAgentTraits {
        let w = (script_weight * priority).clamp(0.0, 1.0);
        HybridAgentTraits {
            ambition: clamp01(lerp(sim_traits.ambition, sim_traits.ambition + bias.ambition, w)),
            paranoia: clamp01(lerp(sim_traits.paranoia, sim_traits.paranoia + bias.paranoia, w)),
            rationality: clamp01(lerp(
                sim_traits.rationality,
                sim_traits.rationality + bias.rationality,
                w,
            )),
            cruelty: clamp01(lerp(sim_traits.cruelty, sim_traits.cruelty + bias.cruelty, w)),
            nationalism: clamp01(lerp(
                sim_traits.nationalism,
                sim_traits.nationalism + bias.nationalism,
                w,
            )),
            empathy: clamp01(lerp(sim_traits.empathy, sim_traits.empathy + bias.empathy, w)),
            aggression: clamp01(lerp(
                sim_traits.aggression,
                sim_traits.aggression + bias.aggression,
                w,
            )),
            instability: clamp01(lerp(
                sim_traits.instability,
                sim_traits.instability + bias.instability,
                w,
            )),
            risk_tolerance: clamp01(lerp(
                sim_traits.risk_tolerance,
                sim_traits.risk_tolerance + bias.risk_tolerance,
                w,
            )),
        }
    }
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a.mul_add(1.0 - t, b * t)
}

#[inline]
fn clamp01(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}
