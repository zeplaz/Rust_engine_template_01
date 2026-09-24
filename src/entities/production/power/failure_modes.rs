//! Failure and derate hooks **scoped by capability**, not by exhaustively matching every `PowerPlantType`.
//! Each system queries a marker (`SteamCycle`, `ContainmentBuilding`, `VariableRenewable`) so new fuel
//! types only need archetype + markers, not new `match` arms everywhere.
//!
//! Nuclear P1 (**COD-NUCLEAR-GRID-LINK-001** / **LOOP-SCRAM** / **DIESEL** / **EVENTS**):
//! offsite feed from [`UtilityConnection`] ↔ UtilityGraph → LOOP → auto SCRAM + diesels.
//! Nuclear P2 (**COD-NUCLEAR-COOLING-001**): diesel fail → decay heat drives [`ThermalComponent`].
//! Nuclear P3 (**COD-NUCLEAR-MELTDOWN-001**): `meltdown_threshold_reached` → [`MeltdownEvent`] →
//! containment breach + WSS fallout hook ([`ContainmentBreachEvent`]).

use bevy::prelude::*;

use crate::entities::components::Operational;
use crate::entities::production::power::capabilities::ContainmentBuilding;
use crate::entities::production::power::capabilities::{SteamCycle, VariableRenewable};
use crate::entities::production::power::components::{PowerPlant, ThermalComponent};
use crate::entities::production::power::plant_definition::NuclearFailureProfile;
use crate::entities::production::power::plant_registry::PlantDefinitionRegistry;
use crate::entities::production::power::power_states::PowerPlantType;
use crate::entities::types::{EmergencyType, OperationalStatus};
use crate::infrastructure::{UtilityConnection, UtilityNetworkKind};
use crate::systems::weather::GlobalRenewableWeatherFactors;

/// Placeholder: steam leak, condenser vacuum, feedwater chemistry — adjust `efficiency` or `status` later.
pub fn steam_system_placeholder(_query: Query<&PowerPlant, With<SteamCycle>>) {}

/// Why a containment plant SCRAMmed (P1 messages).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NuclearScramReason {
    LossOfOffsitePower,
}

/// Emitted once when auto-SCRAM completes after LOOP delay.
#[derive(Message, Debug, Clone)]
pub struct NuclearScramEvent {
    pub entity: Entity,
    pub reason: NuclearScramReason,
}

/// Emitted once when diesels fail and uncooled decay heat begins (**COD-NUCLEAR-COOLING-001**).
#[derive(Message, Debug, Clone)]
pub struct NuclearCoolingDegradedEvent {
    pub entity: Entity,
}

/// Emitted once when core heat crosses the meltdown threshold (**COD-NUCLEAR-MELTDOWN-001**).
#[derive(Message, Debug, Clone)]
pub struct MeltdownEvent {
    pub entity: Entity,
}

/// Emitted once after meltdown + [`NuclearFailureProfile::breach_delay_s`].
/// `fallout_hook` is the WSS radiation injection seam — consumed by
/// [`apply_containment_breach_radiation_hook_system`] (slab `contamination.radiation` writer).
#[derive(Message, Debug, Clone)]
pub struct ContainmentBreachEvent {
    pub entity: Entity,
    pub fallout_hook: bool,
}

/// Deterministic fallout seed written into slab `contamination.radiation` on breach.
pub const CONTAINMENT_BREACH_RADIATION_SEED: f32 = 1.0;

/// World units per chunk for plant → [`ChunkKey`] (matches `DEBUG_CHUNK_SPACING_WORLD`).
const FALLOUT_CHUNK_SPACING_WORLD: f32 = 32.0;

#[inline]
fn world_xz_to_chunk_key(x: f32, z: f32) -> crate::substrate::ChunkKey {
    let s = FALLOUT_CHUNK_SPACING_WORLD.max(1.0);
    crate::substrate::ChunkKey::new((x / s).floor() as i32, (z / s).floor() as i32)
}

