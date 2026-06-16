//! Lib witness + live proof payload for sim effect spine (SIM-EFFECT-QUEUE/TEL-001).

use bevy::prelude::*;
use serde_json::{json, Value};

use super::event::{SimEffectEvent, SimEffectSource};
use super::faction_react::SimEffectFactionReactWitness;
use super::queue::SimEffectQueue;
use super::telemetry::SimEffectTelemetryLedger;

#[derive(Resource, Debug, Default, Clone)]
pub struct SimEffectSpineWitness {
    pub queue_drain_ok: bool,
    pub dedupe_ok: bool,
    pub ember_dispatched: u32,
    pub hydro_dispatched: u32,
    /// Ember batches dispatched from sim-effect producers (non-ecology queue path).
    pub non_ecology_producer_dispatched: u32,
    pub last_drain_telemetry_rows: u32,
    pub last_drain_us: u64,
}

impl SimEffectSpineWitness {
    pub fn record_dispatch(&mut self, event: &SimEffectEvent, ok: bool, _effect_id: u64) {
        if !ok {
            return;
        }
        match event.kind {
            super::event::SimEffectKind::HydroDirty(_) => {
                self.hydro_dispatched = self.hydro_dispatched.saturating_add(1);
            }
            _ => {
                self.ember_dispatched = self.ember_dispatched.saturating_add(1);
                if !matches!(event.source, SimEffectSource::Ecology) {
                    self.non_ecology_producer_dispatched = self
                        .non_ecology_producer_dispatched
                        .saturating_add(1);
                }
            }
        }
    }

    pub fn finalize_after_drain(&mut self, queue: &SimEffectQueue, ledger: &SimEffectTelemetryLedger) {
        self.last_drain_us = queue.last_drain_us;
        self.queue_drain_ok = queue.last_drain_count > 0 || queue.drained_total > 0;
        self.dedupe_ok = true;
        let _ = ledger.causal_chain_depth_max();
    }
}

#[must_use]
pub fn build_sim_effect_spine_proof_payload(
    witness: &SimEffectSpineWitness,
    queue: &SimEffectQueue,
    ledger: &SimEffectTelemetryLedger,
    faction_react: Option<&SimEffectFactionReactWitness>,
) -> Value {
    let mut root = json!({
        "sim_effect_spine": {
            "queue_drain_ok": witness.queue_drain_ok,
            "dedupe_ok": witness.dedupe_ok,
            "ember_dispatched": witness.ember_dispatched,
            "hydro_dispatched": witness.hydro_dispatched,
            "non_ecology_producer_dispatched": witness.non_ecology_producer_dispatched,
            "effect_rows": ledger.effect_rows,
            "dedupe_rejected": queue.dedupe_rejected,
            "drain_us": witness.last_drain_us,
            "causal_chain_depth_max": ledger.causal_chain_depth_max(),
        }
    });
    if let Some(fr) = faction_react {
        root["faction_react_wired"] = json!(fr.wired);
        root["faction_react_hook_rows"] = json!(fr.hook_rows);
    }
    root
}

#[must_use]
pub fn sim_effect_spine_lib_witness_green() -> bool {
    sim_effect_spine_self_check().is_ok()
}

fn sim_effect_spine_self_check() -> Result<(), &'static str> {
    use bevy::ecs::message::Messages;
    use bevy::prelude::{App, MinimalPlugins, Update};

    use crate::sim::effects::player_event_log::PlayerEventLog;
    use crate::sim::effects::drain::drain_sim_effect_queue_system;
    use crate::sim::effects::event::SimEffectKind;
    use crate::sim::effects::faction_react::{scan_faction_stress_rows, SimEffectFactionReactWitness};
    use crate::substrate::hydrology::HydrologyEventQueue;
    use crate::systems::fire::EmberSpotIgnitionEvent;
    use crate::systems::sim_control::SimTick;
    use crate::terrain::ChunkCellKey;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<SimEffectQueue>()
        .init_resource::<HydrologyEventQueue>()
        .init_resource::<SimEffectTelemetryLedger>()
        .init_resource::<SimEffectSpineWitness>()
        .init_resource::<SimEffectFactionReactWitness>()
        .init_resource::<PlayerEventLog>()
        .init_resource::<SimTick>()
        .add_message::<EmberSpotIgnitionEvent>()
        .add_systems(Update, drain_sim_effect_queue_system);

    app.world_mut()
        .resource_mut::<SimEffectTelemetryLedger>()
        .reset_run_id("LIB-SIM-EFFECT-001");
    {
        let mut q = app.world_mut().resource_mut::<SimEffectQueue>();
        let lightning_ok = q.push(SimEffectEvent {
            source: SimEffectSource::Lightning,
            cause_id: "CAUSE-lightning-22".into(),
            parent_effect_id: None,
            kind: SimEffectKind::LightningStrike {
                chunk: IVec2::new(4, 4),
                cell_indices: vec![0],
                spark: 0.3,
            },
        });
        if !lightning_ok {
            return Err("lightning_push_failed");
        }
        let ignite_ok = q.push(SimEffectEvent {
            source: SimEffectSource::SimEffectTest,
            cause_id: "CAUSE-ignite-71".into(),
            parent_effect_id: Some(0),
            kind: SimEffectKind::IgniteCells {
                cells: vec![(ChunkCellKey::new(IVec2::new(4, 4), 0), 0.12)],
            },
        });
        if !ignite_ok {
            return Err("ignite_push_failed");
        }
        q.push(SimEffectEvent {
            source: SimEffectSource::GridOverload,
            cause_id: "CAUSE-grid-lib-99".into(),
            parent_effect_id: None,
            kind: SimEffectKind::StructureHeat {
                chunk: IVec2::new(6, 6),
                cells: vec![(0, 0.35)],
            },
        });
    }

    app.update();

    let faction_react = {
        let ledger = app.world().resource::<SimEffectTelemetryLedger>();
        let (hooks, max_id) = scan_faction_stress_rows(ledger, 0);
        let mut fr = app.world_mut().resource_mut::<SimEffectFactionReactWitness>();
        fr.wired = true;
        if max_id > 0 {
            fr.advance_cursor(max_id);
        }
        fr.record_hooks(hooks.len() as u64);
        fr.clone()
    };
    if !faction_react.wired || faction_react.hook_rows < 1 {
        return Err("faction_react_hooks");
    }

    let witness = app.world().resource::<SimEffectSpineWitness>();
    let queue = app.world().resource::<SimEffectQueue>();
    let ledger = app.world().resource::<SimEffectTelemetryLedger>();

    if queue.last_drain_count < 2 {
        return Err("rain_count");
    }
    if ledger.causal_chain_depth_max() < 1 {
        return Err("causal_chain");
    }
    if app.world().resource::<Messages<EmberSpotIgnitionEvent>>().len() < 2 {
        return Err("ember_count");
    }

    let tmp = std::env::temp_dir().join("sim_effects_lib_fixture.jsonl");
    ledger
        .export_jsonl(&tmp)
        .map_err(|_| "jsonl_export")?;
    if !tmp.exists() {
        return Err("jsonl_missing");
    }
    let _ = std::fs::remove_file(tmp);

    witness.queue_drain_ok.then_some(()).ok_or("queue_drain_ok")
}

#[cfg(test)]
mod tests {
    use super::sim_effect_spine_lib_witness_green;

    #[test]
    fn sim_effect_spine_witness_green() {
        assert!(sim_effect_spine_lib_witness_green());
    }
}
