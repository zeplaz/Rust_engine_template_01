//! **EFFECTS-SYSTEM-ES-6-001** — clipmap L0 authority + DEBT-006/010/010b/010c/**011**/**012** witness.
//!
//! Cleanup: ES-6-3b deleted `bridge_legacy` after packet
//! d6e130cf → re-eval 0778f3f4 (A_obsolete · all unblock_when PASS).
//! DEBT-010/010b/010c: L0 owns all atmosphere channels; GPU field projection-owned.
//! DEBT-011: unify ChunkSmoke + Field mirror + GPU smoke extract — **AtmosphereField deleted**.
//! DEBT-012: optional AtmosphereCell overlay retire — **kept** (class C dormant; overlays ¬L0-direct).

use bevy::input::InputPlugin;
use bevy::prelude::*;

use crate::gui::InputBindings;
use crate::substrate::atmosphere::{
    advect_l0_with_wind, clipmap_l0_smoke_max, max_blend_l0_ash_at, max_blend_l0_ember_at,
    max_blend_l0_fog_at, max_blend_l0_heat_at, max_blend_l0_smoke_at, max_blend_l0_toxicity_at,
    sample_tactical_ash_from_l0, sample_tactical_ember_from_l0, sample_tactical_fog_from_l0,
    sample_tactical_heat_from_l0, sample_tactical_smoke_from_l0, sample_tactical_toxicity_from_l0,
    AtmosphereClipmapStack, CLIPMAP_L0_AUTHORITATIVE, DEBT_010B_TOXICITY_FOG_L0,
    DEBT_010C_ASH_EMBER_HEAT_L0, DEBT_010_WRITERS_L0, DEBT_011_FIELD_DELETED,
    GPU_FIELD_CONSUMERS_ON_L0,
};
use crate::substrate::ecs_retire::smoke_bridge_from_clipmap;
use crate::systems::atmosphere::{
    atmos_chunk_to_tile, sample_tactical_visibility_from_l0, AtmospherePlugin,
    DEBT_010_WRITERS_ON_L0, DEBT_011_SMOKE_UNIFIED, FIELD_L0_SHIM_RETIRED, FIELD_L0_SHIM_WIRED,
    SMOKE_EXTRACT_BRIDGE,
};
use crate::systems::fire::CHUNK_SMOKE_L0_PULL;
use crate::systems::sim_control::SimControlPlugin;

pub const EFFECTS_SYSTEM_ES6_LIVE_JSON: &str = "debug_runs/effects_system_es6_live.json";

