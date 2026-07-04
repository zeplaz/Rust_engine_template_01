//! WSS substrate witness — `debug_runs/wss_substrate_live.json` (DEV-CONTAIN-006).

use bevy::prelude::*;
use serde_json::Value;

use crate::engine::states::BaseState;
use crate::substrate::witness_collectors::build_wss_substrate_payload;
use crate::substrate::{
    substrate_plugin_enabled, ActiveRuntimeState, AtmosphereClipmapStack,
    AtmosphereClipmapWitness, DeformationTickState, DualWriteShimState, EcsRetireState,
    HydrologyConstructionCouplingWitness, HydrologyEventQueue, HydrologyRuntimeWitness,
    PostSpineWitness, SubstratePr4Witness, WorldSubstrateRegistry, WssSubstrateWitness,
};

use super::common::WitnessWriteCadence;
use super::io::{write_enveloped_witness, write_enveloped_witness_unchecked};

pub const WSS_SUBSTRATE_LIVE_JSON: &str = "debug_runs/wss_substrate_live.json";

#[derive(Resource, Debug, Clone)]
pub struct WssSubstrateLiveProofState {
    pub cadence: WitnessWriteCadence,
}

impl Default for WssSubstrateLiveProofState {
    fn default() -> Self {
        Self {
            cadence: WitnessWriteCadence {
                write_interval: 60,
                ..Default::default()
            },
        }
    }
}

#[must_use]
pub fn commit_wss_substrate_live_proof_body(body: Value, source_system: &str) -> bool {
    write_enveloped_witness_unchecked(
        "WSS_SUBSTRATE",
        source_system,
        WSS_SUBSTRATE_LIVE_JSON,
        body,
    )
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn commit_wss_substrate_live_proof(
    registry: &WorldSubstrateRegistry,
    witness: &WssSubstrateWitness,
    plugin_enabled: bool,
    smoke: Option<&crate::render::extraction::SmokeVisualBridgeWitness>,
    clipmap: Option<&AtmosphereClipmapStack>,
    clipmap_witness: Option<&AtmosphereClipmapWitness>,
    hydrology_witness: Option<&HydrologyRuntimeWitness>,
    hydro_queue: Option<&HydrologyEventQueue>,
    hydro_coupling: Option<&HydrologyConstructionCouplingWitness>,
    dual_write: Option<&DualWriteShimState>,
    active_runtime: Option<&ActiveRuntimeState>,
    pr4: Option<&SubstratePr4Witness>,
    retire: Option<&EcsRetireState>,
    post_spine: Option<&PostSpineWitness>,
    deformation_tick: Option<&DeformationTickState>,
    source_system: &str,
) -> bool {
    let body = build_wss_substrate_payload(
        registry,
        witness,
        plugin_enabled,
        smoke,
        clipmap,
        clipmap_witness,
        hydrology_witness,
        hydro_queue,
        hydro_coupling,
        dual_write,
        active_runtime,
        pr4,
        retire,
        post_spine,
        deformation_tick,
    );
    commit_wss_substrate_live_proof_body(body, source_system)
}

#[allow(clippy::too_many_arguments)]
pub fn write_wss_substrate_live_proof_system(
    base: Res<State<BaseState>>,
    mut state: ResMut<WssSubstrateLiveProofState>,
    registry: Res<WorldSubstrateRegistry>,
    witness: Res<WssSubstrateWitness>,
    smoke: Option<Res<crate::render::extraction::SmokeVisualBridgeWitness>>,
    clipmap: Option<Res<AtmosphereClipmapStack>>,
    clipmap_witness: Option<Res<AtmosphereClipmapWitness>>,
    hydrology_witness: Option<Res<HydrologyRuntimeWitness>>,
    hydro_queue: Option<Res<HydrologyEventQueue>>,
    hydro_coupling: Option<Res<HydrologyConstructionCouplingWitness>>,
    dual_write: Option<Res<DualWriteShimState>>,
    active_runtime: Option<Res<ActiveRuntimeState>>,
    pr4: Option<Res<SubstratePr4Witness>>,
    retire: Option<Res<EcsRetireState>>,
    post_spine: Option<Res<PostSpineWitness>>,
    deformation_tick: Option<Res<DeformationTickState>>,
) {
    if !matches!(base.get(), BaseState::Simulation) || !substrate_plugin_enabled() {
        return;
    }
    let body = build_wss_substrate_payload(
        registry.as_ref(),
        witness.as_ref(),
        true,
        smoke.as_deref(),
        clipmap.as_deref(),
        clipmap_witness.as_deref(),
        hydrology_witness.as_deref(),
        hydro_queue.as_deref(),
        hydro_coupling.as_deref(),
        dual_write.as_deref(),
        active_runtime.as_deref(),
        pr4.as_deref(),
        retire.as_deref(),
        post_spine.as_deref(),
        deformation_tick.as_deref(),
    );
    if write_enveloped_witness(
        "WSS_SUBSTRATE",
        "write_wss_substrate_live_proof_system",
        WSS_SUBSTRATE_LIVE_JSON,
        body,
    ) {
        state.cadence.written = true;
    }
}
