//! **VSS-T2-002 / T2-003** — spectator entry-path parity witness (`debug_runs/spectator_parity_live.json`).
//!
//! Harness fire seeds are `debug_fallback` — product proof requires scenario / save / interactive paths.

use crate::engine::launch_args::SpectatorEntryPath;
use crate::engine::play_scenario::g_play_fire_001_lib_witness_green;
use crate::scenario::scenario_types::ScenarioFileV1;
use crate::scenario::trigger_registry::scenario_trigger_registry_witness_green;
use crate::sim::effects::sim_effect_spine_lib_witness_green;

pub const SPECTATOR_PARITY_LIVE_JSON: &str = "debug_runs/spectator_parity_live.json";

/// Product scenario ignite path green (SimEffect waist, not harness seed).
#[must_use]
pub fn scenario_entry_path_parity_green() -> bool {
    g_play_fire_001_lib_witness_green()
        && scenario_trigger_registry_witness_green()
        && scenario_ignite_enqueue_lib_green()
}

#[must_use]
pub fn scenario_ignite_enqueue_lib_green() -> bool {
    scenario_ignite_enqueue_self_check().is_ok()
}

fn scenario_ignite_enqueue_self_check() -> Result<(), &'static str> {
    use bevy::input::InputPlugin;
    use bevy::prelude::*;
    use crate::gui::InputBindings;
    use crate::scenario::scenario_plugin::ScenarioScriptingPlugin;
    use crate::scenario::scenario_steps::ScenarioStep;
    use crate::scenario::script_host::EngineScriptHost;
    use crate::sim::effects::SimEffectQueue;
    use crate::systems::sim_control::SimControlPlugin;

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets/scenarios/tests/vss_t2_002_ignite_at.scenario.ron");
    let text = std::fs::read_to_string(&path).map_err(|_| "fixture_missing")?;
    let file: ScenarioFileV1 = ron::from_str(&text).map_err(|_| "fixture_parse")?;
    if !file.steps.iter().any(|s| matches!(s, ScenarioStep::IgniteAt { .. })) {
        return Err("missing_ignite_at");
    }
    if !file
        .steps
        .iter()
        .any(|s| matches!(s, ScenarioStep::TriggerEffect { .. }))
    {
        return Err("missing_trigger_effect");
    }

    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin));
    app.init_resource::<InputBindings>();
    app.init_resource::<SimEffectQueue>();
    app.add_plugins((SimControlPlugin, ScenarioScriptingPlugin));

    {
        let mut host = app.world_mut().resource_mut::<EngineScriptHost>();
        host.load_script(file);
    }

    for _ in 0..4 {
        app.update();
    }

    let host = app.world().resource::<EngineScriptHost>();
    if !host.execution_log.iter().any(|l| l.contains("IgniteAt:")) {
        return Err("ignite_at_not_executed");
    }
    if !host
        .execution_log
        .iter()
        .any(|l| l.contains("TriggerEffect:"))
    {
        return Err("trigger_effect_not_executed");
    }

    let queue = app.world().resource::<SimEffectQueue>();
    if queue.pushed_total < 2 {
        return Err("sim_effect_enqueue");
    }
    Ok(())
}

/// Interactive session: direct `SimEffectQueue` enqueue → drain (no harness / scenario script).
#[must_use]
pub fn interactive_entry_path_parity_green() -> bool {
    sim_effect_spine_lib_witness_green()
}

/// Save hydrate + fire-field dirty → chunk save queue spine (Wave S manifest round-trip + persist hook).
#[must_use]
pub fn save_entry_path_parity_green() -> bool {
    save_fire_dirty_queue_lib_green()
        && crate::io::save::settlement_books_manifest_roundtrip_witness_green()
}

#[must_use]
pub fn save_fire_dirty_queue_lib_green() -> bool {
    save_fire_dirty_queue_self_check().is_ok()
}

fn save_fire_dirty_queue_self_check() -> Result<(), &'static str> {
    use bevy::prelude::*;
    use crate::io::save::enqueue_dirty_chunks_from_environment_hooks;
    use crate::io::save::DirtyChunkSaveQueue;
    use crate::systems::chunk_environment_persist::{
        ChunkEnvironmentDirty, ChunkEnvironmentPersistHooks,
    };
    use crate::terrain::generation::Chunk;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.init_resource::<ChunkEnvironmentPersistHooks>();
    app.init_resource::<DirtyChunkSaveQueue>();

    let coord = IVec2::new(3, 4);
    app.world_mut().spawn((Chunk { coord }, ChunkEnvironmentDirty {
        fire_field: true,
    }));
    app.world_mut()
        .resource_mut::<ChunkEnvironmentPersistHooks>()
        .fire_field_dirty_events = 1;

    app.add_systems(Update, enqueue_dirty_chunks_from_environment_hooks);
    app.update();

    let queue = app.world().resource::<DirtyChunkSaveQueue>();
    if queue.dirty_chunks.contains(&coord) {
        Ok(())
    } else {
        Err("fire_dirty_not_enqueued")
    }
}

#[must_use]
pub fn all_product_entry_paths_green() -> bool {
    scenario_entry_path_parity_green()
        && save_entry_path_parity_green()
        && interactive_entry_path_parity_green()
}

