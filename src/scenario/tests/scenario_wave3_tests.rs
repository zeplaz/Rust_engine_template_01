use bevy::input::InputPlugin;
use bevy::prelude::*;

use crate::gui::InputBindings;
use crate::scenario::objectives::ScenarioObjectiveMarker;
use crate::scenario::scenario_plugin::ScenarioScriptingPlugin;
use crate::scenario::scenario_types::ScenarioFileV1;
use crate::scenario::script_host::EngineScriptHost;
use crate::systems::sim_control::SimControlPlugin;

#[test]
fn wave3_register_objectives_spawns_markers() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin));
    app.init_resource::<InputBindings>();
    app.add_plugins((SimControlPlugin, ScenarioScriptingPlugin));

    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("assets/scenarios/tests/wave3_objectives.scenario.ron");
    let text = std::fs::read_to_string(&path).expect("fixture");
    let file: ScenarioFileV1 = ron::from_str(&text).expect("parse");

    {
        let mut host = app.world_mut().resource_mut::<EngineScriptHost>();
        host.load_script(file);
    }

    // Log, RegisterObjectives, NoOp — run 2 updates to finish RegisterObjectives step
    app.update();
    app.update();

    let mut q = app.world_mut().query::<&ScenarioObjectiveMarker>();
    let markers: Vec<_> = q.iter(app.world()).collect();
    assert_eq!(markers.len(), 2);
    assert!(markers.iter().any(|m| m.objective_id == "cap_pass"));
    assert!(markers.iter().any(|m| m.objective_id == "keep_line"));
    let cap = markers
        .iter()
        .find(|m| m.objective_id == "cap_pass")
        .expect("cap_pass");
    assert_eq!(
        cap.target,
        Some(crate::scenario::objectives::ObjectiveTargetRef::Region(
            "region:test/north_pass".into()
        ))
    );
}

#[test]
fn wave3_clear_existing_despawns_previous() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, InputPlugin));
    app.init_resource::<InputBindings>();
    app.add_plugins((SimControlPlugin, ScenarioScriptingPlugin));

    let ron1 = r#"(
        schema_version: 2,
        metadata: (id: "a", display_name: "A", author: None, description: None),
        steps: [
            Log(message: "first"),
            RegisterObjectives(clear_existing: false, objectives: [
                (objective_id: "o1", kind: CaptureRegion, label: "L"),
            ]),
        ],
    )"#;
    {
        let mut host = app.world_mut().resource_mut::<EngineScriptHost>();
        host.load_script(ron::from_str(ron1).unwrap());
    }
    app.update();
    app.update();

    assert_eq!(
        app.world_mut()
            .query::<&ScenarioObjectiveMarker>()
            .iter(app.world())
            .count(),
        1
    );

    let ron2 = r#"(
        schema_version: 2,
        metadata: (id: "b", display_name: "B", author: None, description: None),
        steps: [
            RegisterObjectives(clear_existing: true, objectives: [
                (objective_id: "o2", kind: DestroyInfrastructure, label: "X", target: Some(Region("corridor:test"))),
            ]),
        ],
    )"#;
    {
        let mut host = app.world_mut().resource_mut::<EngineScriptHost>();
        host.load_script(ron::from_str(ron2).unwrap());
    }
    app.update();

    let markers: Vec<_> = app
        .world_mut()
        .query::<&ScenarioObjectiveMarker>()
        .iter(app.world())
        .collect();
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].objective_id, "o2");
}
