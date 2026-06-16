use bevy::input::InputPlugin;
use bevy::prelude::*;

use crate::gui::InputBindings;
use crate::scenario::scenario_plugin::ScenarioScriptingPlugin;
use crate::scenario::scenario_steps::ScenarioStep;
use crate::scenario::scenario_types::ScenarioFileV1;
use crate::scenario::script_host::EngineScriptHost;
use crate::systems::sim_control::{SimControlPlugin, SimControlState, SimTick};

#[test]
fn scenario_wave1_load_and_execute_minimal_script() {
    let mut app = App::new();

    app.add_plugins((MinimalPlugins, InputPlugin));
    app.init_resource::<InputBindings>();
    app.add_plugins((SimControlPlugin, ScenarioScriptingPlugin));

    let ron = r#"(
        schema_version: 2,
        metadata: (id: "test", display_name: "Test Scenario", author: None, description: None),
        steps: [NoOp, SimAdvance(ticks: 0)],
    )"#;

    let parsed: ScenarioFileV1 = ron::from_str(ron).expect("Scenario RON should deserialize");

    {
        let mut host = app.world_mut().resource_mut::<EngineScriptHost>();
        host.load_script(parsed);
    }

    app.update();
    app.update();
    app.update();

    let host = app.world().resource::<EngineScriptHost>();

    assert!(host.execution_log.iter().any(|x| x.contains("NoOp")));
    assert!(host.execution_log.iter().any(|x| x.contains("SimAdvance")));

    let sim_tick = app.world().resource::<SimTick>();
    assert_eq!(sim_tick.0, 3);

    let sim_control = app.world().resource::<SimControlState>();
    assert_eq!(sim_control.steps_remaining, 0);
}

#[test]
fn scenario_wave1_fixture_minimal_from_assets_dir() {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("assets/scenarios/tests/minimal_wave1.scenario.ron");
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!("read {}: {e}", path.display());
    });
    let file: ScenarioFileV1 = ron::from_str(&text).expect("fixture RON");
    assert_eq!(file.schema_version, 2);
    assert_eq!(file.metadata.id, "wave1_minimal");
    assert!(
        file.steps.len() >= 3,
        "fixture should list at least 3 steps"
    );
}

#[test]
fn scenario_wave2_scenario_file_ron_roundtrip() {
    let file = ScenarioFileV1 {
        schema_version: 2,
        metadata: crate::scenario::scenario_types::ScenarioMetadata {
            id: "rt".into(),
            display_name: "Roundtrip".into(),
            author: None,
            description: None,
        },
        steps: vec![
            ScenarioStep::NoOp,
            ScenarioStep::Log {
                message: "hi".into(),
            },
            ScenarioStep::SimAdvance { ticks: 2 },
        ],
    };
    let s = file.to_ron_string_pretty().expect("serialize");
    let back: ScenarioFileV1 = ron::from_str(&s).expect("deserialize");
    assert_eq!(back.metadata.id, "rt");
    assert_eq!(back.steps.len(), 3);
}

#[test]
fn scenario_script_host_resume_after_stop() {
    let mut host = EngineScriptHost::default();
    let file = ScenarioFileV1 {
        schema_version: 2,
        metadata: crate::scenario::scenario_types::ScenarioMetadata {
            id: "x".into(),
            display_name: "X".into(),
            author: None,
            description: None,
        },
        steps: vec![ScenarioStep::NoOp, ScenarioStep::NoOp],
    };
    host.load_script(file);
    assert_eq!(host.pending_steps.len(), 2);
    host.stop();
    assert!(!host.running);
    host.resume();
    assert!(host.running);
    assert_eq!(host.pending_steps.len(), 2);
}

#[test]
fn g_play_demo_fire_scenario_deserializes_emit_sim_effect() {
    use crate::engine::play_scenario::DEFAULT_INDUSTRIAL_DEMO_FIRE_SCENARIO;
    use crate::scenario::scenario_steps::ScenarioStep;

    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_INDUSTRIAL_DEMO_FIRE_SCENARIO);
    let text = std::fs::read_to_string(&path).expect("demo fire scenario");
    let file: ScenarioFileV1 = ron::from_str(&text).expect("parse demo fire");
    assert!(
        file.steps
            .iter()
            .any(|s| matches!(s, ScenarioStep::EmitSimEffect { .. }))
    );
}
