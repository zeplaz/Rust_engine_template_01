//! Scripted scenario execution (Wave 1+): `EngineScriptHost`, RON scenarios, `SimControlState` stepping, Wave 3 objectives.
//! Runbook: `prompts/guides/scenario_campaign_scripted_tools_runbook_v1.md`

pub mod objectives;
pub mod scenario_plugin;
pub mod scenario_runner;
pub mod scenario_steps;
pub mod scenario_types;
pub mod script_host;
pub mod validation;

pub use objectives::{
    ObjectiveTargetRef, ScenarioObjectiveKindV1, ScenarioObjectiveMarker, ScenarioObjectiveV1,
};
pub use scenario_plugin::ScenarioScriptingPlugin;
pub use scenario_types::ScenarioFileV1;
pub use validation::{
    validate_scenario, ScenarioValidationReport, ScenarioValidationSeverity,
};

#[cfg(test)]
mod tests;
