//! Presentation dump for in-flight projectiles + recent impacts (trails / explosion cues).

use bevy::prelude::*;

use super::ballistics::BallisticFlightBook;

/// View/HUD sample of ballistic activity (sole writer: [`publish_ballistic_present_system`]).
#[derive(Resource, Debug, Default, Clone)]
pub struct BallisticPresentFrame {
    pub revision: u64,
    /// Trail polylines in world metres: each flight contributes up to 2 points (prev→pos).
    pub trail_segments: Vec<(Vec2, Vec2)>,
    /// Recent impact flashes (metres, age_s).
    pub impacts: Vec<(Vec2, f32)>,
    pub in_flight: u32,
}

#[derive(Resource, Debug, Default)]
pub struct BallisticImpactRing {
    hits: Vec<(Vec2, f32)>,
}

/// Capture trails + age impact flashes for render/HUD.
pub(crate) fn publish_ballistic_present_system(
    book: Res<BallisticFlightBook>,
    clock: Res<crate::systems::sim_control::SimStepClock>,
    mut ring: ResMut<BallisticImpactRing>,
    mut frame: ResMut<BallisticPresentFrame>,
    mut last_impacts: Local<u64>,
) {
    let dt = clock.step_secs() * clock.due.max(1) as f32;
    for hit in ring.hits.iter_mut() {
        hit.1 += dt;
    }
    ring.hits.retain(|h| h.1 < 2.5);

    if book.impacts > *last_impacts {
        // Approximate: new impacts this frame — use flight targets that just cleared.
        // Prefer last known positions from trails; ring is filled by companion on impact enqueue.
        *last_impacts = book.impacts;
    }

    frame.trail_segments.clear();
    // OP-8: a short tail behind each projectile only. The old origin→position segment drew a
    // growing yellow streak from the launcher across the map for the whole flight.
    const TAIL_M: f32 = 160.0;
    for f in &book.flights {
        let flown = (f.pos - f.origin).length();
        if flown > 1.0 {
            let dir = (f.target - f.origin).normalize_or_zero();
            frame.trail_segments.push((f.pos - dir * flown.min(TAIL_M), f.pos));
        }
    }
    frame.in_flight = book.flights.len() as u32;
    frame.impacts = ring.hits.clone();
    frame.revision = frame.revision.wrapping_add(1);
}

/// Record an explosion flash (called from blast present or ballistics hook).
pub(crate) fn push_ballistic_impact_flash(ring: &mut BallisticImpactRing, pos_m: Vec2) {
    ring.hits.push((pos_m, 0.0));
    if ring.hits.len() > 32 {
        ring.hits.remove(0);
    }
}

pub(crate) fn record_impacts_from_blast_craters(
    craters: Res<crate::construction::BlastCraterBook>,
    mut ring: ResMut<BallisticImpactRing>,
    mut last: Local<u64>,
) {
    if craters.blasts_applied <= *last {
        return;
    }
    *last = craters.blasts_applied;
    use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;
    if let Some(&(tx, ty)) = craters.tiles.iter().next() {
        push_ballistic_impact_flash(
            &mut ring,
            Vec2::new((tx as f32 + 0.5) * TILE_METRES_GAMEPLAY, (ty as f32 + 0.5) * TILE_METRES_GAMEPLAY),
        );
    }
}
