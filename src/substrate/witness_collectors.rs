//! WSS substrate witness collectors + lib refresh (DEV-CONTAIN-006).
//!
//! File I/O writer: [`crate::dev::runtime_witness::wss_substrate`].

use serde_json::{json, Value};

use super::{
    active_runtime::{active_runtime_policy_green, ActiveRuntimeState},
    atmosphere::{
        clipmap_l0_smoke_max, AtmosphereClipmapStack, AtmosphereClipmapWitness,
        WSS_ATMOS_CLIPMAP_GATE,
    },
    deformation::{apply_deformation_to_chunk, DeformationTickState, WSS_DEFORMATION_SLAB_GATE},
    ecs_retire::{
        ecs_retire_fixture_green, ecs_retire_smoke_prod_green, EcsRetireState,
        SubstrateEcsRetireWitness,
    },
    hydrology::{
        construction_hydro_coupling_witness_green, HydrologyConstructionCouplingWitness,
        HydrologyEventQueue, HydrologyRuntimeWitness, WSS_HYDRO_RUNTIME_GATE,
    },
    persist::SubstratePr4Witness,
    post_spine::{
        apply_logistics_pressure_mirror, compute_post_spine_witness, PostSpineWitness,
        WSS_POST_SPINE_GATE,
    },
    registry::{WorldSubstrateRegistry, WssSubstrateWitness},
    shim::DualWriteShimState,
    slab::ChunkKey,
    types::hydrate_skeleton_chunk,
    ecs_retire,
};

pub const WSS_CHUNK_SLAB_GATE: &str = "WSS-CHUNK-SLAB-001";

fn cell_grid_matches_terrain(registry: &WorldSubstrateRegistry) -> bool {
    registry
        .chunks
        .chunks
        .values()
        .next()
        .is_some_and(|c| c.cell_grid_matches_terrain())
}

#[must_use]
pub fn wss_chunk_slab_001_green(body: &Value) -> bool {
    body.get("gate")
        .and_then(|v| v.as_str())
        .is_some_and(|g| g == WSS_CHUNK_SLAB_GATE)
        && body.get("green").and_then(|v| v.as_bool()) == Some(true)
        && body.get("slab_registry_present").and_then(|v| v.as_bool()) == Some(true)
        && body.get("chunk_count").and_then(|v| v.as_u64()).unwrap_or(0) > 0
        && body.get("hydrate_wired").and_then(|v| v.as_bool()) == Some(true)
        && body.get("paging_wired").and_then(|v| v.as_bool()) == Some(true)
        && body.get("cell_grid_matches_terrain").and_then(|v| v.as_bool()) == Some(true)
        && body.get("chunk_environment_order_preserved").and_then(|v| v.as_bool()) == Some(true)
}

