//! WSS-SLAB-PR-2 — ECS → slab dual-write shim (weather + fire); ECS remains authoritative.

use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::substrate::registry::WorldSubstrateRegistry;
use crate::substrate::slab::ChunkKey;
use crate::substrate::types::{ChunkWeatherLocal, ThermalState};
use crate::systems::fire::{ChunkSmokeField, ChunkSurfaceFire};
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::Chunk;
use crate::terrain::DynamicTerrainOverlay;

pub const DUAL_WRITE_DRIFT_EPSILON: f32 = 1e-5;

#[derive(Resource, Clone, Debug, Default)]
pub struct DualWriteShimState {
    pub enabled: bool,
    pub drift_max: f32,
    pub synced_resident_keys: u32,
}

#[inline]
pub fn sync_chunk_smoke_to_slab(
    contamination: &mut crate::substrate::types::ContaminationState,
    smoke: &ChunkSmokeField,
) {
    if contamination.airborne.is_empty() {
        contamination.airborne.push(smoke.density);
    } else {
        contamination.airborne[0] = smoke.density;
    }
    contamination.airborne[0] = contamination.airborne[0].max(smoke.toxicity * 0.25);
}

#[inline]
pub fn sync_chunk_weather_to_slab(local: &mut ChunkWeatherLocal, wx: &ChunkWeather) {
    local.rain_intensity = wx.rain_intensity;
    local.fog_density = wx.fog_density;
    local.snow_depth = wx.snow_depth;
    local.wind_speed = wx.wind_speed;
    local.lightning_risk = wx.lightning_risk;
    local.visibility_factor = wx.visibility_factor;
    local.soil_moisture = wx.soil_moisture;
}

#[inline]
pub fn sync_surface_fire_to_thermal(thermal: &mut ThermalState, fire: &ChunkSurfaceFire) {
    let heat = fire.heat;
    for h in &mut thermal.surface_heat {
        *h = heat;
    }
    let fuel_proxy = fire.fuel;
    for f in &mut thermal.ash_cover {
        *f = f.max(fuel_proxy * 0.01);
    }
}

#[inline]
fn weather_field_drift(wx: &ChunkWeather, local: &ChunkWeatherLocal) -> f32 {
    [
        (wx.rain_intensity - local.rain_intensity).abs(),
        (wx.fog_density - local.fog_density).abs(),
        (wx.snow_depth - local.snow_depth).abs(),
        (wx.wind_speed - local.wind_speed).abs(),
        (wx.lightning_risk - local.lightning_risk).abs(),
        (wx.visibility_factor - local.visibility_factor).abs(),
        (wx.soil_moisture - local.soil_moisture).abs(),
    ]
    .into_iter()
    .fold(0.0_f32, f32::max)
}

#[inline]
fn fire_field_drift(fire: &ChunkSurfaceFire, thermal: &ThermalState) -> f32 {
    let slab_heat = thermal.surface_heat.first().copied().unwrap_or(0.0);
    (fire.heat - slab_heat).abs()
}

#[inline]
fn smoke_field_drift(smoke: &ChunkSmokeField, contamination: &crate::substrate::types::ContaminationState) -> f32 {
    let slab = contamination.airborne.first().copied().unwrap_or(0.0);
    (smoke.density - slab).abs()
}

/// Weather + fire (+ optional smoke) drift for ECS retire window — excludes overlay rollup.
#[must_use]
pub fn ecs_hybrid_field_drift_max(
    registry: &WorldSubstrateRegistry,
    query: &Query<(
        &Chunk,
        &ChunkWeather,
        Option<&ChunkSurfaceFire>,
        Option<&ChunkSmokeField>,
    )>,
    include_smoke: bool,
) -> f32 {
    let mut max_drift = 0.0_f32;
    for (chunk, wx, fire, smoke) in query.iter() {
        let key = ChunkKey::from(chunk.coord);
        let Some(state) = registry.chunks.get(key) else {
            continue;
        };
        max_drift = max_drift.max(weather_field_drift(wx, &state.atmosphere.local));
        if let Some(f) = fire {
            max_drift = max_drift.max(fire_field_drift(f, &state.thermal));
        }
        if include_smoke {
            if let Some(s) = smoke {
                max_drift = max_drift.max(smoke_field_drift(s, &state.contamination));
            }
        }
    }
    max_drift
}

