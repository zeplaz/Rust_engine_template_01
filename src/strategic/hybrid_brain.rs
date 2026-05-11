//! Hybrid **behavioral + statistical** brain: intent accumulation → probabilistic resolution → field feedback.
//!
//! **Emergent layer (runbook §9):** noise via [`HybridAgentTraits::instability`]; belief distortion via
//! [`HybridBeliefBias`]; emotional drift via [`hybrid_emotion_drift_system`]; internal faction conflict — stub
//! [`StateControlModel`] / [`control_variance`]; statistical smoothing via [`super::hybrid_fields::smooth`] and
//! [`super::hybrid_fields::region_stats_spatial_smoothing_system`].
//!
//! - Agents **do not** own world truth — they nudge [`WorldIntentField`].
//! - [`WorldFields`](super::WorldFields) **bias** resolution, not individual actions.
//! - Schedule: [`super::hybrid_fields::region_stats_spatial_smoothing_system`] in **PreUpdate**;
//!   intent reset → **emotion drift** → contributions in **Update** via [`crate::strategic::sim::HybridSimPipeline`];
//!   [`hybrid_resolve_and_feedback_system`] in **PostUpdate** (after overlay coupling in **Update**).

use bevy::prelude::*;
use rand::Rng;

use super::hybrid_fields::{smooth, WorldFields};

/// Frame counter / future scratch for deterministic streams.
#[derive(Resource, Default)]
pub struct HybridSimScratch {
    pub frame_counter: u64,
}

// --- Intent (aggregated probability pressure) ---

/// Global intent **accumulator** for a tick; cleared at frame start, filled by agents, then resolved.
///
/// Maps to runbook `war` / `trade` / `revolt` / `cooperation` as scalar **weights** (field names use
/// `*_probability` historically — resolution treats them as nonnegative pressure, not calibrated probabilities).
#[derive(Resource, Clone, Debug)]
pub struct WorldIntentField {
    pub war_probability: f32,
    pub trade_probability: f32,
    pub revolt_probability: f32,
    pub cooperation_probability: f32,
}

impl Default for WorldIntentField {
    fn default() -> Self {
        Self {
            war_probability: 0.0,
            trade_probability: 0.0,
            revolt_probability: 0.0,
            cooperation_probability: 0.0,
        }
    }
}

impl WorldIntentField {
    pub fn clear_accumulation(&mut self) {
        *self = Self::default();
    }

    pub fn clamp_nonnegative(&mut self) {
        self.war_probability = self.war_probability.max(0.0);
        self.trade_probability = self.trade_probability.max(0.0);
        self.revolt_probability = self.revolt_probability.max(0.0);
        self.cooperation_probability = self.cooperation_probability.max(0.0);
    }
}

/// Traits driving perturbation (0…1 typical). Used only inside scoring / intent contributions, never as direct actions.
#[derive(Clone, Copy, Debug)]
pub struct HybridAgentTraits {
    pub ambition: f32,
    pub paranoia: f32,
    pub rationality: f32,
    /// Runbook “cruelty” — pushes war intent with anger.
    pub cruelty: f32,
    /// Scales `territorial_gain` in [`agent_decision_score`].
    pub nationalism: f32,
    /// Softens cooperation intent when high.
    pub empathy: f32,
    /// Legacy aggressiveness — prefer [`Self::cruelty`] for war pressure; kept for callers / blends.
    pub aggression: f32,
    /// Scales decision noise amplitude.
    pub instability: f32,
    /// Runbook — low values penalize risky (`exposure`) actions more in [`agent_decision_score`].
    pub risk_tolerance: f32,
}

impl Default for HybridAgentTraits {
    fn default() -> Self {
        Self {
            ambition: 0.0,
            paranoia: 0.0,
            rationality: 0.0,
            cruelty: 0.0,
            nationalism: 0.0,
            empathy: 0.5,
            aggression: 0.0,
            instability: 0.0,
            risk_tolerance: 0.5,
        }
    }
}

/// Emotional state multipliers (0…1 typical).
#[derive(Clone, Copy, Debug, Default)]
pub struct HybridAgentEmotions {
    pub fear: f32,
    pub anger: f32,
    pub confidence: f32,
    /// Accumulated workload / exhaustion (runbook §2.3).
    pub fatigue: f32,
}

