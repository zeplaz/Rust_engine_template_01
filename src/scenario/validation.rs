//! Load-time validation for `*.scenario.ron` — hard errors vs warnings (runbook §4 Q5).

use std::collections::HashSet;

use bevy::prelude::Reflect;

use super::scenario_steps::ScenarioStep;
use super::scenario_types::ScenarioFileV1;

/// Severity for individual validation messages (future: structured diagnostics).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioValidationSeverity {
    Warning,
    Error,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Reflect)]
pub struct ScenarioValidationReport {
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

impl ScenarioValidationReport {
    pub fn push(&mut self, severity: ScenarioValidationSeverity, msg: impl Into<String>) {
        let s = msg.into();
        match severity {
            ScenarioValidationSeverity::Warning => self.warnings.push(s),
            ScenarioValidationSeverity::Error => self.errors.push(s),
        }
    }

    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Validate a parsed scenario before [`crate::scenario::script_host::EngineScriptHost::load_script`].
#[must_use]
pub fn validate_scenario(file: &ScenarioFileV1) -> ScenarioValidationReport {
    let mut r = ScenarioValidationReport::default();

    match file.schema_version {
        1 => r.push(
            ScenarioValidationSeverity::Warning,
            "schema_version 1 is deprecated; prefer 2 (stable objective_id, target, factions, tags).",
        
        ),
        2 => {}
        _ => {
            r.push(
                ScenarioValidationSeverity::Error,
                format!(
                    "Unsupported schema_version {} (supported: 1, 2)",
                    file.schema_version
                ),
            );
            return r;
        }
    }

    if file.metadata.id.trim().is_empty() {
        r.push(
            ScenarioValidationSeverity::Error,
            "metadata.id must be non-empty",
        );
    }

    let mut seen_ids: HashSet<&str> = HashSet::new();

    for (si, step) in file.steps.iter().enumerate() {
        if let ScenarioStep::RegisterObjectives { objectives, .. } = step {
            for (oi, obj) in objectives.iter().enumerate() {
                let id = obj.objective_id.as_str();
                if id.trim().is_empty() {
                    r.push(
                        ScenarioValidationSeverity::Error,
                        format!(
                            "RegisterObjectives: empty objective_id at step {si}, objective index {oi}"
                        ),
                    );
                    continue;
                }
                if !seen_ids.insert(id) {
                    r.push(
                        ScenarioValidationSeverity::Error,
                        format!("Duplicate objective_id `{id}` across scenario"),
                    );
                }
                if obj.label.trim().is_empty() {
                    r.push(
                        ScenarioValidationSeverity::Warning,
                        format!("Objective `{id}` has empty label"),
                    );
                }
                if obj.target.is_some() && obj.region_key.is_some() {
                    r.push(
                        ScenarioValidationSeverity::Warning,
                        format!(
                            "Objective `{id}` sets both `target` and legacy `region_key`; prefer `target` only"
                        ),
                    );
                }
            }
        }
        if let ScenarioStep::EmitSimEffect { cells, cause_id, .. } = step {
            if cells.is_empty() {
                r.push(
                    ScenarioValidationSeverity::Error,
                    format!("EmitSimEffect at step {si} has empty cells"),
                );
            }
            if cause_id.trim().is_empty() {
                r.push(
                    ScenarioValidationSeverity::Error,
                    format!("EmitSimEffect at step {si} has empty cause_id"),
                );
            }
        }
    }

    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::objectives::{ScenarioObjectiveKindV1, ScenarioObjectiveV1};
    use crate::scenario::scenario_types::ScenarioMetadata;

    fn file_with_objectives(ids: &[&str]) -> ScenarioFileV1 {
        ScenarioFileV1 {
            schema_version: 2,
            metadata: ScenarioMetadata {
                id: "t".into(),
                display_name: "T".into(),
                author: None,
                description: None,
            },
            steps: vec![ScenarioStep::RegisterObjectives {
                clear_existing: false,
                objectives: ids
                    .iter()
                    .map(|id| ScenarioObjectiveV1 {
                        objective_id: (*id).to_string(),
                        kind: ScenarioObjectiveKindV1::CaptureRegion,
                        label: if id.is_empty() {
                            String::new()
                        } else {
                            (*id).to_string()
                        },
                        target: None,
                        region_key: None,
                        owning_faction: None,
                        opposing_faction: None,
                        tags: vec![],
                    })
                    .collect(),
            }],
        }
    }

    #[test]
    fn duplicate_objective_ids_are_errors() {
        let f = file_with_objectives(&["a", "a"]);
        let r = validate_scenario(&f);
        assert!(!r.is_ok());
        assert!(r.errors.iter().any(|e| e.contains("Duplicate")));
    }

    #[test]
    fn empty_label_is_warning_only() {
        let mut f = file_with_objectives(&["o1"]);
        if let ScenarioStep::RegisterObjectives { ref mut objectives, .. } = f.steps[0] {
            objectives[0].label.clear();
        }
        let r = validate_scenario(&f);
        assert!(r.is_ok());
        assert!(!r.warnings.is_empty());
    }
}
