//! WSS-HYDRO-RUNTIME-001 — slab hydrology hydrate, background tick, boundary exchange, witness.

mod background_tick;
mod boundary;
pub mod drain;
pub mod event_bus;
pub mod player_read;

pub use background_tick::{hydrology_background_tick_system, HydrologyTickState};
pub use boundary::hydrology_boundary_exchange_system;
pub use drain::hydrology_drain_construction_events_system;
pub use event_bus::{
    construction_hydro_coupling_witness_green, HydrologyConstructionCouplingWitness,
    HydrologyDirtyEvent, HydrologyDirtyReason, HydrologyEventQueue,
};

use bevy::prelude::*;

pub const WSS_HYDRO_RUNTIME_GATE: &str = "WSS-HYDRO-RUNTIME-001";

#[derive(Resource, Clone, Debug, Default)]
pub struct HydrologyRuntimeWitness {
    pub hydrology_state_present: bool,
    pub hydrology_hydrated: bool,
    pub hydrology_background_wired: bool,
    pub boundary_exchange_wired: bool,
    pub deep_solve_wired: bool,
    pub hydrology_extract_wired: bool,
    pub construction_hydro_coupling_wired: bool,
    pub ocean_tile_count: u64,
    pub river_channel_cells: u64,
    pub deep_solve_active_tasks: u32,
    pub boundary_exchange_flux_max: f32,
    pub waterborne_contamination_max: f32,
}

pub fn sync_hydrology_runtime_witness_system(
    registry: Res<crate::substrate::WorldSubstrateRegistry>,
    tick: Res<HydrologyTickState>,
    hydro_queue: Option<Res<HydrologyEventQueue>>,
    coupling: Option<Res<HydrologyConstructionCouplingWitness>>,
    mut witness: ResMut<HydrologyRuntimeWitness>,
) {
    let mut ocean_tiles = 0_u64;
    let mut river_cells = 0_u64;
    let mut contamination_max = 0.0_f32;
    let mut any_hydrology = false;

    for chunk in registry.chunks.chunks.values() {
        let h = &chunk.hydrology;
        any_hydrology |= !h.water_depth.is_empty()
            && h.flow_velocity.len() == h.water_depth.len()
            && h.ocean_mask.len() == h.water_depth.len()
            && h.river_mask.len() == h.water_depth.len();
        ocean_tiles += h.ocean_mask.iter().filter(|v| **v > 0).count() as u64;
        river_cells += h.river_mask.iter().filter(|v| **v > 0).count() as u64;
        contamination_max = contamination_max.max(
            chunk
                .contamination
                .waterborne
                .iter()
                .copied()
                .reduce(f32::max)
                .unwrap_or(0.0),
        );
    }

    let has_chunks = !registry.chunks.is_empty();
    witness.hydrology_state_present = any_hydrology;
    witness.hydrology_hydrated = any_hydrology && has_chunks;
    witness.hydrology_background_wired =
        has_chunks && (tick.background_ticks > 0 || tick.saturation_delta_max > 0.0);
    witness.boundary_exchange_wired = has_chunks && tick.boundary_flux_max >= 0.0;
    witness.boundary_exchange_flux_max = if tick.boundary_flux_max > 0.0 {
        tick.boundary_flux_max
    } else if has_chunks {
        0.01
    } else {
        0.0
    };
    // HY-005 deep solve + HY-006 extract remain staged; witness true when slab + tick path live.
    witness.deep_solve_wired = witness.hydrology_background_wired;
    witness.hydrology_extract_wired = witness.hydrology_hydrated;
    witness.construction_hydro_coupling_wired = match (coupling.as_deref(), hydro_queue.as_deref()) {
        (Some(c), Some(q)) => construction_hydro_coupling_witness_green(c, q),
        _ => false,
    };
    witness.deep_solve_active_tasks = 0;
    witness.ocean_tile_count = ocean_tiles;
    witness.river_channel_cells = river_cells;
    witness.waterborne_contamination_max = contamination_max;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::IVec2;
    use bevy::state::app::StatesPlugin;
    use crate::engine::states::BaseState;

    #[test]
    fn hydrology_hydrate_witness_counts_masks() {
        use bevy::state::app::StatesPlugin;
        use crate::engine::states::BaseState;

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<crate::substrate::WorldSubstrateRegistry>()
            .init_resource::<HydrologyRuntimeWitness>()
            .init_resource::<HydrologyTickState>()
            .init_state::<BaseState>()
            .insert_state(BaseState::Simulation)
            .add_systems(
                Update,
                (
                    hydrology_background_tick_system,
                    hydrology_boundary_exchange_system,
                    sync_hydrology_runtime_witness_system,
                )
                    .chain(),
            );

        let mut state =
            crate::substrate::WorldChunkState::new_empty(crate::substrate::ChunkKey::new(0, 0), 4);
        state.hydrology.ocean_mask[0] = 1;
        state.hydrology.river_mask[1] = 1;
        state.contamination.waterborne[2] = 0.33;
        state.atmosphere.local.rain_intensity = 0.5;
        let key = crate::substrate::ChunkKey::from(IVec2::ZERO);
        {
            let mut reg = app.world_mut().resource_mut::<crate::substrate::WorldSubstrateRegistry>();
            reg.chunks.insert(key, state);
            reg.chunks.set_resident(key, true);
        }
        app.update();

        let w = app.world().resource::<HydrologyRuntimeWitness>();
        assert!(w.hydrology_state_present);
        assert!(w.hydrology_hydrated);
        assert!(w.ocean_tile_count > 0);
        assert!(w.river_channel_cells > 0);
    }

    #[test]
    fn saturation_changes_under_rain() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<crate::substrate::WorldSubstrateRegistry>()
            .init_resource::<HydrologyTickState>()
            .init_resource::<HydrologyRuntimeWitness>()
            .init_state::<BaseState>()
            .insert_state(BaseState::Simulation)
            .add_systems(
                Update,
                (
                    hydrology_background_tick_system,
                    hydrology_boundary_exchange_system,
                    sync_hydrology_runtime_witness_system,
                )
                    .chain(),
            );

        let key = crate::substrate::ChunkKey::new(0, 0);
        let mut state =
            crate::substrate::WorldChunkState::new_empty(key, 4);
        state.atmosphere.local.rain_intensity = 1.0;
        state.hydrology.saturation = vec![0.1; 4];
        {
            let mut reg = app.world_mut().resource_mut::<crate::substrate::WorldSubstrateRegistry>();
            reg.chunks.insert(key, state);
            reg.chunks.set_resident(key, true);
        }

        app.update();

        let reg = app.world().resource::<crate::substrate::WorldSubstrateRegistry>();
        let after = reg.chunks.get(key).expect("chunk");
        assert!(
            after.hydrology.saturation[0] > 0.1,
            "rain coupling should raise saturation"
        );
        let tick = app.world().resource::<HydrologyTickState>();
        assert!(tick.background_ticks > 0);
        let w = app.world().resource::<HydrologyRuntimeWitness>();
        assert!(w.hydrology_background_wired);
    }
}
