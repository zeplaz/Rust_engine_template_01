use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::objectives::ScenarioObjectiveKindV1;
use super::scenario_steps::ScenarioStep;

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub struct ScenarioFileV1 {
    pub schema_version: u32,
    pub metadata: ScenarioMetadata,
    pub steps: Vec<ScenarioStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub struct ScenarioMetadata {
    pub id: String,
    pub display_name: String,
    pub author: Option<String>,
    pub description: Option<String>,
}

impl ScenarioFileV1 {
    pub fn to_ron_string_pretty(&self) -> Result<String, ron::Error> {
        ron::ser::to_string_pretty(
            self,
            ron::ser::PrettyConfig::new().depth_limit(64),
        )
    }

    /// Runtime / interchange subset — **not** an authoring save. Full lossless state stays in RON (`to_ron_string_pretty`).
    pub fn export_runtime_json_subset(&self) -> Result<String, serde_json::Error> {
        #[derive(serde::Serialize)]
        struct ObjectiveOut<'a> {
            objective_id: &'a str,
            kind: ScenarioObjectiveKindV1,
            label: &'a str,
        }

        #[derive(serde::Serialize)]
        struct Out<'a> {
            schema_version: u32,
            id: &'a str,
            display_name: &'a str,
            objectives: Vec<ObjectiveOut<'a>>,
        }

        let mut objectives = Vec::new();
        for step in &self.steps {
            if let ScenarioStep::RegisterObjectives { objectives: obs, .. } = step {
                for o in obs {
                    objectives.push(ObjectiveOut {
                        objective_id: o.objective_id.as_str(),
                        kind: o.kind,
                        label: o.label.as_str(),
                    });
                }
            }
        }

        let body = Out {
            schema_version: self.schema_version,
            id: self.metadata.id.as_str(),
            display_name: self.metadata.display_name.as_str(),
            objectives,
        };

        serde_json::to_string_pretty(&body)
    }
}

impl Default for ScenarioFileV1 {
    fn default() -> Self {
        Self {
            schema_version: 2,
            metadata: ScenarioMetadata {
                id: "unnamed".into(),
                display_name: "Unnamed Scenario".into(),
                author: None,
                description: None,
            },
            steps: vec![],
        }
    }
}
