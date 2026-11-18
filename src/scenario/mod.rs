//! Scripted scenario execution (Wave 1+): `EngineScriptHost`, RON scenarios, `SimControlState` stepping, Wave 3 objectives.
//! Runbook: `prompts/guides/scenario_campaign_scripted_tools_runbook_v1.md`
//!
//! Example playthroughs (not sim domains): [`dam_flood`] configures general water/construction
//! for the DamFlood benchmark — see PLAN-SCENARIO-VS-SYSTEMS.

pub mod dam_flood;
pub mod ignite;
pub mod objectives;
pub mod scenario_plugin;
pub mod scenario_runner;
pub mod scenario_steps;
pub mod scenario_types;
pub mod script_host;
pub mod trigger_registry;
pub mod trigger_spec;
pub mod validation;

pub use dam_flood::DamFloodPlugin;

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
