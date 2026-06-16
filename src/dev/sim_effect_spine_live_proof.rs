//! SIM-EFFECT-SPINE-001 — lib refresh for `debug_runs/sim_effect_spine_live.json` + JSONL ledger.

use bevy::prelude::{App, IntoScheduleConfigs, MinimalPlugins, Update};
use bevy::ecs::message::Messages;

use crate::dev::runtime_witness::sim_effects::commit_sim_effect_spine_live_proof_unchecked;
use crate::sim::effects::{
    build_sim_effect_spine_proof_payload, drain_sim_effect_queue_system, sim_effect_spine_lib_witness_green,
    PlayerEventLog, SimEffectEvent, SimEffectFactionReactWitness, SimEffectKind, SimEffectQueue,
    SimEffectSource, SimEffectSpineWitness, SimEffectTelemetryLedger, SIM_EFFECTS_JSONL,
};
use crate::strategic::{
    apply_sim_effect_telemetry_faction_stress_system, FractureEventBus, PressureField,
};
use crate::substrate::hydrology::HydrologyEventQueue;
use crate::systems::fire::EmberSpotIgnitionEvent;
use crate::systems::sim_control::SimTick;
use crate::terrain::ChunkCellKey;

#[must_use]
pub fn sim_effect_spine_proof_state() -> (
    SimEffectSpineWitness,
    SimEffectQueue,
    SimEffectTelemetryLedger,
    SimEffectFactionReactWitness,
) {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SimEffectQueue>()
        .init_resource::<HydrologyEventQueue>()
        .init_resource::<SimEffectTelemetryLedger>()
        .init_resource::<SimEffectSpineWitness>()
        .init_resource::<SimEffectFactionReactWitness>()
        .init_resource::<PlayerEventLog>()
        .init_resource::<PressureField>()
        .init_resource::<FractureEventBus>()
        .init_resource::<SimTick>()
        .add_message::<EmberSpotIgnitionEvent>()
        .add_systems(
            Update,
            (
                drain_sim_effect_queue_system,
                apply_sim_effect_telemetry_faction_stress_system,
            )
                .chain(),
        );

    app.world_mut()
        .resource_mut::<SimEffectTelemetryLedger>()
        .reset_run_id("LIB-SIM-EFFECT-LIVE-001");

    {
        let mut q = app.world_mut().resource_mut::<SimEffectQueue>();
        q.push(SimEffectEvent {
            source: SimEffectSource::Lightning,
            cause_id: "CAUSE-lightning-live-22".into(),
            parent_effect_id: None,
            kind: SimEffectKind::LightningStrike {
                chunk: bevy::math::IVec2::new(2, 2),
                cell_indices: vec![0],
                spark: 0.31,
            },
        });
        q.push(SimEffectEvent {
            source: SimEffectSource::SimEffectTest,
            cause_id: "CAUSE-ignite-live-71".into(),
            parent_effect_id: Some(0),
            kind: SimEffectKind::IgniteCells {
                cells: vec![(ChunkCellKey::new(bevy::math::IVec2::new(2, 2), 0), 0.14)],
            },
        });
        q.push(SimEffectEvent {
            source: SimEffectSource::GridOverload,
            cause_id: "CAUSE-grid-live-99".into(),
            parent_effect_id: None,
            kind: SimEffectKind::StructureHeat {
                chunk: bevy::math::IVec2::new(5, 5),
                cells: vec![(0, 0.4)],
            },
        });
    }

    app.update();
    assert!(
        app.world().resource::<Messages<EmberSpotIgnitionEvent>>().len() >= 2,
        "fixture must drain ember events"
    );

    let witness = app.world().resource::<SimEffectSpineWitness>().clone();
    let queue = app.world().resource::<SimEffectQueue>().clone();
    let ledger = app.world().resource::<SimEffectTelemetryLedger>().clone();
    let faction_react = app.world().resource::<SimEffectFactionReactWitness>().clone();
    (witness, queue, ledger, faction_react)
}

/// Writes live JSON + JSONL when lib self-check passes (ignores runtime witness gate).
#[must_use]
pub fn refresh_sim_effect_spine_live_witness() -> bool {
    if !sim_effect_spine_lib_witness_green() {
        return false;
    }
    let (witness, queue, ledger, faction_react) = sim_effect_spine_proof_state();
    if !witness.queue_drain_ok || ledger.causal_chain_depth_max() < 1 {
        return false;
    }
    if !faction_react.wired || faction_react.hook_rows < 1 {
        return false;
    }
    let json_ok = commit_sim_effect_spine_live_proof_unchecked(
        &witness,
        &queue,
        &ledger,
        &faction_react,
    );
    let jsonl_ok = ledger
        .export_jsonl(std::path::Path::new(SIM_EFFECTS_JSONL))
        .is_ok()
        && std::path::Path::new(SIM_EFFECTS_JSONL).exists();
    json_ok && jsonl_ok
}

#[must_use]
pub fn sim_effect_spine_live_proof_body_green() -> bool {
    let (witness, queue, ledger, faction_react) = sim_effect_spine_proof_state();
    let body = build_sim_effect_spine_proof_payload(&witness, &queue, &ledger, Some(&faction_react));
    body["sim_effect_spine"]["queue_drain_ok"].as_bool() == Some(true)
        && body["sim_effect_spine"]["causal_chain_depth_max"]
            .as_u64()
            .is_some_and(|d| d >= 1)
        && body["faction_react_wired"].as_bool() == Some(true)
        && body["faction_react_hook_rows"].as_u64().is_some_and(|n| n >= 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::runtime_witness::sim_effects::SIM_EFFECT_SPINE_JSON;

    #[test]
    fn sim_effects_spine_live_witness_refresh_green() {
        assert!(refresh_sim_effect_spine_live_witness());
    }

    #[test]
    fn sim_effects_spine_live_json_and_jsonl_exist() {
        assert!(refresh_sim_effect_spine_live_witness());
        assert!(
            std::path::Path::new(SIM_EFFECT_SPINE_JSON).exists(),
            "expected {SIM_EFFECT_SPINE_JSON}"
        );
        assert!(
            std::path::Path::new(SIM_EFFECTS_JSONL).exists(),
            "expected {SIM_EFFECTS_JSONL}"
        );
    }
}
