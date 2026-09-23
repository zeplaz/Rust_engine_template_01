//! **VSS-T2-002** — resolve `TriggerEffect(effect_id)` → trigger RON → SimEffect enqueue.

use std::path::{Path, PathBuf};

use super::scenario_steps::ScenarioStep;
use super::trigger_spec::TriggerSpecV1;

pub const TRIGGER_ASSET_DIR: &str = "assets/scenarios/triggers";

/// Known trigger ids whose on-disk filename differs from `{id}.trigger.ron`.
const TRIGGER_ID_ALIASES: &[(&str, &str)] = &[("demo_ignite_v0", "demo_ignite")];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriggerLoadError {
    NotFound,
    ReadFailed,
    ParseFailed,
    EmptyCells,
    UnsupportedSource,
}

fn triggers_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(TRIGGER_ASSET_DIR)
}

fn trigger_path_for_id(effect_id: &str) -> PathBuf {
    let root = triggers_root();
    let direct = root.join(format!("{effect_id}.trigger.ron"));
    if direct.is_file() {
        return direct;
    }
    for (id, stem) in TRIGGER_ID_ALIASES {
        if *id == effect_id {
            let alias = root.join(format!("{stem}.trigger.ron"));
            if alias.is_file() {
                return alias;
            }
        }
    }
    direct
}

/// Load a trigger spec by logical effect id (filename or alias).
pub fn load_trigger_spec(effect_id: &str) -> Result<TriggerSpecV1, TriggerLoadError> {
    let path = trigger_path_for_id(effect_id);
    let text = std::fs::read_to_string(&path).map_err(|_| TriggerLoadError::NotFound)?;
    let spec: TriggerSpecV1 = ron::from_str(&text).map_err(|_| TriggerLoadError::ParseFailed)?;
    if spec.id != effect_id {
        return Err(TriggerLoadError::NotFound);
    }
    if spec.cells.is_empty() {
        return Err(TriggerLoadError::EmptyCells);
    }
    Ok(spec)
}

/// Resolve trigger id to an emit step (validation / script drain).
pub fn trigger_effect_to_emit_step(effect_id: &str) -> Result<ScenarioStep, TriggerLoadError> {
    let spec = load_trigger_spec(effect_id)?;
    spec.to_emit_step().ok_or(TriggerLoadError::UnsupportedSource)
}

#[must_use]
pub fn scenario_trigger_registry_witness_green() -> bool {
    scenario_trigger_registry_self_check().is_ok()
}

fn scenario_trigger_registry_self_check() -> Result<(), &'static str> {
    let step = trigger_effect_to_emit_step("demo_ignite_v0").map_err(|_| "resolve")?;
    if !step.routes_fire_via_sim_effect() {
        return Err("route");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_effect_resolves_demo_ignite_alias() {
        let step = trigger_effect_to_emit_step("demo_ignite_v0").expect("demo trigger");
        assert!(step.routes_fire_via_sim_effect());
    }
}