/// Inject radiation into [`WorldSubstrateRegistry`] when [`ContainmentBreachEvent::fallout_hook`].
/// Sole writer for breach-driven `contamination.radiation` (WSS P3 fallout hook).
pub fn apply_containment_breach_radiation_hook_system(
    mut reader: MessageReader<ContainmentBreachEvent>,
    mut registry: Option<ResMut<crate::substrate::WorldSubstrateRegistry>>,
    transforms: Query<&Transform>,
) {
    let Some(ref mut registry) = registry else {
        return;
    };
    for ev in reader.read() {
        if !ev.fallout_hook {
            continue;
        }
        let key = transforms
            .get(ev.entity)
            .map(|tf| world_xz_to_chunk_key(tf.translation.x, tf.translation.z))
            .unwrap_or_else(|_| crate::substrate::ChunkKey::new(0, 0));
        if !registry.chunks.contains(key) {
            crate::substrate::hydrate_skeleton_chunk(
                registry.as_mut(),
                bevy::math::IVec2::new(key.x, key.y),
            );
        }
        let Some(state) = registry.chunks.get_mut(key) else {
            continue;
        };
        if state.contamination.radiation.is_empty() {
            state.contamination.radiation.push(CONTAINMENT_BREACH_RADIATION_SEED);
        } else {
            for cell in &mut state.contamination.radiation {
                *cell = (*cell).max(CONTAINMENT_BREACH_RADIATION_SEED);
            }
        }
        state.version = state.version.saturating_add(1);
        registry.chunks.set_resident(key, true);
    }
}

