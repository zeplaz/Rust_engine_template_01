use bevy::prelude::*;
use std::collections::VecDeque;

use crate::sim::effects::{SimEffectEvent, SimEffectKind, SimEffectQueue};
use crate::systems::sim_control::SimControlState;
use crate::terrain::ChunkCellKey;

use super::objectives::ScenarioObjectiveMarker;
use super::scenario_steps::ScenarioStep;
use super::scenario_types::ScenarioFileV1;
use super::validation::{validate_scenario, ScenarioValidationReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
#[reflect(Default)]
pub enum ScenarioExecutionState {
    Idle,
    Running,
    Completed,
    Failed,
}

impl Default for ScenarioExecutionState {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct EngineScriptHost {
    pub active_script: Option<ScenarioFileV1>,
    pub pending_steps: VecDeque<ScenarioStep>,
    pub execution_log: Vec<String>,
    pub running: bool,
    pub current_state: ScenarioExecutionState,
    pub current_step_index: usize,
    pub last_error: Option<String>,
    /// Last load validation (warnings may be present even when `last_error` is None).
    pub last_validation: Option<ScenarioValidationReport>,
}

impl EngineScriptHost {
    pub fn load_script(&mut self, script: ScenarioFileV1) {
        self.last_error = None;
        let report = validate_scenario(&script);
        self.last_validation = Some(report.clone());
        if !report.is_ok() {
            self.last_error = Some(report.errors.join("\n"));
            self.current_state = ScenarioExecutionState::Failed;
            self.running = false;
            self.pending_steps.clear();
            self.active_script = None;
            self.current_step_index = 0;
            return;
        }

        self.pending_steps.clear();
        for step in &script.steps {
            self.pending_steps.push_back(step.clone());
        }

        self.execution_log.clear();
        self.execution_log.push(format!(
            "Loaded scenario '{}'",
            script.metadata.display_name
        ));

        self.active_script = Some(script);
        self.running = true;
        self.current_state = ScenarioExecutionState::Running;
        self.current_step_index = 0;
    }

    pub fn stop(&mut self) {
        self.running = false;
        if self.current_state == ScenarioExecutionState::Running {
            self.current_state = ScenarioExecutionState::Idle;
        }
    }

    /// Continue after [`Self::stop`] mid-run (`pending_steps` preserved).
    pub fn resume(&mut self) {
        if self.active_script.is_none() {
            return;
        }
        if self.pending_steps.is_empty() {
            return;
        }
        self.running = true;
        self.current_state = ScenarioExecutionState::Running;
    }

    /// Re-queue every step from [`Self::active_script`] and start draining again.
    pub fn restart_from_active(&mut self) {
        let Some(script) = self.active_script.as_ref() else {
            self.execution_log
                .push("Restart: no active scenario — use Load first.".into());
            return;
        };
        self.pending_steps.clear();
        for step in &script.steps {
            self.pending_steps.push_back(step.clone());
        }
        self.current_step_index = 0;
        self.running = true;
        self.current_state = ScenarioExecutionState::Running;
        self.last_error = None;
        self.execution_log
            .push("Re-queued all steps from active scenario.".into());
    }
}

pub fn drain_script_steps(
    mut commands: Commands,
    mut host: ResMut<EngineScriptHost>,
    mut sim_control: ResMut<SimControlState>,
    mut sim_effect_queue: ResMut<SimEffectQueue>,
    objective_entities: Query<Entity, With<ScenarioObjectiveMarker>>,
) {
    if !host.running {
        return;
    }

    let Some(step) = host.pending_steps.pop_front() else {
        host.execution_log.push("Scenario complete".into());
        host.running = false;
        host.current_state = ScenarioExecutionState::Completed;
        return;
    };

    host.current_step_index = host.current_step_index.saturating_add(1);

    match step {
        ScenarioStep::NoOp => {
            host.execution_log.push("Executed NoOp".into());
        }
        ScenarioStep::Log { message } => {
            host.execution_log.push(format!("LOG: {message}"));
        }
        ScenarioStep::SimAdvance { ticks } => {
            sim_control.steps_remaining = sim_control
                .steps_remaining
                .saturating_add(ticks);
            host
                .execution_log
                .push(format!("Queued SimAdvance for {ticks} ticks"));
        }
        ScenarioStep::RegisterObjectives {
            clear_existing,
            objectives,
        } => {
            if clear_existing {
                let to_clear: Vec<Entity> = objective_entities.iter().collect();
                for e in to_clear {
                    commands.entity(e).despawn();
                }
            }
            let n = objectives.len();
            for obj in &objectives {
                let m = ScenarioObjectiveMarker::from(obj);
                let label = format!("Scenario objective {}", m.objective_id);
                commands.spawn((m, Name::new(label)));
            }
            host.execution_log.push(format!(
                "RegisterObjectives: spawned {n} ScenarioObjectiveMarker entities (clear_existing={clear_existing})"
            ));
        }
        ScenarioStep::EmitSimEffect {
            source,
            cause_id,
            parent_effect_id,
            cells,
        } => {
            if cells.is_empty() {
                host.execution_log.push("EmitSimEffect: rejected — empty cells".into());
            } else {
                let mapped: Vec<(ChunkCellKey, f32)> = cells
                    .iter()
                    .map(|c| {
                        (
                            ChunkCellKey {
                                chunk: IVec2::new(c.chunk_x, c.chunk_y),
                                cell_index: c.cell,
                            },
                            c.spark,
                        )
                    })
                    .collect();
                let pushed = sim_effect_queue.push(SimEffectEvent {
                    source,
                    cause_id: cause_id.clone(),
                    parent_effect_id,
                    kind: SimEffectKind::IgniteCells { cells: mapped },
                });
                host.execution_log.push(format!(
                    "EmitSimEffect: cause={cause_id} cells={} pushed={pushed}",
                    cells.len()
                ));
            }
        }
    }
}
