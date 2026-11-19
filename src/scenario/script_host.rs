use bevy::prelude::*;
use std::collections::VecDeque;

use crate::sim::effects::SimEffectQueue;
use crate::strategic::StrategicRasterConfig;
use crate::scenario::dam_flood::{
    dam_channel_span_footprint, DamControlIntent, DamFloodBenchmarkBook, DamFloodPlayerOwner,
    DamFloodStage,
};
use crate::systems::water::{
    apply_slow_domain_time_skip, BarrierBreachRequest, BarrierBreachSpec, BarrierInstallRequest,
    BarrierInstallSpec, SlowDomainTimeSkip,
};
use crate::systems::sim_control::{SimControlState, SimStepClock};
use crate::systems::weather::WeatherPollingStations;

use super::ignite::{
    enqueue_scenario_ignite_sim_effect, ignite_cell_from_world_tile,
    ignite_cells_from_scenario_cells,
};
use super::objectives::ScenarioObjectiveMarker;
use super::scenario_steps::ScenarioStep;
use super::scenario_types::ScenarioFileV1;
use super::trigger_registry::{
    build_trigger_fact_memory, reorder_pending_trigger_effects, trigger_effect_to_emit_step,
    SCENARIO_TRIGGER_RULES,
};
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
    /// DF-1: absolute `SimStepClock.steps` that must be reached before draining more steps.
    pub advance_wait_until_steps: Option<u64>,
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
            self.advance_wait_until_steps = None;
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
        self.advance_wait_until_steps = None;
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
        self.advance_wait_until_steps = None;
        self.running = true;
        self.current_state = ScenarioExecutionState::Running;
        self.last_error = None;
        self.execution_log
            .push("Re-queued all steps from active scenario.".into());
    }
}

/// Release a front `WaitUntilSiteOperational` once a site with that catalog id is Operational.
/// Narrow system (scenario host + sites); `drain_script_steps` only blocks on the step.
pub fn gate_wait_until_site_operational_system(
    mut host: ResMut<EngineScriptHost>,
    sites: Query<(&crate::strategic::ConstructionSite, &crate::strategic::PlannedSite)>,
) {
    let Some(ScenarioStep::WaitUntilSiteOperational { catalog_id }) = host.pending_steps.front()
    else {
        return;
    };
    let want = catalog_id.clone();
    let done = sites.iter().any(|(site, planned)| {
        planned.catalog_id.as_deref() == Some(want.as_str())
            && site.phase == crate::strategic::SiteConstructionPhase::Operational
    });
    if done {
        host.pending_steps.pop_front();
        host.current_step_index = host.current_step_index.saturating_add(1);
        host.execution_log.push(format!("WaitUntilSiteOperational reached: {want}"));
    }
}

