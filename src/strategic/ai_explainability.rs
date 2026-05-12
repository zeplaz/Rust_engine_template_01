//! L7 **AI / pipeline explainability** — snapshots for egui tooling (`simulation_explainability_runbook_v1.md`).
//!
//! Surfaces **interpreted** contributor lines, not raw tuning coefficients.

use bevy::prelude::*;

use crate::systems::sim_control::SimTick;

use super::behavior_entities::{Agent, AgentFactionLink, Faction};
use super::behavior_pipeline::{
    compose_decision_score, sample_decision_components, DecisionScoreComponents,
};
use super::behavior_pressure::PressureField;
use super::behavior_script::ScriptInfluence;
use super::hybrid_brain::{HybridResolutionTelemetry, WorldEvent};
use super::hybrid_fields::WorldFields;

/// Latest **micro** decision sample (per-agent pipeline composition).
#[derive(Resource, Clone, Debug, Default)]
pub struct DecisionExplainabilitySnapshot {
    pub sim_tick: u64,
    pub sample_entity: Option<Entity>,
    pub components: Option<DecisionScoreComponents>,
    pub composed: f32,
    pub pipeline_contributors: Vec<String>,
}

#[must_use]
pub fn format_pipeline_contributors(c: &DecisionScoreComponents, total: f32) -> Vec<String> {
    if total.abs() < 1e-5 {
        return vec!["Pipeline pulse ~neutral — components cancel or agents idle.".into()];
    }
    let pct = |x: f32| (100.0 * x / total).clamp(-999.0, 999.0);
    let mut v = Vec::new();
    if c.base_traits > 1e-4 {
        v.push(format!(
            "+ strategic posture (traits) — ~{:.0}% of this pulse",
            pct(c.base_traits)
        ));
    }
    if c.emotional > 1e-4 {
        v.push(format!(
            "+ emotional stance (confidence / fear / anger) — ~{:.0}%",
            pct(c.emotional)
        ));
    }
    if c.faction_pressure > 1e-4 {
        v.push(format!(
            "+ faction stress (cohesion / control / resources) — ~{:.0}%",
            pct(c.faction_pressure)
        ));
    }
    if c.script_influence > 1e-4 {
        v.push(format!(
            "+ mission / script cue (priority & forced intents) — ~{:.0}%",
            pct(c.script_influence)
        ));
    }
    if c.environmental > 1e-4 {
        v.push(format!(
            "+ environment & field pressure (world + cards) — ~{:.0}%",
            pct(c.environmental)
        ));
    }
    if v.is_empty() {
        v.push("All composition channels near zero — check `Agent` + world fields.".into());
    }
    v
}

#[must_use]
pub fn format_hybrid_telemetry_explain(t: &HybridResolutionTelemetry) -> Vec<String> {
    let m = t.masses;
    vec![
        format!(
            "Macro draw → {:?} (uniform draw {:.2} in cumulative mass ordering)",
            t.resolved, t.roll
        ),
        format!(
            "War pathway — {:.2} (intent × tension × instability; expansionist / crisis read)",
            m.p_war
        ),
        format!(
            "Revolt pathway — {:.2} (intent × instability × dissent margin)",
            m.p_revolt
        ),
        format!(
            "Trade pulse — {:.2} (intent × economic pressure; logistics relief read)",
            m.p_trade
        ),
        format!(
            "Cooperation pulse — {:.2} (intent × public sentiment; stabilization read)",
            m.p_coop
        ),
        explain_event_readout(t.resolved),
    ]
}

#[inline]
fn explain_event_readout(ev: WorldEvent) -> String {
    match ev {
        WorldEvent::WarBreakout => {
            "Readout: theaters tilting toward open contention — sustain and strike loads rise.".into()
        }
        WorldEvent::RevoltRisk => {
            "Readout: internal fracture risk climbing — garrison and legitimacy stress.".into()
        }
        WorldEvent::TradeSurge => {
            "Readout: logistics / exchange windows opening — convoy utility improves.".into()
        }
        WorldEvent::CooperationPulse => {
            "Readout: consolidation or deal-making window — coordination costs fall slightly.".into()
        }
        WorldEvent::StabilityDrift => {
            "Readout: no flagship macro event — slow drift (watch field gradients).".into()
        }
    }
}

pub fn decision_explainability_capture_system(
    tick: Option<Res<SimTick>>,
    world: Res<WorldFields>,
    pressure_field: Res<PressureField>,
    agents: Query<(Entity, &Agent, Option<&ScriptInfluence>), With<Agent>>,
    links: Query<&AgentFactionLink>,
    factions: Query<&Faction>,
    mut snap: ResMut<DecisionExplainabilitySnapshot>,
) {
    let sim_tick = tick.map(|t| t.0).unwrap_or(0);
    let mut best: Option<(Entity, DecisionScoreComponents, f32)> = None;
    for (entity, agent, script) in agents.iter() {
        let fac = links
            .iter()
            .find(|l| l.agent == entity)
            .and_then(|l| factions.get(l.faction).ok());
        let c = sample_decision_components(agent, world.as_ref(), pressure_field.as_ref(), fac, script);
        let s = compose_decision_score(&c);
        match best {
            None => best = Some((entity, c, s)),
            Some((_, _, bs)) if s > bs => best = Some((entity, c, s)),
            _ => {}
        }
    }
    if let Some((e, c, s)) = best {
        snap.sim_tick = sim_tick;
        snap.sample_entity = Some(e);
        snap.components = Some(c);
        snap.composed = s;
        snap.pipeline_contributors = format_pipeline_contributors(&c, s);
    } else {
        snap.sim_tick = sim_tick;
        snap.sample_entity = None;
        snap.components = None;
        snap.composed = 0.0;
        snap.pipeline_contributors =
            vec!["No agents with `Agent` — nothing to score this frame.".into()];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contributors_sum_narrative() {
        let c = DecisionScoreComponents {
            base_traits: 0.1,
            emotional: 0.05,
            faction_pressure: 0.02,
            script_influence: 0.03,
            environmental: 0.04,
        };
        let t = compose_decision_score(&c);
        let lines = format_pipeline_contributors(&c, t);
        assert!(lines.iter().any(|l| l.contains("strategic posture")), "{lines:?}");
    }
}
