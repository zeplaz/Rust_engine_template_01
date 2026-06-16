//! FACTION-REACT-001 — strategic read-only consumer of SimEffect telemetry rows.
//!
//! Applies soft faction stress via [`PressureField`] + [`FractureEventBus`]; never writes the sim-effect queue/ledger.

use bevy::prelude::*;

use crate::sim::effects::{
    scan_faction_stress_rows, FactionStressTelemetryClass, SimEffectFactionReactWitness,
    SimEffectTelemetryLedger,
};

use super::behavior_entities::Faction;
use super::behavior_fracture::{FractureDriver, FractureEvent, FractureEventBus};
use super::behavior_pressure::{PressureField, PressureProfile};

/// Scan new telemetry rows after sim-effect drain; accumulate mission climate pressure only.
pub fn apply_sim_effect_telemetry_faction_stress_system(
    ledger: Res<SimEffectTelemetryLedger>,
    mut pressure: ResMut<PressureField>,
    mut fracture_bus: ResMut<FractureEventBus>,
    mut witness: ResMut<SimEffectFactionReactWitness>,
    factions: Query<Entity, With<Faction>>,
) {
    witness.wired = true;
    let cursor = witness.scan_cursor();
    let (hooks, max_id) = scan_faction_stress_rows(&ledger, cursor);
    if max_id > cursor {
        witness.advance_cursor(max_id);
    }
    if hooks.is_empty() {
        return;
    }

    let fallback_faction = factions.iter().next();
    for hook in &hooks {
        pressure.accumulate(
            &PressureProfile {
                instability: hook.severity * 0.65,
                cohesion_drift: hook.severity,
                ..default()
            },
            1.0,
        );
        if let Some(f_ent) = fallback_faction {
            let driver = match hook.class {
                FactionStressTelemetryClass::PowerLoss => FractureDriver::EconomicCollapse,
                FactionStressTelemetryClass::StructureFire => FractureDriver::MissionPressureOverflow,
                FactionStressTelemetryClass::EcologicalDisturbance => {
                    FractureDriver::MissionPressureOverflow
                }
            };
            fracture_bus.push(FractureEvent {
                faction: f_ent,
                pressure: hook.severity,
                drivers: vec![driver],
            });
        }
    }
    witness.record_hooks(hooks.len() as u64);
}
