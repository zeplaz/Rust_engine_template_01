//! World substrate spine — chunk slab authority, dual-write shim, hydrology, atmosphere clipmap.

pub mod active_runtime;
pub mod atmosphere;
pub mod deformation;
pub mod ecs_retire;
pub mod hydrate;
pub mod hydrology;
pub mod persist;
pub mod post_spine;
pub mod registry;
pub mod shim;
pub mod slab;
pub mod types;

pub use active_runtime::{
    activate_hot_chunks_system, active_runtime_policy_green, deactivate_stale_runtime_system,
    sync_active_runtime_witness_flags_system, ActiveRuntimeState,
};
pub use atmosphere::{
    clipmap_l0_smoke_max, contamination_tick_system, legacy_atmosphere_bridge_system,
    sync_atmos_clipmap_witness_system, AtmosphereClipmapStack, AtmosphereClipmapWitness,
    WSS_ATMOS_CLIPMAP_GATE,
};
pub use deformation::{
    apply_deformation_to_chunk, deformation_apply_tick_system, DeformationTickState,
    WSS_DEFORMATION_SLAB_GATE,
};
pub use ecs_retire::{
    ecs_retire_fixture_green, ecs_retire_pass_system, ecs_retire_smoke_prod_green,
    slab_surface_heat, EcsRetireState, SubstrateEcsRetireWitness,
};
pub use post_spine::apply_slab_traction_to_logistics_snapshot;
pub use hydrate::{hydrate_chunk_into_substrate, sync_substrate_hydrate_system};
pub use hydrology::{
    construction_hydro_coupling_witness_green, hydrology_background_tick_system,
    hydrology_boundary_exchange_system, hydrology_drain_construction_events_system,
    sync_hydrology_runtime_witness_system, HydrologyConstructionCouplingWitness,
    HydrologyEventQueue, HydrologyRuntimeWitness, HydrologyTickState, WSS_HYDRO_RUNTIME_GATE,
};
pub use persist::{
    mirror_overlay_cell_to_slab, sync_dynamic_overlay_migrate_system,
    sync_substrate_persist_witness_system, SubstratePr4Witness,
};
pub use post_spine::{
    apply_logistics_pressure_mirror, compute_post_spine_witness,
    mirror_logistics_pressure_to_slab_system, sync_post_spine_witness_system,
    sync_regional_weather_from_clipmap_system, update_global_renewable_weather_from_clipmap_system,
    PostSpineWitness, WSS_POST_SPINE_GATE,
};
pub use registry::{
    ChunkPagingState, SubstratePersistBook, WorldSubstrateRegistry, WssSubstrateWitness,
};
pub use shim::{
    compare_dual_write_drift_system, compare_dynamic_overlay_drift_system, dual_write_shim_green,
    sync_ecs_to_substrate_dual_write_system, DualWriteShimState,
};
pub use slab::ChunkKey;
pub use types::{hydrate_skeleton_chunk, WorldChunkState, SUBSTRATE_SKELETON_CELL_GRID};
pub use bevy::prelude::UVec2;

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::systems::sim_control::SimControlSystemSet;

#[must_use]
pub fn substrate_plugin_enabled() -> bool {
    !matches!(
        std::env::var("RUST_ENGINE_SUBSTRATE").as_deref(),
        Ok("0") | Ok("false") | Ok("off")
    )
}

pub fn sync_substrate_paging_system(
    base: Res<State<BaseState>>,
    mut witness: ResMut<WssSubstrateWitness>,
    mut registry: ResMut<WorldSubstrateRegistry>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    let keys: Vec<ChunkKey> = registry.chunks.chunks.keys().copied().collect();
    for key in keys {
        registry.chunks.set_resident(key, true);
    }
    if !registry.chunks.is_empty() {
        witness.paging_wired = true;
        witness.chunk_environment_order_preserved = true;
    }
}

pub mod witness_collectors;

pub use witness_collectors::{
    build_wss_substrate_payload, refresh_wss_substrate_live_witness, wss_chunk_slab_001_green,
    WSS_CHUNK_SLAB_GATE,
};
pub use crate::dev::runtime_witness::wss_substrate::{
    commit_wss_substrate_live_proof, commit_wss_substrate_live_proof_body,
    write_wss_substrate_live_proof_system, WssSubstrateLiveProofState, WSS_SUBSTRATE_LIVE_JSON,
};

pub struct SubstratePlugin;

impl Plugin for SubstratePlugin {
    fn build(&self, app: &mut App) {
        if !substrate_plugin_enabled() {
            return;
        }
        app.init_resource::<WorldSubstrateRegistry>()
            .init_resource::<ChunkPagingState>()
            .init_resource::<SubstratePersistBook>()
            .init_resource::<WssSubstrateWitness>()
            .init_resource::<WssSubstrateLiveProofState>()
            .init_resource::<SubstratePr4Witness>()
            .init_resource::<DualWriteShimState>()
            .init_resource::<ActiveRuntimeState>()
            .init_resource::<EcsRetireState>()
            .init_resource::<SubstrateEcsRetireWitness>()
            .init_resource::<AtmosphereClipmapStack>()
            .init_resource::<AtmosphereClipmapWitness>()
            .init_resource::<HydrologyRuntimeWitness>()
            .init_resource::<HydrologyTickState>()
            .init_resource::<HydrologyEventQueue>()
            .init_resource::<HydrologyConstructionCouplingWitness>()
            .init_resource::<DeformationTickState>()
            .init_resource::<PostSpineWitness>()
            .add_systems(
                Update,
                (
                    sync_substrate_hydrate_system,
                    sync_substrate_paging_system,
                    sync_ecs_to_substrate_dual_write_system,
                    compare_dual_write_drift_system,
                    compare_dynamic_overlay_drift_system,
                    sync_dynamic_overlay_migrate_system,
                    sync_substrate_persist_witness_system,
                    activate_hot_chunks_system,
                    deactivate_stale_runtime_system,
                    sync_active_runtime_witness_flags_system,
                )
                    .after(SimControlSystemSet::AdvanceSimTick)
                    .run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(
                Update,
                deformation_apply_tick_system
                    .after(SimControlSystemSet::AdvanceSimTick)
                    .run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(
                Update,
                (
                    hydrology_background_tick_system,
                    hydrology_boundary_exchange_system,
                    hydrology_drain_construction_events_system,
                    sync_hydrology_runtime_witness_system,
                    legacy_atmosphere_bridge_system,
                    atmosphere::contamination_tick_system,
                    sync_atmos_clipmap_witness_system,
                    mirror_logistics_pressure_to_slab_system,
                    sync_regional_weather_from_clipmap_system,
                    update_global_renewable_weather_from_clipmap_system,
                )
                    .after(SimControlSystemSet::AdvanceSimTick)
                    .run_if(in_state(BaseState::Simulation)),
            )
            .add_systems(
                Update,
                (
                    sync_post_spine_witness_system,
                    ecs_retire_pass_system,
                    write_wss_substrate_live_proof_system
                        .run_if(crate::dev::runtime_witness::wss_substrate_live_proof_due),
                )
                    .after(SimControlSystemSet::AdvanceSimTick)
                    .run_if(in_state(BaseState::Simulation)),
            );
    }
}