#[must_use]
pub fn substrate_dual_write_mirror_enabled() -> bool {
    matches!(
        std::env::var("RUST_ENGINE_SUBSTRATE_DUAL_WRITE").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    )
}

/// ECS → slab mirror for resident keys only (PR-2: no slab → ECS writeback).
pub fn sync_ecs_to_substrate_dual_write_system(
    base: Res<State<BaseState>>,
    mut registry: ResMut<WorldSubstrateRegistry>,
    mut shim: ResMut<DualWriteShimState>,
    query: Query<(
        &Chunk,
        &ChunkWeather,
        Option<&ChunkSurfaceFire>,
        Option<&ChunkSmokeField>,
    )>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    if !substrate_dual_write_mirror_enabled() {
        shim.enabled = false;
        return;
    }

    let mut synced = 0_u32;
    for (chunk, wx, fire, smoke) in &query {
        let key = ChunkKey::from(chunk.coord);
        if !registry.chunks.is_resident(key) {
            continue;
        }
        let Some(state) = registry.chunks.get_mut(key) else {
            continue;
        };
        sync_chunk_weather_to_slab(&mut state.atmosphere.local, wx);
        if let Some(f) = fire {
            sync_surface_fire_to_thermal(&mut state.thermal, f);
        }
        if let Some(s) = smoke {
            sync_chunk_smoke_to_slab(&mut state.contamination, s);
        }
        synced += 1;
    }

    if synced > 0 {
        shim.enabled = true;
        shim.synced_resident_keys = synced;
    }
}

#[inline]
fn overlay_map_drift(
    dynamic: &crate::substrate::types::DynamicOverlaySlice,
    cell_index: usize,
    expected: f32,
    read: impl Fn(&crate::substrate::types::DynamicOverlaySlice, usize) -> f32,
) -> f32 {
    if cell_index >= dynamic.mud.len() {
        return f32::MAX;
    }
    (read(dynamic, cell_index) - expected).abs()
}

/// PR4-2: include sparse `DynamicTerrainOverlay` slices in drift rollup.
pub fn compare_dynamic_overlay_drift_system(
    registry: Res<WorldSubstrateRegistry>,
    mut shim: ResMut<DualWriteShimState>,
    overlay: Option<Res<DynamicTerrainOverlay>>,
) {
    if !shim.enabled {
        return;
    }
    let Some(overlay) = overlay else {
        return;
    };
    let mut max_drift = shim.drift_max;
    for (cell_key, expected) in &overlay.mud {
        let slab_key = ChunkKey::from(cell_key.chunk);
        let Some(state) = registry.chunks.get(slab_key) else {
            max_drift = max_drift.max(f32::MAX);
            continue;
        };
        let i = cell_key.cell_index as usize;
        max_drift = max_drift.max(overlay_map_drift(
            &state.dynamic,
            i,
            *expected,
            |d, idx| d.mud[idx],
        ));
    }
    for (cell_key, expected) in &overlay.snow {
        let slab_key = ChunkKey::from(cell_key.chunk);
        let Some(state) = registry.chunks.get(slab_key) else {
            max_drift = max_drift.max(f32::MAX);
            continue;
        };
        let i = cell_key.cell_index as usize;
        max_drift = max_drift.max(overlay_map_drift(
            &state.dynamic,
            i,
            *expected,
            |d, idx| d.snow_accum[idx],
        ));
    }
    for (cell_key, expected) in &overlay.danger {
        let slab_key = ChunkKey::from(cell_key.chunk);
        let Some(state) = registry.chunks.get(slab_key) else {
            max_drift = max_drift.max(f32::MAX);
            continue;
        };
        let i = cell_key.cell_index as usize;
        max_drift = max_drift.max(overlay_map_drift(
            &state.dynamic,
            i,
            *expected,
            |d, idx| d.danger[idx],
        ));
    }
    for (cell_key, expected) in &overlay.congestion {
        let slab_key = ChunkKey::from(cell_key.chunk);
        let Some(state) = registry.chunks.get(slab_key) else {
            max_drift = max_drift.max(f32::MAX);
            continue;
        };
        let i = cell_key.cell_index as usize;
        max_drift = max_drift.max(overlay_map_drift(
            &state.dynamic,
            i,
            *expected,
            |d, idx| d.congestion[idx],
        ));
    }
    shim.drift_max = max_drift;
}

