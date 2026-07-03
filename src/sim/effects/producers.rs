//! External sim-effect producers — lightning + grid overload → [`super::queue::SimEffectQueue`] only.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::entities::production::power::GridOverloadEvent;
use crate::gui::DEBUG_CHUNK_SPACING_WORLD;
use crate::systems::sim_control::{SimControlState, SimTick};
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

use super::event::{SimEffectEvent, SimEffectKind, SimEffectSource};
use super::queue::SimEffectQueue;

const LIGHTNING_STRIKE_THRESHOLD: f32 = 0.68;
const LIGHTNING_SALT: u32 = 0x4C47_0001;

#[derive(Resource, Debug, Default)]
pub struct LightningRiskLatch {
    prev: HashMap<IVec2, f32>,
}

#[inline]
fn sim_effect_hash01(sim: u64, chunk: IVec2, cell: u32, salt: u32) -> f32 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    sim.hash(&mut h);
    chunk.hash(&mut h);
    cell.hash(&mut h);
    salt.hash(&mut h);
    let v = h.finish();
    (v as f64 / u64::MAX as f64) as f32
}

#[inline]
fn lightning_strike_roll(tick: u64, chunk: IVec2, risk: f32) -> bool {
    if risk < LIGHTNING_STRIKE_THRESHOLD {
        return false;
    }
    let p = (risk - LIGHTNING_STRIKE_THRESHOLD + 0.06).clamp(0.02, 0.35);
    sim_effect_hash01(tick, chunk, 0, LIGHTNING_SALT) < p
}

#[inline]
fn pick_strike_cell(tick: u64, chunk: IVec2, cell_count: u32) -> u32 {
    if cell_count == 0 {
        return 0;
    }
    let h = sim_effect_hash01(tick, chunk, 1, LIGHTNING_SALT);
    ((h * cell_count as f32).floor() as u32).min(cell_count.saturating_sub(1))
}

#[inline]
fn world_xz_to_chunk(x: f32, z: f32, spacing: f32) -> IVec2 {
    let s = spacing.max(1.0);
    IVec2::new((x / s).floor() as i32, (z / s).floor() as i32)
}

/// Sample [`ChunkWeather::lightning_risk`] edges + deterministic roll → [`SimEffectKind::LightningStrike`].
pub fn enqueue_lightning_strike_sim_effects(
    sim: Res<SimControlState>,
    tick: Res<SimTick>,
    mut latch: ResMut<LightningRiskLatch>,
    mut queue: ResMut<SimEffectQueue>,
    chunks: Query<(&Chunk, &ChunkWeather, Option<&ChunkCellMatrix>)>,
) {
    if !sim.should_tick() {
        return;
    }
    for (chunk, weather, matrix) in &chunks {
        let prev = latch.prev.get(&chunk.coord).copied().unwrap_or(0.0);
        let edge = prev < LIGHTNING_STRIKE_THRESHOLD && weather.lightning_risk >= LIGHTNING_STRIKE_THRESHOLD;
        let roll = lightning_strike_roll(tick.0, chunk.coord, weather.lightning_risk);
        latch.prev.insert(chunk.coord, weather.lightning_risk);
        if !edge && !roll {
            continue;
        }
        let cell_count = matrix
            .map(|m| m.size.x.saturating_mul(m.size.y))
            .unwrap_or(1)
            .max(1);
        let cell = pick_strike_cell(tick.0, chunk.coord, cell_count);
        let spark = (weather.lightning_risk * 0.42).clamp(0.18, 0.85);
        queue.push(SimEffectEvent {
            source: SimEffectSource::Lightning,
            cause_id: format!("CAUSE-lightning-{}-{}", chunk.coord.x, chunk.coord.y),
            parent_effect_id: None,
            kind: SimEffectKind::LightningStrike {
                chunk: chunk.coord,
                cell_indices: vec![cell],
                spark,
            },
        });
    }
}