#[must_use]
pub fn build_spectator_parity_body() -> serde_json::Value {
    let scenario_green = scenario_entry_path_parity_green();
    let save_green = save_entry_path_parity_green();
    let interactive_green = interactive_entry_path_parity_green();
    let all_green = scenario_green && save_green && interactive_green;
    let slice_id = if all_green {
        "VSS-T2-003"
    } else if scenario_green {
        "VSS-T2-002"
    } else {
        "VSS-T2-001"
    };
    let entry_path = if all_green {
        SpectatorEntryPath::Scenario.as_str()
    } else if scenario_green {
        SpectatorEntryPath::Scenario.as_str()
    } else {
        SpectatorEntryPath::Harness.as_str()
    };
    serde_json::json!({
        "schema": "spectator_parity_witness_v1",
        "status": if all_green { "green" } else if scenario_green { "partial" } else { "pending" },
        "program_id": "VSS-001",
        "slice_id": slice_id,
        "green": all_green,
        "harness_fire": {
            "debug_fallback_only": true,
            "forbidden_as_sole_proof": true,
            "source": "apply_test_scene_fire_seeds"
        },
        "entry_path": entry_path,
        "entry_paths": {
            "harness": {
                "role": "debug_fallback",
                "green": false,
                "proof_grade": "debug_only"
            },
            "scenario": {
                "role": "product",
                "green": scenario_green,
                "proof_grade": if scenario_green { "product" } else { "pending" },
                "api": ["IgniteAt", "TriggerEffect", "EmitSimEffect"],
                "fixture": "assets/scenarios/tests/vss_t2_002_ignite_at.scenario.ron",
                "demo_scenario": crate::engine::play_scenario::DEFAULT_INDUSTRIAL_DEMO_FIRE_SCENARIO,
                "ignition_waist": "SimEffectQueue → EmberSpotIgnitionEvent"
            },
            "save": {
                "role": "product",
                "green": save_green,
                "proof_grade": if save_green { "product" } else { "pending" },
                "spine": "Wave S manifest round-trip + fire_field → DirtyChunkSaveQueue",
                "manifest_roundtrip": "settlement_books_manifest_roundtrip_witness_green"
            },
            "interactive": {
                "role": "product",
                "green": interactive_green,
                "proof_grade": if interactive_green { "product" } else { "pending" },
                "ignition_waist": "SimEffectQueue direct enqueue → drain",
                "lib_witness": "sim_effect_spine_lib_witness_green"
            }
        },
        "product_entry_path_required": ["scenario", "save", "interactive"],
        "notes": if all_green {
            "VSS-T2-003 — all product entry paths green; harness remains debug_fallback_only."
        } else {
            "Product paths in progress — scenario/save/interactive parity lib checks."
        }
    })
}

#[must_use]
pub fn refresh_spectator_parity_live_witness() -> bool {
    let body = build_spectator_parity_body();
    let profile = body
        .get("slice_id")
        .and_then(|v| v.as_str())
        .unwrap_or("VSS-T2-003")
        .to_string();
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        &profile,
        "refresh_spectator_parity_live_witness",
        SPECTATOR_PARITY_LIVE_JSON,
        body,
    );
    crate::dev::debug_run_envelope::write_debug_run_json(SPECTATOR_PARITY_LIVE_JSON, wrapped)
}

/// Legacy stub name — delegates to product witness refresh.
#[must_use]
pub fn refresh_spectator_parity_live_witness_stub() -> bool {
    refresh_spectator_parity_live_witness()
}

#[must_use]
pub fn spectator_parity_stub_body() -> serde_json::Value {
    build_spectator_parity_body()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::launch_args::{EngineLaunchArgs, TestScene};
    use crate::engine::test_harness::TestWorldHarness;

    #[test]
    fn harness_fire_is_debug_fallback_not_product_proof() {
        let fire = EngineLaunchArgs::from_cli(Some("fire".into()), false, None);
        assert!(fire.harness_fire_is_debug_fallback());
        assert!(TestScene::Fire.seeds_fire_overlay());
        assert!(!TestScene::Weather.seeds_fire_overlay());
        assert!(TestWorldHarness::default().harness_fire_debug_fallback);
    }

    #[test]
    fn scenario_ignite_enqueue_via_sim_effect_queue() {
        assert!(scenario_ignite_enqueue_lib_green());
    }

    #[test]
    fn scenario_entry_path_parity_green_lib() {
        assert!(scenario_entry_path_parity_green());
    }

    #[test]
    fn interactive_entry_path_parity_green_lib() {
        assert!(interactive_entry_path_parity_green());
    }

    #[test]
    fn save_entry_path_parity_green_lib() {
        assert!(save_entry_path_parity_green());
    }

    #[test]
    fn all_product_entry_paths_green_lib() {
        assert!(all_product_entry_paths_green());
    }

    #[test]
    fn spectator_parity_refresh_writes_json() {
        assert!(refresh_spectator_parity_live_witness());
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SPECTATOR_PARITY_LIVE_JSON);
        assert!(path.is_file(), "witness missing at {}", path.display());
        let raw = std::fs::read_to_string(&path).expect("read witness");
        let v: serde_json::Value = serde_json::from_str(&raw).expect("parse witness");
        assert_eq!(
            v.pointer("/entry_paths/scenario/green")
                .and_then(|x| x.as_bool()),
            Some(true)
        );
        assert_eq!(
            v.pointer("/entry_paths/save/green")
                .and_then(|x| x.as_bool()),
            Some(true)
        );
        assert_eq!(
            v.pointer("/entry_paths/interactive/green")
                .and_then(|x| x.as_bool()),
            Some(true)
        );
        assert_eq!(v.get("green").and_then(|x| x.as_bool()), Some(true));
        assert_eq!(
            v.pointer("/entry_paths/scenario/proof_grade")
                .and_then(|x| x.as_str()),
            Some("product")
        );
        assert_eq!(
            v.pointer("/harness_fire/debug_fallback_only")
                .and_then(|x| x.as_bool()),
            Some(true)
        );
        assert_eq!(v.get("slice_id").and_then(|x| x.as_str()), Some("VSS-T2-003"));
    }
}