/// Sparse ECS tag: attach to entities that participate in hybrid intent sampling.
#[derive(Component, Clone, Debug)]
pub struct HybridBrainSample {
    pub traits: HybridAgentTraits,
    pub emotions: HybridAgentEmotions,
}

/// Distorted view of global fields from an agent’s psychology.
#[derive(Clone, Copy, Debug, Default)]
pub struct Perception {
    pub war_risk: f32,
    pub opportunity: f32,
    pub instability: f32,
}

#[inline]
pub fn perceive_world(world: &WorldFields, traits: &HybridAgentTraits) -> Perception {
    Perception {
        war_risk: world.war_tension * traits.paranoia,
        opportunity: world.economic_pressure * traits.ambition,
        instability: world.instability_index * (1.0 - traits.rationality).max(0.0),
    }
}

/// Wrong-world / narrative bias scaled by paranoia (runbook §2.4 stub — single scalar distortion).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct HybridBeliefBias {
    /// Positive amplifies threat-heavy perception channels.
    pub distortion: f32,
}

#[inline]
pub fn perceive_world_biased(
    world: &WorldFields,
    traits: &HybridAgentTraits,
    belief: &HybridBeliefBias,
) -> Perception {
    let mut p = perceive_world(world, traits);
    let d = 1.0 + belief.distortion.clamp(-0.85, 0.85) * traits.paranoia;
    p.war_risk *= d;
    p.instability *= d;
    p
}

/// Runbook §5 `apply_intent` — agents contribute weights, not decisions.
#[inline]
pub fn apply_agent_intent(
    field: &mut WorldIntentField,
    traits: &HybridAgentTraits,
    emotions: &HybridAgentEmotions,
) {
    field.war_probability += traits.cruelty * emotions.anger;
    field.trade_probability += traits.rationality * emotions.confidence;
    field.revolt_probability += emotions.fear * traits.paranoia;
    field.cooperation_probability +=
        traits.rationality * traits.empathy * (1.0 - emotions.anger).max(0.0);
}

/// Runbook §4 / MVP §11 action families for `base_value(world)` bias.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HybridActionKind {
    #[default]
    War,
    Trade,
    Stabilize,
}

#[inline]
pub fn hybrid_action_base_value(kind: HybridActionKind, world: &WorldFields) -> f32 {
    match kind {
        HybridActionKind::War => world.war_tension * 0.5 + world.instability_index * 0.3,
        HybridActionKind::Trade => {
            world.economic_pressure * 0.4 + (1.0 - world.resource_scarcity) * 0.2
        }
        HybridActionKind::Stabilize => {
            world.public_sentiment * 0.3 + (1.0 - world.instability_index) * 0.2
        }
    }
}

/// Weights for scoring a candidate discrete action (stub — real actions gain richer schema later).
#[derive(Clone, Copy, Debug, Default)]
pub struct ActionWeights {
    /// Extra EV from context (runbook `base_value` is added in [`agent_decision_score_with_world`]).
    pub expected_value: f32,
    pub safety_weight: f32,
    pub aggression_weight: f32,
    pub gain: f32,
    pub exposure: f32,
    /// Couples with [`HybridAgentTraits::nationalism`] (territorial / prestige plays).
    pub territorial_gain: f32,
}

/// Runbook §4 `score_action`: fuzzy sum, no hard thresholds.
/// Use [`agent_decision_score_with_world`] when `base_value(world)` applies.
#[inline]
pub fn agent_decision_score(
    traits: &HybridAgentTraits,
    emotions: &HybridAgentEmotions,
    action: &ActionWeights,
    noise: f32,
) -> f32 {
    let base = action.expected_value;
    let emotional_bias =
        emotions.fear * action.safety_weight + emotions.anger * action.aggression_weight;
    let exposure_w = (1.0 - traits.risk_tolerance).max(0.0);
    let trait_bias = traits.ambition * action.gain
        - traits.paranoia * action.exposure * exposure_w
        + traits.nationalism * action.territorial_gain;
    let n = noise * traits.instability;
    base + emotional_bias + trait_bias + n
}

