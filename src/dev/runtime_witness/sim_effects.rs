//! Sim effect spine live proof — `debug_runs/sim_effect_spine_live.json`.

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::sim::effects::{
    build_sim_effect_spine_proof_payload, SimEffectFactionReactWitness, SimEffectQueue,
    SimEffectSpineWitness, SimEffectTelemetryLedger, SIM_EFFECTS_JSONL,
};

use super::io::{write_enveloped_witness, write_enveloped_witness_unchecked};

pub const SIM_EFFECT_SPINE_JSON: &str = "debug_runs/sim_effect_spine_live.json";

fn commit_sim_effect_spine_live_proof_inner(
    witness: &SimEffectSpineWitness,
    queue: &SimEffectQueue,
    ledger: &SimEffectTelemetryLedger,
    faction_react: &SimEffectFactionReactWitness,
    write_json: impl FnOnce(&str, &str, &str, serde_json::Value) -> bool,
) -> bool {
    let json_ok = write_json(
        "SIM-EFFECT-SPINE-001",
        "sim_effect_spine_live_proof",
        SIM_EFFECT_SPINE_JSON,
        build_sim_effect_spine_proof_payload(witness, queue, ledger, Some(faction_react)),
    );
    let jsonl_ok = ledger
        .export_jsonl(std::path::Path::new(SIM_EFFECTS_JSONL))
        .is_ok();
    json_ok && jsonl_ok
}

#[must_use]
pub fn commit_sim_effect_spine_live_proof(
    witness: &SimEffectSpineWitness,
    queue: &SimEffectQueue,
    ledger: &SimEffectTelemetryLedger,
    faction_react: &SimEffectFactionReactWitness,
) -> bool {
    commit_sim_effect_spine_live_proof_inner(witness, queue, ledger, faction_react, write_enveloped_witness)
}

#[must_use]
pub fn commit_sim_effect_spine_live_proof_unchecked(
    witness: &SimEffectSpineWitness,
    queue: &SimEffectQueue,
    ledger: &SimEffectTelemetryLedger,
    faction_react: &SimEffectFactionReactWitness,
) -> bool {
    commit_sim_effect_spine_live_proof_inner(
        witness,
        queue,
        ledger,
        faction_react,
        write_enveloped_witness_unchecked,
    )
}

pub fn write_sim_effect_spine_live_proof_system(
    base: Option<Res<State<BaseState>>>,
    witness: Res<SimEffectSpineWitness>,
    queue: Res<SimEffectQueue>,
    ledger: Res<SimEffectTelemetryLedger>,
    faction_react: Res<SimEffectFactionReactWitness>,
) {
    if !matches!(base.as_deref().map(|s| s.get()), Some(BaseState::Simulation)) {
        return;
    }
    if !witness.queue_drain_ok && ledger.effect_rows == 0 {
        return;
    }
    let _ = commit_sim_effect_spine_live_proof(
        witness.as_ref(),
        queue.as_ref(),
        ledger.as_ref(),
        faction_react.as_ref(),
    );
}

#[must_use]
pub fn refresh_sim_effect_spine_live_witness() -> bool {
    crate::dev::sim_effect_spine_live_proof::refresh_sim_effect_spine_live_witness()
}
