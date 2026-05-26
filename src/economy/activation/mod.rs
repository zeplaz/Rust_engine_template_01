//! Industrial activation — spawn production ECS when strategic sites go operational.

mod bridge;
mod concrete_chain_e2e;
mod live_proof;

pub use concrete_chain_e2e::{
    commit_concrete_portland_chain_in_play, fast_forward_portland_chain_sites_to_operational,
    refresh_concrete_chain_e2e_witness_system, reset_ind_e03_grid_overload_seed_on_enter_simulation,
    reset_stage7_play_chain_seed_on_enter_simulation, seed_ind_e03_grid_overload_witness_once,
    seed_stage7_play_concrete_chain_once, spawn_concrete_portland_chain_operational,
    spawn_ind_e03_grid_overload_cluster, ConcreteChainE2eWitness, IndE03GridOverloadSeedState,
    Stage7PlayChainSeedState, CONCRETE_PORTLAND_CHAIN, CONCRETE_PORTLAND_STEPS,
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
