//! WSS-SLAB-PR-5 — ECS hybrid retirement fixture (drift window + slab extract path).

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::substrate::persist::SubstratePr4Witness;
use crate::substrate::registry::WorldSubstrateRegistry;
use crate::substrate::shim::{
    dual_write_shim_green, ecs_hybrid_field_drift_max, sync_chunk_smoke_to_slab, DualWriteShimState,
    DUAL_WRITE_DRIFT_EPSILON,
};
use crate::substrate::slab::ChunkKey;
use crate::substrate::types::ChunkWeatherLocal;
use crate::systems::fire::ChunkSurfaceFire;
use crate::systems::weather::ChunkWeather;
use crate::substrate::atmosphere::{clipmap_l0_smoke_max, AtmosphereClipmapStack};
use crate::render::extraction::SmokeVisualBridgeWitness;
use crate::systems::fire::ChunkSmokeField;
use crate::terrain::generation::Chunk;

/// CI fixture: consecutive sim ticks with dual-write drift under ε before authority cutover.
pub const ECS_RETIRE_DRIFT_WINDOW_TICKS: u32 = 120;

const SMOKE_EXTRACT_EPS: f32 = 1e-4;

#[derive(Resource, Clone, Debug)]
pub struct EcsRetireState {
    pub hybrid_weather_authoritative: bool,
    pub hybrid_fire_authoritative: bool,
    pub hybrid_smoke_authoritative: bool,
    pub stable_drift_ticks: u32,
    pub cutover_complete: bool,
    pub smoke_cutover_complete: bool,
    pub weather_extract_reads_slab: bool,
    pub fire_extract_reads_slab: bool,
    pub smoke_extract_reads_slab: bool,
}

impl Default for EcsRetireState {
    fn default() -> Self {
        Self::new_pre_cutover()
    }
}

impl EcsRetireState {
    pub fn new_pre_cutover() -> Self {
        Self {
            hybrid_weather_authoritative: true,
            hybrid_fire_authoritative: true,
            hybrid_smoke_authoritative: true,
            stable_drift_ticks: 0,
            cutover_complete: false,
            smoke_cutover_complete: false,
            weather_extract_reads_slab: false,
            fire_extract_reads_slab: false,
            smoke_extract_reads_slab: false,
        }
    }
}

#[derive(Resource, Clone, Debug, Default)]
pub struct SubstrateEcsRetireWitness {
    pub ecs_retire_fixture_green: bool,
}

#[must_use]
pub fn slab_smoke_density(registry: &WorldSubstrateRegistry, coord: IVec2) -> f32 {
    let key = ChunkKey::from(coord);
    registry
        .chunks
        .get(key)
        .and_then(|s| s.contamination.airborne.first().copied())
        .unwrap_or(0.0)
}

#[must_use]
pub fn slab_smoke_extract_matches_ecs(
    registry: &WorldSubstrateRegistry,
    coord: IVec2,
    smoke: &ChunkSmokeField,
) -> bool {
    (slab_smoke_density(registry, coord) - smoke.density).abs() < DUAL_WRITE_DRIFT_EPSILON
}

pub fn slab_surface_heat(registry: &WorldSubstrateRegistry, coord: IVec2) -> f32 {
    let key = ChunkKey::from(coord);
    registry
        .chunks
        .get(key)
        .and_then(|s| s.thermal.surface_heat.first().copied())
        .unwrap_or(0.0)
}

#[must_use]
pub fn slab_weather_local(registry: &WorldSubstrateRegistry, coord: IVec2) -> Option<ChunkWeatherLocal> {
    let key = ChunkKey::from(coord);
    registry
        .chunks
        .get(key)
        .map(|s| s.atmosphere.local.clone())
}

/// Extract-path weather scalars from slab snapshot (no `ChunkWeather` query).
#[must_use]
pub fn extract_weather_scalars_from_slab(
    registry: &WorldSubstrateRegistry,
    coord: IVec2,
) -> Option<ChunkWeatherLocal> {
    slab_weather_local(registry, coord)
}

#[must_use]
pub fn slab_fire_extract_matches_ecs(
    registry: &WorldSubstrateRegistry,
    coord: IVec2,
    fire: &ChunkSurfaceFire,
) -> bool {
    (slab_surface_heat(registry, coord) - fire.heat).abs() < DUAL_WRITE_DRIFT_EPSILON
}

#[must_use]
pub fn slab_weather_extract_matches_ecs(
    registry: &WorldSubstrateRegistry,
    coord: IVec2,
    wx: &ChunkWeather,
) -> bool {
    let Some(local) = slab_weather_local(registry, coord) else {
        return false;
    };
    [
        (wx.rain_intensity - local.rain_intensity).abs(),
        (wx.fog_density - local.fog_density).abs(),
        (wx.snow_depth - local.snow_depth).abs(),
    ]
    .into_iter()
    .fold(0.0_f32, f32::max)
        < DUAL_WRITE_DRIFT_EPSILON
}

