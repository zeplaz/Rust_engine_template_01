//! Canonical home for runtime witness JSON orchestration (DEV-ARTIFACT-CONTAINMENT-001).
//!
//! Domain modules expose read-only collectors; this tree owns envelope wrap, cadence,
//! file I/O, and the [`gate`] switch.

pub mod cadence_plugin;
pub mod common;
pub mod containment;
pub mod construction;
pub mod economy;
pub mod fire;
pub mod gate;
pub mod io;
pub mod minimap;
pub mod parity;
pub mod sim_effects;
pub mod stage6;
pub mod stage7_behavioral;
pub mod stage7_play;
pub mod view_runtime;
pub mod wave_c;
pub mod wave_p;
pub mod wave_s;
pub mod wss_substrate;

pub use construction::{
    commit_construction_stage_live_proof, write_construction_live_proof_system,
    ConstructionLiveProofState, CONSTRUCTION_STAGE_JSON,
};
pub use economy::{
    commit_industrial_activation_live_proof, commit_logistics_throughput_live_proof,
    write_industrial_activation_live_proof_system, write_logistics_throughput_live_proof_system,
    IndustrialActivationLiveProofState, LogisticsThroughputLiveProofState,
    INDUSTRIAL_ACTIVATION_JSON, LOGISTICS_THROUGHPUT_JSON,
};
pub use fire::{
    commit_fire_ecology_live_proof, write_fire_ecology_live_proof_system,
    FireEcologyLiveProofState, FIRE_ECOLOGY_JSON,
};
pub use containment::{
    phase0_containment_green, scan_live_proof_containment_violations, LiveProofContainmentViolation,
};
pub use cadence_plugin::{
    arm_construction_live_proof_cadence, arm_fire_ecology_live_proof_cadence,
    arm_global_live_proof_cadence, arm_wss_substrate_live_proof_cadence,
    construction_live_proof_due, fire_ecology_live_proof_due,
    fire_streaming_live_proof_due, industrial_activation_live_proof_due,
    logistics_throughput_live_proof_due, stage7_behavioral_live_proof_due,
    stage7_play_live_proof_due, view_runtime_live_proof_due, wave_p_live_proof_due,
    wss_substrate_live_proof_due, LiveProofCadencePlugin,
};
pub use common::{
    arm_live_proof_cadence, arm_witness_write_cadence, live_proof_cadence_due,
    live_proof_write_latched, tick_live_proof_cadence, LiveProofCadence, LiveProofWriteLatch,
};
pub use gate::{
    witness_gate_snapshot, witness_writes_enabled, ENV_RUNTIME_WITNESS_WRITES,
    ENV_RUNTIME_WITNESS_WRITES_FORCE_OFF,
};
pub use io::{write_enveloped_witness, write_enveloped_witness_unchecked};
pub use wave_c::{
    build_wave_c_live_proof_payload, commit_wave_c_live_proof, wc_depth_001_green,
    write_wave_c_live_proof_system, WaveCLiveProofState, WAVE_C_LIVE_JSON,
};
pub use wave_s::{
    build_wave_s_hydrate_proof_payload, write_wave_s_hydrate_live_proof_system,
    WaveSLiveProofState, WAVE_S_HYDRATE_JSON,
};
pub use parity::{refresh_migrated_witness_parity_bundle, strip_envelope_for_parity, witness_has_required_keys};
pub use sim_effects::{
    commit_sim_effect_spine_live_proof, commit_sim_effect_spine_live_proof_unchecked,
    refresh_sim_effect_spine_live_witness, write_sim_effect_spine_live_proof_system,
    SIM_EFFECT_SPINE_JSON,
};
pub use minimap::{
    commit_minimap_compositor_live_proof, refresh_perf_vis_p1b_gpu_default_live_witness,
    refresh_ui_oh_m2_001_live_witness, refresh_ui_w3_m2_001_live_witness,
    refresh_ui_w3_m3_001_live_witness, refresh_ui_w3_m3_001_stage7_operational_witness,
    write_minimap_compositor_live_proof_system,
    MinimapCompositorLiveProofState, MINIMAP_COMPOSITOR_JSON,
};
pub use stage6::{
    build_stage6_proof_payload, commit_stage6_virtualization_live_proof,
    ops_f01_perf_attribution_section_present, refresh_infra_slice3_001_live_witnesses,
    refresh_stage6_virtualization_witness, refresh_wc_d04_stage6_virtualization_live_witness,
    refresh_wc_d04_stage6_virtualization_live_witness_with_source,
    stage6_readiness_violations, wc_d04_green, wc_d04_witness_fields,
    write_ops_f01_perf_attribution_section, write_stage6_virtualization_live_proof_system,
    Stage6LiveProofState, Stage6VirtualizationWitness, PERF_ATTRIBUTION_60S_MD,
    STAGE6_VIRTUALIZATION_JSON,
};
pub use stage7_behavioral::{
    commit_stage7_behavioral_witness, write_stage7_behavioral_witness_system,
    Stage7BehavioralLiveProofState, STAGE7_BEHAVIORAL_LIVE_JSON,
};
pub use stage7_play::{
    commit_stage7_play_witness, write_stage7_play_witness_system,
    Stage7PlayLiveProofState, STAGE7_PLAY_LIVE_JSON,
};
pub use view_runtime::{
    build_infrastructure_view_isolation_payload, refresh_infrastructure_view_isolation_live_witness,
    write_view_runtime_live_proof_system, ViewRuntimeLiveProofState,
    INFRASTRUCTURE_VIEW_ISOLATION_JSON,
};
pub use wave_p::{
    commit_wave_p_witness, write_wave_p_witness_system, WavePLiveProofState,
};
pub use wss_substrate::{
    commit_wss_substrate_live_proof, commit_wss_substrate_live_proof_body,
    write_wss_substrate_live_proof_system, WssSubstrateLiveProofState, WSS_SUBSTRATE_LIVE_JSON,
};
pub use crate::dev::proof_grade::ProofGrade;

/// Relative paths allowed outside `src/dev/runtime_witness/` during migration (Slice B–D).
/// CI warns on any other `*live_proof*.rs` until listed in manifest; `-HardFail` active (DEV-CONTAIN-HARDFAIL-CI-001).
pub const MIGRATION_SHIM_PATHS: &[&str] = &[
    "src/construction/live_proof.rs",
    "src/economy/activation/live_proof.rs",
    "src/economy/logistics/live_proof.rs",
    "src/systems/fire/live_proof.rs",
    "src/gui/editor/world_preview/wave_p_live_proof.rs",
    "src/dev/stage7_behavioral_live_proof.rs",
    "src/dev/stage7_play_live_proof.rs",
    "src/substrate/live_proof.rs",
];
