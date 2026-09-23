//! **EFFECTS-SYSTEM-ES-6-001** — clipmap L0 authority + DEBT-006 gated witness.

use bevy::input::InputPlugin;
use bevy::prelude::*;

use crate::gui::InputBindings;
use crate::substrate::atmosphere::{
    clipmap_l0_smoke_max, legacy_atmosphere_bridge_enabled, AtmosphereClipmapStack,
    CLIPMAP_L0_AUTHORITATIVE,
};
use crate::substrate::ecs_retire::smoke_bridge_from_clipmap;
use crate::systems::atmosphere::{AtmosphereField, AtmospherePlugin};
use crate::systems::sim_control::SimControlPlugin;

pub const EFFECTS_SYSTEM_ES6_LIVE_JSON: &str = "debug_runs/effects_system_es6_live.json";

#[must_use]
pub fn effects_system_es6_witness_body() -> serde_json::Value {
    let _ = std::env::remove_var("RUST_ENGINE_ATMOS_LEGACY_BRIDGE");

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
    let bridge_gated = !legacy_atmosphere_bridge_enabled();
    let smoke_from_clipmap = smoke_bridge_from_clipmap(stack).is_some();
    let authoritative = CLIPMAP_L0_AUTHORITATIVE
        && clipmap_levels_present
        && bridge_gated
        && smoke_from_clipmap
        && l0_smoke_max > 0.0;
    let green = authoritative && legacy_field_present;

    serde_json::json!({
        "program_id": "EFFECTS-SYSTEM-UNIFY-001",
        "phase": "ES-6",
        "slice": "ES-6-001",
        "status": if green { "done" } else { "ready" },
        "clipmap_l0_authoritative": CLIPMAP_L0_AUTHORITATIVE && authoritative,
        "legacy_bridge_deleted_or_gated": bridge_gated,
        "legacy_bridge_env_rollback": "RUST_ENGINE_ATMOS_LEGACY_BRIDGE=1",
        "clipmap_levels_present": clipmap_levels_present,
        "clipmap_level_count": levels,
        "clipmap_l0_smoke_max": l0_smoke_max,
        "smoke_bridge_from_clipmap_ok": smoke_from_clipmap,
        "legacy_atmosphere_field_present": legacy_field_present,
        "legacy_field_role": "transitional_fold_advect_particles_visibility_chunk_smoke",
        "debt_006_delete_safe": false,
        "debt_006_gated_off_default": bridge_gated,
        "cleanup_class": "B_transitional_bridge_gated",
        "residual": [
            "ES-6-2: field.rs fold/advect still write AtmosphereField (transitional, not clipmap dual-write)",
            "ES-6-4: clipmap save/load snapshot contract absent",
        ],
        "exit_met": green,
        "green": green,
        "plan_note": "ES-6: L0 authoritative for clipmap smoke consumers; DEBT-006 hard-gated OFF (env rollback).",
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
        assert!(!legacy_atmosphere_bridge_enabled());
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
        assert_eq!(body.get("green").and_then(|v| v.as_bool()), Some(true));
    }
}