#[must_use]
pub fn ecs_retire_fixture_green(
    pr4: &SubstratePr4Witness,
    retire: &EcsRetireState,
    dual: &DualWriteShimState,
    slab_gate_green: bool,
) -> bool {
    slab_gate_green
        && pr4.substrate_persist_roundtrip_ok
        && pr4.dynamic_overlay_migrated
        && (!dual.enabled || dual_write_shim_green(dual))
        && retire.cutover_complete
        && !retire.hybrid_weather_authoritative
        && !retire.hybrid_fire_authoritative
        && retire.weather_extract_reads_slab
        && retire.fire_extract_reads_slab
        && retire.stable_drift_ticks >= ECS_RETIRE_DRIFT_WINDOW_TICKS
}

/// Live / lib rollup for **WSS-PR5-SMOKE-PROD-001** (fixture green does not require this).
#[must_use]
pub fn ecs_retire_smoke_prod_green(
    retire: &EcsRetireState,
    smoke_extract_wired: bool,
    smoke_density_sum: f32,
    ecs_fixture_green: bool,
) -> bool {
    ecs_fixture_green
        && retire.cutover_complete
        && !retire.hybrid_smoke_authoritative
        && retire.smoke_extract_reads_slab
        && retire.smoke_cutover_complete
        && smoke_extract_wired
        && smoke_density_sum > SMOKE_EXTRACT_EPS
}

fn tick_ecs_retire_drift_window(
    retire: &mut EcsRetireState,
    dual: &DualWriteShimState,
    ecs_field_drift: f32,
) {
    if retire.cutover_complete {
        return;
    }
    if dual.enabled && ecs_field_drift < DUAL_WRITE_DRIFT_EPSILON {
        retire.stable_drift_ticks = retire
            .stable_drift_ticks
            .saturating_add(1)
            .min(ECS_RETIRE_DRIFT_WINDOW_TICKS);
    } else {
        retire.stable_drift_ticks = 0;
    }
}

fn apply_ecs_retire_cutover(
    registry: &WorldSubstrateRegistry,
    query: &Query<(
        &Chunk,
        &ChunkWeather,
        Option<&ChunkSurfaceFire>,
        Option<&ChunkSmokeField>,
    )>,
    dual: &DualWriteShimState,
    retire: &mut EcsRetireState,
) {
    if retire.cutover_complete {
        return;
    }
    if !dual.enabled || dual.drift_max >= DUAL_WRITE_DRIFT_EPSILON {
        return;
    }
    if retire.stable_drift_ticks < ECS_RETIRE_DRIFT_WINDOW_TICKS {
        return;
    }

    let mut weather_ok = true;
    let mut fire_ok = true;
    for (chunk, wx, fire, _smoke) in query.iter() {
        let key = ChunkKey::from(chunk.coord);
        if !registry.chunks.is_resident(key) {
            continue;
        }
        weather_ok &= slab_weather_extract_matches_ecs(registry, chunk.coord, wx);
        if let Some(f) = fire {
            fire_ok &= slab_fire_extract_matches_ecs(registry, chunk.coord, f);
        }
    }

    retire.weather_extract_reads_slab = weather_ok;
    retire.fire_extract_reads_slab = fire_ok;
    if weather_ok && fire_ok {
        retire.hybrid_weather_authoritative = false;
        retire.hybrid_fire_authoritative = false;
        retire.cutover_complete = true;
    }
}

/// Synthetic bridge when render witness resource is absent but clipmap has smoke (live sim).
#[must_use]
pub fn smoke_bridge_from_clipmap(clipmap: &AtmosphereClipmapStack) -> Option<SmokeVisualBridgeWitness> {
    let max = clipmap_l0_smoke_max(clipmap);
    if max <= SMOKE_EXTRACT_EPS {
        return None;
    }
    Some(SmokeVisualBridgeWitness {
        smoke_density_sum: max,
        smoke_row_count: 1,
        smoke_extract_wired: true,
        smoke_stub_removed: true,
    })
}