#[inline]
pub fn agent_decision_score_with_world(
    traits: &HybridAgentTraits,
    emotions: &HybridAgentEmotions,
    action: &ActionWeights,
    world: &WorldFields,
    kind: HybridActionKind,
    noise: f32,
) -> f32 {
    let base = hybrid_action_base_value(kind, world) + action.expected_value;
    let emotional_bias =
        emotions.fear * action.safety_weight + emotions.anger * action.aggression_weight;
    let exposure_w = (1.0 - traits.risk_tolerance).max(0.0);
    let trait_bias = traits.ambition * action.gain
        - traits.paranoia * action.exposure * exposure_w
        + traits.nationalism * action.territorial_gain;
    let n = noise * traits.instability;
    base + emotional_bias + trait_bias + n
}

// --- Resolution ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorldEvent {
    WarBreakout,
    TradeSurge,
    RevoltRisk,
    CooperationPulse,
    StabilityDrift,
}

/// Marginal masses before cumulative draw (runbook §7e war_chance-style telemetry).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct HybridResolutionMasses {
    pub p_war: f32,
    pub p_revolt: f32,
    pub p_trade: f32,
    pub p_coop: f32,
}

#[inline]
pub fn resolution_masses(intent: &WorldIntentField, stats: &WorldFields) -> HybridResolutionMasses {
    HybridResolutionMasses {
        p_war: (intent.war_probability * stats.war_tension * (1.0 + stats.instability_index))
            .clamp(0.0, 1.0),
        p_revolt: (intent.revolt_probability
            * stats.instability_index
            * (1.2 - stats.public_sentiment).max(0.0))
        .clamp(0.0, 1.0),
        p_trade: (intent.trade_probability * stats.economic_pressure).clamp(0.0, 1.0) * 0.5,
        p_coop: (intent.cooperation_probability * stats.public_sentiment).clamp(0.0, 1.0) * 0.35,
    }
}

#[inline]
pub fn resolve_world_state_from_masses(m: &HybridResolutionMasses, roll: f32) -> WorldEvent {
    let roll = roll.clamp(0.0, 1.0);
    let mut acc = m.p_war;
    if roll < acc {
        return WorldEvent::WarBreakout;
    }
    acc += m.p_revolt;
    if roll < acc {
        return WorldEvent::RevoltRisk;
    }
    acc += m.p_trade;
    if roll < acc {
        return WorldEvent::TradeSurge;
    }
    acc += m.p_coop;
    if roll < acc {
        return WorldEvent::CooperationPulse;
    }
    WorldEvent::StabilityDrift
}

/// Probabilistic collapse: intent × stats → at most one flagship event this tick (cumulative `roll` ∈ [0,1)).
pub fn resolve_world_state(intent: &WorldIntentField, stats: &WorldFields, roll: f32) -> WorldEvent {
    let m = resolution_masses(intent, stats);
    resolve_world_state_from_masses(&m, roll)
}

/// Single-tick debug snapshot (HUD / [`crate::strategic::sim::SimDebugView`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HybridResolutionTelemetry {
    pub masses: HybridResolutionMasses,
    pub roll: f32,
    pub resolved: WorldEvent,
}

// --- Fractured state ---

#[derive(Clone, Copy, Debug, Default)]
pub struct StateControlModel {
    pub military_control: f32,
    pub economic_control: f32,
    pub oligarch_control: f32,
    pub regional_autonomy: f32,
}

#[inline]
pub fn control_variance(m: &StateControlModel) -> f32 {
    let v = [m.military_control, m.economic_control, m.oligarch_control];
    let mean = (v[0] + v[1] + v[2]) / 3.0;
    let var: f32 = v.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / 3.0;
    var.sqrt()
}

#[inline]
pub fn fracture_pressure_exceeds(m: &StateControlModel, threshold: f32) -> bool {
    control_variance(m) > threshold
}

// --- Last resolved (HUD / downstream) ---

#[derive(Resource, Clone, Debug, Default)]
pub struct HybridSimLastResolved {
    pub event: Option<WorldEvent>,
    pub telemetry: Option<HybridResolutionTelemetry>,
}

/// Monotonic counter for N-tick / rare phases (runbook §3 — macro cadence stub).
#[derive(Resource, Clone, Debug, Default)]
pub struct HybridSimPhaseClock {
    pub tick: u64,
}

// --- Systems ---

pub fn hybrid_intent_reset_system(mut intent: ResMut<WorldIntentField>) {
    intent.clear_accumulation();
}

