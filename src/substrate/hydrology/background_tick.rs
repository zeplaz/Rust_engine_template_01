//! HY-003 — resident-only hydrology background tick (saturation + flow refresh stub).

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::substrate::registry::WorldSubstrateRegistry;
use crate::substrate::slab::ChunkKey;

/// Per-frame hydrology tick diagnostics for witness rollup.
#[derive(Resource, Clone, Debug, Default)]
pub struct HydrologyTickState {
    pub background_ticks: u32,
    pub saturation_delta_max: f32,
    pub boundary_flux_max: f32,
}

pub fn hydrology_background_tick_system(
    base: Res<State<BaseState>>,
    mut registry: ResMut<WorldSubstrateRegistry>,
    mut tick: ResMut<HydrologyTickState>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }

    tick.saturation_delta_max = 0.0;
    let keys: Vec<ChunkKey> = registry
        .chunks
        .resident
        .iter()
        .copied()
        .collect();

    for key in keys {
        let Some(chunk) = registry.chunks.get_mut(key) else {
            continue;
        };
        let rain = chunk.atmosphere.local.rain_intensity;
        let soil = chunk.atmosphere.local.soil_moisture;
        let moisture_driver = rain.max(soil * 0.5);
        if moisture_driver <= 0.05 {
            continue;
        }

        let n = chunk.hydrology.saturation.len();
        for i in 0..n {
            let before = chunk.hydrology.saturation[i];
            let after = (before + moisture_driver * 0.03).min(1.0);
            chunk.hydrology.saturation[i] = after;
            tick.saturation_delta_max = tick.saturation_delta_max.max(after - before);

            // Flow direction refresh from terrain height gradient (HY-003 stub).
            if i > 0 && i < chunk.terrain.height.len() {
                let dh = chunk.terrain.height[i] - chunk.terrain.height[i - 1];
                chunk.hydrology.flow_velocity[i] = Vec2::new(dh.signum() * 0.1, 0.0);
            }
        }
        tick.background_ticks = tick.background_ticks.saturating_add(1);
    }
}
