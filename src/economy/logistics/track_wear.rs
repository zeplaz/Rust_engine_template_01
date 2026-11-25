//! RD-5 worn tracks (PLAN-ROADS-v1): traffic off the road network wears the ground, wear fades
//! without traffic, and a well-worn run along a truck leg becomes a **Track** road (graph edges,
//! faster than open ground). A track is not construction: no bill, nobody ordered it; it is
//! placed through the normal construction funnel like any uncharged road, so the player or AI can
//! later upgrade it to a built tier.
//!
//! - [`accumulate_track_wear_system`]: sole writer of [`TrackWearBook::wear`]. Trucks add passes to
//!   the off-road tiles of the legs they drive (vehicles on the leg ÷ trip time, per second);
//!   moving formations add wear at their tile by unit class. Everything decays by a half-life.
//! - [`form_worn_tracks_system`]: walks each planned leg's off-road tiles in order; a connected run
//!   of at least `min_run_tiles` tiles at `passes_to_form` becomes Track road segments (enqueued on
//!   [`ConstructionPlanQueue`]); those tiles are marked formed (sole writer of `formed`).
//!
//! Tuning: `road_tiers.ron` `worn_tracks`.

use std::collections::{BTreeMap, HashMap, HashSet};

use bevy::prelude::*;

use super::orders::model::{SiteId, VehiclePhase};
use super::orders::plugin::LogisticsOrderSimRes;
use super::orders::SiteLegRouteBook;
use super::routes::TruckRoute;
use crate::combat::balance::UnitKind;
use crate::combat::formation::Formation;
use crate::construction::{ConstructionIntent, ConstructionPlanQueue, ConstructionType};
use crate::infrastructure::road_tier::WornTrackTuning;
use crate::infrastructure::RoadTier;
use crate::strategic::BuildSiteTile;
use crate::systems::sim_control::SimStepClock;

/// Truck speed (m/s) trip times are taken at when spreading passes over a leg.
const WEAR_REF_TRUCK_MPS: f32 = 12.0;
/// Track forming is checked this often (sim steps).
const FORM_EVERY_STEPS: u64 = 5;
/// Wear below this many passes is dropped.
const WEAR_FLOOR: f32 = 0.05;

/// Ground wear (passes) per tile, and tiles already turned into track.
#[derive(Resource, Debug, Default, Clone)]
pub struct TrackWearBook {
    pub wear: HashMap<IVec2, f32>,
    pub formed: HashSet<IVec2>,
    /// Track runs formed so far (dump).
    pub tracks_formed: u32,
    pub revision: u64,
}

/// Off-road tiles a route drives, in driving order (stretches at or below the off-road speed;
/// consecutive off-road stretches join into one run).
#[must_use]
pub fn offroad_tile_runs(route: &TruckRoute, offroad_mps: f32) -> Vec<Vec<IVec2>> {
    let mut runs: Vec<Vec<IVec2>> = Vec::new();
    let mut cur: Vec<IVec2> = Vec::new();
    for (w, cap) in route.points.windows(2).zip(&route.caps_mps) {
        if *cap > offroad_mps + 1e-3 {
            if !cur.is_empty() {
                runs.push(std::mem::take(&mut cur));
            }
            continue;
        }
        let n = (w[0].distance(w[1]).ceil() as usize).max(1) * 2;
        for i in 0..=n {
            let t = w[0].lerp(w[1], i as f32 / n as f32).floor().as_ivec2();
            if cur.last() != Some(&t) {
                cur.push(t);
            }
        }
    }
    if !cur.is_empty() {
        runs.push(cur);
    }
    runs
}

/// Wear a moving formation of `kind` leaves per second on its tile (passes).
#[must_use]
pub fn formation_wear_per_s(kind: UnitKind, t: &WornTrackTuning) -> f32 {
    match kind {
        UnitKind::Infantry => t.infantry_per_s,
        UnitKind::Recon | UnitKind::Engineer => t.light_per_s,
        UnitKind::Mechanized | UnitKind::Artillery | UnitKind::AirDefense => t.wheeled_per_s,
        UnitKind::Armor => t.tracked_per_s,
    }
}

