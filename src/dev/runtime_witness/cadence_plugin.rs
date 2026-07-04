//! MIG-A7 — global arm systems for live-proof witness writers (cadence tick vs disk write).

use bevy::prelude::*;

use super::common::{arm_live_proof_cadence, arm_witness_write_cadence, LiveProofCadence};
use super::construction::ConstructionLiveProofState;
use super::economy::{IndustrialActivationLiveProofState, LogisticsThroughputLiveProofState};
use super::fire::FireEcologyLiveProofState;
use super::stage7_behavioral::Stage7BehavioralLiveProofState;
use super::stage7_play::Stage7PlayLiveProofState;
use super::view_runtime::ViewRuntimeLiveProofState;
use super::wave_p::WavePLiveProofState;
use super::wss_substrate::WssSubstrateLiveProofState;
use crate::render::fire_streaming::FireStreamingLiveProofState;

pub struct LiveProofCadencePlugin;

impl Plugin for LiveProofCadencePlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<LiveProofCadence>() {
            app.init_resource::<LiveProofCadence>();
        }
        app.add_systems(
            First,
            (
                arm_global_live_proof_cadence,
                arm_fire_ecology_live_proof_cadence,
                arm_wss_substrate_live_proof_cadence,
                arm_construction_live_proof_cadence,
                arm_view_runtime_live_proof_cadence,
                arm_industrial_activation_live_proof_cadence,
                arm_logistics_throughput_live_proof_cadence,
                arm_fire_streaming_live_proof_cadence,
                arm_wave_p_live_proof_cadence,
                arm_stage7_play_live_proof_cadence,
                arm_stage7_behavioral_live_proof_cadence,
            ),
        );
    }
}

pub fn arm_global_live_proof_cadence(mut state: ResMut<LiveProofCadence>) {
    arm_live_proof_cadence(&mut state);
}

pub fn arm_fire_ecology_live_proof_cadence(state: Option<ResMut<FireEcologyLiveProofState>>) {
    if let Some(mut state) = state {
        arm_witness_write_cadence(&mut state.cadence);
    }
}

pub fn arm_wss_substrate_live_proof_cadence(state: Option<ResMut<WssSubstrateLiveProofState>>) {
    if let Some(mut state) = state {
        arm_witness_write_cadence(&mut state.cadence);
    }
}

pub fn arm_construction_live_proof_cadence(state: Option<ResMut<ConstructionLiveProofState>>) {
    if let Some(mut state) = state {
        arm_witness_write_cadence(&mut state.cadence);
    }
}

pub fn arm_view_runtime_live_proof_cadence(state: Option<ResMut<ViewRuntimeLiveProofState>>) {
    if let Some(mut state) = state {
        arm_witness_write_cadence(&mut state.cadence);
    }
}

pub fn arm_industrial_activation_live_proof_cadence(
    state: Option<ResMut<IndustrialActivationLiveProofState>>,
) {
    if let Some(mut state) = state {
        arm_witness_write_cadence(&mut state.cadence);
    }
}

#[must_use]
pub fn logistics_throughput_live_proof_ready(state: Res<LogisticsThroughputLiveProofState>) -> bool {
    state.cadence.write_due || !state.cadence.written()
}

pub fn arm_logistics_throughput_live_proof_cadence(
    state: Option<ResMut<LogisticsThroughputLiveProofState>>,
) {
    if let Some(mut state) = state {
        arm_witness_write_cadence(&mut state.cadence);
    }
}

pub fn arm_fire_streaming_live_proof_cadence(state: Option<ResMut<FireStreamingLiveProofState>>) {
    if let Some(mut state) = state {
        arm_witness_write_cadence(&mut state.cadence);
    }
}

pub fn arm_wave_p_live_proof_cadence(state: Option<ResMut<WavePLiveProofState>>) {
    if let Some(mut state) = state {
        arm_witness_write_cadence(&mut state.cadence);
    }
}

pub fn arm_stage7_play_live_proof_cadence(state: Option<ResMut<Stage7PlayLiveProofState>>) {
    if let Some(mut state) = state {
        arm_witness_write_cadence(&mut state.cadence);
    }
}

pub fn arm_stage7_behavioral_live_proof_cadence(
    state: Option<ResMut<Stage7BehavioralLiveProofState>>,
) {
    if let Some(mut state) = state {
        arm_witness_write_cadence(&mut state.cadence);
    }
}

#[must_use]
pub fn fire_ecology_live_proof_due(state: Res<FireEcologyLiveProofState>) -> bool {
    state.cadence.write_due
}

#[must_use]
pub fn wss_substrate_live_proof_due(state: Res<WssSubstrateLiveProofState>) -> bool {
    state.cadence.write_due
}

#[must_use]
pub fn construction_live_proof_due(state: Res<ConstructionLiveProofState>) -> bool {
    state.cadence.write_due
}

#[must_use]
pub fn view_runtime_live_proof_due(state: Res<ViewRuntimeLiveProofState>) -> bool {
    state.cadence.write_due
}

#[must_use]
pub fn industrial_activation_live_proof_due(state: Res<IndustrialActivationLiveProofState>) -> bool {
    state.cadence.write_due
}

#[must_use]
pub fn logistics_throughput_live_proof_due(state: Res<LogisticsThroughputLiveProofState>) -> bool {
    logistics_throughput_live_proof_ready(state)
}

#[must_use]
pub fn fire_streaming_live_proof_due(state: Res<FireStreamingLiveProofState>) -> bool {
    state.cadence.write_due
}

#[must_use]
pub fn wave_p_live_proof_due(state: Res<WavePLiveProofState>) -> bool {
    state.cadence.write_due
}

#[must_use]
pub fn stage7_play_live_proof_due(state: Res<Stage7PlayLiveProofState>) -> bool {
    state.cadence.write_due
}

#[must_use]
pub fn stage7_behavioral_live_proof_due(state: Res<Stage7BehavioralLiveProofState>) -> bool {
    state.cadence.write_due
}