/// Max `contamination.radiation` across resident slabs (witness / lib proof).
#[must_use]
pub fn substrate_contamination_radiation_max(
    registry: &crate::substrate::WorldSubstrateRegistry,
) -> f32 {
    registry
        .chunks
        .chunks
        .values()
        .flat_map(|c| c.contamination.radiation.iter().copied())
        .fold(0.0_f32, f32::max)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NuclearDieselState {
    #[default]
    Off,
    Starting,
    Running,
    Failed,
}

/// Runtime state for entities with [`ContainmentBuilding`] + a failure profile.
#[derive(Component, Debug, Clone)]
pub struct NuclearContainmentRuntime {
    pub offsite_connected: bool,
    /// Seconds since LOOP detected while still waiting for SCRAM.
    pub loop_elapsed_s: f32,
    pub scrammed: bool,
    pub diesel: NuclearDieselState,
    pub diesel_start_elapsed_s: f32,
    /// Remaining diesel fuel in **hours** (game-compressed burn applies in tick).
    pub diesel_fuel_hours_remaining: f32,
    /// P2: diesels failed (or equivalent) while LOOP persists — core heat rising.
    pub cooling_degraded: bool,
    /// P2 gate — P3 [`nuclear_meltdown_breach_system`] consumes this under `meltdown_enabled`.
    pub meltdown_threshold_reached: bool,
    /// One-shot for [`NuclearCoolingDegradedEvent`].
    pub cooling_alert_emitted: bool,
    /// P3: meltdown ladder active (event emitted).
    pub meltdown_active: bool,
    /// Seconds since meltdown became active (breach timer).
    pub meltdown_elapsed_s: f32,
    /// P3: containment breached + fallout hook emitted.
    pub containment_breached: bool,
    /// One-shot for [`MeltdownEvent`].
    pub meltdown_event_emitted: bool,
    /// One-shot for [`ContainmentBreachEvent`].
    pub breach_event_emitted: bool,
}

impl NuclearContainmentRuntime {
    #[must_use]
    pub fn from_profile(profile: &NuclearFailureProfile) -> Self {
        Self {
            offsite_connected: true,
            loop_elapsed_s: 0.0,
            scrammed: false,
            diesel: NuclearDieselState::Off,
            diesel_start_elapsed_s: 0.0,
            diesel_fuel_hours_remaining: profile.diesel_fuel_hours.max(0.0),
            cooling_degraded: false,
            meltdown_threshold_reached: false,
            cooling_alert_emitted: false,
            meltdown_active: false,
            meltdown_elapsed_s: 0.0,
            containment_breached: false,
            meltdown_event_emitted: false,
            breach_event_emitted: false,
        }
    }
}

fn thermal_from_profile(profile: &NuclearFailureProfile) -> ThermalComponent {
    ThermalComponent {
        current_temperature: profile.core_stable_temp_c.max(0.0),
        max_temperature: profile.meltdown_threshold_c.max(profile.core_stable_temp_c.max(1.0)),
    }
}

/// Attach [`NuclearContainmentRuntime`] + [`ThermalComponent`] when a containment plant resolves a failure profile.
pub fn attach_nuclear_containment_runtime(
    mut commands: Commands,
    defs: Res<PlantDefinitionRegistry>,
    q: Query<(Entity, &PowerPlant), (With<ContainmentBuilding>, Without<NuclearContainmentRuntime>)>,
) {
    for (entity, plant) in &q {
        let Some(profile) = nuclear_profile_for(plant, &defs) else {
            continue;
        };
        commands.entity(entity).insert((
            NuclearContainmentRuntime::from_profile(profile),
            thermal_from_profile(profile),
        ));
    }
}

/// Sync offsite feed from [`UtilityConnection`] (UtilityGraph reachability).
/// **COD-NUCLEAR-GRID-LINK-001** — no radius hack; coal/gas without `ContainmentBuilding` never enter.
pub fn sync_nuclear_offsite_from_utility(
    mut q: Query<
        (&mut NuclearContainmentRuntime, Option<&UtilityConnection>),
        With<ContainmentBuilding>,
    >,
) {
    for (mut runtime, utility) in &mut q {
        runtime.offsite_connected = match utility {
            Some(c) if c.kind == UtilityNetworkKind::Power => c.connected,
            // No utility link yet (editor / pre-activation): treat as connected to avoid false SCRAM.
            _ => true,
        };
    }
}

/// LOOP detect → SCRAM delay → auto SCRAM + diesel start (**COD-NUCLEAR-LOOP-SCRAM-001** + DIESEL).
pub fn nuclear_loop_scram_system(
    time: Res<Time>,
    defs: Res<PlantDefinitionRegistry>,
    mut events: MessageWriter<NuclearScramEvent>,
    mut q: Query<(
        Entity,
        &mut PowerPlant,
        &mut NuclearContainmentRuntime,
    ), With<ContainmentBuilding>>,
) {
    let dt = time.delta_secs().max(0.0);
    if dt <= 0.0 {
        return;
    }

    for (entity, mut plant, mut runtime) in &mut q {
        let Some(profile) = nuclear_profile_for(&plant, &defs) else {
            continue;
        };
        if !profile.scram_on_loop {
            continue;
        }

        let needs_offsite = status_requires_offsite(plant.status, profile);
        let lost_offsite = needs_offsite && !runtime.offsite_connected;

        if !runtime.scrammed {
            if lost_offsite {
                runtime.loop_elapsed_s += dt;
                if runtime.loop_elapsed_s >= profile.scram_delay_s.max(0.0) {
                    plant.status = OperationalStatus::ExternalShutdown;
                    plant.current_output = 0.0;
                    runtime.scrammed = true;
                    runtime.diesel = NuclearDieselState::Starting;
                    runtime.diesel_start_elapsed_s = 0.0;
                    events.write(NuclearScramEvent {
                        entity,
                        reason: NuclearScramReason::LossOfOffsitePower,
                    });
                }
            } else {
                runtime.loop_elapsed_s = 0.0;
            }
            continue;
        }

        // Post-SCRAM diesel window.
        match runtime.diesel {
            NuclearDieselState::Starting => {
                runtime.diesel_start_elapsed_s += dt;
                if runtime.diesel_start_elapsed_s >= profile.diesel_start_delay_s.max(0.0) {
                    runtime.diesel = if runtime.diesel_fuel_hours_remaining > 0.0 {
                        NuclearDieselState::Running
                    } else {
                        NuclearDieselState::Failed
                    };
                }
            }
            NuclearDieselState::Running => {
                if runtime.offsite_connected {
                    // Offsite restored — diesels stand down; plant stays SCRAMmed until operator restart.
                    runtime.diesel = NuclearDieselState::Off;
                } else {
                    let burn_h = (dt / 3600.0) * profile.diesel_time_compress.max(1.0);
                    runtime.diesel_fuel_hours_remaining =
                        (runtime.diesel_fuel_hours_remaining - burn_h).max(0.0);
                    if runtime.diesel_fuel_hours_remaining <= 0.0 {
                        runtime.diesel = NuclearDieselState::Failed;
                    }
                }
            }
            NuclearDieselState::Off | NuclearDieselState::Failed => {}
        }
    }
}

/// **COD-NUCLEAR-COOLING-001** — after SCRAM, drive [`ThermalComponent`] from decay heat vs cooling.
/// Cooling available: passive profile, offsite restored, or diesels Starting/Running.
/// Cooling lost: diesels Failed while still islanded → core heat rises toward meltdown threshold (P2 flag only).
pub fn nuclear_decay_heat_cooling_system(
    time: Res<Time>,
    defs: Res<PlantDefinitionRegistry>,
    mut degraded: MessageWriter<NuclearCoolingDegradedEvent>,
    mut q: Query<
        (
            Entity,
            &PowerPlant,
            &mut NuclearContainmentRuntime,
            &mut ThermalComponent,
        ),
        With<ContainmentBuilding>,
    >,
) {
    let dt = time.delta_secs().max(0.0);
    if dt <= 0.0 {
        return;
    }

    for (entity, plant, mut runtime, mut thermal) in &mut q {
        let Some(profile) = nuclear_profile_for(plant, &defs) else {
            continue;
        };
        if !runtime.scrammed {
            continue;
        }

        let cooling_ok = cooling_available(profile, &runtime);
        runtime.cooling_degraded = !cooling_ok;

        let stable = profile.core_stable_temp_c.max(0.0);
        let threshold = profile
            .meltdown_threshold_c
            .max(stable + 1.0);
        thermal.max_temperature = threshold;

        if cooling_ok {
            let cool = profile.decay_heat_cool_c_per_s.max(0.0)
                * profile.decay_heat_time_compress.max(1.0)
                * dt;
            if thermal.current_temperature > stable {
                thermal.current_temperature =
                    (thermal.current_temperature - cool).max(stable);
            } else {
                thermal.current_temperature = stable;
            }
            continue;
        }

        // Uncooled decay heat — P2 sets threshold flag only; P3 emits MeltdownEvent.
        if !runtime.cooling_alert_emitted {
            runtime.cooling_alert_emitted = true;
            degraded.write(NuclearCoolingDegradedEvent { entity });
        }

        let rise = profile.decay_heat_rise_c_per_s.max(0.0)
            * profile.decay_heat_time_compress.max(1.0)
            * dt;
        thermal.current_temperature =
            (thermal.current_temperature + rise).min(threshold);

        if profile.meltdown_enabled && thermal.current_temperature >= threshold - f32::EPSILON {
            runtime.meltdown_threshold_reached = true;
        }
    }
}

/// **COD-NUCLEAR-MELTDOWN-001** — act on `meltdown_threshold_reached` under `meltdown_enabled`.
/// Emits [`MeltdownEvent`], then after `breach_delay_s` emits [`ContainmentBreachEvent`] (WSS fallout hook).
/// Authority: [`ContainmentBuilding`] + profile only — never bare `PowerPlantType::Nuclear`.
pub fn nuclear_meltdown_breach_system(
    time: Res<Time>,
    defs: Res<PlantDefinitionRegistry>,
    mut meltdown_events: MessageWriter<MeltdownEvent>,
    mut breach_events: MessageWriter<ContainmentBreachEvent>,
    mut q: Query<
        (
            Entity,
            &PowerPlant,
            &mut NuclearContainmentRuntime,
            Option<&mut Operational>,
        ),
        With<ContainmentBuilding>,
    >,
) {
    let dt = time.delta_secs().max(0.0);
    if dt <= 0.0 {
        return;
    }

    for (entity, plant, mut runtime, operational) in &mut q {
        let Some(profile) = nuclear_profile_for(plant, &defs) else {
            continue;
        };
        if !profile.meltdown_enabled {
            continue;
        }
        if !runtime.meltdown_threshold_reached {
            continue;
        }

        if !runtime.meltdown_event_emitted {
            runtime.meltdown_event_emitted = true;
            runtime.meltdown_active = true;
            runtime.meltdown_elapsed_s = 0.0;
            meltdown_events.write(MeltdownEvent { entity });
        }

        if runtime.containment_breached {
            continue;
        }

        runtime.meltdown_elapsed_s += dt;
        if runtime.meltdown_elapsed_s < profile.breach_delay_s.max(0.0) {
            continue;
        }

        runtime.containment_breached = true;
        runtime.breach_event_emitted = true;
        if let Some(mut op) = operational {
            op.emergencies.insert(EmergencyType::ReactorBreach);
        }
        breach_events.write(ContainmentBreachEvent {
            entity,
            fallout_hook: true,
        });
    }
}

fn cooling_available(profile: &NuclearFailureProfile, runtime: &NuclearContainmentRuntime) -> bool {
    if profile.passive_cooling {
        return true;
    }
    if runtime.offsite_connected {
        return true;
    }
    matches!(
        runtime.diesel,
        NuclearDieselState::Starting | NuclearDieselState::Running
    )
}

fn nuclear_profile_for<'a>(
    plant: &PowerPlant,
    defs: &'a PlantDefinitionRegistry,
) -> Option<&'a NuclearFailureProfile> {
    if plant.definition_id.is_empty() {
        return None;
    }
    defs.get(plant.definition_id.as_str())
        .and_then(|d| d.nuclear_failure_profile.as_ref())
}

