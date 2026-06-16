//! SimEffect queue spine — PLAN-SIM-EFFECT-SPINE-001 P0/P1.

mod drain;
mod event;
mod faction_react;
mod player_event_log;
mod producers;
mod queue;
mod telemetry;
mod witness;

pub use drain::{drain_sim_effect_queue_system, SimEffectSystemSet};
pub use producers::{
    enqueue_grid_overload_sim_effects, enqueue_lightning_strike_sim_effects, LightningRiskLatch,
};
pub use event::{SimEffectEvent, SimEffectKind, SimEffectSource};
pub use queue::SimEffectQueue;
pub use telemetry::{SimEffectTelemetryLedger, SimEffectTelemetryRecord, SIM_EFFECTS_JSONL};
pub use player_event_log::{
    clear_player_event_crit_unread, event_log_ui_001_witness_green, event_log_ui_001_witness_json,
    event_log_ui_format_witness_green, event_log_ui_impl_witness_green,
    event_log_ui_ops_strip_witness_green, event_log_ui_projection_witness_green,
    format_ops_strip_alerts_line, format_ops_strip_event_crit_line,
    format_player_event_row_line, format_player_event_tray_body, format_player_event_tray_row_line,
    project_player_event_log_from_drain, PlayerEventLog, PLAYER_EVENT_DEDUPE_TICKS,
    PLAYER_EVENT_LOG_CAP, PLAYER_EVENT_TRAY_BODY_MAX_ROWS,
};
pub use faction_react::{
    classify_faction_stress_row, scan_faction_stress_rows, stress_severity,
    FactionStressHook, FactionStressTelemetryClass, SimEffectFactionReactWitness,
    STRUCTURE_HEAT_KIND_TAG,
};
pub use witness::{
    build_sim_effect_spine_proof_payload, sim_effect_spine_lib_witness_green,
    SimEffectSpineWitness,
};

use bevy::prelude::*;

use crate::systems::sim_control::SimControlSystemSet;

pub struct SimEffectsPlugin;

impl Plugin for SimEffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SimEffectQueue>()
            .init_resource::<SimEffectTelemetryLedger>()
            .init_resource::<SimEffectSpineWitness>()
            .init_resource::<SimEffectFactionReactWitness>()
            .init_resource::<PlayerEventLog>()
            .init_resource::<LightningRiskLatch>()
            .register_type::<SimEffectSource>()
            .add_systems(
                Update,
                (
                    enqueue_lightning_strike_sim_effects,
                    enqueue_grid_overload_sim_effects,
                    drain_sim_effect_queue_system,
                )
                    .chain()
                    .in_set(SimEffectSystemSet::Drain),
            )
            .configure_sets(
                Update,
                SimEffectSystemSet::Drain.after(SimControlSystemSet::AdvanceSimTick),
            );
    }
}
