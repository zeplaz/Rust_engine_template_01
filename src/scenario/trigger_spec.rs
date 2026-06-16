//! **SCENARIO-TRIGGER-001** — RON trigger sketch for scripted sim-effect enqueue.

use serde::{Deserialize, Serialize};

use super::scenario_steps::{ScenarioIgniteCell, ScenarioStep};

/// Data-driven trigger → [`ScenarioStep::EmitSimEffect`] (sketch v1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerSpecV1 {
    pub schema_version: u32,
    pub id: String,
    /// Optional sim tick gate (inclusive minimum).
    pub when_tick_min: Option<u64>,
    pub source: String,
    pub cause_id: String,
    #[serde(default)]
    pub parent_effect_id: Option<u64>,
    pub cells: Vec<ScenarioIgniteCell>,
}

impl TriggerSpecV1 {
    #[must_use]
    pub fn to_emit_step(&self) -> Option<ScenarioStep> {
        use crate::sim::effects::SimEffectSource;
        let source = match self.source.as_str() {
            "scenario_script" => SimEffectSource::ScenarioScript,
            "ecology" => SimEffectSource::Ecology,
            "lightning" => SimEffectSource::Lightning,
            "grid_overload" => SimEffectSource::GridOverload,
            "construction" => SimEffectSource::Construction,
            _ => return None,
        };
        if self.cells.is_empty() {
            return None;
        }
        Some(ScenarioStep::EmitSimEffect {
            source,
            cause_id: self.cause_id.clone(),
            parent_effect_id: self.parent_effect_id,
            cells: self.cells.clone(),
        })
    }
}

#[must_use]
pub fn scenario_trigger_001_witness_green() -> bool {
    scenario_trigger_001_self_check().is_ok()
}

fn scenario_trigger_001_self_check() -> Result<(), &'static str> {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest.join("assets/scenarios/triggers/demo_ignite.trigger.ron");
    let text = std::fs::read_to_string(&path).map_err(|_| "ron_missing")?;
    let spec: TriggerSpecV1 = ron::from_str(&text).map_err(|_| "ron_parse")?;
    if spec.schema_version != 1 {
        return Err("schema");
    }
    spec.to_emit_step().ok_or("step")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_trigger_001_fixture_ron_green() {
        assert!(scenario_trigger_001_witness_green());
    }
}
