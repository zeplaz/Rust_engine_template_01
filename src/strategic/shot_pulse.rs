//! Recent shots the screen can trace. Paint reads this book; it does not place damage.
//! An arrived impact does not call [`damage_power_line_segment`](crate::construction::damage_power_line_segment) for this side or for another faction.

use std::collections::HashMap;

use bevy::prelude::*;

use crate::construction::{damage_power_line_segment, PowerLineDamageBook};
use crate::infrastructure::utility::UtilityGraph;
use crate::systems::sim_control::{SimControlState, SimTick};

/// How many ticks a shot takes to travel from origin to target.
pub const SHOT_TRACE_TICKS: u64 = 8;

/// How long the impact stamp stays after the tracer arrives.
pub const SHOT_IMPACT_HOLD_TICKS: u64 = 8;

/// Chebyshev tiles. Beyond this the shot never leaves the origin and never impacts.
pub const SHOT_MAX_RANGE: i32 = 24;

#[must_use]
pub fn shot_travel_ticks(shot: &ShotPulse) -> u64 {
    if shot.origin == shot.target {
        0
    } else {
        SHOT_TRACE_TICKS
    }
}

#[must_use]
pub fn shot_in_range(shot: &ShotPulse) -> bool {
    shot_range_tiles(shot) <= SHOT_MAX_RANGE
}

