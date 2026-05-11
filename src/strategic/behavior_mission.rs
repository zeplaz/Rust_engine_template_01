//! **Mission = pressure package** — goal + pressure profile + participants + duration; **no** logic trees.
//!
//! Success lines on missions are **readouts** for tooling/HUD (evaluate-only), never enforcement triggers.

use bevy::prelude::*;

use super::behavior_entities::Faction;
use super::behavior_pressure::{PressureField, PressureProfile};
use super::behavior_script::ScriptInfluence;
use super::hybrid_brain::HybridAgentTraits;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MissionPressure {
    pub aggression_bias: f32,
    pub cooperation_bias: f32,
    pub fear_bias: f32,
    pub instability_bias: f32,
    pub paranoia_bias: f32,
}

impl MissionPressure {
    #[inline]
    pub fn is_effective(self) -> bool {
        self.aggression_bias != 0.0
            || self.cooperation_bias != 0.0
            || self.fear_bias != 0.0
            || self.instability_bias != 0.0
            || self.paranoia_bias != 0.0
    }

    /// Narrative pressure folded into [`ScriptInfluence::bias_vector`] (weather, not orders).
    pub fn accumulate_into_traits(self, target: &mut HybridAgentTraits) {
        target.aggression += self.aggression_bias;
        target.empathy += self.cooperation_bias;
        target.instability += self.instability_bias;
        target.paranoia += self.paranoia_bias;
        target.risk_tolerance -= self.fear_bias * 0.25;
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MissionId(pub u64);

/// One goal — reward/penalty **pressure** + optional emotional bias (designer data; systems interpret later).
#[derive(Clone, Debug, Default)]
pub struct Objective {
    pub label: String,
    pub reward_pressure: f32,
    pub penalty_pressure: f32,
    /// Narrative salience into agent cognition / emotion (interpretation layer — stub scalar).
    pub emotional_bias_hint: f32,
}

/// Mission bundles participants + [`ScriptInfluence`] + optional **global** [`PressureProfile`] injection.
#[derive(Clone, Debug)]
pub struct Mission {
    pub id: MissionId,
    pub participants: Vec<Entity>,
    pub objectives: Vec<Objective>,
    /// Designer “weather” biases (mission does not script outcomes).
    pub pressure: MissionPressure,
    /// Optional chunks to prioritize for worldgen / preview (see [`crate::terrain::generation::ChunkGenMissionChunkHints`]).
    pub influenced_chunks: Vec<IVec2>,
    pub influence: ScriptInfluence,
    /// None ⇒ runs until removed from [`ActiveMissions`]. Elapsed advances once per frame this plugin runs.
    pub duration_ticks: Option<u64>,
    pub ticks_elapsed: u64,
    /// Fed into [`PressureField`] while active (climate / “weather system”).
    pub global_pressure: PressureProfile,
    /// Optional label for tooling: “success” is evaluated elsewhere, **not** triggered here.
    pub success_readout_label: Option<String>,
}

impl Mission {
    pub fn new(id: MissionId, participants: Vec<Entity>, influence: ScriptInfluence) -> Self {
        Self {
            id,
            participants,
            objectives: Vec::new(),
            pressure: MissionPressure::default(),
            influenced_chunks: Vec::new(),
            influence,
            duration_ticks: None,
            ticks_elapsed: 0,
            global_pressure: PressureProfile::default(),
            success_readout_label: None,
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct ActiveMissions {
    pub missions: Vec<Mission>,
}

/// Remove missions that exceeded [`Mission::duration_ticks`] **before** this frame’s pressure apply.
pub fn active_missions_expire_system(mut active: ResMut<ActiveMissions>) {
    active.missions.retain(|m| {
        m.duration_ticks
            .map_or(true, |d| m.ticks_elapsed < d)
    });
}

/// Decay previous field, then accumulate each active mission’s [`Mission::global_pressure`].
pub fn pressure_field_from_active_missions_system(
    mut field: ResMut<PressureField>,
    missions: Res<ActiveMissions>,
) {
    field.relax(0.05);
    for m in &missions.missions {
        let scale = m.influence.priority.clamp(0.0, 1.0).max(0.05);
        field.accumulate(&m.global_pressure, scale);
    }
}

/// Applies mission [`ScriptInfluence`] to participant entities (**overwrite** component each tick — simple v1).
/// Copies mission [`Mission::global_pressure`] into [`ScriptInfluence::pressure_profile`] for inspector visibility.
pub fn narrative_mission_influence_apply_system(
    mut commands: Commands,
    missions: Res<ActiveMissions>,
) {
    for m in &missions.missions {
        for &p in &m.participants {
            let mut inf = m.influence.clone();
            if !m.objectives.is_empty() {
                let sum_r: f32 = m.objectives.iter().map(|o| o.reward_pressure).sum();
                let sum_p: f32 = m.objectives.iter().map(|o| o.penalty_pressure).sum();
                inf.priority = (inf.priority + (sum_r - sum_p) * 0.02).clamp(0.0, 1.0);
            }
            if m.pressure.is_effective() {
                m.pressure.accumulate_into_traits(&mut inf.bias_vector);
                inf.priority = (inf.priority + 0.2).clamp(0.0, 1.0);
            }
            inf.pressure_profile = m.global_pressure;
            commands.entity(p).insert(inf);
        }
    }
}

pub fn active_missions_advance_elapsed_system(mut active: ResMut<ActiveMissions>) {
    for m in &mut active.missions {
        m.ticks_elapsed = m.ticks_elapsed.saturating_add(1);
    }
}

/// Tooling readout only — evaluates optional text hints (e.g. `cohesion < 0.3`) against participant **faction** entities.
#[must_use]
pub fn mission_success_readout_note(
    mission: &Mission,
    factions: &Query<&Faction>,
    participant_entities: &[Entity],
    world_instability: f32,
) -> String {
    let mut min_coh = f32::MAX;
    let mut n = 0usize;
    for &p in participant_entities {
        if let Ok(f) = factions.get(p) {
            n += 1;
            min_coh = min_coh.min(f.cohesion);
        }
    }
    let thresh = mission
        .success_readout_label
        .as_deref()
        .and_then(parse_cohesion_hint);
    let mut out = String::new();
    if let Some(ref l) = mission.success_readout_label {
        out.push_str(l);
        out.push_str(" → ");
    }
    if n == 0 {
        out.push_str("participants lack `Faction` (pick faction entities or extend to Agent+Link)");
        return out;
    }
    let coh = min_coh;
    let p_frag = (1.0 - coh) * world_instability.clamp(0.0, 1.0);
    out.push_str(&format!(
        "min cohesion {:.2} | heuristic ~{:.2}",
        coh,
        p_frag.clamp(0.0, 1.0)
    ));
    if let Some(t) = thresh {
        out.push_str(&format!(" | hint < {:.2}? {}", t, coh < t));
    }
    out
}

fn parse_cohesion_hint(label: &str) -> Option<f32> {
    let low = label.to_ascii_lowercase();
    if !low.contains("cohesion") {
        return None;
    }
    for part in low.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-')) {
        if part.is_empty() {
            continue;
        }
        if let Ok(v) = part.parse::<f32>() {
            return Some(v);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::behavior_script::ScriptInfluence;

    #[test]
    fn mission_pumps_pressure_then_expires() {
        let mut world = World::new();
        world.init_resource::<ActiveMissions>();
        world.init_resource::<PressureField>();
        let p = world.spawn(()).id();
        world.resource_mut::<ActiveMissions>().missions.push(Mission {
            id: MissionId(1),
            participants: vec![p],
            objectives: vec![],
            pressure: MissionPressure::default(),
            influenced_chunks: vec![],
            influence: ScriptInfluence {
                priority: 1.0,
                ..Default::default()
            },
            duration_ticks: Some(1),
            ticks_elapsed: 0,
            global_pressure: PressureProfile {
                instability: 0.5,
                ..Default::default()
            },
            success_readout_label: None,
        });

        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                active_missions_expire_system,
                pressure_field_from_active_missions_system,
                active_missions_advance_elapsed_system,
            )
                .chain(),
        );
        schedule.run(&mut world);

        let mid = world.resource::<PressureField>().instability;
        assert!(mid > 0.15, "mission should raise global instability channel");

        schedule.run(&mut world);
        assert!(
            world.resource::<ActiveMissions>().missions.is_empty(),
            "mission should be removed after duration"
        );
    }

    #[test]
    fn mission_pressure_accumulates_into_script_bias() {
        let mut world = World::new();
        world.init_resource::<ActiveMissions>();
        let p = world.spawn(()).id();
        world.resource_mut::<ActiveMissions>().missions.push(Mission {
            id: MissionId(2),
            participants: vec![p],
            objectives: vec![],
            pressure: MissionPressure {
                aggression_bias: 0.15,
                ..Default::default()
            },
            influenced_chunks: vec![],
            influence: ScriptInfluence {
                priority: 0.5,
                ..Default::default()
            },
            duration_ticks: None,
            ticks_elapsed: 0,
            global_pressure: PressureProfile::default(),
            success_readout_label: None,
        });
        let mut schedule = Schedule::default();
        schedule.add_systems(narrative_mission_influence_apply_system);
        schedule.run(&mut world);
        let inf = world.entity(p).get::<ScriptInfluence>().expect("ScriptInfluence");
        assert!(
            inf.bias_vector.aggression > 0.1,
            "aggression bias should accumulate from MissionPressure"
        );
        assert!(inf.priority > 0.5);
    }
}