pub fn drain_script_steps(
    mut commands: Commands,
    mut host: ResMut<EngineScriptHost>,
    mut sim_control: ResMut<SimControlState>,
    mut sim_effect_queue: ResMut<SimEffectQueue>,
    clock: Option<Res<SimStepClock>>,
    stations: Option<Res<WeatherPollingStations>>,
    raster: Option<Res<StrategicRasterConfig>>,
    objective_entities: Query<Entity, With<ScenarioObjectiveMarker>>,
    mut dam_control: Option<ResMut<DamControlIntent>>,
    mut time_skip: Option<ResMut<SlowDomainTimeSkip>>,
    mut dam_book: Option<ResMut<DamFloodBenchmarkBook>>,
    player_owner: Option<Res<DamFloodPlayerOwner>>,
    mut barrier_install: Option<ResMut<BarrierInstallRequest>>,
    mut barrier_breach: Option<ResMut<BarrierBreachRequest>>,
    mut commit: MessageWriter<crate::strategic::CommitConstructionSiteEvent>,
    mut strike_orders: Option<ResMut<crate::combat::StrikeOrderQueue>>,
) {
    if !host.running {
        return;
    }

    let tick = clock.map_or(0, |c| c.steps);

    // DF-1: SimAdvance blocks until the clock reaches the wait target.
    if let Some(until) = host.advance_wait_until_steps {
        if tick < until {
            return;
        }
        host.advance_wait_until_steps = None;
        host.execution_log
            .push(format!("SimAdvance complete at tick={tick}"));
    }

    let sample = stations
        .as_deref()
        .filter(|s| s.ready)
        .map(|s| s.sample_at_tile(bevy::math::IVec2::ZERO));
    let drought = sample.map(|s| (1.0 - s.humidity).clamp(0.0, 1.0)).unwrap_or(0.0);
    let rain = sample
        .map(|s| (s.rain_mm_h.max(0.0) / 10.0).clamp(0.0, 1.0))
        .unwrap_or(0.0);
    let fuel = 0.25_f32;
    let mem = build_trigger_fact_memory(tick, drought, fuel, rain);
    reorder_pending_trigger_effects(
        &mut host.pending_steps,
        &mem,
        SCENARIO_TRIGGER_RULES,
    );

    // DF-1: WaitUntilTick — peek without popping until ready.
    if let Some(ScenarioStep::WaitUntilTick { tick: need }) = host.pending_steps.front() {
        if tick < *need {
            return;
        }
    }
    // Site waits are released by `gate_wait_until_site_operational_system` (own domain query).
    if matches!(host.pending_steps.front(), Some(ScenarioStep::WaitUntilSiteOperational { .. })) {
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
        ScenarioStep::WaitUntilSiteOperational { .. } => {}
        ScenarioStep::SetSimSpeed { speed } => {
            sim_control.speed = speed.clamp(0.0, 1000.0);
            host.execution_log.push(format!("SetSimSpeed: {:.1}x", sim_control.speed));
        }
        ScenarioStep::NoOp => {
            host.execution_log.push("Executed NoOp".into());
        }
        ScenarioStep::Log { message } => {
            host.execution_log.push(format!("LOG: {message}"));
        }
        ScenarioStep::SimAdvance { ticks } => {
            let ticks = ticks.max(0);
            sim_control.steps_remaining = sim_control.steps_remaining.saturating_add(ticks);
            if ticks > 0 {
                host.advance_wait_until_steps = Some(tick.saturating_add(u64::from(ticks)));
            }
            host.execution_log
                .push(format!("Queued SimAdvance for {ticks} ticks (block until done)"));
        }
        ScenarioStep::WaitUntilTick { tick: need } => {
            host.execution_log
                .push(format!("WaitUntilTick reached need={need} now={tick}"));
        }
        ScenarioStep::CommitSite {
            origin_x,
            origin_z,
            width,
            depth,
            catalog_id,
        } => {
            let owner = player_owner
                .as_ref()
                .and_then(|p| p.0)
                .unwrap_or_else(|| commands.spawn(Name::new("ScenarioSiteOwner")).id());
            let is_dam = catalog_id
                .as_deref()
                .is_some_and(|id| id.to_ascii_lowercase().contains("dam"));
            let archetype = if is_dam {
                crate::strategic::SiteArchetype::Dam
            } else {
                crate::strategic::SiteArchetype::Factory
            };
            let (origin, footprint) = if is_dam {
                dam_channel_span_footprint((origin_x, origin_z), width.max(1), depth.max(1))
            } else {
                (
                    crate::strategic::BuildSiteTile {
                        x: origin_x.max(0) as u32,
                        z: origin_z.max(0) as u32,
                    },
                    crate::strategic::FootprintTiles {
                        width: width.max(1),
                        depth: depth.max(1),
                    },
                )
            };
            commit.write(crate::strategic::CommitConstructionSiteEvent {
                site_id: crate::strategic::SiteId::UNASSIGNED,
                owner,
                archetype,
                origin,
                footprint,
                layer: crate::strategic::LayerType::Surface,
                catalog_id,
                placement: None,
            });
            if let Some(book) = dam_book.as_mut() {
                book.enter_stage(DamFloodStage::S1Build, tick);
            }
            host.execution_log.push(format!(
                "CommitSite: origin=({},{}) {}x{} archetype={archetype:?}",
                origin.x, origin.z, footprint.width, footprint.depth
            ));
        }
        ScenarioStep::SetDamControl {
            gate_open01,
            outlet_open01,
            turbine_flow_setpoint_m3s,
        } => {
            if let Some(ctrl) = dam_control.as_mut() {
                ctrl.gate_open01 = gate_open01.clamp(0.0, 1.0);
                ctrl.outlet_open01 = outlet_open01.clamp(0.0, 1.0);
                // Legacy scenario step: single turbine setpoint lands on unit 0.
                ctrl.set_legacy_turbine_setpoint(turbine_flow_setpoint_m3s.max(0.0));
            }
            if let Some(book) = dam_book.as_mut() {
                let closed = outlet_open01 <= 0.2 && gate_open01 <= 0.2;
                if closed {
                    book.enter_stage(DamFloodStage::S2Impound, tick);
                } else {
                    book.enter_stage(DamFloodStage::S4Operate, tick);
                }
            }
            host.execution_log.push(format!(
                "SetDamControl: gate={gate_open01:.2} outlet={outlet_open01:.2} turbine={turbine_flow_setpoint_m3s:.1}"
            ));
        }
        ScenarioStep::InstallBarrier {
            world_z,
            crest_m,
            half_span,
            center_world_x,
        } => {
            if let Some(req) = barrier_install.as_mut() {
                req.pending = Some(BarrierInstallSpec {
                    tiles: Vec::new(),
                    world_z,
                    crest_m: crest_m.max(0.0),
                    half_span: half_span.max(1),
                    center_world_x,
                });
            }
            // General water API (e.g. a ROI with no dam). Book values are mirrored from water
            // and DamBody, never written by script (CLOSE-S1).
            host.execution_log.push(format!(
                "InstallBarrier: z={world_z} crest_m={crest_m:.1} half_span={half_span}"
            ));
        }
        ScenarioStep::ApplyBreach {
            clear_crest,
            breach_flow_m3s,
        } => {
            if let Some(req) = barrier_breach.as_mut() {
                req.push(BarrierBreachSpec {
                    dam: None,
                    clear_crest,
                    breach_flow_m3s: breach_flow_m3s.max(0.0),
                });
            }
            host.execution_log.push(format!(
                "ApplyBreach: clear_crest={clear_crest} breach_flow_m3s={breach_flow_m3s:.1}"
            ));
        }
        ScenarioStep::StrikeOrder {
            target_x,
            target_z,
            warhead,
        } => {
            if let Some(book) = dam_book.as_mut() {
                book.enter_stage(DamFloodStage::S5Strike, tick);
            }
            // One warhead flight via StrikeOrderQueue (DF-7). Breach waits for impact.
            let tm = crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;
            let launch = crate::scenario::dam_flood::load_dam_valley_fixture().launch_site_origin;
            let origin = bevy::math::Vec2::new((launch.0 - 18) as f32 * tm, launch.1 as f32 * tm);
            let target = bevy::math::Vec2::new(target_x as f32 * tm, target_z as f32 * tm);
            if let Some(q) = strike_orders.as_mut() {
                q.enqueue(crate::combat::StrikeOrder {
                    origin_xz_m: origin,
                    target_xz_m: target,
                    warhead_id: warhead.clone(),
                });
            }
            host.execution_log.push(format!(
                "StrikeOrder: target=({target_x},{target_z}) warhead={warhead} (missile launch; breach on impact)"
            ));
        }
        ScenarioStep::SetTimeSkip {
            enabled,
            dt_slow_secs,
        } => {
            if let Some(skip) = time_skip.as_mut() {
                apply_slow_domain_time_skip(skip, enabled, dt_slow_secs);
            }
            host.execution_log.push(format!(
                "SetTimeSkip: enabled={enabled} dt_slow_secs={dt_slow_secs}"
            ));
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
                let mapped = ignite_cells_from_scenario_cells(&cells);
                let pushed = enqueue_scenario_ignite_sim_effect(
                    sim_effect_queue.as_mut(),
                    source,
                    cause_id.clone(),
                    parent_effect_id,
                    mapped,
                );
                host.execution_log.push(format!(
                    "EmitSimEffect: cause={cause_id} cells={} pushed={pushed}",
                    cells.len()
                ));
            }
        }
        ScenarioStep::IgniteAt {
            tile,
            spark,
            cause_id,
        } => {
            let cells_per_chunk = raster
                .as_ref()
                .map(|r| r.cells_per_chunk)
                .unwrap_or_else(|| StrategicRasterConfig::default().cells_per_chunk);
            let tile_x = tile.tile_x;
            let tile_z = tile.tile_z;
            let mapped = vec![ignite_cell_from_world_tile(tile, cells_per_chunk, spark)];
            let pushed = enqueue_scenario_ignite_sim_effect(
                sim_effect_queue.as_mut(),
                crate::sim::effects::SimEffectSource::ScenarioScript,
                cause_id.clone(),
                None,
                mapped,
            );
            if let Some(book) = dam_book.as_mut() {
                book.enter_stage(DamFloodStage::S9DryBurn, tick);
            }
            host.execution_log.push(format!(
                "IgniteAt: tile=({}, {}) spark={spark} cause={cause_id} pushed={pushed}",
                tile_x, tile_z
            ));
        }
        ScenarioStep::TriggerEffect { effect_id } => {
            match trigger_effect_to_emit_step(&effect_id) {
                Ok(ScenarioStep::EmitSimEffect {
                    source,
                    cause_id,
                    parent_effect_id,
                    cells,
                }) => {
                    let mapped = ignite_cells_from_scenario_cells(&cells);
                    let pushed = enqueue_scenario_ignite_sim_effect(
                        sim_effect_queue.as_mut(),
                        source,
                        cause_id.clone(),
                        parent_effect_id,
                        mapped,
                    );
                    host.execution_log.push(format!(
                        "TriggerEffect: id={effect_id} cause={cause_id} cells={} pushed={pushed}",
                        cells.len()
                    ));
                }
                Ok(_) => {
                    host.execution_log.push(format!(
                        "TriggerEffect: id={effect_id} rejected — non-ignite step"
                    ));
                }
                Err(err) => {
                    host.execution_log.push(format!(
                        "TriggerEffect: id={effect_id} failed — {err:?}"
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scenario::scenario_steps::ScenarioStep;
    use crate::scenario::scenario_types::{ScenarioFileV1, ScenarioMetadata};

    #[test]
    fn dam_scene_timeline_wait_until_tick_stays_pending() {
        let mut host = EngineScriptHost::default();
        host.load_script(ScenarioFileV1 {
            schema_version: 2,
            metadata: ScenarioMetadata {
                id: "dam_timeline".into(),
                display_name: "Dam timeline".into(),
                author: None,
                description: None,
            },
            steps: vec![
                ScenarioStep::WaitUntilTick { tick: 5 },
                ScenarioStep::Log {
                    message: "after".into(),
                },
            ],
        });
        assert!(host.running);
        assert_eq!(host.pending_steps.len(), 2);
        // Mimic drain gate: front WaitUntilTick with need=5 blocks while clock=0.
        let tick = 0u64;
        if let Some(ScenarioStep::WaitUntilTick { tick: need }) = host.pending_steps.front() {
            assert!(tick < *need);
        } else {
            panic!("expected WaitUntilTick at front");
        }
        // After clock advances past need, step may be popped.
        let tick = 5u64;
        if let Some(ScenarioStep::WaitUntilTick { tick: need }) = host.pending_steps.front() {
            assert!(tick >= *need);
            let _ = host.pending_steps.pop_front();
        }
        assert_eq!(host.pending_steps.len(), 1);
    }

    #[test]
    fn dam_scene_timeline_sim_advance_sets_wait() {
        let mut host = EngineScriptHost::default();
        host.running = true;
        let tick = 10u64;
        let ticks = 3u32;
        host.advance_wait_until_steps = Some(tick.saturating_add(u64::from(ticks)));
        assert_eq!(host.advance_wait_until_steps, Some(13));
        // Still blocked at tick 12.
        assert!(host.advance_wait_until_steps.is_some_and(|u| 12 < u));
        // Clear when reached.
        if host.advance_wait_until_steps.is_some_and(|u| 13 >= u) {
            host.advance_wait_until_steps = None;
        }
        assert!(host.advance_wait_until_steps.is_none());
    }
}
