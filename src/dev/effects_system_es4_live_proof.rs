//! **EFFECTS-SYSTEM-ES-4-002** — quarantine witness `debug_runs/effects_system_es4_live.json`.

use bevy::input::InputPlugin;
use bevy::prelude::*;

use crate::gui::InputBindings;
use crate::systems::atmosphere::{
    AtmospherePlugin, AtmosphereRenderLayers, ATMOSPHERE_COMPOSITE_WIRED,
    ATMOSPHERE_WGSL_QUARANTINED,
};
use crate::systems::sim_control::SimControlPlugin;

pub const EFFECTS_SYSTEM_ES4_LIVE_JSON: &str = "debug_runs/effects_system_es4_live.json";

#[must_use]
pub fn effects_system_es4_witness_body() -> serde_json::Value {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(InputPlugin);
    app.init_resource::<InputBindings>();
    app.add_plugins(SimControlPlugin);
    app.add_plugins(AtmospherePlugin);

    app.world_mut()
        .resource_mut::<AtmosphereRenderLayers>()
        .ground_haze = true;
    for _ in 0..2 {
        app.update();
    }

    let layers = app.world().resource::<AtmosphereRenderLayers>();
    let layers_off = !layers.any_composite_enabled();
    let green = !ATMOSPHERE_COMPOSITE_WIRED
        && ATMOSPHERE_WGSL_QUARANTINED
        && layers_off;

    serde_json::json!({
        "program_id": "EFFECTS-SYSTEM-UNIFY-001",
        "phase": "ES-4",
        "slice": "ES-4-002",
        "path": "ALTERNATIVE_quarantine",
        "atmosphere_composite_wired": ATMOSPHERE_COMPOSITE_WIRED,
        "atmosphere_wgsl_quarantined": ATMOSPHERE_WGSL_QUARANTINED,
        "atmosphere_render_layers_all_off": layers_off,
        "wgsl_experiment_root": "assets/shaders/experiments/atmosphere/",
        "green": green,
    })
}

#[must_use]
pub fn refresh_effects_system_es4_live_witness() -> bool {
    let body = effects_system_es4_witness_body();
    if !body
        .get("green")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return false;
    }
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "EFFECTS-SYSTEM-ES-4-002",
        "refresh_effects_system_es4_live_witness",
        EFFECTS_SYSTEM_ES4_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(EFFECTS_SYSTEM_ES4_LIVE_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_system_es4_live_witness_refresh_green() {
        assert!(refresh_effects_system_es4_live_witness());
    }
}
