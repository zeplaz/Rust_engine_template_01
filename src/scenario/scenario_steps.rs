use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::sim::effects::SimEffectSource;

use super::objectives::ScenarioObjectiveV1;

/// One ignite cell for [`ScenarioStep::EmitSimEffect`] (RON: `chunk_x`, `chunk_y`, `cell`, `spark`).
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Serialize, Deserialize)]
pub struct ScenarioIgniteCell {
    pub chunk_x: i32,
    pub chunk_y: i32,
    pub cell: u32,
    pub spark: f32,
}

/// World tile for [`ScenarioStep::IgniteAt`] — same axes as map editor / [`crate::strategic::BuildSiteTile`].
#[derive(Debug, Clone, Serialize, Deserialize, Reflect, PartialEq)]
#[reflect(Serialize, Deserialize)]
pub struct ScenarioWorldTile {
    pub tile_x: u32,
    pub tile_z: u32,
}

fn default_ignite_spark() -> f32 {
    0.55
}

fn default_scenario_ignite_cause_id() -> String {
    "CAUSE-scenario-ignite".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
#[reflect(Serialize, Deserialize)]
pub enum ScenarioStep {
    NoOp,
    SimAdvance { ticks: u32 },
    Log { message: String },
    /// Spawn [`super::objectives::ScenarioObjectiveMarker`] entities (Wave 3).
    RegisterObjectives {
        clear_existing: bool,
        objectives: Vec<ScenarioObjectiveV1>,
    },
    /// **SCENARIO-TRIGGER-001 / G-PLAY-FIRE-001** — enqueue sim-effect ignite (Path A, not harness seed).
    EmitSimEffect {
        source: SimEffectSource,
        cause_id: String,
        #[serde(default)]
        parent_effect_id: Option<u64>,
        cells: Vec<ScenarioIgniteCell>,
    },
    /// **VSS-T2-002** — single world-tile ignite via SimEffect waist (product path).
    IgniteAt {
        tile: ScenarioWorldTile,
        #[serde(default = "default_ignite_spark")]
        spark: f32,
        #[serde(default = "default_scenario_ignite_cause_id")]
        cause_id: String,
    },
    /// **VSS-T2-002** — resolve `assets/scenarios/triggers/{effect_id}.trigger.ron` → SimEffect ignite.
    TriggerEffect {
        effect_id: String,
    },
    /// DF-1 — hold until `SimStepClock.steps >= tick` (do not pop until ready).
    WaitUntilTick {
        tick: u64,
    },
    /// Block the script until a committed site with this catalog id is Operational (built through
    /// the real construction + logistics path). Gate: `gate_wait_until_site_operational_system`.
    WaitUntilSiteOperational {
        catalog_id: String,
    },
    /// DF-1 — commit a construction site through the sole funnel (stub until DF-4 dam archetype).
    CommitSite {
        origin_x: i32,
        origin_z: i32,
        width: u32,
        depth: u32,
        #[serde(default)]
        catalog_id: Option<String>,
    },
    /// DF-1 — dam gate / outlet / turbine setpoints (stored on `DamControlIntent`).
    SetDamControl {
        #[serde(default)]
        gate_open01: f32,
        #[serde(default)]
        outlet_open01: f32,
        #[serde(default)]
        turbine_flow_setpoint_m3s: f32,
    },
    /// DF-2 — install barrier crest on the general water field (post-commit / player build).
    InstallBarrier {
        world_z: i32,
        #[serde(default = "default_barrier_crest")]
        crest_m: f32,
        #[serde(default = "default_barrier_half_span")]
        half_span: u32,
        #[serde(default)]
        center_world_x: Option<i32>,
    },
    /// DF-2 / destroy cascade — clear crest and/or arm breach outflow (stage falls → intakes trip).
    ApplyBreach {
        #[serde(default = "default_true")]
        clear_crest: bool,
        #[serde(default = "default_breach_flow")]
        breach_flow_m3s: f32,
    },
    /// DF-1 / DF-7 — strike order intent (launch wiring lands in DF-7).
    StrikeOrder {
        target_x: i32,
        target_z: i32,
        #[serde(default = "default_warhead_tag")]
        warhead: String,
    },
    /// DF-1 / D4 — coarse-step time skip for slow domains.
    SetTimeSkip {
        enabled: bool,
        #[serde(default = "default_dt_slow")]
        dt_slow_secs: u32,
    },
}

fn default_warhead_tag() -> String {
    "medium".into()
}

fn default_dt_slow() -> u32 {
    600
}

fn default_barrier_crest() -> f32 {
    40.0
}

fn default_barrier_half_span() -> u32 {
    6
}

fn default_true() -> bool {
    true
}

fn default_breach_flow() -> f32 {
    400.0
}

impl ScenarioStep {
    /// True when the step enqueues fire through the SimEffect spine (not harness seed).
    #[must_use]
    pub fn routes_fire_via_sim_effect(&self) -> bool {
        matches!(
            self,
            Self::EmitSimEffect { .. } | Self::IgniteAt { .. } | Self::TriggerEffect { .. }
        )
    }
}
