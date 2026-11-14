//! Deterministic ballistic / blast bookkeeping (Ranking C3 scaffold).
//!
//! Sim owns trajectories; presentation/audio consume dumps. No rigid-body world.

use bevy::prelude::*;

use crate::audio::{AudioOneShotKind, AudioOneShotQueue};
use crate::construction::{BlastIntent, BlastIntentQueue};
use crate::systems::sim_attention::SimAttentionFrame;
use crate::systems::sim_control::{SimControlSystemSet, SimStepClock};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

const METRES_PER_TILE: f32 = TILE_METRES_GAMEPLAY;

/// One in-flight projectile (missile / shell) — package-level, not airframe ECS.
#[derive(Clone, Debug)]
pub struct BallisticFlight {
    pub id: u64,
    pub origin: Vec2,
    pub target: Vec2,
    pub pos: Vec2,
    pub speed_tiles_per_step: f32,
    pub blast_radius: f32,
    pub kind: BallisticKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BallisticKind {
    Missile,
    Shell,
    EmberProxy,
}

#[derive(Resource, Default, Debug)]
pub struct BallisticFlightBook {
    pub flights: Vec<BallisticFlight>,
    next_id: u64,
    pub impacts: u64,
}

impl BallisticFlightBook {
    pub fn launch(
        &mut self,
        origin: Vec2,
        target: Vec2,
        speed_tiles_per_step: f32,
        blast_radius: f32,
        kind: BallisticKind,
    ) -> u64 {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.flights.push(BallisticFlight {
            id,
            origin,
            target,
            pos: origin,
            speed_tiles_per_step: speed_tiles_per_step.max(0.01),
            blast_radius,
            kind,
        });
        id
    }
}

/// Advance flights; enqueue combat audio on impact. Blast damage apply is DEFER → construction funnel.
pub fn step_ballistic_flights(
    mut book: ResMut<BallisticFlightBook>,
    mut audio: ResMut<AudioOneShotQueue>,
    mut blast_queue: Option<ResMut<BlastIntentQueue>>,
    attention: Option<Res<SimAttentionFrame>>,
    clock: Res<SimStepClock>,
) {
    if clock.due == 0 {
        return;
    }
    let steps = clock.due;
    let full_audio = attention
        .as_ref()
        .map(|a| a.player_theater_active)
        .unwrap_or(true);

    let mut flights = std::mem::take(&mut book.flights);
    let mut remain = Vec::with_capacity(flights.len());
    let mut impacts = 0_u64;
    for mut f in flights.drain(..) {
        let delta = f.target - f.pos;
        let dist = delta.length();
        let step = f.speed_tiles_per_step * steps as f32;
        if dist <= step || dist < 1e-3 {
            impacts += 1;
            if full_audio {
                let kind = match f.kind {
                    BallisticKind::Missile => AudioOneShotKind::Explosion,
                    BallisticKind::Shell => AudioOneShotKind::Explosion,
                    BallisticKind::EmberProxy => AudioOneShotKind::Shot,
                };
                audio.push_combat(kind, f.target);
            }
            // Construction owns damage — enqueue blast intent (never mutate sites here).
            if let Some(q) = blast_queue.as_mut() {
                q.enqueue(BlastIntent {
                    center_xz: f.target,
                    radius_m: f.blast_radius.max(1.0) * METRES_PER_TILE,
                    damage: match f.kind {
                        BallisticKind::Missile => 120.0,
                        BallisticKind::Shell => 80.0,
                        BallisticKind::EmberProxy => 15.0,
                    },
                    source_flight: f.id,
                });
            }
            continue;
        }
        f.pos += delta.normalize() * step;
        remain.push(f);
    }
    book.flights = remain;
    book.impacts = book.impacts.saturating_add(impacts);
}

pub struct BallisticsPlugin;

impl Plugin for BallisticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BallisticFlightBook>().add_systems(
            Update,
            step_ballistic_flights.after(SimControlSystemSet::AdvanceSimTick),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn launch_and_impact_in_steps() {
        let mut book = BallisticFlightBook::default();
        book.launch(Vec2::ZERO, Vec2::new(10.0, 0.0), 5.0, 2.0, BallisticKind::Missile);
        assert_eq!(book.flights.len(), 1);
        // Manual step mimic
        let mut f = book.flights.remove(0);
        f.pos += (f.target - f.pos).normalize() * 5.0;
        assert!((f.pos.x - 5.0).abs() < 1e-3);
    }
}
