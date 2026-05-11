//! **Fracture overlay** — secondary instability lens (stress fractures, not the main sim driver).
//!
//! - Emits **soft** [`FractureEvent`]s and informational heuristics only.
//! - Does **not** own war spawning, map topology, or guaranteed splits. See [`FractureOverlaySettings`].

use std::collections::HashMap;

use bevy::prelude::*;

use super::behavior_emergence_log::{format_fracture_log_line, StrategicEmergenceLog};
use super::behavior_entities::{Agent, AgentFactionLink, Faction, FactionInternalStage};
use super::behavior_pressure::PressureField;
use super::hybrid_brain::HybridSimPhaseClock;
use super::hybrid_fields::WorldFields;

// --- Legacy shorthand signal (HUD / quick taps); prefer [`FractureEvent`] for sim semantics. ---

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FractureType {
    Political,
    Economic,
    Military,
    Ideological,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FractureSignal {
    pub source: Entity,
    pub strength: f32,
    pub type_: FractureType,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct FractureSignalBus {
    pub pending: Vec<FractureSignal>,
}

impl FractureSignalBus {
    #[inline]
    pub fn push(&mut self, signal: FractureSignal) {
        self.pending.push(signal);
    }

    pub fn drain(&mut self) -> Vec<FractureSignal> {
        std::mem::take(&mut self.pending)
    }
}

// --- Event-driven fracture (locked design) ---

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FractureDriver {
    EconomicCollapse,
    IdeologicalSplit,
    MilitaryCoupPressure,
    OligarchCapture,
    /// Mission climate overloaded faction stress (never a guaranteed split).
    MissionPressureOverflow,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FractureEvent {
    pub faction: Entity,
    pub pressure: f32,
    pub drivers: Vec<FractureDriver>,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct FractureEventBus {
    pub pending: Vec<FractureEvent>,
}

impl FractureEventBus {
    #[inline]
    pub fn push(&mut self, ev: FractureEvent) {
        self.pending.push(ev);
    }

    pub fn drain(&mut self) -> Vec<FractureEvent> {
        std::mem::take(&mut self.pending)
    }
}

/// Tunables for lightweight / dev-only fracture hooks (no strategic authority on map or wars).
#[derive(Resource, Clone, Debug)]
pub struct FractureOverlaySettings {
    /// When false, [`FractureEvent`]s are still logged; [`SubFactionStub`] entities are not spawned.
    pub spawn_sub_faction_stub_entities: bool,
}

impl Default for FractureOverlaySettings {
    fn default() -> Self {
        Self {
            spawn_sub_faction_stub_entities: true,
        }
    }
}

/// Read-only **informational** rollup for UI / tooling (not a hard simulation gate).
#[derive(Resource, Clone, Debug, Default)]
pub struct FractureProbabilityOverlay {
    /// Mean per-faction heuristic in \[0, 1\].
    pub mean_heuristic: f32,
    pub max_heuristic: f32,
}

/// Aggregate instability lens from cohesion, loyalty spread, control, and world stress.
pub fn fracture_probability_overlay_system(
    world: Res<WorldFields>,
    members: Query<&AgentFactionLink, With<Agent>>,
    factions: Query<(Entity, &Faction), With<Faction>>,
    mut overlay: ResMut<FractureProbabilityOverlay>,
) {
    let instab = world.instability_index.clamp(0.0, 1.0);
    let economic = world.economic_pressure.clamp(0.0, 1.0);
    let mut sum = 0.0_f32;
    let mut n = 0_usize;
    let mut max_h = 0.0_f32;
    for (f_ent, fac) in factions.iter() {
        let loyalties: Vec<f32> = members
            .iter()
            .filter(|l| l.faction == f_ent)
            .map(|l| l.loyalty)
            .collect();
        let cohesion_term = (1.0 - fac.cohesion).clamp(0.0, 1.0);
        let spread_term = if loyalties.len() >= 2 {
            let m = loyalties.len() as f32;
            let mean = loyalties.iter().copied().sum::<f32>() / m;
            let var: f32 = loyalties.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / m;
            var.sqrt().clamp(0.0, 1.0)
        } else {
            0.0
        };
        let control_term = (1.0 - fac.control_strength).clamp(0.0, 1.0);
        let h = (cohesion_term * 0.42 + spread_term * 0.28 + control_term * 0.2 + economic * 0.1) * instab;
        let h = h.clamp(0.0, 1.0);
        sum += h;
        n += 1;
        max_h = max_h.max(h);
    }
    overlay.mean_heuristic = if n > 0 { sum / n as f32 } else { 0.0 };
    overlay.max_heuristic = max_h;
}

#[derive(Component, Clone, Debug)]
pub struct SubFactionStub {
    pub parent_faction: Entity,
    pub fracture_type: FractureType,
}

/// **Meso** cadence — slow resource / control drift (not every frame).
pub fn faction_meso_internal_tick_system(
    clock: Res<HybridSimPhaseClock>,
    mut factions: Query<&mut Faction>,
) {
    if clock.tick % 5 != 0 && clock.tick != 0 {
        return;
    }
    for mut f in factions.iter_mut() {
        f.resources = (f.resources - 0.2).max(0.0);
        let drift = 0.012 * (1.0 - f.cohesion);
        f.control_strength = (f.control_strength - drift).clamp(0.0, 1.0);
    }
}

/// Cohesion shrinks when loyalty dispersion × global instability is high.
pub fn faction_cohesion_pressure_system(
    world: Res<WorldFields>,
    pressure_field: Res<PressureField>,
    members: Query<&AgentFactionLink, With<Agent>>,
    mut factions: Query<(Entity, &mut Faction), With<Faction>>,
) {
    let instab = world.instability_index.clamp(0.0, 1.0);
    let cohesion_pull = pressure_field.cohesion_drift.clamp(0.0, 1.0) * 0.012;
    for (f_ent, mut fac) in factions.iter_mut() {
        let loyalties: Vec<f32> = members
            .iter()
            .filter(|l| l.faction == f_ent)
            .map(|l| l.loyalty)
            .collect();
        if loyalties.is_empty() {
            continue;
        }
        let n = loyalties.len() as f32;
        let mean = loyalties.iter().copied().sum::<f32>() / n;
        let var: f32 = loyalties.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / n;
        let spread = var.sqrt();
        let pressure = spread.clamp(0.0, 1.0) * instab * (1.0 - fac.cohesion).max(0.0);
        fac.cohesion = (fac.cohesion - pressure * 0.02 - cohesion_pull).clamp(0.0, 1.0);
    }
}

/// Stages 1→3 before topology hook fires ([`FactionInternalStage`]).
pub fn faction_internal_stage_system(mut factions: Query<&mut Faction>) {
    for mut f in factions.iter_mut() {
        if f.cohesion < 0.55 && f.internal_stage == FactionInternalStage::Unified {
            f.internal_stage = FactionInternalStage::Divergence;
        } else if f.cohesion < 0.4 && f.internal_stage == FactionInternalStage::Divergence {
            f.internal_stage = FactionInternalStage::Autonomy;
        } else if f.cohesion < 0.28 && f.internal_stage == FactionInternalStage::Autonomy {
            f.internal_stage = FactionInternalStage::Split;
        }
    }
}

#[derive(Resource, Default)]
pub struct FractureSignalScratch {
    pub prev_cohesion: HashMap<Entity, f32>,
}

#[derive(Resource, Default)]
pub struct FractureStageScratch {
    pub prev_stage: HashMap<Entity, FactionInternalStage>,
}

/// Emit [`FractureEvent`] (+ legacy signal) on cohesion threshold cross **or** entering [`Split`].
pub fn fracture_event_emit_system(
    world: Res<WorldFields>,
    q: Query<(Entity, &Faction)>,
    mut signal_bus: ResMut<FractureSignalBus>,
    mut event_bus: ResMut<FractureEventBus>,
    mut coh_scratch: ResMut<FractureSignalScratch>,
    mut stage_scratch: ResMut<FractureStageScratch>,
) {
    const THRESH: f32 = 0.28;
    for (e, f) in q.iter() {
        let prev_c = coh_scratch.prev_cohesion.get(&e).copied().unwrap_or(1.0);
        let prev_s = *stage_scratch
            .prev_stage
            .get(&e)
            .unwrap_or(&FactionInternalStage::Unified);

        let crossed = prev_c >= THRESH && f.cohesion < THRESH;
        let split_now = f.internal_stage == FactionInternalStage::Split && prev_s != FactionInternalStage::Split;

        if crossed || split_now {
            let mut drivers = Vec::new();
            if world.economic_pressure > 0.65 {
                drivers.push(FractureDriver::EconomicCollapse);
            }
            drivers.push(FractureDriver::IdeologicalSplit);
            if f.control_strength < 0.35 {
                drivers.push(FractureDriver::MilitaryCoupPressure);
            }
            if f.resources < 30.0 {
                drivers.push(FractureDriver::OligarchCapture);
            }

            let pressure = ((THRESH - f.cohesion).max(0.0) + if split_now { 0.35 } else { 0.0 }).min(1.0);

            event_bus.push(FractureEvent {
                faction: e,
                pressure,
                drivers: drivers.clone(),
            });
            signal_bus.push(FractureSignal {
                source: e,
                strength: pressure,
                type_: FractureType::Political,
            });
        }

        coh_scratch.prev_cohesion.insert(e, f.cohesion);
        stage_scratch.prev_stage.insert(e, f.internal_stage);
    }
}

/// **Stub only** — consumes [`FractureEventBus`] (event-first). Does not spawn wars or kill entities.
pub fn sub_faction_stub_hook_system(
    settings: Res<FractureOverlaySettings>,
    mut event_bus: ResMut<FractureEventBus>,
    mut log: ResMut<StrategicEmergenceLog>,
    mut commands: Commands,
) {
    for ev in event_bus.drain() {
        log.push(format_fracture_log_line(&ev));
        if settings.spawn_sub_faction_stub_entities {
            commands.spawn(SubFactionStub {
                parent_faction: ev.faction,
                fracture_type: FractureType::Ideological,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::behavior_entities::{
        Agent, AgentFactionLink, AgentMode, CognitiveState, FactionInternalStage,
    };
    use crate::strategic::behavior_pressure::PressureField;
    use crate::strategic::hybrid_brain::{HybridAgentEmotions, HybridAgentTraits};

    #[test]
    fn cohesion_drops_when_loyalties_diverge() {
        let mut world = World::new();
        world.init_resource::<WorldFields>();
        world.init_resource::<PressureField>();
        world.resource_mut::<WorldFields>().instability_index = 1.0;

        let f = world
            .spawn(Faction {
                id: Entity::PLACEHOLDER,
                cohesion: 0.9,
                ideology: vec![],
                resources: 100.0,
                internal_blocks: vec![],
                control_strength: 0.7,
                internal_stage: FactionInternalStage::Unified,
                sub_factions: vec![],
            })
            .id();

        for loyalty in [0.2_f32, 0.95] {
            world.spawn((
                Agent {
                    id: Entity::PLACEHOLDER,
                    traits: HybridAgentTraits::default(),
                    emotional_state: HybridAgentEmotions::default(),
                    cognition: CognitiveState::default(),
                    mode: AgentMode::Free,
                },
                AgentFactionLink {
                    agent: Entity::PLACEHOLDER,
                    faction: f,
                    loyalty,
                    influence: 0.5,
                    autonomy: 0.3,
                },
            ));
        }

        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                crate::strategic::behavior_entities::behavior_sync_entity_ids_system,
                faction_cohesion_pressure_system,
            )
                .chain(),
        );
        schedule.run(&mut world);

        let fac = world.entity(f).get::<Faction>().expect("faction");
        assert!(fac.cohesion < 0.9, "cohesion should drop under loyalty spread + instability");
    }

    #[test]
    fn fracture_event_on_threshold_cross() {
        let mut world = World::new();
        world.init_resource::<WorldFields>();
        world.init_resource::<FractureSignalBus>();
        world.init_resource::<FractureEventBus>();
        world.init_resource::<FractureSignalScratch>();
        world.init_resource::<FractureStageScratch>();

        let f = world
            .spawn(Faction {
                id: Entity::PLACEHOLDER,
                cohesion: 0.2,
                ideology: vec![],
                resources: 100.0,
                internal_blocks: vec![],
                control_strength: 0.7,
                internal_stage: FactionInternalStage::Unified,
                sub_factions: vec![],
            })
            .id();

        let mut schedule = Schedule::default();
        schedule.add_systems(fracture_event_emit_system);
        schedule.run(&mut world);

        let bus = world.resource::<FractureEventBus>();
        assert!(bus.pending.iter().any(|ev| ev.faction == f));
    }

    #[test]
    fn stub_hook_drains_fracture_events() {
        let mut world = World::new();
        world.init_resource::<FractureEventBus>();
        world.init_resource::<crate::strategic::StrategicEmergenceLog>();
        world.init_resource::<FractureOverlaySettings>();
        let src = world.spawn(()).id();
        world.resource_mut::<FractureEventBus>().push(FractureEvent {
            faction: src,
            pressure: 0.5,
            drivers: vec![FractureDriver::IdeologicalSplit],
        });
        let mut schedule = Schedule::default();
        schedule.add_systems(sub_faction_stub_hook_system);
        schedule.run(&mut world);
        assert!(world.resource::<FractureEventBus>().pending.is_empty());

        let mut q = world.query::<&SubFactionStub>();
        assert_eq!(q.iter(&world).count(), 1);
    }

    #[test]
    fn stub_hook_skips_entity_spawn_when_disabled() {
        let mut world = World::new();
        world.init_resource::<FractureEventBus>();
        world.init_resource::<crate::strategic::StrategicEmergenceLog>();
        world.insert_resource(FractureOverlaySettings {
            spawn_sub_faction_stub_entities: false,
        });
        let src = world.spawn(()).id();
        world.resource_mut::<FractureEventBus>().push(FractureEvent {
            faction: src,
            pressure: 0.4,
            drivers: vec![FractureDriver::IdeologicalSplit],
        });
        let mut schedule = Schedule::default();
        schedule.add_systems(sub_faction_stub_hook_system);
        schedule.run(&mut world);
        assert!(world.resource::<FractureEventBus>().pending.is_empty());
        assert_eq!(world.query::<&SubFactionStub>().iter(&world).count(), 0);
    }

    #[test]
    fn fracture_probability_overlay_is_informational() {
        let mut world = World::new();
        world.init_resource::<WorldFields>();
        world.init_resource::<PressureField>();
        world.insert_resource(FractureOverlaySettings::default());
        world.init_resource::<crate::strategic::StrategicEmergenceLog>();
        world.insert_resource(FractureProbabilityOverlay::default());
        let f = world
            .spawn(Faction {
                id: Entity::PLACEHOLDER,
                cohesion: 0.3,
                ideology: vec![],
                resources: 80.0,
                internal_blocks: vec![],
                control_strength: 0.6,
                internal_stage: FactionInternalStage::Unified,
                sub_factions: vec![],
            })
            .id();
        world.spawn((
            Agent {
                id: Entity::PLACEHOLDER,
                traits: HybridAgentTraits::default(),
                emotional_state: HybridAgentEmotions::default(),
                cognition: CognitiveState::default(),
                mode: AgentMode::Free,
            },
            AgentFactionLink {
                agent: Entity::PLACEHOLDER,
                faction: f,
                loyalty: 0.1,
                influence: 0.5,
                autonomy: 0.3,
            },
        ));

        let mut schedule = Schedule::default();
        schedule.add_systems(fracture_probability_overlay_system);
        schedule.run(&mut world);

        let overlay = world.resource::<FractureProbabilityOverlay>();
        assert!(
            overlay.mean_heuristic > 0.0 && overlay.mean_heuristic <= 1.0,
            "mean stays in (0,1] for stressed faction"
        );
        assert!(
            overlay.max_heuristic > 0.0 && overlay.max_heuristic <= 1.0,
            "max stays in (0,1]"
        );
    }
}