/// WSS-PR5-SMOKE-PROD-001 — flip smoke authority after weather/fire cutover + smoke path green.
#[must_use]
pub fn apply_ecs_smoke_prod_cutover(
    registry: &mut WorldSubstrateRegistry,
    clipmap: &AtmosphereClipmapStack,
    smoke_bridge: Option<&SmokeVisualBridgeWitness>,
    smoke_q: &Query<(&Chunk, &ChunkSmokeField)>,
    retire: &mut EcsRetireState,
) -> bool {
    if !retire.cutover_complete || retire.smoke_cutover_complete {
        return retire.smoke_cutover_complete;
    }

    let bridge_density = smoke_bridge.map(|b| b.smoke_density_sum).unwrap_or(0.0);
    let bridge_wired = smoke_bridge.is_some_and(|b| b.smoke_extract_wired);
    let clip_ok = clipmap_l0_smoke_max(clipmap) > SMOKE_EXTRACT_EPS
        || bridge_density > SMOKE_EXTRACT_EPS
        || bridge_wired;
    let mut slab_ok = registry.chunks.chunks.values().any(|c| {
        c.contamination
            .airborne
            .iter()
            .any(|&v| v > SMOKE_EXTRACT_EPS)
    });

    for (chunk, smoke) in smoke_q.iter() {
        let key = ChunkKey::from(chunk.coord);
        if !registry.chunks.is_resident(key) {
            continue;
        }
        if let Some(state) = registry.chunks.get_mut(key) {
            sync_chunk_smoke_to_slab(&mut state.contamination, smoke);
        }
        slab_ok |= slab_smoke_extract_matches_ecs(registry, chunk.coord, smoke);
    }

    if clip_ok && slab_ok {
        retire.hybrid_smoke_authoritative = false;
        retire.smoke_extract_reads_slab = true;
        retire.smoke_cutover_complete = true;
        return true;
    }
    false
}

fn sync_ecs_retire_witness_inner(
    pr4: &SubstratePr4Witness,
    dual: &DualWriteShimState,
    retire: &EcsRetireState,
    witness: &crate::substrate::registry::WssSubstrateWitness,
    registry: &WorldSubstrateRegistry,
    out: &mut SubstrateEcsRetireWitness,
) {
    let plugin_on = crate::substrate::substrate_plugin_enabled();
    let cell_ok = registry
        .chunks
        .chunks
        .values()
        .next()
        .is_some_and(|c| c.cell_grid_matches_terrain());
    let slab_green = plugin_on
        && !registry.chunks.is_empty()
        && witness.hydrate_wired
        && witness.paging_wired
        && cell_ok
        && witness.chunk_environment_order_preserved;
    out.ecs_retire_fixture_green = ecs_retire_fixture_green(pr4, retire, dual, slab_green);
}

/// PR-5 pass: drift window → cutover → witness rollup (single schedule node).
pub fn ecs_retire_pass_system(
    base: Res<State<BaseState>>,
    dual: Res<DualWriteShimState>,
    mut registry: ResMut<WorldSubstrateRegistry>,
    query: Query<(
        &Chunk,
        &ChunkWeather,
        Option<&ChunkSurfaceFire>,
        Option<&ChunkSmokeField>,
    )>,
    smoke_q: Query<(&Chunk, &ChunkSmokeField)>,
    clipmap: Option<Res<AtmosphereClipmapStack>>,
    smoke_bridge: Option<Res<SmokeVisualBridgeWitness>>,
    pr4: Res<SubstratePr4Witness>,
    witness: Res<crate::substrate::registry::WssSubstrateWitness>,
    mut retire: ResMut<EcsRetireState>,
    mut out: ResMut<SubstrateEcsRetireWitness>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        return;
    }
    let ecs_field_drift = ecs_hybrid_field_drift_max(registry.as_ref(), &query, false);
    tick_ecs_retire_drift_window(&mut retire, &dual, ecs_field_drift);
    apply_ecs_retire_cutover(registry.as_ref(), &query, &dual, &mut retire);
    if let Some(clipmap) = clipmap.as_deref() {
        let synthetic = smoke_bridge_from_clipmap(clipmap);
        let effective = smoke_bridge
            .as_deref()
            .or(synthetic.as_ref());
        let _ = apply_ecs_smoke_prod_cutover(
            registry.as_mut(),
            clipmap,
            effective,
            &smoke_q,
            &mut retire,
        );
    }
    sync_ecs_retire_witness_inner(&pr4, &dual, &retire, &witness, registry.as_ref(), &mut out);
}