#[must_use]
pub fn effects_system_es6_witness_body() -> serde_json::Value {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(InputPlugin);
    app.init_resource::<InputBindings>();
    app.add_plugins(SimControlPlugin);
    app.add_plugins(AtmospherePlugin);

    // Seed L0 so smoke_bridge_from_clipmap proves clipmap consumers stay wired.
    {
        let mut stack = app.world_mut().resource_mut::<AtmosphereClipmapStack>();
        if let Some(cell) = stack
            .levels
            .first_mut()
            .and_then(|l| l.smoke_density.first_mut())
        {
            *cell = 0.55;
        }
    }

    for _ in 0..2 {
        app.update();
    }

    let stack = app.world().resource::<AtmosphereClipmapStack>();
    let levels = stack.levels.len();
    let clipmap_levels_present = levels == 4;
    let field_resource_present = false; // AtmosphereField resource deleted (DEBT-011)
    let l0_smoke_max = clipmap_l0_smoke_max(stack);
    let bridge_deleted = true;
    let smoke_from_clipmap = smoke_bridge_from_clipmap(stack).is_some();
    let authoritative = CLIPMAP_L0_AUTHORITATIVE
        && clipmap_levels_present
        && bridge_deleted
        && smoke_from_clipmap
        && l0_smoke_max > 0.0;
    let green = authoritative && DEBT_011_FIELD_DELETED && !field_resource_present;

    let es_6_4 = crate::io::save::atmosphere_clipmap_save_roundtrip_green()
        && crate::io::save::atmosphere_clipmap_manifest_roundtrip_green();

    // ES-6-2 — L0 tactical sample path (shim retired; Field deleted).
    let mut shim_stack = AtmosphereClipmapStack::default();
    max_blend_l0_smoke_at(&mut shim_stack, 0, 0, 0.42);
    let es_6_2 = FIELD_L0_SHIM_RETIRED
        && !FIELD_L0_SHIM_WIRED
        && clipmap_l0_smoke_max(&shim_stack) >= 0.42
        && sample_tactical_smoke_from_l0(&shim_stack, 0, 0) == Some(0.42);

    // ES-6-3a — readers prefer L0 (visibility free fn).
    let fog_vis = (1.0 - 0.42_f32 * 0.7 - 0.0_f32 * 0.45).clamp(0.05, 1.0);
    let es_6_3a =
        (sample_tactical_visibility_from_l0(&shim_stack, 0, 0) - fog_vis).abs() < 1e-5;

    let debt_006_delete_safe = CLIPMAP_L0_AUTHORITATIVE
        && authoritative
        && es_6_2
        && es_6_3a
        && bridge_deleted;

    // DEBT-010 — writers write L0.
    let mut debt_stack = AtmosphereClipmapStack::default();
    max_blend_l0_smoke_at(&mut debt_stack, 0, 0, 0.61);
    if let Some(level0) = debt_stack.levels.first_mut() {
        advect_l0_with_wind(level0, Vec2::X, 0.0, 0.016);
    }
    let debt_010_writers_l0 = DEBT_010_WRITERS_L0
        && DEBT_010_WRITERS_ON_L0
        && sample_tactical_smoke_from_l0(&debt_stack, 0, 0) == Some(0.61);

    // DEBT-010b — L0 fog/toxicity lanes + chunk_smoke L0 pull.
    let mut b_stack = AtmosphereClipmapStack::default();
    max_blend_l0_smoke_at(&mut b_stack, 0, 0, 0.5);
    max_blend_l0_fog_at(&mut b_stack, 0, 0, 0.4);
    max_blend_l0_toxicity_at(&mut b_stack, 0, 0, 0.35);
    let fog_vis_b = (1.0 - 0.5_f32 * 0.7 - 0.4_f32 * 0.45).clamp(0.05, 1.0);
    let debt_010b_toxicity_fog_l0 = DEBT_010B_TOXICITY_FOG_L0
        && CHUNK_SMOKE_L0_PULL
        && sample_tactical_fog_from_l0(&b_stack, 0, 0) == Some(0.4)
        && sample_tactical_toxicity_from_l0(&b_stack, 0, 0) == Some(0.35)
        && (sample_tactical_visibility_from_l0(&b_stack, 0, 0) - fog_vis_b).abs() < 1e-5
        && DEBT_011_FIELD_DELETED;

    // DEBT-010c — L0 ash/ember/heat + GPU field consumers projection-owned.
    let mut c_stack = AtmosphereClipmapStack::default();
    max_blend_l0_smoke_at(&mut c_stack, 0, 0, 0.5);
    max_blend_l0_fog_at(&mut c_stack, 0, 0, 0.2);
    max_blend_l0_toxicity_at(&mut c_stack, 0, 0, 0.1);
    max_blend_l0_ash_at(&mut c_stack, 0, 0, 0.28);
    max_blend_l0_ember_at(&mut c_stack, 0, 0, 0.19);
    max_blend_l0_heat_at(&mut c_stack, 0, 0, 0.23);
    if let Some(level0) = c_stack.levels.first_mut() {
        advect_l0_with_wind(level0, Vec2::X, 0.0, 0.016);
    }
    let debt_010c_ash_ember_heat_l0 = DEBT_010C_ASH_EMBER_HEAT_L0
        && GPU_FIELD_CONSUMERS_ON_L0
        && sample_tactical_ash_from_l0(&c_stack, 0, 0) == Some(0.28)
        && sample_tactical_ember_from_l0(&c_stack, 0, 0) == Some(0.19)
        && sample_tactical_heat_from_l0(&c_stack, 0, 0) == Some(0.23)
        && DEBT_011_FIELD_DELETED;

    // DEBT-011 — smoke unify + Field delete.
    let debt_011_smoke_unified = DEBT_011_SMOKE_UNIFIED
        && DEBT_011_FIELD_DELETED
        && CHUNK_SMOKE_L0_PULL
        && SMOKE_EXTRACT_BRIDGE == "L0→ChunkSmoke→SimChunkSmokeVisualExtract"
        && atmos_chunk_to_tile(IVec2::new(0, 0)) == Some((0, 0))
        && !field_resource_present;

    let field_deleted = debt_011_smoke_unified;
    let field_delete_blocked = !field_deleted;

    // DEBT-012 — AtmosphereCell overlay sample: keep (class C). Overlays are not L0-direct;
    // logistics merge still takes the cell DTO. No open residual while kept.
    const OVERLAYS_L0_DIRECT: bool = false;
    let debt_012_cell_kept = debt_011_smoke_unified && !OVERLAYS_L0_DIRECT;
    let debt_012_cell_deleted = false;

    let mut residual = Vec::new();
    if !es_6_2 {
        residual.push(
            "ES-6-2: L0 tactical sample / Field→L0 shim retirement incomplete".to_string(),
        );
    }
    if !es_6_3a {
        residual.push(
            "ES-6-3a: migrate visibility/coupling/diagnostics onto L0 sample_tactical_*".to_string(),
        );
    }
    if !es_6_4 {
        residual.push(
            "ES-6-4: clipmap save/load snapshot — debug_runs/atmosphere_clipmap_save_roundtrip_live.json"
                .to_string(),
        );
    }
    if !debt_006_delete_safe && es_6_2 && es_6_3a {
        residual.push(
            "ES-6-3b: DELETE DEBT-006 bridge_legacy when debt_006_delete_safe".to_string(),
        );
    }
    if debt_006_delete_safe && !debt_010_writers_l0 {
        residual.push(
            "DEBT-010: retire AtmosphereField writers (update/advect) onto clipmap — separate from DEBT-006"
                .to_string(),
        );
    }
    if debt_010_writers_l0 && !debt_010b_toxicity_fog_l0 {
        residual.push(
            "DEBT-010b: migrate toxicity/fog clipmap lanes + chunk_smoke L0 pull (class B transitional)"
                .to_string(),
        );
    }
    if debt_010b_toxicity_fog_l0 && !debt_010c_ash_ember_heat_l0 {
        residual.push(
            "DEBT-010c: migrate ash/ember/heat clipmap lanes + GPU field consumers on L0"
                .to_string(),
        );
    }
    if debt_010c_ash_ember_heat_l0 && !debt_011_smoke_unified {
        residual.push(
            "DEBT-011: unify ChunkSmoke + Field mirror + GPU field smoke extract bridge — then delete AtmosphereField"
                .to_string(),
        );
    }
    // DEBT-012 open residual only if overlays go L0-direct but cell still present (delete path).
    if debt_011_smoke_unified && OVERLAYS_L0_DIRECT && !debt_012_cell_deleted {
        residual.push(
            "DEBT-012: overlays L0-direct — delete AtmosphereCell sample DTO + overlay/logistics callers"
                .to_string(),
        );
    }

    let exit_met = green
        && debt_006_delete_safe
        && debt_010_writers_l0
        && debt_010b_toxicity_fog_l0
        && debt_010c_ash_ember_heat_l0
        && debt_011_smoke_unified
        && debt_012_cell_kept;

    serde_json::json!({
        "program_id": "EFFECTS-SYSTEM-UNIFY-001",
        "phase": "ES-6",
        "slice": "DEBT-012",
        "status": if exit_met { "done" } else { "ready" },
        "clipmap_l0_authoritative": CLIPMAP_L0_AUTHORITATIVE && authoritative,
        "legacy_bridge_deleted_or_gated": bridge_deleted,
        "legacy_bridge_env_rollback": "deleted — RUST_ENGINE_ATMOS_LEGACY_BRIDGE retired with DEBT-006",
        "clipmap_levels_present": clipmap_levels_present,
        "clipmap_level_count": levels,
        "clipmap_l0_smoke_max": l0_smoke_max,
        "smoke_bridge_from_clipmap_ok": smoke_from_clipmap,
        "legacy_atmosphere_field_present": field_resource_present,
        "legacy_field_role": "deleted_debt_011",
        "field_cleanup_class": "A_obsolete_deleted",
        "field_delete_blocked": field_delete_blocked,
        "field_deleted": field_deleted,
        "es_6_2_field_l0_shim": es_6_2,
        "es_6_3a_field_readers_l0": es_6_3a,
        "field_l0_shim_wired": FIELD_L0_SHIM_WIRED,
        "field_l0_shim_retired": FIELD_L0_SHIM_RETIRED,
        "debt_006_delete_safe": debt_006_delete_safe,
        "debt_006_gated_off_default": bridge_deleted,
        "debt_010_writers_l0": debt_010_writers_l0,
        "debt_010b_toxicity_fog_l0": debt_010b_toxicity_fog_l0,
        "debt_010c_ash_ember_heat_l0": debt_010c_ash_ember_heat_l0,
        "gpu_field_consumers_on_l0": GPU_FIELD_CONSUMERS_ON_L0,
        "chunk_smoke_l0_pull": CHUNK_SMOKE_L0_PULL,
        "debt_011_smoke_unified": debt_011_smoke_unified,
        "debt_011_field_deleted": DEBT_011_FIELD_DELETED,
        "smoke_extract_bridge": SMOKE_EXTRACT_BRIDGE,
        "debt_012_cell_action": if debt_012_cell_kept { "kept" } else if debt_012_cell_deleted { "deleted" } else { "open" },
        "debt_012_cleanup_class": "C_dormant",
        "overlays_l0_direct": OVERLAYS_L0_DIRECT,
        "debt_012_unblock_when": "overlays + logistics sample L0 channels without AtmosphereCell DTO",
        "cleanup_class": "C_dormant_kept",
        "cleanup_packet": "DEBT-012 AtmosphereCell overlay sample — keep until overlays_l0_direct",
        "residual": residual,
        "es_6_4_clipmap_snapshot": es_6_4,
        "exit_met": exit_met,
        "green": green,
        "plan_note": "DEBT-012 kept: AtmosphereCell is class-C dormant sample DTO for overlays/logistics; overlays not L0-direct. AtmosphereField already deleted (DEBT-011).",
        "atmos_lane_drain": "idle",
    })
}