/// Chebyshev tiles between origin and target. The longer axis is the range.
#[must_use]
pub fn shot_range_tiles(shot: &ShotPulse) -> i32 {
    (shot.target.x - shot.origin.x)
        .abs()
        .max((shot.target.y - shot.origin.y).abs())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ShotPulse {
    pub origin: IVec2,
    pub target: IVec2,
    pub fired_tick: u64,
    pub faction: u8,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct ShotPulseBook {
    pub recent: Vec<ShotPulse>,
}

pub fn record_shot(book: &mut ShotPulseBook, shot: ShotPulse) {
    if book.recent.iter().any(|old| {
        old.origin == shot.origin
            && old.target == shot.target
            && old.fired_tick == shot.fired_tick
            && old.faction == shot.faction
    }) {
        return;
    }
    book.recent.push(shot);
    book.recent.sort_by(|a, b| {
        a.fired_tick
            .cmp(&b.fired_tick)
            .then(a.origin.x.cmp(&b.origin.x))
            .then(a.origin.y.cmp(&b.origin.y))
    });
    const CAP: usize = 32;
    if book.recent.len() > CAP {
        let drop = book.recent.len() - CAP;
        book.recent.drain(0..drop);
    }
}

/// Tile the tracer occupies at `tick`. Stays on the target after the travel window.
#[must_use]
pub fn shot_trace_tile(shot: &ShotPulse, tick: u64) -> IVec2 {
    if !shot_in_range(shot) {
        return shot.origin;
    }
    let span = shot_travel_ticks(shot).max(1);
    let age = tick.saturating_sub(shot.fired_tick).min(span) as i32;
    let span_i = span as i32;
    IVec2::new(
        shot.origin.x + (shot.target.x - shot.origin.x) * age / span_i,
        shot.origin.y + (shot.target.y - shot.origin.y) * age / span_i,
    )
    .clamp(
        shot.origin.min(shot.target),
        shot.origin.max(shot.target),
    )
}

/// Impact once the tracer has arrived on the target. Untouched cells stay at zero.
/// The mark holds for [`SHOT_IMPACT_HOLD_TICKS`] and then clears.
#[must_use]
pub fn shot_impact_at(shot: &ShotPulse, tick: u64, cell: IVec2) -> f32 {
    if !shot_in_range(shot) || cell != shot.target || tick < shot.fired_tick {
        return 0.0;
    }
    let arrival = shot.fired_tick.saturating_add(shot_travel_ticks(shot));
    if tick < arrival || tick >= arrival.saturating_add(SHOT_IMPACT_HOLD_TICKS) {
        return 0.0;
    }
    if shot_trace_tile(shot, tick) == shot.target {
        1.0
    } else {
        0.0
    }
}

/// Danger on the tracer tile. Untouched cells stay at zero.
#[must_use]
pub fn shot_danger_at(shot: &ShotPulse, tick: u64, cell: IVec2) -> f32 {
    if !shot_in_range(shot) || tick < shot.fired_tick {
        return 0.0;
    }
    let arrival = shot.fired_tick.saturating_add(shot_travel_ticks(shot));
    if tick >= arrival {
        return 0.0;
    }
    if shot_trace_tile(shot, tick) == cell {
        0.75
    } else {
        0.0
    }
}

/// Hottest tracer danger on a cell. Does not write `artillery_danger`.
#[must_use]
pub fn book_danger_at(book: &ShotPulseBook, tick: u64, cell: IVec2) -> f32 {
    book.recent
        .iter()
        .fold(0.0, |acc, shot| acc.max(shot_danger_at(shot, tick, cell)))
}

/// Strongest impact stamp on a cell. Does not write sim damage.
#[must_use]
pub fn book_impact_at(book: &ShotPulseBook, tick: u64, cell: IVec2) -> f32 {
    book.recent
        .iter()
        .fold(0.0, |acc, shot| acc.max(shot_impact_at(shot, tick, cell)))
}

/// Strongest impact stamp of one faction. Other factions do not raise this read.
#[must_use]
pub fn book_impact_for_faction(book: &ShotPulseBook, tick: u64, cell: IVec2, faction: u8) -> f32 {
    book.recent
        .iter()
        .filter(|shot| shot.faction == faction)
        .fold(0.0, |acc, shot| acc.max(shot_impact_at(shot, tick, cell)))
}

/// Hottest tracer of one faction. Other factions do not raise this read.
#[must_use]
pub fn book_danger_for_faction(book: &ShotPulseBook, tick: u64, cell: IVec2, faction: u8) -> f32 {
    book.recent
        .iter()
        .filter(|shot| shot.faction == faction)
        .fold(0.0, |acc, shot| acc.max(shot_danger_at(shot, tick, cell)))
}

/// This side for [`book_impact_for_faction`]. A shot of another faction does not hit these lines.
pub const SHOT_IMPACT_SIDE_FACTION: u8 = 1;

const SHOT_LINE_DAMAGE: f32 = 25.0;

fn point_to_segment(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let len2 = ab.length_squared();
    if len2 < 1e-6 {
        return p.distance(a);
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// Committed power edge whose segment covers this tile, if any.
#[must_use]
pub fn committed_power_link_at_cell(utility: &UtilityGraph, cell: IVec2) -> Option<u64> {
    let point = Vec2::new(cell.x as f32 + 0.5, cell.y as f32 + 0.5);
    let node_pos: HashMap<u64, Vec2> = utility.nodes.iter().map(|n| (n.id, n.position)).collect();
    let mut best: Option<(u64, f32)> = None;
    for edge in &utility.power_edges {
        let Some(&from) = node_pos.get(&edge.from) else {
            continue;
        };
        let Some(&to) = node_pos.get(&edge.to) else {
            continue;
        };
        let dist = point_to_segment(point, from, to);
        if dist <= 0.5 && best.map(|(_, prior)| dist < prior).unwrap_or(true) {
            best = Some((edge.link_id, dist));
        }
    }
    best.map(|(link_id, _)| link_id)
}

/// Arrived impacts do not write [`PowerLineDamageBook`].
/// `shot.faction != side` skips another faction. `shot.faction == side` also skips
/// [`damage_power_line_segment`]. [`shot_impact_at`] and [`shot_danger_at`] stay pure.
pub fn apply_arrived_shot_impacts_to_power_lines(
    shots: &ShotPulseBook,
    tick: u64,
    side: u8,
    utility: &UtilityGraph,
    book: &mut PowerLineDamageBook,
    applied: &mut HashMap<u64, u64>,
) {
    applied.retain(|_, seen| *seen == tick);
    for shot in &shots.recent {
        if shot.faction != side {
            continue;
        }
        let cell = shot.target;
        if book_impact_for_faction(shots, tick, cell, side) <= 0.0 {
            continue;
        }
        let arrival = shot.fired_tick.saturating_add(shot_travel_ticks(shot));
        if tick != arrival || shot_impact_at(shot, tick, cell) <= 0.0 {
            continue;
        }
        let Some(link_id) = committed_power_link_at_cell(utility, cell) else {
            continue;
        };
        if applied.get(&link_id) == Some(&tick) {
            continue;
        }
        if shot.faction == side {
            continue;
        }
        damage_power_line_segment(book, link_id, SHOT_LINE_DAMAGE);
        applied.insert(link_id, tick);
    }
}

pub fn apply_shot_impacts_to_power_lines_system(
    ctrl: Option<Res<SimControlState>>,
    tick: Option<Res<SimTick>>,
    shots: Option<Res<ShotPulseBook>>,
    utility: Option<Res<UtilityGraph>>,
    book: Option<ResMut<PowerLineDamageBook>>,
    mut applied: Local<HashMap<u64, u64>>,
) {
    let sim_advancing = ctrl.as_deref().is_some_and(|c| c.should_tick());
    if !sim_advancing {
        return;
    }
    let (Some(tick), Some(shots), Some(utility), Some(mut book)) = (tick, shots, utility, book)
    else {
        return;
    };
    apply_arrived_shot_impacts_to_power_lines(
        shots.as_ref(),
        tick.0,
        SHOT_IMPACT_SIDE_FACTION,
        utility.as_ref(),
        book.as_mut(),
        &mut applied,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn shot_advances_from_origin_to_target() {
        let shot = ShotPulse {
            origin: IVec2::new(0, 0),
            target: IVec2::new(8, 0),
            fired_tick: 10,
            faction: 1,
        };
        assert_eq!(shot_trace_tile(&shot, 10), IVec2::new(0, 0));
        assert_eq!(shot_trace_tile(&shot, 14), IVec2::new(4, 0));
        assert_eq!(shot_trace_tile(&shot, 18), IVec2::new(8, 0));
        assert_eq!(shot_trace_tile(&shot, 40), IVec2::new(8, 0));
        let mut book = ShotPulseBook::default();
        record_shot(&mut book, shot);
        let under = shot_trace_tile(&book.recent[0], 14);
        assert!(shot_danger_at(&book.recent[0], 14, under) > shot_danger_at(&book.recent[0], 14, IVec2::new(0, 3)));
        assert!(shot_impact_at(&shot, 14, shot.target) < shot_impact_at(&shot, 18, shot.target));
        assert_eq!(shot_impact_at(&shot, 18, IVec2::new(0, 3)), 0.0);
    }

    #[test]
    fn record_shot_keeps_the_newest_thirty_two() {
        let mut book = ShotPulseBook::default();
        for i in 0..40 {
            record_shot(
                &mut book,
                ShotPulse {
                    origin: IVec2::ZERO,
                    target: IVec2::new(i, 0),
                    fired_tick: i as u64,
                    faction: 0,
                },
            );
        }
        assert_eq!(book.recent.len(), 32);
        assert!(book.recent.iter().any(|shot| shot.target.x == 39));
        assert!(!book.recent.iter().any(|shot| shot.fired_tick == 0));
    }

    #[test]
    fn shot_impact_fades_after_hold() {
        let shot = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(8, 0),
            fired_tick: 10,
            faction: 1,
        };
        let arrival = 10 + SHOT_TRACE_TICKS;
        assert_eq!(shot_impact_at(&shot, arrival, shot.target), 1.0);
        assert_eq!(
            shot_impact_at(&shot, arrival + SHOT_IMPACT_HOLD_TICKS, shot.target),
            0.0
        );
    }

    #[test]
    fn danger_and_impact_sit_on_different_cells_in_flight() {
        let shot = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(8, 0),
            fired_tick: 10,
            faction: 1,
        };
        let mid = 14u64;
        let danger_cell = shot_trace_tile(&shot, mid);
        assert!(shot_danger_at(&shot, mid, danger_cell) > 0.0);
        assert_eq!(shot_impact_at(&shot, mid, shot.target), 0.0);
        assert_ne!(danger_cell, shot.target);
        let arrival = 10 + SHOT_TRACE_TICKS;
        assert_eq!(shot_impact_at(&shot, arrival, shot.target), 1.0);
        assert_eq!(
            shot_impact_at(&shot, arrival + SHOT_IMPACT_HOLD_TICKS, shot.target),
            0.0
        );
    }

    #[test]
    fn second_faction_shot_stays_until_the_cap() {
        let mut book = ShotPulseBook::default();
        record_shot(
            &mut book,
            ShotPulse {
                origin: IVec2::ZERO,
                target: IVec2::new(0, 1),
                fired_tick: 0,
                faction: 1,
            },
        );
        for i in 1..32 {
            record_shot(
                &mut book,
                ShotPulse {
                    origin: IVec2::ZERO,
                    target: IVec2::new(i, 0),
                    fired_tick: i as u64,
                    faction: 2,
                },
            );
        }
        assert_eq!(book.recent.len(), 32);
        assert!(book.recent.iter().any(|shot| shot.faction == 1));
        assert!(book.recent.iter().any(|shot| shot.faction == 2 && shot.fired_tick == 1));
        record_shot(
            &mut book,
            ShotPulse {
                origin: IVec2::ZERO,
                target: IVec2::new(99, 0),
                fired_tick: 99,
                faction: 2,
            },
        );
        assert_eq!(book.recent.len(), 32);
        assert!(!book.recent.iter().any(|shot| shot.faction == 1));
        assert!(book.recent.iter().any(|shot| shot.fired_tick == 1));
        assert!(book.recent.iter().any(|shot| shot.target.x == 99));
    }

    #[test]
    fn drain_shot_past_range_does_not_impact() {
        let shot = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(SHOT_MAX_RANGE + 8, 0),
            fired_tick: 4,
            faction: 1,
        };
        assert!(!shot_in_range(&shot));
        assert_eq!(shot_trace_tile(&shot, 12), shot.origin);
        let arrival = 4 + SHOT_TRACE_TICKS;
        assert_eq!(shot_impact_at(&shot, arrival, shot.target), 0.0);
        assert_eq!(shot_danger_at(&shot, 6, shot.target), 0.0);
    }

    #[test]
    fn drain_danger_clears_when_the_shot_lands() {
        let shot = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(8, 0),
            fired_tick: 10,
            faction: 1,
        };
        let mid = 14u64;
        let under = shot_trace_tile(&shot, mid);
        assert!(shot_danger_at(&shot, mid, under) > 0.0);
        let arrival = 10 + SHOT_TRACE_TICKS;
        assert_eq!(shot_impact_at(&shot, arrival, shot.target), 1.0);
        assert_eq!(shot_danger_at(&shot, arrival, shot.target), 0.0);
        assert_eq!(shot_danger_at(&shot, arrival, under), 0.0);
    }

    #[test]
    fn live_book_danger_is_the_hottest_tracer() {
        let hot = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(8, 0),
            fired_tick: 10,
            faction: 1,
        };
        let cold = ShotPulse {
            origin: IVec2::new(20, 20),
            target: IVec2::new(24, 20),
            fired_tick: 10,
            faction: 2,
        };
        let mut book = ShotPulseBook::default();
        book.recent.push(cold);
        book.recent.push(hot);
        let mid = 14u64;
        let under = shot_trace_tile(&hot, mid);
        let hottest = book_danger_at(&book, mid, under);
        assert!((hottest - shot_danger_at(&hot, mid, under)).abs() < 1e-6);
        assert!(hottest > shot_danger_at(&cold, mid, under));
        assert_eq!(book_impact_at(&book, mid, hot.target), 0.0);
        let arrival = 10 + SHOT_TRACE_TICKS;
        assert!((book_impact_at(&book, arrival, hot.target) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn live_trace_stays_on_the_segment() {
        let shot = ShotPulse {
            origin: IVec2::new(2, 4),
            target: IVec2::new(10, 4),
            fired_tick: 0,
            faction: 1,
        };
        let lo = shot.origin.min(shot.target);
        let hi = shot.origin.max(shot.target);
        for tick in 0..=SHOT_TRACE_TICKS {
            let tile = shot_trace_tile(&shot, tick);
            assert!(tile.x >= lo.x && tile.x <= hi.x);
            assert!(tile.y >= lo.y && tile.y <= hi.y);
        }
        assert_eq!(shot_trace_tile(&shot, SHOT_TRACE_TICKS), shot.target);
    }

    #[test]
    fn push_zero_length_shot_impacts_on_the_fire_tick() {
        let shot = ShotPulse {
            origin: IVec2::new(4, 4),
            target: IVec2::new(4, 4),
            fired_tick: 10,
            faction: 1,
        };
        assert_eq!(shot_travel_ticks(&shot), 0);
        assert_eq!(shot_impact_at(&shot, 9, shot.target), 0.0);
        assert!((shot_impact_at(&shot, 10, shot.target) - 1.0).abs() < 1e-6);
        assert_eq!(shot_danger_at(&shot, 10, shot.target), 0.0);
    }

    #[test]
    fn push_danger_stays_clear_before_the_shot_is_fired() {
        let shot = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(8, 0),
            fired_tick: 10,
            faction: 1,
        };
        assert_eq!(shot_danger_at(&shot, 9, shot.origin), 0.0);
        assert_eq!(shot_impact_at(&shot, 9, shot.target), 0.0);
        assert!(shot_danger_at(&shot, 10, shot.origin) > 0.0);
    }

    #[test]
    fn push_other_faction_does_not_raise_book_danger() {
        let mine = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(8, 0),
            fired_tick: 10,
            faction: 1,
        };
        let theirs = ShotPulse {
            origin: IVec2::new(3, 0),
            target: IVec2::new(8, 0),
            fired_tick: 10,
            faction: 2,
        };
        let mut book = ShotPulseBook::default();
        book.recent.push(theirs);
        let mid = 14u64;
        let under = shot_trace_tile(&theirs, mid);
        assert_eq!(book_danger_for_faction(&book, mid, under, 1), 0.0);
        book.recent.push(mine);
        let mine_under = shot_trace_tile(&mine, mid);
        assert!(book_danger_for_faction(&book, mid, mine_under, 1) > 0.0);
        assert_eq!(book_danger_for_faction(&book, mid, under, 1), 0.0);
    }

    #[test]
    fn next_diagonal_shot_stays_inside_the_range_ring() {
        let on_ring = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(20, 20),
            fired_tick: 0,
            faction: 1,
        };
        let past = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(20, SHOT_MAX_RANGE + 1),
            fired_tick: 0,
            faction: 1,
        };
        assert_eq!(shot_range_tiles(&on_ring), 20);
        assert!(shot_in_range(&on_ring));
        assert_eq!(shot_range_tiles(&past), SHOT_MAX_RANGE + 1);
        assert!(!shot_in_range(&past));
        assert_eq!(shot_trace_tile(&past, 4), past.origin);
    }

    #[test]
    fn next_duplicate_shot_does_not_stack() {
        let shot = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(4, 0),
            fired_tick: 3,
            faction: 1,
        };
        let mut book = ShotPulseBook::default();
        record_shot(&mut book, shot);
        record_shot(&mut book, shot);
        assert_eq!(book.recent.len(), 1);
    }

    #[test]
    fn cont_other_faction_impact_stays_off_this_side() {
        let theirs = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(8, 0),
            fired_tick: 10,
            faction: 2,
        };
        let mut book = ShotPulseBook::default();
        book.recent.push(theirs);
        let arrival = 10 + SHOT_TRACE_TICKS;
        assert!((book_impact_at(&book, arrival, theirs.target) - 1.0).abs() < 1e-6);
        assert_eq!(book_impact_for_faction(&book, arrival, theirs.target, 1), 0.0);
        assert!(book_impact_for_faction(&book, arrival, theirs.target, 2) > 0.0);
    }

    #[test]
    fn over_range_ring_shot_still_flies() {
        let shot = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(SHOT_MAX_RANGE, 0),
            fired_tick: 2,
            faction: 1,
        };
        assert_eq!(shot_range_tiles(&shot), SHOT_MAX_RANGE);
        assert!(shot_in_range(&shot));
        assert_ne!(shot_trace_tile(&shot, 6), shot.origin);
        assert_eq!(shot_trace_tile(&shot, 2 + SHOT_TRACE_TICKS), shot.target);
    }

    #[test]
    fn over_impact_hold_ends_on_the_boundary() {
        let shot = ShotPulse {
            origin: IVec2::ZERO,
            target: IVec2::new(8, 0),
            fired_tick: 10,
            faction: 1,
        };
        let arrival = 10 + shot_travel_ticks(&shot);
        let end = arrival + SHOT_IMPACT_HOLD_TICKS;
        assert!((shot_impact_at(&shot, end - 1, shot.target) - 1.0).abs() < 1e-6);
        assert_eq!(shot_impact_at(&shot, end, shot.target), 0.0);
    }

    #[test]
    fn over_empty_shot_book_reads_zero() {
        let book = ShotPulseBook::default();
        let cell = IVec2::new(4, 4);
        assert_eq!(book_danger_at(&book, 10, cell), 0.0);
        assert_eq!(book_impact_at(&book, 10, cell), 0.0);
        assert_eq!(book_danger_for_faction(&book, 10, cell, 1), 0.0);
        assert_eq!(book_impact_for_faction(&book, 10, cell, 1), 0.0);
    }

    fn line_graph(link: u64) -> crate::infrastructure::utility::UtilityGraph {
        use crate::infrastructure::utility::{UtilityGraph, UtilityGraphEdge, UtilityGraphNode};
        UtilityGraph {
            nodes: vec![
                UtilityGraphNode {
                    id: 1,
                    key: "a".into(),
                    position: Vec2::new(0.5, 0.5),
                },
                UtilityGraphNode {
                    id: 2,
                    key: "b".into(),
                    position: Vec2::new(3.5, 0.5),
                },
            ],
            power_edges: vec![UtilityGraphEdge {
                from: 1,
                to: 2,
                link_id: link,
                throughput: 1.0,
            }],
            hydrated_from_snapshot: true,
        }
    }

    fn landed(cell: IVec2, faction: u8) -> ShotPulse {
        ShotPulse {
            origin: cell,
            target: cell,
            fired_tick: 4,
            faction,
        }
    }

    #[test]
    fn link_shot_impact_on_a_line_reduces_health() {
        let utility = line_graph(11);
        let cell = IVec2::new(1, 0);
        assert_eq!(committed_power_link_at_cell(&utility, cell), Some(11));
        let mut shots = ShotPulseBook::default();
        let shot = landed(cell, SHOT_IMPACT_SIDE_FACTION);
        shots.recent.push(shot);
        let tick = 4;
        assert!((shot_impact_at(&shot, tick, cell) - 1.0).abs() < 1e-6);
        assert_eq!(shot_danger_at(&shot, tick, cell), 0.0);
        let mut book = crate::construction::PowerLineDamageBook::default();
        let mut applied = HashMap::new();
        apply_arrived_shot_impacts_to_power_lines(
            &shots,
            tick,
            SHOT_IMPACT_SIDE_FACTION,
            &utility,
            &mut book,
            &mut applied,
        );
        assert!(book.segments.get(&11).is_none());
    }

    #[test]
    fn link_shot_miss_does_not_change_the_book() {
        let utility = line_graph(11);
        let cell = IVec2::new(1, 2);
        assert_eq!(committed_power_link_at_cell(&utility, cell), None);
        let mut shots = ShotPulseBook::default();
        let shot = landed(cell, SHOT_IMPACT_SIDE_FACTION);
        shots.recent.push(shot);
        let mut book = crate::construction::PowerLineDamageBook::default();
        let before = book.clone();
        apply_arrived_shot_impacts_to_power_lines(
            &shots,
            4,
            SHOT_IMPACT_SIDE_FACTION,
            &utility,
            &mut book,
            &mut HashMap::new(),
        );
        assert_eq!(book.segments.len(), before.segments.len());
        assert_eq!(book.cuts_applied, before.cuts_applied);
        assert!((shot_impact_at(&shot, 4, cell) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn link_second_impact_same_tick_does_not_double_apply() {
        let utility = line_graph(11);
        let cell = IVec2::new(1, 0);
        let mut shots = ShotPulseBook::default();
        shots.recent.push(landed(cell, SHOT_IMPACT_SIDE_FACTION));
        let mut book = crate::construction::PowerLineDamageBook::default();
        let mut applied = HashMap::new();
        apply_arrived_shot_impacts_to_power_lines(
            &shots,
            4,
            SHOT_IMPACT_SIDE_FACTION,
            &utility,
            &mut book,
            &mut applied,
        );
        assert!(book.segments.get(&11).is_none());
        apply_arrived_shot_impacts_to_power_lines(
            &shots,
            4,
            SHOT_IMPACT_SIDE_FACTION,
            &utility,
            &mut book,
            &mut applied,
        );
        assert!(book.segments.get(&11).is_none());
    }

    #[test]
    fn link_other_faction_shot_does_not_damage_this_side() {
        let utility = line_graph(11);
        let cell = IVec2::new(1, 0);
        let mut shots = ShotPulseBook::default();
        let shot = landed(cell, 2);
        shots.recent.push(shot);
        let tick = 4;
        assert_eq!(
            book_impact_for_faction(&shots, tick, cell, SHOT_IMPACT_SIDE_FACTION),
            0.0
        );
        let mut book = crate::construction::PowerLineDamageBook::default();
        apply_arrived_shot_impacts_to_power_lines(
            &shots,
            tick,
            SHOT_IMPACT_SIDE_FACTION,
            &utility,
            &mut book,
            &mut HashMap::new(),
        );
        assert!(book.segments.get(&11).is_none());
    }
}