fn status_requires_offsite(status: OperationalStatus, profile: &NuclearFailureProfile) -> bool {
    if profile.requires_offsite_power_when.is_empty() {
        matches!(
            status,
            OperationalStatus::Standby
                | OperationalStatus::Operational
                | OperationalStatus::ReducedCapacity
                | OperationalStatus::OverCapacity
                | OperationalStatus::StartingUp
        )
    } else {
        profile.requires_offsite_power_when.contains(&status)
    }
}

/// Cloud / wind coupling via world-averaged chunk weather (see `GlobalRenewableWeatherFactors`).
pub fn variable_renewable_placeholder(
    factors: Res<GlobalRenewableWeatherFactors>,
    mut query: Query<&mut PowerPlant, With<VariableRenewable>>,
) {
    for mut plant in &mut query {
        let m = match plant.plant_type {
            PowerPlantType::Wind => factors.wind_capacity_factor,
            PowerPlantType::Solar => factors.solar_capacity_factor,
            _ => 1.0,
        };
        plant.current_output *= m.clamp(0.05, 1.2);
    }
}

/// Project [`NuclearScramEvent`] into the player event log (P1 alert copy).
pub fn project_nuclear_scram_to_player_event_log(
    mut reader: MessageReader<NuclearScramEvent>,
    tick: Option<Res<crate::systems::sim_control::SimTick>>,
    log: Option<ResMut<crate::sim::effects::PlayerEventLog>>,
) {
    let Some(mut log) = log else {
        return;
    };
    let tick = tick.map(|t| t.0).unwrap_or(0);
    for ev in reader.read() {
        crate::sim::effects::push_player_event_row(
            &mut log,
            crate::sim::effects::PlayerEventRow {
                tick,
                category: crate::sim::effects::PlayerEventCategory::Grid,
                severity: crate::sim::effects::PlayerEventSeverity::Warn,
                target_ref: format!("entity({})", ev.entity),
                label: "Nuclear · offsite power lost — SCRAM initiated".into(),
                effect_id: 0,
                parent_id: None,
                dispatch_ok: true,
            },
        );
    }
}

