//! **SCENARIO-TRIGGER-001** — RON trigger sketch for scripted sim-effect enqueue.
//!
//! **ECS-12** (gea4v2 17.8.11.1–2 *Data pathway communication* / *pros and cons of GUI wiring*) ·
//! LEGACY `events_message_passing` (gea_v2_e4 p632): a trigger is a *data pathway* —
//! `TriggerEffect(id)` (script edge) → registry lookup → this spec → one intent step.
//! The spec is plain data; it holds no handles into the ECS and can only be turned into a
//! [`ScenarioStep::EmitSimEffect`] **intent**. Applying that intent to sim truth is the
//! drain system's job in `script_host` (single funnel), never this module's.
//! Book trade-off kept: wiring as data is diffable/validatable (see `validation.rs`), but an
//! edge only exists if the registry declares it — an unknown id is a rejected edge, not a
//! silent no-op.

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
    /// Emit-intent for this pathway; `None` when the source tag is not a known sim-effect
    /// source or the pathway carries no cells (nothing to broadcast).
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

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(source: &str, cells: usize) -> TriggerSpecV1 {
        TriggerSpecV1 {
            schema_version: 1,
            id: "t".into(),
            when_tick_min: None,
            source: source.into(),
            cause_id: "CAUSE-t".into(),
            parent_effect_id: Some(7),
            cells: (0..cells)
                .map(|i| ScenarioIgniteCell {
                    chunk_x: 0,
                    chunk_y: 0,
                    cell: i as u32,
                    spark: 0.5,
                })
                .collect(),
        }
    }

    #[test]
    fn ecs12_known_source_with_cells_emits_intent_only() {
        let step = spec("lightning", 2).to_emit_step().expect("edge fires");
        assert!(step.routes_fire_via_sim_effect());
        match step {
            ScenarioStep::EmitSimEffect {
                cause_id,
                parent_effect_id,
                cells,
                ..
            } => {
                assert_eq!(cause_id, "CAUSE-t");
                assert_eq!(parent_effect_id, Some(7));
                assert_eq!(cells.len(), 2);
            }
            other => panic!("trigger must only yield EmitSimEffect intent, got {other:?}"),
        }
    }

    #[test]
    fn ecs12_unknown_source_or_empty_cells_do_not_broadcast() {
        assert!(spec("not_a_source", 1).to_emit_step().is_none());
        assert!(spec("ecology", 0).to_emit_step().is_none());
    }
}
