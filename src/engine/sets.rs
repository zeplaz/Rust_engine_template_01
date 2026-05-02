//! Navigation / motion **`SystemSet`s** — wired from [`crate::systems::navigation::NavigationSchedulePlugin`]
//! **after** [`crate::systems::transport::TransportSchedule::CostCache`].
//!
//! **Runbooks:** [`../../prompts/guides/ecs_systems_schedule_runbook_v1.md`](../../prompts/guides/ecs_systems_schedule_runbook_v1.md)
//! · **S2** [`../../prompts/matrix/engine_bevy/runbook/s2_schedule_navigation_steps_v1.md`](../../prompts/matrix/engine_bevy/runbook/s2_schedule_navigation_steps_v1.md).

use bevy::prelude::SystemSet;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum NavSets {
    /// Road-vehicle damage / max-speed caps before integrating motion (S2).
    DamageSpeedAdjustment,
    /// Path followers / displacement — after cost cache + damage (W3+).
    MotionCalculation,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum GameSystemSet {
    TerrainGeneration,
    RegionGeneration,
    AgentGeneration,
    AgentRelationships,
    Simulation,
}