/// Project [`NuclearCoolingDegradedEvent`] — critical alert before meltdown (P2).
pub fn project_nuclear_cooling_degraded_to_player_event_log(
    mut reader: MessageReader<NuclearCoolingDegradedEvent>,
    tick: Option<Res<crate::systems::sim_control::SimTick>>,
    log: Option<ResMut<crate::sim::effects::PlayerEventLog>>,
) {
    let Some(mut log) = log else {
        return;
    };
    let tick = tick.map(|t| t.0).unwrap_or(0);
    for ev in reader.read() {
        crate::sim::effects::push_player_event_row(
            &mut log,
            crate::sim::effects::PlayerEventRow {
                tick,
                category: crate::sim::effects::PlayerEventCategory::Grid,
                severity: crate::sim::effects::PlayerEventSeverity::Crit,
                target_ref: format!("entity({})", ev.entity),
                label: "Nuclear · core cooling degraded · restore power".into(),
                effect_id: 0,
                parent_id: None,
                dispatch_ok: true,
            },
        );
    }
}

/// Project [`MeltdownEvent`] — emergency alert (P3).
pub fn project_meltdown_to_player_event_log(
    mut reader: MessageReader<MeltdownEvent>,
    tick: Option<Res<crate::systems::sim_control::SimTick>>,
    log: Option<ResMut<crate::sim::effects::PlayerEventLog>>,
) {
    let Some(mut log) = log else {
        return;
    };
    let tick = tick.map(|t| t.0).unwrap_or(0);
    for ev in reader.read() {
        crate::sim::effects::push_player_event_row(
            &mut log,
            crate::sim::effects::PlayerEventRow {
                tick,
                category: crate::sim::effects::PlayerEventCategory::Grid,
                severity: crate::sim::effects::PlayerEventSeverity::Crit,
                target_ref: format!("entity({})", ev.entity),
                label: "Nuclear · meltdown in progress · evacuate district".into(),
                effect_id: 0,
                parent_id: None,
                dispatch_ok: true,
            },
        );
    }
}