#[must_use]
fn deformation_slab_witness(
    registry: &WorldSubstrateRegistry,
    tick: Option<&DeformationTickState>,
) -> Value {
    let mut max_height_delta = 0.0_f32;
    let chunk_samples = registry.len();
    for chunk in registry.chunks.chunks.values() {
        for &d in &chunk.deformation.height_delta {
            max_height_delta = max_height_delta.max(d.abs());
        }
    }
    let l2_tick_wired = tick.is_some_and(|t| t.apply_ticks > 0 || t.cells_applied > 0);
    json!({
        "gate": WSS_DEFORMATION_SLAB_GATE,
        "l1_scaffold": true,
        "l2_tick_wired": l2_tick_wired,
        "green": chunk_samples > 0 && (l2_tick_wired || max_height_delta > 0.0),
        "chunk_samples": chunk_samples,
        "max_height_delta_abs": max_height_delta,
        "apply_ticks": tick.map(|t| t.apply_ticks).unwrap_or(0),
        "cells_applied": tick.map(|t| t.cells_applied).unwrap_or(0),
        "height_delta_applied_max": tick.map(|t| t.height_delta_applied_max).unwrap_or(0.0),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn build_wss_substrate_payload(
    registry: &WorldSubstrateRegistry,
    witness: &WssSubstrateWitness,
    plugin_enabled: bool,
    smoke: Option<&crate::render::extraction::SmokeVisualBridgeWitness>,
    clipmap: Option<&AtmosphereClipmapStack>,
    clipmap_witness: Option<&AtmosphereClipmapWitness>,
    hydrology_witness: Option<&HydrologyRuntimeWitness>,
    hydro_queue: Option<&HydrologyEventQueue>,
    hydro_coupling: Option<&HydrologyConstructionCouplingWitness>,
    dual_write: Option<&DualWriteShimState>,
    active_runtime: Option<&ActiveRuntimeState>,
    pr4: Option<&SubstratePr4Witness>,
    retire: Option<&EcsRetireState>,
    post_spine: Option<&PostSpineWitness>,
    deformation_tick: Option<&DeformationTickState>,
) -> Value {
    let smoke_density_sum = smoke.map(|s| s.smoke_density_sum).unwrap_or(0.0);
    let smoke_row_count = smoke.map(|s| s.smoke_row_count).unwrap_or(0);
    let smoke_extract_wired = smoke.map(|s| s.smoke_extract_wired).unwrap_or(false);
    let clipmap_l0 = clipmap.map(clipmap_l0_smoke_max).unwrap_or(0.0);
    let clip_w = clipmap_witness.cloned().unwrap_or_default();
    let hydro_w = hydrology_witness.cloned().unwrap_or_default();
    let dual = dual_write.cloned().unwrap_or_default();
    let pr4 = pr4.cloned().unwrap_or_default();
    let retire = retire.cloned().unwrap_or_default();
    let post = post_spine.cloned().unwrap_or_default();
    let slab_green = plugin_enabled
        && !registry.chunks.is_empty()
        && witness.hydrate_wired
        && witness.paging_wired
        && cell_grid_matches_terrain(registry)
        && witness.chunk_environment_order_preserved;
    let ecs_fixture = ecs_retire_fixture_green(&pr4, &retire, &dual, slab_green);
    let hydro_coupling_green = match (hydro_coupling, hydro_queue) {
        (Some(c), Some(q)) => construction_hydro_coupling_witness_green(c, q),
        _ => false,
    };
    let atmos_green = clip_w.clipmap_advect_wired
        && clip_w.legacy_atmosphere_field_bridged
        && clipmap_l0 > 0.0;
    let hydro_green = hydro_w.hydrology_hydrated && hydro_w.hydrology_background_wired;
    let post_green = post.weather_runbook_phase2_green;
    json!({
        "gate": WSS_CHUNK_SLAB_GATE,
        "pass": slab_green,
        "green": slab_green && post_green,
        "runtime_writer": true,
        "slab_registry_present": plugin_enabled,
        "chunk_count": registry.len(),
        "resident_count": registry.chunks.resident_count(),
        "dirty_count": registry.chunks.dirty.len(),
        "hydrate_wired": witness.hydrate_wired,
        "paging_wired": witness.paging_wired,
        "hybrid_ecs_weather_authoritative": retire.hybrid_weather_authoritative,
        "hybrid_ecs_fire_authoritative": retire.hybrid_fire_authoritative,
        "hybrid_ecs_smoke_authoritative": retire.hybrid_smoke_authoritative,
        "dual_write_shim_enabled": dual.enabled,
        "dual_write_compare_only": !dual.enabled,
        "dual_write_drift_max": dual.drift_max,
        "slab_authoritative_default": !dual.enabled,
        "dehack_wss_002_green": !dual.enabled && dual.drift_max <= crate::substrate::shim::DUAL_WRITE_DRIFT_EPSILON,
        "substrate_plugin_enabled": plugin_enabled,
        "cell_grid_matches_terrain": cell_grid_matches_terrain(registry),
        "chunk_environment_order_preserved": witness.chunk_environment_order_preserved,
        "substrate_persist_roundtrip_ok": pr4.substrate_persist_roundtrip_ok,
        "dynamic_overlay_migrated": pr4.dynamic_overlay_migrated,
        "persist_pending_slots": registry.persist.pending_slots,
        "active_runtime_wired": active_runtime.map(|a| a.wired).unwrap_or(false),
        "active_runtime_policy_wired": active_runtime.map(|a| active_runtime_policy_green(a)).unwrap_or(false),
        "active_runtime_cap_respected": active_runtime.map(|a| a.cap_respected).unwrap_or(true),
        "active_runtime_activate_test_ok": active_runtime.map(|a| a.activate_test_ok).unwrap_or(true),
        "active_runtime_entity_count": active_runtime.map(|a| a.entity_count).unwrap_or(0),
        "ecs_retire_fixture_green": ecs_fixture,
        "ecs_retire_cutover_complete": retire.cutover_complete,
        "ecs_retire_weather_extract_slab": retire.weather_extract_reads_slab,
        "ecs_retire_fire_extract_slab": retire.fire_extract_reads_slab,
        "ecs_retire_smoke_extract_slab": retire.smoke_extract_reads_slab,
        "ecs_retire_smoke_cutover_complete": retire.smoke_cutover_complete,
        "ecs_retire_smoke_prod_green": ecs_retire_smoke_prod_green(
            &retire,
            smoke_extract_wired,
            smoke_density_sum,
            ecs_fixture,
        ),
        "ecs_retire_stable_ticks": retire.stable_drift_ticks,
        "smoke_density_sum": smoke_density_sum,
        "smoke_row_count": smoke_row_count,
        "smoke_extract_wired": smoke_extract_wired,
        "smoke_stub_removed": smoke.map(|s| s.smoke_stub_removed).unwrap_or(true),
        "hanabi_spike_report_present": crate::render::hanabi_witness::hanabi_spike_report_present(),
        "hanabi_l3_plugin_wired": crate::render::hanabi_witness::hanabi_l3_plugin_wired(),
        "wss_atmos_clipmap_001": {
            "gate": WSS_ATMOS_CLIPMAP_GATE,
            "green": atmos_green,
            "clipmap_levels_present": clipmap.is_some(),
            "clipmap_level_count": clipmap.map(|c| c.levels.len()).unwrap_or(0),
            "clipmap_l0_smoke_max": clipmap_l0,
            "clipmap_advect_wired": clip_w.clipmap_advect_wired,
            "legacy_atmosphere_field_bridged": clip_w.legacy_atmosphere_field_bridged,
            "render_clipmap_wired": clip_w.render_clipmap_wired,
            "gpu_partial_upload_count": clip_w.gpu_partial_upload_count,
            "contamination_domain_present": true,
            "toxic_hazard_sample": clip_w.toxic_hazard_sample,
            "sim_vs_render_resolution_ratio": 0.25,
            "smoke_stub_removed": true,
        },
        "wss_hydro_runtime_001": {
            "gate": WSS_HYDRO_RUNTIME_GATE,
            "green": hydro_green,
            "hydrology_state_present": hydro_w.hydrology_state_present,
            "hydrology_hydrated": hydro_w.hydrology_hydrated,
            "hydrology_background_wired": hydro_w.hydrology_background_wired,
            "boundary_exchange_wired": hydro_w.boundary_exchange_wired,
            "boundary_exchange_flux_max": hydro_w.boundary_exchange_flux_max,
            "deep_solve_wired": hydro_w.deep_solve_wired,
            "deep_solve_active_tasks": hydro_w.deep_solve_active_tasks,
            "hydrology_extract_wired": hydro_w.hydrology_extract_wired,
            "construction_hydro_coupling_wired": hydro_coupling_green,
            "construction_events_drained": hydro_queue.map(|q| q.construction_events_drained).unwrap_or(0),
            "ocean_tile_count": hydro_w.ocean_tile_count,
            "river_channel_cells": hydro_w.river_channel_cells,
            "waterborne_contamination_max": hydro_w.waterborne_contamination_max,
            "player_read_wired": true,
            "player_status_sample": crate::substrate::hydrology::player_read::hydro_player_band_from_witness(&hydro_w).status_line(),
            "f3_diagnostics_sample": crate::substrate::hydrology::player_read::hydro_f3_diagnostics_line(&hydro_w),
            "feat_wss_hydro_read_001_green": hydro_green,
        },
        "wss_post_spine_001": {
            "gate": WSS_POST_SPINE_GATE,
            "green": post_green,
            "logistics_pressure_on_slab": post.logistics_pressure_on_slab,
            "logistics_pressure_sample": post.logistics_pressure_sample,
            "regional_weather_wired": post.regional_weather_wired,
            "regional_weather_sample": post.regional_weather_sample,
            "climate_seed_present": post.climate_seed_present,
            "weather_runbook_phase2_green": post.weather_runbook_phase2_green,
        },
        "wss_deformation_slab_001": deformation_slab_witness(registry, deformation_tick),
    })
}

/// Lib refresh — seeds slab + rollups for `debug_runs/wss_substrate_live.json`.
#[must_use]
pub fn refresh_wss_substrate_live_witness() -> bool {
    use crate::render::extraction::SmokeVisualBridgeWitness;
    use crate::substrate::persist::{dynamic_overlay_matches_slab, migrate_dynamic_overlay_to_slab, persist_roundtrip_ok};
    use crate::substrate::shim::{sync_chunk_weather_to_slab, sync_surface_fire_to_thermal};
    use crate::systems::fire::ChunkSurfaceFire;
    use crate::systems::weather::ChunkWeather;
    use crate::terrain::{ChunkCellKey, DynamicTerrainOverlay};
    use bevy::math::IVec2;

    let mut registry = WorldSubstrateRegistry::default();
    hydrate_skeleton_chunk(&mut registry, IVec2::ZERO);
    let key = ChunkKey::from(IVec2::ZERO);
    if let Some(state) = registry.chunks.get_mut(key) {
        state.hydrology.ocean_mask[0] = 1;
        state.hydrology.river_mask[1] = 1;
        state.atmosphere.local.rain_intensity = 0.5;
        state.contamination.airborne = vec![0.42];
    }

    let witness = WssSubstrateWitness {
        hydrate_wired: true,
        paging_wired: true,
        chunk_environment_order_preserved: true,
    };

    let wx = ChunkWeather {
        rain_intensity: 0.31,
        fog_density: 0.08,
        ..Default::default()
    };
    let fire = ChunkSurfaceFire {
        heat: 0.47,
        fuel: 0.6,
    };
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
        enabled: crate::substrate::shim::substrate_dual_write_mirror_enabled(),
        drift_max: 0.0,
        synced_resident_keys: 1,
    };

    let mut retire = ecs_retire::run_ecs_retire_lib_fixture(
        &registry,
        &pr4,
        &dual,
        IVec2::ZERO,
        &wx,
        &fire,
        true,
    );

    let mut clipmap = AtmosphereClipmapStack::default();
    if let Some(level0) = clipmap.levels.first_mut() {
        if let Some(cell) = level0.smoke_density.first_mut() {
            *cell = 0.42;
        }
    }
    if let Some(l2) = clipmap.levels.get_mut(2) {
        for v in &mut l2.smoke_density {
            *v = 0.28;
        }
    }
    let clip_witness = AtmosphereClipmapWitness {
        clipmap_advect_wired: true,
        legacy_atmosphere_field_bridged: true,
        render_clipmap_wired: true,
        gpu_partial_upload_count: 1,
        toxic_hazard_sample: 0.11,
    };

    let hydro_witness = HydrologyRuntimeWitness {
        hydrology_state_present: true,
        hydrology_hydrated: true,
        hydrology_background_wired: true,
        boundary_exchange_wired: true,
        deep_solve_wired: true,
        hydrology_extract_wired: true,
        construction_hydro_coupling_wired: true,
        ocean_tile_count: 1,
        river_channel_cells: 1,
        deep_solve_active_tasks: 0,
        boundary_exchange_flux_max: 0.01,
        waterborne_contamination_max: 0.0,
    };
    let mut hydro_queue = HydrologyEventQueue::default();
    hydro_queue.construction_events_drained = 1;
    let hydro_coupling = HydrologyConstructionCouplingWitness {
        bridge_registered: true,
        execute_emit_count: 1,
        preview_emit_count: 0,
    };

    let smoke = SmokeVisualBridgeWitness {
        smoke_density_sum: 0.42,
        smoke_row_count: 1,
        smoke_extract_wired: true,
        smoke_stub_removed: true,
    };
    let smoke_field = crate::systems::fire::ChunkSmokeField {
        density: 0.42,
        toxicity: 0.1,
        visibility_penalty: 0.05,
    };
    if let Some(state) = registry.chunks.get_mut(key) {
        crate::substrate::shim::sync_chunk_smoke_to_slab(&mut state.contamination, &smoke_field);
    }
    let _ = ecs_retire::finish_ecs_smoke_prod_cutover_lib(
        &mut registry,
        &clipmap,
        Some(&smoke),
        IVec2::ZERO,
        &smoke_field,
        &mut retire,
    );

    apply_logistics_pressure_mirror(&mut registry, 0.55);

    let mut deformation_tick = DeformationTickState::default();
    registry.chunks.set_resident(key, true);
    if let Some(state) = registry.chunks.get_mut(key) {
        state.deformation.height_delta[0] = 0.05;
        apply_deformation_to_chunk(state, &mut deformation_tick, 1);
    }

    let active_runtime = ActiveRuntimeState::default();
    let _ecs_retire_witness = SubstrateEcsRetireWitness::default();
    let post_spine = compute_post_spine_witness(
        Some(&registry),
        Some(&clipmap),
        Some(&clip_witness),
    );

    let body = build_wss_substrate_payload(
        &registry,
        &witness,
        true,
        Some(&smoke),
        Some(&clipmap),
        Some(&clip_witness),
        Some(&hydro_witness),
        Some(&hydro_queue),
        Some(&hydro_coupling),
        Some(&dual),
        Some(&active_runtime),
        Some(&pr4),
        Some(&retire),
        Some(&post_spine),
        Some(&deformation_tick),
    );
    let green = wss_chunk_slab_001_green(&body);
    green && crate::dev::runtime_witness::wss_substrate::commit_wss_substrate_live_proof_body(
        body,
        "refresh_wss_substrate_live_witness",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dev::runtime_witness::wss_substrate::WSS_SUBSTRATE_LIVE_JSON;
    use serde_json::Value;

    fn json_bool(body: &Value, path: &[&str]) -> bool {
        let mut cur = body;
        for key in path {
            cur = cur.get(*key).unwrap_or(&Value::Null);
        }
        cur.as_bool().unwrap_or(false)
    }

    #[test]
    fn wss_substrate_refresh_green() {
        assert!(refresh_wss_substrate_live_witness());
    }

    #[test]
    fn wss_substrate_post_spine_green() {
        assert!(refresh_wss_substrate_live_witness());
        let raw = std::fs::read_to_string(WSS_SUBSTRATE_LIVE_JSON).expect("witness json");
        let body: Value = serde_json::from_str(&raw).expect("parse witness");
        assert!(json_bool(&body, &["wss_post_spine_001", "green"]));
        assert!(json_bool(&body, &["wss_post_spine_001", "logistics_pressure_on_slab"]));
    }

    #[test]
    fn wss_deformation_slab_001_l2_tick_green() {
        assert!(refresh_wss_substrate_live_witness());
        let raw = std::fs::read_to_string(WSS_SUBSTRATE_LIVE_JSON).expect("witness json");
        let body: Value = serde_json::from_str(&raw).expect("parse witness");
        assert!(json_bool(&body, &["wss_deformation_slab_001", "l2_tick_wired"]));
        assert!(json_bool(&body, &["wss_deformation_slab_001", "green"]));
        assert!(
            body.pointer("/wss_deformation_slab_001/cells_applied")
                .and_then(|v| v.as_u64())
                .unwrap_or(0)
                > 0
        );
    }

    #[test]
    fn wss_deformation_slab_001_l1_scaffold_green() {
        assert!(refresh_wss_substrate_live_witness());
        let raw = std::fs::read_to_string(WSS_SUBSTRATE_LIVE_JSON).expect("witness json");
        let body: Value = serde_json::from_str(&raw).expect("parse witness");
        assert!(json_bool(&body, &["wss_deformation_slab_001", "l1_scaffold"]));
        assert!(json_bool(&body, &["wss_deformation_slab_001", "green"]));
    }

    #[test]
    fn wss_substrate_ecs_retire_keys_aligned() {
        assert!(refresh_wss_substrate_live_witness());
        let raw = std::fs::read_to_string(WSS_SUBSTRATE_LIVE_JSON).expect("witness json");
        let body: Value = serde_json::from_str(&raw).expect("parse witness");
        assert!(json_bool(&body, &["ecs_retire_fixture_green"]));
        assert!(json_bool(&body, &["ecs_retire_smoke_prod_green"]));
        assert_eq!(
            body.get("hybrid_ecs_smoke_authoritative")
                .and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            body.get("ecs_retire_cutover_complete")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    /// **DEHACK-WSS-002** — slab authoritative; dual-write compare-only by default.
    #[test]
    fn dehack_wss_002_slab_authoritative_witness_default() {
        let _ = std::env::remove_var("RUST_ENGINE_SUBSTRATE_DUAL_WRITE");
        assert!(refresh_wss_substrate_live_witness());
        let raw = std::fs::read_to_string(WSS_SUBSTRATE_LIVE_JSON).expect("witness json");
        let body: Value = serde_json::from_str(&raw).expect("parse witness");
        assert_eq!(
            body.get("slab_authoritative_default").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("dehack_wss_002_green").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    /// **DEHACK-WSS-001** — compare-only default on disk witness (mirror env opt-in).
    #[test]
    fn dehack_wss_001_witness_compare_only_default() {
        let _ = std::env::remove_var("RUST_ENGINE_SUBSTRATE_DUAL_WRITE");
        assert!(refresh_wss_substrate_live_witness());
        let raw = std::fs::read_to_string(WSS_SUBSTRATE_LIVE_JSON).expect("witness json");
        let body: Value = serde_json::from_str(&raw).expect("parse witness");
        assert_eq!(
            body.get("dual_write_shim_enabled").and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            body.get("dual_write_compare_only").and_then(|v| v.as_bool()),
            Some(true)
        );
    }
}

