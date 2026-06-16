//! Tick drain + domain dispatch adapters (fire ember waist, hydrology queue).

use std::time::Instant;

use bevy::prelude::*;

use crate::substrate::hydrology::HydrologyEventQueue;
use crate::systems::ecology::{DisturbanceKind, LandscapeDisturbanceQueue};
use crate::systems::fire::EmberSpotIgnitionEvent;
use crate::systems::sim_control::SimTick;
use crate::terrain::ChunkCellKey;

use super::event::{SimEffectEvent, SimEffectKind};
use super::queue::SimEffectQueue;
use super::telemetry::SimEffectTelemetryLedger;
use super::witness::SimEffectSpineWitness;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum SimEffectSystemSet {
    Drain,
}

pub fn drain_sim_effect_queue_system(
    tick: Res<SimTick>,
    mut queue: ResMut<SimEffectQueue>,
    mut hydro: ResMut<HydrologyEventQueue>,
    mut ember_writer: MessageWriter<EmberSpotIgnitionEvent>,
    mut ledger: ResMut<SimEffectTelemetryLedger>,
    mut witness: ResMut<SimEffectSpineWitness>,
    mut player_log: ResMut<super::player_event_log::PlayerEventLog>,
    mut landscape_disturbances: Option<ResMut<LandscapeDisturbanceQueue>>,
) {
    let start = Instant::now();
    queue.last_drain_count = 0;
    witness.last_drain_telemetry_rows = 0;

    let events: Vec<SimEffectEvent> = queue.pending.drain(..).collect();
    let mut drained: Vec<(SimEffectEvent, u64, bool)> = Vec::with_capacity(events.len());
    for event in events {
        let ok = dispatch_one(
            &event,
            &mut hydro,
            &mut ember_writer,
            landscape_disturbances.as_deref_mut(),
        );
        let effect_id = ledger.record_drain(tick.0, &event, ok);
        witness.record_dispatch(&event, ok, effect_id);
        drained.push((event, effect_id, ok));
        queue.last_drain_count = queue.last_drain_count.saturating_add(1);
        queue.drained_total = queue.drained_total.saturating_add(1);
    }

    let projection_refs: Vec<(&SimEffectEvent, u64, bool)> =
        drained.iter().map(|(e, id, ok)| (e, *id, *ok)).collect();
    super::player_event_log::project_player_event_log_from_drain(
        tick.0,
        &projection_refs,
        &mut player_log,
    );

    queue.last_drain_us = start.elapsed().as_micros() as u64;
    witness.last_drain_telemetry_rows = queue.last_drain_count;
    witness.last_drain_us = queue.last_drain_us;
    if queue.last_drain_count > 0 {
        witness.queue_drain_ok = true;
    }
    queue.clear_tick_dedupe();
}

fn dispatch_one(
    event: &SimEffectEvent,
    hydro: &mut HydrologyEventQueue,
    ember_writer: &mut MessageWriter<EmberSpotIgnitionEvent>,
    landscape_disturbances: Option<&mut LandscapeDisturbanceQueue>,
) -> bool {
    match &event.kind {
        SimEffectKind::IgniteCells { cells } => {
            if cells.is_empty() {
                return false;
            }
            for (target, spark) in cells {
                if *spark <= 1e-6 {
                    continue;
                }
                ember_writer.write(EmberSpotIgnitionEvent {
                    target: *target,
                    spark: *spark,
                });
            }
            true
        }
        SimEffectKind::LightningStrike {
            chunk,
            cell_indices,
            spark,
        } => {
            if cell_indices.is_empty() || *spark <= 1e-6 {
                return false;
            }
            for idx in cell_indices {
                ember_writer.write(EmberSpotIgnitionEvent {
                    target: ChunkCellKey::new(*chunk, *idx),
                    spark: *spark,
                });
            }
            true
        }
        SimEffectKind::HydroDirty(dirty) => hydro.push(dirty.clone()),
        SimEffectKind::LandscapeDisturbance { chunk, harvest } => {
            if let Some(q) = landscape_disturbances {
                q.pending.push((
                    *chunk,
                    if *harvest {
                        DisturbanceKind::Harvest
                    } else {
                        DisturbanceKind::ConstructionClear
                    },
                ));
                true
            } else {
                false
            }
        }
        SimEffectKind::StructureHeat { chunk, cells } => {
            if cells.is_empty() {
                return false;
            }
            for (idx, heat) in cells {
                ember_writer.write(EmberSpotIgnitionEvent {
                    target: ChunkCellKey::new(*chunk, *idx),
                    spark: *heat,
                });
            }
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sim::effects::event::{SimEffectKind, SimEffectSource};
    use crate::sim::effects::queue::SimEffectQueue;
    use crate::sim::effects::telemetry::SimEffectTelemetryLedger;
    use crate::sim::effects::witness::SimEffectSpineWitness;
    use crate::substrate::hydrology::HydrologyEventQueue;
    use crate::systems::sim_control::SimTick;
    use bevy::ecs::message::Messages;
    use bevy::prelude::{App, MinimalPlugins, Update};

    #[test]
    fn drain_dispatches_ember_and_hydro_without_double_apply() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimEffectQueue>()
            .init_resource::<HydrologyEventQueue>()
            .init_resource::<SimEffectTelemetryLedger>()
            .init_resource::<SimEffectSpineWitness>()
            .init_resource::<SimTick>()
            .add_message::<EmberSpotIgnitionEvent>()
            .add_systems(Update, drain_sim_effect_queue_system);

        {
            let mut q = app.world_mut().resource_mut::<SimEffectQueue>();
            q.push(SimEffectEvent {
                source: SimEffectSource::SimEffectTest,
                cause_id: "CAUSE-test-ember".into(),
                parent_effect_id: None,
                kind: SimEffectKind::IgniteCells {
                    cells: vec![(ChunkCellKey::new(IVec2::ZERO, 0), 0.15)],
                },
            });
            let parent = q.push(SimEffectEvent {
                source: SimEffectSource::Lightning,
                cause_id: "CAUSE-lightning-22".into(),
                parent_effect_id: None,
                kind: SimEffectKind::LightningStrike {
                    chunk: IVec2::new(1, 0),
                    cell_indices: vec![2],
                    spark: 0.25,
                },
            });
            assert!(parent);
            q.push(SimEffectEvent {
                source: SimEffectSource::Lightning,
                cause_id: "CAUSE-lightning-22".into(),
                parent_effect_id: None,
                kind: SimEffectKind::LightningStrike {
                    chunk: IVec2::new(1, 0),
                    cell_indices: vec![2],
                    spark: 0.25,
                },
            });
            q.push(SimEffectEvent {
                source: SimEffectSource::SimEffectTest,
                cause_id: "CAUSE-test-ignite-child".into(),
                parent_effect_id: Some(1),
                kind: SimEffectKind::IgniteCells {
                    cells: vec![(ChunkCellKey::new(IVec2::new(1, 0), 2), 0.1)],
                },
            });
        }

        app.update();

        let embers = app.world().resource::<Messages<EmberSpotIgnitionEvent>>();
        assert_eq!(embers.len(), 3);

        let q = app.world().resource::<SimEffectQueue>();
        assert_eq!(q.last_drain_count, 3);
        assert_eq!(q.dedupe_rejected, 1);

        let ledger = app.world().resource::<SimEffectTelemetryLedger>();
        assert!(ledger.causal_chain_depth_max() >= 1);

        app.update();
        let embers2 = app.world().resource::<Messages<EmberSpotIgnitionEvent>>();
        assert_eq!(embers2.len(), 3, "second drain must not re-apply");
    }
}
