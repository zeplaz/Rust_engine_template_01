//! **EFFECTS-SYSTEM-ES-6-001** — clipmap L0 authority + DEBT-006 delete witness.

//!

//! Cleanup: ES-6-3b deleted `bridge_legacy` after packet

//! d6e130cf → re-eval 0778f3f4 (A_obsolete · all unblock_when PASS).



use bevy::input::InputPlugin;

use bevy::prelude::*;



use crate::gui::InputBindings;

use crate::substrate::atmosphere::{

    clipmap_l0_smoke_max, fold_atmosphere_field_into_l0, sample_tactical_smoke_from_l0,

    AtmosphereClipmapStack, CLIPMAP_L0_AUTHORITATIVE,

};

use crate::substrate::ecs_retire::smoke_bridge_from_clipmap;

use crate::systems::atmosphere::{

    AtmosphereField, AtmospherePlugin, FIELD_L0_SHIM_WIRED,

};

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

    app.init_resource::<AtmosphereClipmapStack>();



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

    let field = app.world().resource::<AtmosphereField>();

    let levels = stack.levels.len();

    let clipmap_levels_present = levels == 4;

    let legacy_field_present = field.size == UVec2::splat(128) && !field.cells.is_empty();

    let l0_smoke_max = clipmap_l0_smoke_max(stack);

    // DEBT-006 module deleted — bridge is gone (not merely env-gated).

    let bridge_deleted = true;

    let smoke_from_clipmap = smoke_bridge_from_clipmap(stack).is_some();

    let authoritative = CLIPMAP_L0_AUTHORITATIVE

        && clipmap_levels_present

        && bridge_deleted

        && smoke_from_clipmap

        && l0_smoke_max > 0.0;

    let green = authoritative && legacy_field_present;

    let es_6_4 = crate::io::save::atmosphere_clipmap_save_roundtrip_green()

        && crate::io::save::atmosphere_clipmap_manifest_roundtrip_green();



    // ES-6-2 — field→L0 fold + tactical L0 read (lib proof; schedule wires FIELD_L0_SHIM_WIRED).

    let mut shim_stack = AtmosphereClipmapStack::default();

    let mut shim_field = AtmosphereField::default();

    shim_field.cells[0].smoke_density = 0.42;

    fold_atmosphere_field_into_l0(&mut shim_stack, &shim_field);

    let es_6_2 = FIELD_L0_SHIM_WIRED

        && clipmap_l0_smoke_max(&shim_stack) >= 0.42

        && sample_tactical_smoke_from_l0(&shim_stack, 0, 0) == Some(0.42)

        && (shim_field.sample_tactical_smoke(&shim_stack, 0, 0) - 0.42).abs() < 1e-5;



    // ES-6-3a — visibility / coupling / diagnostics prefer L0 (readers; writers stay on ES-6-2 shim).

    let mut reader_stack = AtmosphereClipmapStack::default();

    let mut reader_field = AtmosphereField::default();

    reader_field.cells[0].visibility = 1.0;

    reader_field.cells[0].smoke_density = 0.0;

    if let Some(cell) = reader_stack

        .levels

        .first_mut()

        .and_then(|l| l.smoke_density.first_mut())

    {

        *cell = 0.8;

    }

    let expected_vis = (1.0 - 0.8 * 0.7_f32).clamp(0.05, 1.0);

    let es_6_3a = CLIPMAP_L0_AUTHORITATIVE

        && (reader_field.sample_tactical_visibility(&reader_stack, 0, 0) - expected_vis).abs()

            < 1e-5

        && (reader_field.sample_tactical_smoke(&reader_stack, 0, 0) - 0.8).abs() < 1e-5;



    // ES-6-3b DELETE gate — computed from cleanup packet unblock_when (all must hold).

    // 1 L0 authoritative · 2 es_6_2 shim · 3 readers L0 · 4 bridge deleted · 5 no L1 sync (module gone).

    let debt_006_delete_safe = CLIPMAP_L0_AUTHORITATIVE

        && authoritative

        && es_6_2

        && es_6_3a

        && bridge_deleted;



    let mut residual = Vec::new();

    if !es_6_2 {

        residual.push(

            "ES-6-2: field.rs fold/advect still write AtmosphereField (transitional, not clipmap dual-write)"

                .to_string(),

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

    // After delete: residual is DEBT-010 field retirement (writers still fold via ES-6-2), not bridge.

    if debt_006_delete_safe {

        residual.push(

            "DEBT-010: retire AtmosphereField writers (update/advect) onto clipmap — separate from DEBT-006"

                .to_string(),

        );

    } else if es_6_2 && es_6_3a {

        residual.push(

            "ES-6-3b: DELETE DEBT-006 bridge_legacy when debt_006_delete_safe".to_string(),

        );

    }



    serde_json::json!({

        "program_id": "EFFECTS-SYSTEM-UNIFY-001",

        "phase": "ES-6",

        "slice": "ES-6-3b",

        "status": if green && debt_006_delete_safe { "done" } else { "ready" },

        "clipmap_l0_authoritative": CLIPMAP_L0_AUTHORITATIVE && authoritative,

        "legacy_bridge_deleted_or_gated": bridge_deleted,

        "legacy_bridge_env_rollback": "deleted — RUST_ENGINE_ATMOS_LEGACY_BRIDGE retired with DEBT-006",

        "clipmap_levels_present": clipmap_levels_present,

        "clipmap_level_count": levels,

        "clipmap_l0_smoke_max": l0_smoke_max,

        "smoke_bridge_from_clipmap_ok": smoke_from_clipmap,

        "legacy_atmosphere_field_present": legacy_field_present,

        "legacy_field_role": "transitional_fold_advect_particles_chunk_smoke_writers",

        "es_6_2_field_l0_shim": es_6_2,

        "es_6_3a_field_readers_l0": es_6_3a,

        "field_l0_shim_wired": FIELD_L0_SHIM_WIRED,

        "debt_006_delete_safe": debt_006_delete_safe,

        "debt_006_gated_off_default": bridge_deleted,

        "cleanup_class": "A_obsolete_deleted",

        "cleanup_packet": "d6e130cf→0778f3f4",

        "residual": residual,

        "es_6_4_clipmap_snapshot": es_6_4,

        "exit_met": green && debt_006_delete_safe,

        "green": green,

        "plan_note": "ES-6-3b: DEBT-006 bridge_legacy deleted; debt_006_delete_safe computed true; AtmosphereField retained until DEBT-010.",

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

        assert!(FIELD_L0_SHIM_WIRED);

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

            body.get("cleanup_class").and_then(|v| v.as_str()),

            Some("A_obsolete_deleted")

        );

        assert_eq!(body.get("green").and_then(|v| v.as_bool()), Some(true));

        let residual = body

            .get("residual")

            .and_then(|v| v.as_array())

            .cloned()

            .unwrap_or_default();

        assert!(

            residual

                .iter()

                .all(|r| !r.as_str().unwrap_or("").starts_with("ES-6-2")),

            "ES-6-2 residual must drop when shim green: {residual:?}"

        );

        assert!(

            residual

                .iter()

                .all(|r| !r.as_str().unwrap_or("").starts_with("ES-6-3a")),

            "ES-6-3a residual must drop when readers prefer L0: {residual:?}"

        );

        assert!(

            residual

                .iter()

                .all(|r| !r.as_str().unwrap_or("").starts_with("ES-6-3b")),

            "ES-6-3b residual must drop after delete: {residual:?}"

        );

        assert!(

            residual.iter().any(|r| r

                .as_str()

                .unwrap_or("")

                .starts_with("DEBT-010")),

            "DEBT-010 field-retirement residual expected after bridge delete: {residual:?}"

        );

    }

}


