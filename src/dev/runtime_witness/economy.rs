//! Economy witness writers — industrial + logistics (DEV-CONTAIN-003).

use bevy::prelude::*;

use crate::dev::industrial_activation_todos::{
    IndustrialActivationTodoBoard, IndustrialActivationWitness,
};
use crate::dev::logistics_throughput_todos::{
    LogisticsThroughputTodoBoard, LogisticsThroughputWitness,
};
use crate::economy::activation::ConcreteChainE2eWitness;
use crate::economy::logistics::types::{LogisticsDiagnostics, LogisticsThroughputRuntimeWitness};
use crate::economy::resource_flow::ResourceFlowRegistry;
use crate::engine::states::BaseState;
use crate::strategic::PlannedSite;
use crate::systems::transport::{TransportEdgeDirectory, TransportNavExport};

use super::common::WitnessWriteCadence;
use super::io::write_enveloped_witness;

pub const INDUSTRIAL_ACTIVATION_JSON: &str = "debug_runs/industrial_activation_live.json";
pub const LOGISTICS_THROUGHPUT_JSON: &str = "debug_runs/logistics_throughput_live.json";

#[derive(Resource, Debug)]
pub struct IndustrialActivationLiveProofState {
    pub cadence: WitnessWriteCadence,
}

impl Default for IndustrialActivationLiveProofState {
    fn default() -> Self {
        Self {
            cadence: WitnessWriteCadence {
                write_interval: 90,
                ..Default::default()
            },
        }
    }
}

impl IndustrialActivationLiveProofState {
    #[must_use]
    pub fn written(&self) -> bool {
        self.cadence.written()
    }
}

#[derive(Resource, Debug)]
pub struct LogisticsThroughputLiveProofState {
    pub cadence: WitnessWriteCadence,
}

impl Default for LogisticsThroughputLiveProofState {
    fn default() -> Self {
        Self {
            cadence: WitnessWriteCadence {
                write_interval: 90,
                ..Default::default()
            },
        }
    }
}

impl LogisticsThroughputLiveProofState {
    #[must_use]
    pub fn written(&self) -> bool {
        self.cadence.written()
    }
}

#[must_use]
pub fn commit_industrial_activation_live_proof(
    board: Option<&IndustrialActivationTodoBoard>,
    witness: &IndustrialActivationWitness,
    governance_violations: usize,
    district: Option<&crate::economy::spatial_district::IndustrialDistrictSnapshot>,
    chain: &ConcreteChainE2eWitness,
    flow: Option<&crate::economy::resource_flow::ResourceFlowSimWitness>,
    toast: Option<&crate::economy::activation::grid_overload_ux::GridOverloadToastState>,
) -> bool {
    let body = crate::economy::activation::build_industrial_activation_proof_payload(
        board,
        witness,
        governance_violations,
        district,
        chain,
        flow,
        toast,
    );
    write_enveloped_witness(
        "INDUSTRIAL_ACTIVATION",
        "industrial_activation_live_proof",
        INDUSTRIAL_ACTIVATION_JSON,
        body,
    )
}

pub fn write_industrial_activation_live_proof_system(
    base: Option<Res<State<BaseState>>>,
    mut state: ResMut<IndustrialActivationLiveProofState>,
    board: Option<Res<IndustrialActivationTodoBoard>>,
    witness: Res<IndustrialActivationWitness>,
    chain: Res<ConcreteChainE2eWitness>,
    buildings: Option<Res<crate::construction::BuildingDefinitionRegistry>>,
    district: Option<Res<crate::economy::spatial_district::IndustrialDistrictSnapshot>>,
    flow: Option<Res<crate::economy::resource_flow::ResourceFlowSimWitness>>,
    toast: Option<Res<crate::economy::activation::grid_overload_ux::GridOverloadToastState>>,
) {
    if !matches!(base.as_deref().map(|s| s.get()), Some(BaseState::Simulation)) {
        return;
    }

    let gov_count = buildings
        .as_deref()
        .map(|r| r.governance_violations.len())
        .unwrap_or(0);
    let mut witness_snap = witness.as_ref().clone();
    witness_snap.proof_json = true;

    if commit_industrial_activation_live_proof(
        board.as_deref(),
        &witness_snap,
        gov_count,
        district.as_deref(),
        chain.as_ref(),
        flow.as_deref(),
        toast.as_deref(),
    ) {
        state.cadence.written = true;
    }
}

#[must_use]
pub fn commit_logistics_throughput_live_proof(
    board: Option<&LogisticsThroughputTodoBoard>,
    witness: &LogisticsThroughputWitness,
    diagnostics: &LogisticsDiagnostics,
    runtime: Option<&LogisticsThroughputRuntimeWitness>,
    infra_e5: Option<crate::economy::logistics::witness_collectors::InfraE502ProofExtension>,
) -> bool {
    let body = crate::economy::logistics::witness_collectors::build_logistics_throughput_proof_payload(
        board,
        witness,
        diagnostics,
        runtime,
        infra_e5,
    );
    write_enveloped_witness(
        "LOGISTICS_THROUGHPUT",
        "logistics_throughput_live_proof",
        LOGISTICS_THROUGHPUT_JSON,
        body,
    )
}

#[must_use]
fn infra_e5_002_proof_extension(
    flow: Option<&ResourceFlowRegistry>,
    nav: Option<&TransportNavExport>,
    directory: Option<&TransportEdgeDirectory>,
    portals: &Query<(Entity, &crate::economy::logistics::FacilityPortal, &PlannedSite)>,
) -> crate::economy::logistics::witness_collectors::InfraE502ProofExtension {
    use crate::economy::logistics::routes::{collect_portal_entity_tiles, flow_paths_match_nav_export};

    let (Some(flow), Some(nav), Some(directory)) = (flow, nav, directory) else {
        return crate::economy::logistics::witness_collectors::InfraE502ProofExtension::default();
    };
    let tiles = collect_portal_entity_tiles(portals);
    crate::economy::logistics::witness_collectors::InfraE502ProofExtension {
        graph_only_paths: flow_paths_match_nav_export(flow, nav, directory, &tiles),
        no_harness_tile_paint: true,
    }
}

pub fn write_logistics_throughput_live_proof_system(
    base: Option<Res<State<BaseState>>>,
    mut state: ResMut<LogisticsThroughputLiveProofState>,
    board: Option<Res<LogisticsThroughputTodoBoard>>,
    witness: Res<LogisticsThroughputWitness>,
    diagnostics: Res<LogisticsDiagnostics>,
    runtime: Option<Res<LogisticsThroughputRuntimeWitness>>,
    flow: Option<Res<ResourceFlowRegistry>>,
    nav: Option<Res<TransportNavExport>>,
    directory: Option<Res<TransportEdgeDirectory>>,
    portals: Query<(Entity, &crate::economy::logistics::FacilityPortal, &PlannedSite)>,
) {
    if !matches!(base.as_deref().map(|s| s.get()), Some(BaseState::Simulation)) {
        return;
    }

    let infra_e5 = infra_e5_002_proof_extension(
        flow.as_deref(),
        nav.as_deref(),
        directory.as_deref(),
        &portals,
    );
    if commit_logistics_throughput_live_proof(
        board.as_deref(),
        witness.as_ref(),
        diagnostics.as_ref(),
        runtime.as_deref(),
        Some(infra_e5),
    ) {
        state.cadence.written = true;
    }
}