/// Map [`GridOverloadEvent`] → structure heat ember batch (no direct [`EmberSpotIgnitionEvent`] writes).
pub fn enqueue_grid_overload_sim_effects(
    sim: Res<SimControlState>,
    mut reader: MessageReader<GridOverloadEvent>,
    mut queue: ResMut<SimEffectQueue>,
    hosts: Query<&Transform>,
) {
    if !sim.should_tick() {
        return;
    }
    let spacing = DEBUG_CHUNK_SPACING_WORLD;
    for ev in reader.read() {
        let Ok(tf) = hosts.get(ev.grid_entity) else {
            continue;
        };
        let chunk = world_xz_to_chunk(tf.translation.x, tf.translation.z, spacing);
        let ratio = (ev.total_load / ev.total_capacity.max(f32::EPSILON)).clamp(1.0, 2.5);
        let spark = (ratio * 0.22).clamp(0.15, 0.72);
        queue.push(SimEffectEvent {
            source: SimEffectSource::GridOverload,
            cause_id: format!("CAUSE-grid-{}", ev.grid_entity.index()),
            parent_effect_id: None,
            kind: SimEffectKind::StructureHeat {
                chunk,
                cells: vec![(0, spark)],
            },
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::substrate::hydrology::HydrologyEventQueue;
    use crate::sim::effects::drain::drain_sim_effect_queue_system;
    use crate::sim::effects::SimEffectSpineWitness;
    use crate::sim::effects::SimEffectTelemetryLedger;
    use crate::systems::fire::EmberSpotIgnitionEvent;
    use bevy::ecs::message::Messages;
    use bevy::math::UVec2;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<HydrologyEventQueue>()
            .init_resource::<crate::sim::effects::player_event_log::PlayerEventLog>()
            .init_resource::<SimEffectQueue>()
            .init_resource::<SimControlState>()
            .init_resource::<SimTick>()
            .init_resource::<LightningRiskLatch>()
            .init_resource::<SimEffectTelemetryLedger>()
            .init_resource::<SimEffectSpineWitness>()
            .add_message::<GridOverloadEvent>()
            .add_message::<EmberSpotIgnitionEvent>()
            .add_systems(
                Update,
                (
                    enqueue_lightning_strike_sim_effects,
                    enqueue_grid_overload_sim_effects,
                    drain_sim_effect_queue_system,
                )
                    .chain(),
            );
        app
    }

    #[test]
    fn fire_ignition_p0_lightning_producer_enqueues_non_ecology_cause() {
        let mut app = test_app();
        app.world_mut().spawn((
            Chunk {
                coord: IVec2::new(3, 4),
            },
            ChunkWeather {
                lightning_risk: 0.82,
                ..ChunkWeather::default()
            },
            ChunkCellMatrix::new(UVec2::new(4, 4)),
        ));
        app.update();
        let q = app.world().resource::<SimEffectQueue>();
        assert!(q.pushed_total >= 1);
        let ledger = app.world().resource::<SimEffectTelemetryLedger>();
        assert!(
            ledger
                .rows
                .iter()
                .any(|r| r.source == SimEffectSource::Lightning),
            "lightning producer must drain with non-ecology cause"
        );
        assert!(
            app.world().resource::<Messages<EmberSpotIgnitionEvent>>().len() >= 1,
            "drain must emit ember from lightning producer"
        );
    }

    #[test]
    fn fire_ignition_p0_grid_overload_producer_enqueues_structure_heat() {
        let mut app = test_app();
        let grid = app
            .world_mut()
            .spawn(Transform::from_xyz(128.0, 0.0, 64.0))
            .id();
        app.world_mut().write_message(GridOverloadEvent {
            grid_entity: grid,
            total_load: 120.0,
            total_capacity: 80.0,
        });
        app.update();
        let ledger = app.world().resource::<SimEffectTelemetryLedger>();
        assert!(
            ledger
                .rows
                .iter()
                .any(|r| r.source == SimEffectSource::GridOverload),
            "grid overload producer must enqueue+drain structure heat"
        );
    }
}