/// End-of-pass drift metric between ECS components and slab mirror.
pub fn compare_dual_write_drift_system(
    registry: Res<WorldSubstrateRegistry>,
    mut shim: ResMut<DualWriteShimState>,
    query: Query<(
        &Chunk,
        &ChunkWeather,
        Option<&ChunkSurfaceFire>,
        Option<&ChunkSmokeField>,
    )>,
) {
    if !shim.enabled {
        return;
    }

    let mut max_drift = 0.0_f32;
    for (chunk, wx, fire, smoke) in &query {
        let key = ChunkKey::from(chunk.coord);
        let Some(state) = registry.chunks.get(key) else {
            continue;
        };
        max_drift = max_drift.max(weather_field_drift(wx, &state.atmosphere.local));
        if let Some(f) = fire {
            max_drift = max_drift.max(fire_field_drift(f, &state.thermal));
        }
        if let Some(s) = smoke {
            max_drift = max_drift.max(smoke_field_drift(s, &state.contamination));
        }
    }
    shim.drift_max = max_drift;
}

#[must_use]
pub fn dual_write_shim_green(shim: &DualWriteShimState) -> bool {
    shim.enabled && shim.drift_max < DUAL_WRITE_DRIFT_EPSILON
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::state::app::StatesPlugin;
    use crate::engine::states::BaseState;
    use crate::substrate::WorldChunkState;
    use crate::terrain::generation::{Chunk, ChunkCellMatrix};

    #[test]
    fn dual_write_weather_fire_drift_under_epsilon() {
        std::env::set_var("RUST_ENGINE_SUBSTRATE_DUAL_WRITE", "1");
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<WorldSubstrateRegistry>()
            .init_resource::<DualWriteShimState>()
            .init_state::<BaseState>()
            .insert_state(BaseState::Simulation)
            .add_systems(
                Update,
                (
                    sync_ecs_to_substrate_dual_write_system,
                    compare_dual_write_drift_system,
                )
                    .chain(),
            );

        let coord = IVec2::new(3, 2);
        let key = ChunkKey::from(coord);
        let mut state = WorldChunkState::new_empty(key, 4);
        {
            let mut reg = app.world_mut().resource_mut::<WorldSubstrateRegistry>();
            reg.chunks.insert(key, state);
            reg.chunks.set_resident(key, true);
        }

        app.world_mut().spawn((
            Chunk { coord },
            ChunkCellMatrix::new(UVec2::new(4, 4)),
            ChunkWeather {
                rain_intensity: 0.42,
                fog_density: 0.11,
                ..Default::default()
            },
            ChunkSurfaceFire {
                heat: 0.55,
                fuel: 0.8,
            },
        ));

        app.update();

        let shim = app.world().resource::<DualWriteShimState>();
        assert!(shim.enabled);
        assert!(dual_write_shim_green(shim));

        let reg = app.world().resource::<WorldSubstrateRegistry>();
        let slab = reg.chunks.get(key).expect("slab");
        assert!((slab.atmosphere.local.rain_intensity - 0.42).abs() < DUAL_WRITE_DRIFT_EPSILON);
        assert!((slab.thermal.surface_heat[0] - 0.55).abs() < DUAL_WRITE_DRIFT_EPSILON);
    }

    /// **DEHACK-WSS-001** — slab authoritative; mirror only when env opt-in.
    #[test]
    fn dehack_wss_001_compare_only_by_default() {
        let _ = std::env::remove_var("RUST_ENGINE_SUBSTRATE_DUAL_WRITE");
        assert!(!substrate_dual_write_mirror_enabled());

        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<WorldSubstrateRegistry>()
            .init_resource::<DualWriteShimState>()
            .init_state::<BaseState>()
            .insert_state(BaseState::Simulation)
            .add_systems(Update, sync_ecs_to_substrate_dual_write_system);

        let coord = IVec2::new(1, 1);
        let key = ChunkKey::from(coord);
        {
            let mut reg = app.world_mut().resource_mut::<WorldSubstrateRegistry>();
            reg.chunks
                .insert(key, WorldChunkState::new_empty(key, 4));
            reg.chunks.set_resident(key, true);
        }
        app.world_mut().spawn((
            Chunk { coord },
            ChunkCellMatrix::new(UVec2::new(4, 4)),
            ChunkWeather::default(),
        ));
        app.update();

        let shim = app.world().resource::<DualWriteShimState>();
        assert!(!shim.enabled, "mirror disabled without RUST_ENGINE_SUBSTRATE_DUAL_WRITE");
    }
}
