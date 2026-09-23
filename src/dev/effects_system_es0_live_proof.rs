//! **EFFECTS-SYSTEM-ES-0-001** — refresh `debug_runs/effects_system_es0_live.json`.

use bevy::input::InputPlugin;
use bevy::prelude::*;

use crate::systems::atmosphere::{effects_stub_schedule_clean, AtmosphereDiagnostics, AtmospherePlugin};
use crate::systems::sim_control::SimControlPlugin;
use crate::gui::InputBindings;

pub const EFFECTS_SYSTEM_ES0_LIVE_JSON: &str = "debug_runs/effects_system_es0_live.json";

#[must_use]
pub fn effects_system_es0_witness_body() -> serde_json::Value {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(InputPlugin);
    app.init_resource::<InputBindings>();
    app.add_plugins(SimControlPlugin);
    app.add_plugins(AtmospherePlugin);

    for _ in 0..4 {
        app.update();
    }

    let diag = app.world().resource::<AtmosphereDiagnostics>();
    let clean = diag.particle_controller_runs == 0
        && diag.render_prep_runs == 0
        && effects_stub_schedule_clean(diag.field_fill_runs);

    serde_json::json!({
        "program_id": "EFFECTS-SYSTEM-UNIFY-001",
        "phase": "ES-0",
        "effects_stub_schedule_clean": clean,
        "particle_controller_runs": diag.particle_controller_runs,
        "render_prep_runs": diag.render_prep_runs,
        "field_fill_runs": diag.field_fill_runs,
        "green": clean,
    })
}

#[must_use]
pub fn refresh_effects_system_es0_live_witness() -> bool {
    let body = effects_system_es0_witness_body();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "EFFECTS-SYSTEM-ES-0-001",
        "refresh_effects_system_es0_live_witness",
        EFFECTS_SYSTEM_ES0_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(EFFECTS_SYSTEM_ES0_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_system_es0_live_witness_refresh_green() {
        assert!(refresh_effects_system_es0_live_witness());
    }
}