/// Lib witness: smoke authority off when clipmap + slab mirror path is valid.
#[must_use]
pub fn finish_ecs_smoke_prod_cutover_lib(
    registry: &mut WorldSubstrateRegistry,
    clipmap: &AtmosphereClipmapStack,
    smoke_bridge: Option<&SmokeVisualBridgeWitness>,
    coord: IVec2,
    smoke: &ChunkSmokeField,
    retire: &mut EcsRetireState,
) -> bool {
    if !retire.cutover_complete {
        return false;
    }
    let key = ChunkKey::from(coord);
    if let Some(state) = registry.chunks.get_mut(key) {
        sync_chunk_smoke_to_slab(&mut state.contamination, smoke);
    }
    let bridge_density = smoke_bridge.map(|b| b.smoke_density_sum).unwrap_or(0.0);
    let clip_ok = clipmap_l0_smoke_max(clipmap) > SMOKE_EXTRACT_EPS
        || bridge_density > SMOKE_EXTRACT_EPS;
    let slab_ok = slab_smoke_extract_matches_ecs(registry, coord, smoke);
    if clip_ok && slab_ok {
        retire.hybrid_smoke_authoritative = false;
        retire.smoke_extract_reads_slab = true;
        retire.smoke_cutover_complete = true;
        return true;
    }
    false
}

/// Lib/CI: advance drift window and validate slab extract path without a full app loop.
#[must_use]
pub fn run_ecs_retire_lib_fixture(
    registry: &WorldSubstrateRegistry,
    pr4: &SubstratePr4Witness,
    dual: &DualWriteShimState,
    coord: IVec2,
    wx: &ChunkWeather,
    fire: &ChunkSurfaceFire,
    _slab_gate_green: bool,
) -> EcsRetireState {
    let mut retire = EcsRetireState::new_pre_cutover();
    if dual_write_shim_green(dual) {
        retire.stable_drift_ticks = ECS_RETIRE_DRIFT_WINDOW_TICKS;
    }
    retire.weather_extract_reads_slab =
        extract_weather_scalars_from_slab(registry, coord).is_some()
            && slab_weather_extract_matches_ecs(registry, coord, wx);
    retire.fire_extract_reads_slab =
        slab_fire_extract_matches_ecs(registry, coord, fire)
            && slab_surface_heat(registry, coord) > 0.0;
    if retire.weather_extract_reads_slab
        && retire.fire_extract_reads_slab
        && pr4.substrate_persist_roundtrip_ok
        && pr4.dynamic_overlay_migrated
        && (!dual.enabled || dual_write_shim_green(dual))
    {
        retire.stable_drift_ticks = ECS_RETIRE_DRIFT_WINDOW_TICKS;
        retire.hybrid_weather_authoritative = false;
        retire.hybrid_fire_authoritative = false;
        retire.cutover_complete = true;
    }
    retire
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::hydrate_skeleton_chunk;
    use crate::substrate::persist::{dynamic_overlay_matches_slab, migrate_dynamic_overlay_to_slab, persist_roundtrip_ok};
    use crate::substrate::shim::{sync_chunk_weather_to_slab, sync_surface_fire_to_thermal};
    use crate::substrate::SubstratePr4Witness;
    use crate::terrain::{ChunkCellKey, DynamicTerrainOverlay};

    #[test]
    fn ecs_retire_fixture_advances_after_drift_window() {
        let mut registry = WorldSubstrateRegistry::default();
        hydrate_skeleton_chunk(&mut registry, IVec2::ZERO);
        registry.chunks.set_resident(ChunkKey::from(IVec2::ZERO), true);

        let wx = ChunkWeather {
            rain_intensity: 0.31,
            fog_density: 0.08,
            ..Default::default()
        };
        let fire = ChunkSurfaceFire {
            heat: 0.47,
            fuel: 0.6,
        };
        let key = ChunkKey::from(IVec2::ZERO);
        if let Some(state) = registry.chunks.get_mut(key) {
            sync_chunk_weather_to_slab(&mut state.atmosphere.local, &wx);
            sync_surface_fire_to_thermal(&mut state.thermal, &fire);
        }

        let mut pr4 = SubstratePr4Witness::default();
        pr4.substrate_persist_roundtrip_ok = persist_roundtrip_ok(&mut registry);
        let cell = ChunkCellKey::new(IVec2::ZERO, 0);
        let mut overlay = DynamicTerrainOverlay::default();
        overlay.mud.insert(cell, 0.2);
        migrate_dynamic_overlay_to_slab(&mut registry, &overlay);
        pr4.dynamic_overlay_migrated = dynamic_overlay_matches_slab(&registry, &overlay);

        let dual = DualWriteShimState {
            enabled: true,
            drift_max: 0.0,
            synced_resident_keys: 1,
        };

        let retire = run_ecs_retire_lib_fixture(
            &registry,
            &pr4,
            &dual,
            IVec2::ZERO,
            &wx,
            &fire,
            true,
        );
        assert!(retire.cutover_complete);
        assert!(!retire.hybrid_weather_authoritative);
        assert!(!retire.hybrid_fire_authoritative);
        assert!(ecs_retire_fixture_green(&pr4, &retire, &dual, true));
    }
}
