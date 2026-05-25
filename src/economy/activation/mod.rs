//! Industrial activation — spawn production ECS when strategic sites go operational.

mod bridge;
mod concrete_chain_e2e;
mod live_proof;

pub use concrete_chain_e2e::{
    commit_concrete_portland_chain_in_play, fast_forward_portland_chain_sites_to_operational,
    refresh_concrete_chain_e2e_witness_system, spawn_concrete_portland_chain_operational,
    ConcreteChainE2eWitness, CONCRETE_PORTLAND_CHAIN, CONCRETE_PORTLAND_STEPS,
};

pub use bridge::{
    activate_industrial_facilities_system, refresh_industrial_activation_witness_system,
    BuildingDefinitionRef, IndustrialFacilityActivated,
};
pub use bridge::IndustrialActivationPlugin;
pub use live_proof::{
    sync_industrial_proof_witness_flags, write_industrial_activation_live_proof_system,
    IndustrialActivationLiveProofState,
};