/// Decay all wear by `dt` seconds of half-life `half_life_s`; drops faint tiles.
pub fn decay_wear(wear: &mut HashMap<IVec2, f32>, dt: f32, half_life_s: f32) {
    let k = 0.5_f32.powf(dt / half_life_s.max(1.0));
    wear.retain(|_, w| {
        *w *= k;
        *w >= WEAR_FLOOR
    });
}

/// Trucks and formations wear the ground; wear fades.
pub fn accumulate_track_wear_system(
    clock: Option<Res<SimStepClock>>,
    sim: Option<Res<LogisticsOrderSimRes>>,
    legs: Option<Res<SiteLegRouteBook>>,
    formations: Query<&Formation>,
    mut book: ResMut<TrackWearBook>,
) {
    let Some(clock) = clock else {
        return;
    };
    if clock.due == 0 {
        return;
    }
    let dt = clock.due as f32 * clock.step_secs();
    let table = crate::infrastructure::road_tier_table();
    let t = &table.worn_tracks;
    decay_wear(&mut book.wear, dt, t.half_life_s);
    let offroad = table.offroad_speed_mps();
    if let (Some(sim), Some(legs)) = (sim.as_deref(), legs.as_deref()) {
        let mut on_leg: BTreeMap<(SiteId, SiteId), u32> = BTreeMap::new();
        for v in sim.sim.vehicles.values() {
            if !matches!(v.phase, VehiclePhase::ToPickup(_) | VehiclePhase::Hauling(_)) {
                continue;
            }
            if let Some(pair) = crate::systems::navigation::vehicle_leg_pair(v) {
                *on_leg.entry(pair).or_insert(0) += 1;
            }
        }
        for (pair, n) in on_leg {
            let Some(leg) = legs.legs.get(&pair) else {
                continue;
            };
            let trip_s = leg.route.time_s(WEAR_REF_TRUCK_MPS).max(1.0);
            let passes = n as f32 * dt / trip_s;
            for run in offroad_tile_runs(&leg.route, offroad) {
                for tile in run {
                    *book.wear.entry(tile).or_insert(0.0) += passes;
                }
            }
        }
    }
    for f in &formations {
        if f.target.is_some() {
            *book.wear.entry(f.tile).or_insert(0.0) += formation_wear_per_s(f.kind, t) * dt;
        }
    }
}

/// Straight pieces covering a tile chain: a new piece starts where the chain turns.
#[must_use]
pub fn chain_to_pieces(chain: &[IVec2]) -> Vec<(IVec2, IVec2)> {
    let mut pieces = Vec::new();
    if chain.len() < 2 {
        return pieces;
    }
    let mut start = chain[0];
    let mut dir = chain[1] - chain[0];
    for w in chain.windows(2).skip(1) {
        let d = w[1] - w[0];
        if d != dir {
            pieces.push((start, w[0]));
            start = w[0];
            dir = d;
        }
    }
    pieces.push((start, *chain.last().expect("len >= 2")));
    pieces
}

/// Sub-runs of `run` where every tile has `passes_to_form` and is not yet track, at least
/// `min_run_tiles` long.
#[must_use]
pub fn worn_sub_runs(run: &[IVec2], book: &TrackWearBook, t: &WornTrackTuning) -> Vec<Vec<IVec2>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    for tile in run {
        let worn = book.wear.get(tile).copied().unwrap_or(0.0) >= t.passes_to_form && !book.formed.contains(tile);
        if worn {
            cur.push(*tile);
        } else if !cur.is_empty() {
            if cur.len() >= t.min_run_tiles {
                out.push(std::mem::take(&mut cur));
            } else {
                cur.clear();
            }
        }
    }
    if cur.len() >= t.min_run_tiles {
        out.push(cur);
    }
    out
}

