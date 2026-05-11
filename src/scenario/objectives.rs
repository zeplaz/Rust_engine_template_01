//! **Wave 3+** — scenario objective DTOs + ECS stub markers (no win/lose, no triggers yet).
//! Runbook: `prompts/guides/scenario_campaign_scripted_tools_runbook_v1.md` §5 Wave 3, §4 Q1–Q2.

use bevy::math::IVec2;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Stable spatial / logical target for an objective (not tile-only).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub enum ObjectiveTargetRef {
    /// Hierarchical region id, e.g. `region:ukraine/donetsk_city`.
    Region(String),
    Tile(IVec2),
    Chunk(IVec2),
    Corridor(String),
    Site(String),
}

/// Serialized objective (RON). Wire shape stays stable for tooling; ECS marker mirrors this at runtime.
#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub struct ScenarioObjectiveV1 {
    /// Stable id (never entity id, label, or list order).
    #[serde(alias = "id")]
    pub objective_id: String,
    pub kind: ScenarioObjectiveKindV1,
    pub label: String,
    #[serde(default)]
    pub target: Option<ObjectiveTargetRef>,
    /// Legacy Wave 3 field; use [`Self::target`]. Parsed forms like `tile:x,y` / `chunk:x,y` map at runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub region_key: Option<String>,
    #[serde(default)]
    pub owning_faction: Option<String>,
    #[serde(default)]
    pub opposing_faction: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub enum ScenarioObjectiveKindV1 {
    CaptureRegion,
    DestroyInfrastructure,
    MaintainSupply,
}

/// Spawned by [`crate::scenario::scenario_steps::ScenarioStep::RegisterObjectives`].
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
pub struct ScenarioObjectiveMarker {
    pub objective_id: String,
    pub kind: ScenarioObjectiveKindV1,
    pub label: String,
    pub target: Option<ObjectiveTargetRef>,
    pub owning_faction: Option<String>,
    pub opposing_faction: Option<String>,
    pub tags: Vec<String>,
}

/// Effective target: explicit `target`, else legacy `region_key` string (e.g. `tile:2,3`).
#[must_use]
pub fn objective_effective_target(o: &ScenarioObjectiveV1) -> Option<ObjectiveTargetRef> {
    if let Some(t) = &o.target {
        return Some(t.clone());
    }
    o.region_key
        .as_deref()
        .and_then(parse_legacy_region_key)
}

fn parse_legacy_region_key(s: &str) -> Option<ObjectiveTargetRef> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(rest) = s.strip_prefix("tile:") {
        return parse_ivec2_csv(rest).map(ObjectiveTargetRef::Tile);
    }
    if let Some(rest) = s.strip_prefix("chunk:") {
        return parse_ivec2_csv(rest).map(ObjectiveTargetRef::Chunk);
    }
    Some(ObjectiveTargetRef::Region(s.to_string()))
}

fn parse_ivec2_csv(rest: &str) -> Option<IVec2> {
    let mut parts = rest.split(',').map(str::trim);
    let x: i32 = parts.next()?.parse().ok()?;
    let z: i32 = parts.next()?.parse().ok()?;
    Some(IVec2::new(x, z))
}

impl From<&ScenarioObjectiveV1> for ScenarioObjectiveMarker {
    fn from(o: &ScenarioObjectiveV1) -> Self {
        Self {
            objective_id: o.objective_id.clone(),
            kind: o.kind,
            label: o.label.clone(),
            target: objective_effective_target(o),
            owning_faction: o.owning_faction.clone(),
            opposing_faction: o.opposing_faction.clone(),
            tags: o.tags.clone(),
        }
    }
}
