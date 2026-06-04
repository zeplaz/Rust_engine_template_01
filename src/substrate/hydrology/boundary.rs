//! HY-004 — inter-chunk boundary flux peek (resident slabs only).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::substrate::hydrology::background_tick::HydrologyTickState;
use crate::substrate::registry::WorldSubstrateRegistry;
use crate::substrate::slab::ChunkKey;

const NEIGHBOR_OFFSETS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

pub fn hydrology_boundary_exchange_system(
    base: Res<State<BaseState>>,
    registry: Res<WorldSubstrateRegistry>,
    mut tick: ResMut<HydrologyTickState>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }

    let mut flux_max = 0.0_f32;
    for key in registry.chunks.resident.iter().copied() {
        let Some(chunk) = registry.chunks.get(key) else {
            continue;
        };
        let center_depth = chunk.hydrology.water_depth.first().copied().unwrap_or(0.0);

        for (dx, dy) in NEIGHBOR_OFFSETS {
            let neighbor_key = ChunkKey::new(key.x + dx, key.y + dy);
            let Some(neighbor) = registry.chunks.get(neighbor_key) else {
                continue;
            };
            let neighbor_depth = neighbor.hydrology.water_depth.first().copied().unwrap_or(0.0);
            flux_max = flux_max.max((center_depth - neighbor_depth).abs() * 0.25);
        }
    }

    if flux_max > 0.0 {
        tick.saturation_delta_max = tick.saturation_delta_max.max(flux_max);
    }
    tick.boundary_flux_max = flux_max;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;
    use crate::engine::states::BaseState;

    #[test]
    fn boundary_flux_continuous() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<WorldSubstrateRegistry>()
            .init_resource::<HydrologyTickState>()
            .init_state::<BaseState>()
            .insert_state(BaseState::Simulation)
            .add_systems(Update, hydrology_boundary_exchange_system);

        let key_a = ChunkKey::new(0, 0);
        let key_b = ChunkKey::new(1, 0);
        let mut a =
            crate::substrate::WorldChunkState::new_empty(key_a, 4);
        let mut b =
            crate::substrate::WorldChunkState::new_empty(key_b, 4);
        a.hydrology.water_depth = vec![0.8, 0.2, 0.2, 0.2];
        b.hydrology.water_depth = vec![0.2, 0.2, 0.2, 0.2];
        {
            let mut reg = app.world_mut().resource_mut::<WorldSubstrateRegistry>();
            reg.chunks.insert(key_a, a);
            reg.chunks.insert(key_b, b);
            reg.chunks.set_resident(key_a, true);
            reg.chunks.set_resident(key_b, true);
        }

        app.update();

        let tick = app.world().resource::<HydrologyTickState>();
        assert!(tick.boundary_flux_max.is_finite());
        assert!(tick.boundary_flux_max > 0.0);
    }
}