pub fn hybrid_agent_intent_contribution_system(
    mut intent: ResMut<WorldIntentField>,
    world: Res<WorldFields>,
    brains: Query<(&HybridBrainSample, Option<&HybridBeliefBias>)>,
) {
    for (brain, belief) in &brains {
        let _perception = match belief {
            Some(b) => perceive_world_biased(&world, &brain.traits, b),
            None => perceive_world(&world, &brain.traits),
        };
        apply_agent_intent(&mut intent, &brain.traits, &brain.emotions);
    }
    intent.clamp_nonnegative();
}

/// Runbook §2.3 emotional drift — coarse proxies until regional threat is wired per-agent.
pub fn hybrid_emotion_drift_system(
    time: Res<Time>,
    world: Res<WorldFields>,
    mut brains: Query<&mut HybridBrainSample>,
) {
    let dt = time.delta_secs().clamp(0.0, 0.25);
    let k = 0.45_f32;
    for mut brain in &mut brains {
        let paranoia = brain.traits.paranoia;
        let e = &mut brain.emotions;
        e.fear = (e.fear + world.war_tension * paranoia * k * dt).clamp(0.0, 1.0);
        e.anger = (e.anger + world.war_tension * 0.55 * k * dt).clamp(0.0, 1.0);
        e.confidence =
            (e.confidence - (1.0 - world.public_sentiment) * 0.35 * k * dt).clamp(0.0, 1.0);
        e.fatigue = (e.fatigue + world.instability_index * 0.3 * k * dt).clamp(0.0, 1.0);
        let decay = (0.92f32).powf(dt * 30.0);
        e.fear *= decay;
        e.anger *= decay;
    }
}

pub fn hybrid_phase_clock_tick_system(mut clock: ResMut<HybridSimPhaseClock>) {
    clock.tick = clock.tick.wrapping_add(1);
}