/// Well-worn runs along planned legs become Track roads.
pub fn form_worn_tracks_system(
    clock: Option<Res<SimStepClock>>,
    legs: Option<Res<SiteLegRouteBook>>,
    mut book: ResMut<TrackWearBook>,
    mut queue: ResMut<ConstructionPlanQueue>,
    mut last: Local<u64>,
) {
    let (Some(clock), Some(legs)) = (clock, legs) else {
        return;
    };
    if clock.steps < *last + FORM_EVERY_STEPS {
        return;
    }
    *last = clock.steps;
    let table = crate::infrastructure::road_tier_table();
    let offroad = table.offroad_speed_mps();
    let mut formed = 0;
    for leg in legs.legs.values() {
        for run in offroad_tile_runs(&leg.route, offroad) {
            for sub in worn_sub_runs(&run, &book, &table.worn_tracks) {
                for (a, b) in chain_to_pieces(&sub) {
                    if a.x < 0 || a.y < 0 || b.x < 0 || b.y < 0 {
                        continue;
                    }
                    let head = BuildSiteTile { x: a.x as u32, z: a.y as u32 };
                    let tail = BuildSiteTile { x: b.x as u32, z: b.y as u32 };
                    queue.enqueue(ConstructionIntent {
                        entity_type: ConstructionType::RoadSegment { head, tail, tier: RoadTier::Track },
                        world_position: (a.as_vec2() + b.as_vec2()) * 0.5,
                        rotation: 0.0,
                    });
                }
                book.formed.extend(sub.iter().copied());
                formed += 1;
            }
        }
    }
    if formed > 0 {
        book.tracks_formed += formed;
        book.revision = book.revision.wrapping_add(1);
        bevy::log::debug!(target: "logistics::tracks", runs = formed, "worn tracks formed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuning() -> WornTrackTuning {
        WornTrackTuning::default()
    }

    #[test]
    fn only_off_road_stretches_are_worn() {
        // off-road (cap 5) → road (cap 10) → off-road (cap 5)
        let route = TruckRoute {
            points: vec![Vec2::new(0.5, 0.5), Vec2::new(4.5, 0.5), Vec2::new(10.5, 0.5), Vec2::new(10.5, 4.5)],
            caps_mps: vec![5.0, 10.0, 5.0],
        };
        let runs = offroad_tile_runs(&route, 5.0);
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].first(), Some(&IVec2::new(0, 0)));
        assert_eq!(runs[0].last(), Some(&IVec2::new(4, 0)));
        assert!(runs[1].iter().all(|t| t.x == 10), "{:?}", runs[1]);
    }

    #[test]
    fn a_run_needs_enough_wear_over_enough_tiles_and_forms_once() {
        let t = tuning();
        let run: Vec<IVec2> = (0..10).map(|x| IVec2::new(x, 0)).collect();
        let mut book = TrackWearBook::default();
        for tile in &run[..4] {
            book.wear.insert(*tile, t.passes_to_form + 1.0);
        }
        assert!(worn_sub_runs(&run, &book, &t).is_empty(), "4 tiles is too short");
        for tile in &run {
            book.wear.insert(*tile, t.passes_to_form + 1.0);
        }
        let subs = worn_sub_runs(&run, &book, &t);
        assert_eq!(subs, vec![run.clone()]);
        book.formed.extend(run.iter().copied());
        assert!(worn_sub_runs(&run, &book, &t).is_empty(), "formed tiles are not formed again");
    }

    #[test]
    fn wear_fades_without_traffic() {
        let mut wear = HashMap::from([(IVec2::ZERO, 10.0), (IVec2::ONE, 0.06)]);
        decay_wear(&mut wear, 3600.0, 3600.0);
        assert!((wear[&IVec2::ZERO] - 5.0).abs() < 1e-3, "one half-life halves it");
        assert!(!wear.contains_key(&IVec2::ONE), "faint wear is dropped");
    }

    #[test]
    fn armour_wears_more_than_infantry_and_chains_split_at_turns() {
        let t = tuning();
        assert!(formation_wear_per_s(UnitKind::Armor, &t) > formation_wear_per_s(UnitKind::Infantry, &t));
        let chain = [IVec2::new(0, 0), IVec2::new(1, 0), IVec2::new(2, 0), IVec2::new(2, 1), IVec2::new(2, 2)];
        assert_eq!(chain_to_pieces(&chain), vec![(IVec2::new(0, 0), IVec2::new(2, 0)), (IVec2::new(2, 0), IVec2::new(2, 2))]);
    }
}