/// Project [`ContainmentBreachEvent`] — catastrophic + fallout hook notice (P3).
/// Radiation slab write is owned by [`apply_containment_breach_radiation_hook_system`].
pub fn project_containment_breach_to_player_event_log(
    mut reader: MessageReader<ContainmentBreachEvent>,
    tick: Option<Res<crate::systems::sim_control::SimTick>>,
    log: Option<ResMut<crate::sim::effects::PlayerEventLog>>,
) {
    let Some(mut log) = log else {
        return;
    };
    let tick = tick.map(|t| t.0).unwrap_or(0);
    for ev in reader.read() {
        crate::sim::effects::push_player_event_row(
            &mut log,
            crate::sim::effects::PlayerEventRow {
                tick,
                category: crate::sim::effects::PlayerEventCategory::Grid,
                severity: crate::sim::effects::PlayerEventSeverity::Crit,
                target_ref: format!("entity({})", ev.entity),
                label: "Containment breach · radiation release".into(),
                effect_id: 0,
                parent_id: None,
                dispatch_ok: true,
            },
        );
        let _ = ev.fallout_hook;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::production::power::capabilities::attach_power_plant_capabilities;
    use crate::entities::production::power::plant_definition::{
        CapabilityFlags, NuclearFailureProfile, PlantDefinition,
    };
    use crate::entities::production::power::power_states::PowerPlantType;
    use bevy::time::TimeUpdateStrategy;
    use std::time::Duration;

    fn test_profile() -> NuclearFailureProfile {
        NuclearFailureProfile {
            requires_offsite_power_when: vec![
                OperationalStatus::Operational,
                OperationalStatus::Standby,
            ],
            offsite_power_mw: 12.0,
            diesel_backup_mw: 10.0,
            diesel_fuel_hours: 72.0,
            passive_cooling: false,
            scram_on_loop: true,
            meltdown_enabled: true,
            scram_delay_s: 1.0,
            diesel_start_delay_s: 0.5,
            diesel_time_compress: 60.0,
            core_stable_temp_c: 80.0,
            meltdown_threshold_c: 1200.0,
            decay_heat_rise_c_per_s: 0.2,
            decay_heat_time_compress: 60.0,
            decay_heat_cool_c_per_s: 0.05,
            breach_delay_s: 45.0,
        }
    }

    fn cooling_fail_profile() -> NuclearFailureProfile {
        NuclearFailureProfile {
            diesel_fuel_hours: 0.001,
            diesel_start_delay_s: 0.1,
            scram_delay_s: 0.2,
            diesel_time_compress: 3600.0,
            decay_heat_rise_c_per_s: 20.0,
            decay_heat_time_compress: 1.0,
            core_stable_temp_c: 80.0,
            meltdown_threshold_c: 400.0,
            ..test_profile()
        }
    }

    fn test_nuclear_def(id: &str, profile: NuclearFailureProfile) -> PlantDefinition {
        PlantDefinition {
            id: id.into(),
            display_name: "Test PWR".into(),
            plant_type: PowerPlantType::Nuclear,
            narrative: Default::default(),
            output_model: Default::default(),
            operational: Default::default(),
            instance_template: Default::default(),
            capabilities: CapabilityFlags {
                is_steam_cycle: true,
                is_nuclear_containment: true,
                is_variable_renewable: false,
            },
            emissions: Default::default(),
            economics: Default::default(),
            research: Default::default(),
            grid_interface: Default::default(),
            nuclear_failure_profile: Some(profile),
        }
    }

    #[test]
    fn loop_triggers_scram_on_containment_only() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(250)))
            .init_resource::<PlantDefinitionRegistry>()
            .add_message::<NuclearScramEvent>()
            .add_systems(
                Update,
                (
                    attach_power_plant_capabilities,
                    attach_nuclear_containment_runtime,
                    sync_nuclear_offsite_from_utility,
                    nuclear_loop_scram_system,
                )
                    .chain(),
            );

        {
            let mut defs = app.world_mut().resource_mut::<PlantDefinitionRegistry>();
            defs.insert_for_test(test_nuclear_def("pwr_test_loop", test_profile()));
        }

        let nuclear = app
            .world_mut()
            .spawn((
                PowerPlant {
                    definition_id: "pwr_test_loop".into(),
                    plant_type: PowerPlantType::Nuclear,
                    max_output: 1100.0,
                    current_output: 800.0,
                    status: OperationalStatus::Operational,
                    efficiency: 1.0,
                },
                UtilityConnection::power(1, 0.1, false),
            ))
            .id();

        let coal = app
            .world_mut()
            .spawn(PowerPlant {
                definition_id: String::new(),
                plant_type: PowerPlantType::Coal,
                max_output: 500.0,
                current_output: 400.0,
                status: OperationalStatus::Operational,
                efficiency: 1.0,
            })
            .id();

        for _ in 0..8 {
            app.update();
        }

        let plant = app.world().get::<PowerPlant>(nuclear).unwrap();
        assert_eq!(plant.status, OperationalStatus::ExternalShutdown);
        assert!(app.world().get::<NuclearContainmentRuntime>(nuclear).unwrap().scrammed);
        assert!(app.world().get::<ThermalComponent>(nuclear).is_some());

        let coal_plant = app.world().get::<PowerPlant>(coal).unwrap();
        assert_eq!(coal_plant.status, OperationalStatus::Operational);
        assert!(app.world().get::<NuclearContainmentRuntime>(coal).is_none());
    }

    #[test]
    fn diesel_fail_raises_core_heat_on_thermal() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(200)))
            .init_resource::<PlantDefinitionRegistry>()
            .add_message::<NuclearScramEvent>()
            .add_message::<NuclearCoolingDegradedEvent>()
            .add_systems(
                Update,
                (
                    attach_power_plant_capabilities,
                    attach_nuclear_containment_runtime,
                    sync_nuclear_offsite_from_utility,
                    nuclear_loop_scram_system,
                    nuclear_decay_heat_cooling_system,
                )
                    .chain(),
            );

        {
            let mut defs = app.world_mut().resource_mut::<PlantDefinitionRegistry>();
            defs.insert_for_test(test_nuclear_def("pwr_cool_fail", cooling_fail_profile()));
        }

        let nuclear = app
            .world_mut()
            .spawn((
                PowerPlant {
                    definition_id: "pwr_cool_fail".into(),
                    plant_type: PowerPlantType::Nuclear,
                    max_output: 1100.0,
                    current_output: 800.0,
                    status: OperationalStatus::Operational,
                    efficiency: 1.0,
                },
                UtilityConnection::power(1, 0.1, false),
            ))
            .id();

        for _ in 0..40 {
            app.update();
        }

        let runtime = app.world().get::<NuclearContainmentRuntime>(nuclear).unwrap();
        let thermal = app.world().get::<ThermalComponent>(nuclear).unwrap();
        assert!(runtime.scrammed);
        assert_eq!(runtime.diesel, NuclearDieselState::Failed);
        assert!(runtime.cooling_degraded);
        assert!(thermal.current_temperature > 80.0);
        assert!(runtime.cooling_alert_emitted);
    }

    #[test]
    fn meltdown_threshold_emits_event_then_breach() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(200)))
            .init_resource::<PlantDefinitionRegistry>()
            .init_resource::<crate::substrate::WorldSubstrateRegistry>()
            .add_message::<NuclearScramEvent>()
            .add_message::<NuclearCoolingDegradedEvent>()
            .add_message::<MeltdownEvent>()
            .add_message::<ContainmentBreachEvent>()
            .add_systems(
                Update,
                (
                    attach_power_plant_capabilities,
                    attach_nuclear_containment_runtime,
                    sync_nuclear_offsite_from_utility,
                    nuclear_loop_scram_system,
                    nuclear_decay_heat_cooling_system,
                    nuclear_meltdown_breach_system,
                    apply_containment_breach_radiation_hook_system,
                )
                    .chain(),
            );

        let mut melt_profile = cooling_fail_profile();
        melt_profile.meltdown_threshold_c = 200.0;
        melt_profile.decay_heat_rise_c_per_s = 40.0;
        melt_profile.breach_delay_s = 0.4;
        {
            let mut defs = app.world_mut().resource_mut::<PlantDefinitionRegistry>();
            defs.insert_for_test(test_nuclear_def("pwr_melt", melt_profile));
        }

        let nuclear = app
            .world_mut()
            .spawn((
                PowerPlant {
                    definition_id: "pwr_melt".into(),
                    plant_type: PowerPlantType::Nuclear,
                    max_output: 1100.0,
                    current_output: 800.0,
                    status: OperationalStatus::Operational,
                    efficiency: 1.0,
                },
                UtilityConnection::power(1, 0.1, false),
                Transform::from_xyz(4.0, 0.0, 4.0),
                Operational {
                    maintenance_level: 1.0,
                    malfunctions: Default::default(),
                    emergencies: Default::default(),
                    operational_status: OperationalStatus::Operational,
                },
            ))
            .id();

        for _ in 0..60 {
            app.update();
        }

        let runtime = app.world().get::<NuclearContainmentRuntime>(nuclear).unwrap();
        assert!(runtime.meltdown_threshold_reached);
        assert!(runtime.meltdown_active);
        assert!(runtime.meltdown_event_emitted);
        assert!(runtime.containment_breached);
        assert!(runtime.breach_event_emitted);
        let op = app.world().get::<Operational>(nuclear).unwrap();
        assert!(op.emergencies.contains(&EmergencyType::ReactorBreach));
        let rad = substrate_contamination_radiation_max(
            app.world().resource::<crate::substrate::WorldSubstrateRegistry>(),
        );
        assert!(rad >= CONTAINMENT_BREACH_RADIATION_SEED - f32::EPSILON);
    }

    #[test]
    fn meltdown_disabled_skips_events() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(200)))
            .init_resource::<PlantDefinitionRegistry>()
            .add_message::<NuclearScramEvent>()
            .add_message::<NuclearCoolingDegradedEvent>()
            .add_message::<MeltdownEvent>()
            .add_message::<ContainmentBreachEvent>()
            .add_systems(
                Update,
                (
                    attach_power_plant_capabilities,
                    attach_nuclear_containment_runtime,
                    sync_nuclear_offsite_from_utility,
                    nuclear_loop_scram_system,
                    nuclear_decay_heat_cooling_system,
                    nuclear_meltdown_breach_system,
                )
                    .chain(),
            );

        let mut profile = cooling_fail_profile();
        profile.meltdown_enabled = false;
        profile.breach_delay_s = 0.1;
        {
            let mut defs = app.world_mut().resource_mut::<PlantDefinitionRegistry>();
            defs.insert_for_test(test_nuclear_def("pwr_no_melt", profile));
        }

        let nuclear = app
            .world_mut()
            .spawn((
                PowerPlant {
                    definition_id: "pwr_no_melt".into(),
                    plant_type: PowerPlantType::Nuclear,
                    max_output: 1100.0,
                    current_output: 800.0,
                    status: OperationalStatus::Operational,
                    efficiency: 1.0,
                },
                UtilityConnection::power(1, 0.1, false),
            ))
            .id();

        for _ in 0..80 {
            app.update();
        }

        let runtime = app.world().get::<NuclearContainmentRuntime>(nuclear).unwrap();
        assert!(!runtime.meltdown_threshold_reached);
        assert!(!runtime.meltdown_event_emitted);
        assert!(!runtime.containment_breached);
    }
}