pub fn hybrid_resolve_and_feedback_system(
    intent: Res<WorldIntentField>,
    mut fields: ResMut<WorldFields>,
    mut last: ResMut<HybridSimLastResolved>,
    mut scratch: ResMut<HybridSimScratch>,
) {
    let roll: f32 = rand::thread_rng().gen();
    let masses = resolution_masses(&intent, &fields);
    let ev = resolve_world_state_from_masses(&masses, roll);
    last.telemetry = Some(HybridResolutionTelemetry {
        masses,
        roll,
        resolved: ev,
    });
    last.event = Some(ev);

    let inertia = 0.92f32;
    match ev {
        WorldEvent::WarBreakout => {
            fields.war_tension = smooth(fields.war_tension, 0.85, inertia);
            fields.instability_index = smooth(fields.instability_index, 0.65, inertia);
        }
        WorldEvent::RevoltRisk => {
            fields.instability_index = smooth(fields.instability_index, 0.7, inertia);
            fields.public_sentiment = smooth(fields.public_sentiment, 0.35, inertia);
        }
        WorldEvent::TradeSurge => {
            fields.economic_pressure = smooth(fields.economic_pressure, 0.55, inertia);
        }
        WorldEvent::CooperationPulse => {
            fields.public_sentiment = smooth(fields.public_sentiment, 0.62, inertia);
        }
        WorldEvent::StabilityDrift => {
            fields.war_tension = smooth(fields.war_tension, 0.12, 0.98);
        }
    }
    scratch.frame_counter = scratch.frame_counter.wrapping_add(1);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_war_when_roll_zero_and_nonzero_intent() {
        let intent = WorldIntentField {
            war_probability: 0.5,
            ..Default::default()
        };
        let stats = WorldFields {
            war_tension: 0.8,
            instability_index: 0.5,
            ..WorldFields::default()
        };
        assert_eq!(resolve_world_state(&intent, &stats, 0.0), WorldEvent::WarBreakout);
    }

    #[test]
    fn control_variance_high_when_weights_diverge() {
        let m = StateControlModel {
            military_control: 0.9,
            economic_control: 0.2,
            oligarch_control: 0.8,
            regional_autonomy: 0.4,
        };
        assert!(control_variance(&m) > 0.25);
        assert!(fracture_pressure_exceeds(&m, 0.2));
    }

    #[test]
    fn agent_decision_score_increases_with_ambition_and_gain() {
        let traits = HybridAgentTraits {
            ambition: 0.8,
            paranoia: 0.1,
            rationality: 0.5,
            cruelty: 0.2,
            nationalism: 0.0,
            empathy: 0.5,
            aggression: 0.2,
            instability: 0.0,
            risk_tolerance: 0.5,
        };
        let emotions = HybridAgentEmotions::default();
        let action = ActionWeights {
            expected_value: 1.0,
            safety_weight: 0.0,
            aggression_weight: 0.0,
            gain: 2.0,
            exposure: 0.0,
            territorial_gain: 0.0,
        };
        let hi = agent_decision_score(&traits, &emotions, &action, 0.0);
        let mut low_traits = traits;
        low_traits.ambition = 0.1;
        let lo = agent_decision_score(&low_traits, &emotions, &action, 0.0);
        assert!(hi > lo);
    }

    #[test]
    fn agent_decision_score_respects_nationalism_and_territorial_gain() {
        let traits = HybridAgentTraits {
            nationalism: 0.9,
            ..HybridAgentTraits::default()
        };
        let emotions = HybridAgentEmotions::default();
        let mut hi = ActionWeights::default();
        hi.territorial_gain = 2.0;
        let mut lo = ActionWeights::default();
        lo.territorial_gain = 0.0;
        assert!(agent_decision_score(&traits, &emotions, &hi, 0.0) > agent_decision_score(&traits, &emotions, &lo, 0.0));
    }

    #[test]
    fn hybrid_action_base_value_war_rises_with_tension() {
        let calm = WorldFields::default();
        let tense = WorldFields {
            war_tension: 0.95,
            instability_index: 0.9,
            ..WorldFields::default()
        };
        assert!(hybrid_action_base_value(HybridActionKind::War, &tense) > hybrid_action_base_value(HybridActionKind::War, &calm));
    }

    #[test]
    fn apply_intent_matches_runbook_weights() {
        let mut field = WorldIntentField::default();
        let traits = HybridAgentTraits {
            cruelty: 0.5,
            rationality: 0.4,
            paranoia: 0.6,
            empathy: 0.3,
            ..HybridAgentTraits::default()
        };
        let emotions = HybridAgentEmotions {
            anger: 0.8,
            confidence: 0.5,
            fear: 0.7,
            fatigue: 0.0,
        };
        apply_agent_intent(&mut field, &traits, &emotions);
        assert!((field.war_probability - 0.4).abs() < 1e-5); // cruelty * anger
        assert!((field.trade_probability - 0.2).abs() < 1e-5); // rationality * confidence
        assert!((field.revolt_probability - 0.42).abs() < 1e-5); // fear * paranoia
    }

    #[test]
    fn resolution_masses_expose_war_pressure() {
        let intent = WorldIntentField {
            war_probability: 1.0,
            ..Default::default()
        };
        let w = WorldFields::default();
        let m = resolution_masses(&intent, &w);
        assert!(m.p_war > 0.0);
    }

    #[test]
    fn perceive_biased_amplifies_war_risk_when_distortion_positive() {
        let world = WorldFields {
            war_tension: 0.5,
            ..WorldFields::default()
        };
        let traits = HybridAgentTraits {
            paranoia: 0.8,
            ..HybridAgentTraits::default()
        };
        let base = perceive_world(&world, &traits).war_risk;
        let biased = perceive_world_biased(
            &world,
            &traits,
            &HybridBeliefBias { distortion: 0.5 },
        )
        .war_risk;
        assert!(biased > base);
    }

    #[test]
    fn low_risk_tolerance_penalizes_exposure() {
        let hi_risk = HybridAgentTraits {
            risk_tolerance: 0.9,
            paranoia: 0.8,
            ..HybridAgentTraits::default()
        };
        let mut lo_risk = hi_risk;
        lo_risk.risk_tolerance = 0.1;
        let emotions = HybridAgentEmotions::default();
        let action = ActionWeights {
            exposure: 2.0,
            territorial_gain: 0.0,
            expected_value: 0.0,
            ..Default::default()
        };
        assert!(
            agent_decision_score(&hi_risk, &emotions, &action, 0.0)
                > agent_decision_score(&lo_risk, &emotions, &action, 0.0)
        );
    }
}