#[must_use]
pub fn refresh_effects_system_es6_live_witness() -> bool {
    let body = effects_system_es6_witness_body();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "EFFECTS-SYSTEM-ES-6-001",
        "refresh_effects_system_es6_live_witness",
        EFFECTS_SYSTEM_ES6_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(EFFECTS_SYSTEM_ES6_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_system_es6_live_witness_refresh_green() {
        assert!(CLIPMAP_L0_AUTHORITATIVE);
        assert!(FIELD_L0_SHIM_RETIRED);
        assert!(!FIELD_L0_SHIM_WIRED);
        assert!(DEBT_010_WRITERS_L0);
        assert!(DEBT_010B_TOXICITY_FOG_L0);
        assert!(DEBT_010C_ASH_EMBER_HEAT_L0);
        assert!(GPU_FIELD_CONSUMERS_ON_L0);
        assert!(CHUNK_SMOKE_L0_PULL);
        assert!(DEBT_011_SMOKE_UNIFIED);
        assert!(DEBT_011_FIELD_DELETED);
        assert!(refresh_effects_system_es6_live_witness());
        let body = effects_system_es6_witness_body();
        assert_eq!(
            body.get("clipmap_l0_authoritative")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("legacy_bridge_deleted_or_gated")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("es_6_2_field_l0_shim").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("es_6_3a_field_readers_l0")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("debt_006_delete_safe").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("debt_010_writers_l0").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("debt_010b_toxicity_fog_l0")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("debt_010c_ash_ember_heat_l0")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("gpu_field_consumers_on_l0")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("debt_011_smoke_unified")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("field_deleted").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            body.get("field_delete_blocked").and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(body.get("green").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(body.get("slice").and_then(|v| v.as_str()), Some("DEBT-012"));
        assert_eq!(
            body.get("debt_012_cell_action").and_then(|v| v.as_str()),
            Some("kept")
        );
        assert_eq!(
            body.get("overlays_l0_direct").and_then(|v| v.as_bool()),
            Some(false)
        );
        assert_eq!(
            body.get("cleanup_class").and_then(|v| v.as_str()),
            Some("C_dormant_kept")
        );
        assert_eq!(
            body.get("atmos_lane_drain").and_then(|v| v.as_str()),
            Some("idle")
        );
        let residual = body
            .get("residual")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(
            residual
                .iter()
                .all(|r| !r.as_str().unwrap_or("").starts_with("DEBT-011")),
            "DEBT-011 residual must drop when debt_011_smoke_unified: {residual:?}"
        );
        assert!(
            residual
                .iter()
                .all(|r| !r.as_str().unwrap_or("").starts_with("DEBT-012")),
            "DEBT-012 open residual must drop when cell kept (overlays ¬L0-direct): {residual:?}"
        );
    }
}
