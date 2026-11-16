//! Multi-focus interest orbs merged into chunk streaming windows.
//!
//! **ECS-11** (gea4v2 17.8.8 *Registering Interest in Events* · aig3 11.2.2 *Event Casting*):
//! an orb is a subscription; work fans out only to chunks some orb registered
//! ([`registered_priority_for_chunk`] / [`retain_registered_chunks`]) instead of broadcasting
//! to every chunk. LEGACY keywords: interest · subscribe · event filter.
//! Hysteresis-retained chunks (IO-02, `R+H`) are a deliberate residency superset and are
//! merged *after* this gate in `streaming::mod`, not through it.

use std::collections::HashSet;

use bevy::prelude::{IVec2, UVec2};

use crate::gui::{OperationalLodZone, WorldRepresentationFrame};

use super::residency::{chunk_window_coords, chunk_within_load_radius};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InterestOrbKind {
    PlayerFocus,
    LodZone,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InterestOrb {
    pub center: IVec2,
    pub radius_chunks: i32,
    pub priority: u8,
    pub kind: InterestOrbKind,
}

#[must_use]
pub fn primary_interest_orb(world: &WorldRepresentationFrame) -> InterestOrb {
    InterestOrb {
        center: world.focus_chunk,
        radius_chunks: world.interest_radius_chunks.max(1),
        priority: 255,
        kind: InterestOrbKind::PlayerFocus,
    }
}

#[must_use]
pub fn interest_orbs_from_lod_zones(
    zones: &[OperationalLodZone],
    chunk_tiles: UVec2,
) -> Vec<InterestOrb> {
    let cw = chunk_tiles.x.max(1) as f32;
    let ch = chunk_tiles.y.max(1) as f32;
    zones
        .iter()
        .map(|zone| {
            let center = IVec2::new(
                (zone.center.x / cw).floor() as i32,
                (zone.center.z / ch).floor() as i32,
            );
            let radius_chunks = (zone.radius / cw.max(ch)).ceil().max(1.0) as i32;
            InterestOrb {
                center,
                radius_chunks,
                priority: (zone.priority.clamp(0.0, 1.0) * 255.0) as u8,
                kind: InterestOrbKind::LodZone,
            }
        })
        .collect()
}

/// Chunk `coord` lies inside this orb's registered window (square, Chebyshev — same set as
/// [`chunk_window_coords`], without allocating the window).
#[must_use]
pub fn orb_registers_chunk(orb: &InterestOrb, coord: IVec2) -> bool {
    chunk_within_load_radius(coord, orb.center, orb.radius_chunks)
}

/// ECS-11 (gea4v2 17.8.8 / aig3 11.2.2): the **only** interest gate for streaming fan-out.
///
/// `Some(priority)` = at least one orb registered interest in `coord` (highest priority wins);
/// `None` = nobody subscribed, so the chunk receives no load / priority / ghost-band work.
/// LEGACY keywords: interest · subscribe · event filter (`events_message_passing`, `event_managers`).
#[must_use]
pub fn registered_priority_for_chunk(coord: IVec2, orbs: &[InterestOrb]) -> Option<u8> {
    orbs.iter()
        .filter(|orb| orb_registers_chunk(orb, coord))
        .map(|orb| orb.priority)
        .max()
}

/// `true` when any orb registered interest in `coord`.
#[must_use]
pub fn chunk_has_registered_interest(coord: IVec2, orbs: &[InterestOrb]) -> bool {
    orbs.iter().any(|orb| orb_registers_chunk(orb, coord))
}

/// Drop every candidate nobody registered interest in (order preserved).
#[must_use]
pub fn retain_registered_chunks(candidates: &[IVec2], orbs: &[InterestOrb]) -> Vec<IVec2> {
    candidates
        .iter()
        .copied()
        .filter(|&coord| chunk_has_registered_interest(coord, orbs))
        .collect()
}

/// Priority for scheduling; unregistered chunks collapse to `0` (see [`registered_priority_for_chunk`]).
#[must_use]
pub fn priority_for_chunk(coord: IVec2, orbs: &[InterestOrb]) -> u8 {
    registered_priority_for_chunk(coord, orbs).unwrap_or(0)
}

#[must_use]
pub fn merge_interest_orbs_deduped(orbs: &[InterestOrb]) -> Vec<InterestOrb> {
    let mut by_center: std::collections::HashMap<IVec2, InterestOrb> = std::collections::HashMap::new();
    for orb in orbs {
        by_center
            .entry(orb.center)
            .and_modify(|existing| {
                if orb.priority > existing.priority {
                    *existing = *orb;
                }
            })
            .or_insert(*orb);
    }
    let mut merged: Vec<InterestOrb> = by_center.into_values().collect();
    merged.sort_by_key(|orb| (orb.center.y, orb.center.x));
    merged
}

#[must_use]
pub fn merge_interest_chunk_coords(orbs: &[InterestOrb]) -> Vec<IVec2> {
    let mut merged = HashSet::new();
    for orb in orbs {
        merged.extend(chunk_window_coords(orb.center, orb.radius_chunks));
    }
    let mut coords: Vec<IVec2> = merged.into_iter().collect();
    coords.sort_by_key(|coord| (coord.y, coord.x));
    coords
}

#[must_use]
pub fn merge_interest_chunk_coords_with_ghost_bands(orbs: &[InterestOrb]) -> Vec<IVec2> {
    let mut merged = HashSet::new();
    for orb in orbs {
        for center in chunk_window_coords(orb.center, orb.radius_chunks) {
            merged.extend(super::residency::ghost_band_seed_coords(center));
        }
    }
    let mut coords: Vec<IVec2> = merged.into_iter().collect();
    coords.sort_by_key(|coord| (coord.y, coord.x));
    coords
}

#[must_use]
pub fn highest_priority_orb(orbs: &[InterestOrb]) -> Option<InterestOrb> {
    orbs.iter().copied().max_by_key(|orb| orb.priority)
}

/// Keep the nearest chunks to focus when merged interest becomes too large.
#[must_use]
pub fn cap_chunk_coords_by_focus(mut coords: Vec<IVec2>, focus: IVec2, max: usize) -> Vec<IVec2> {
    if max == 0 || coords.len() <= max {
        return coords;
    }
    coords.sort_by(|a, b| {
        let da = (*a - focus).as_vec2().length_squared();
        let db = (*b - focus).as_vec2().length_squared();
        da.total_cmp(&db)
            .then_with(|| a.y.cmp(&b.y))
            .then_with(|| a.x.cmp(&b.x))
    });
    coords.truncate(max);
    coords
}

/// Stable signature for a sorted interest chunk set (scheduler diff / diagnostics).
#[must_use]
pub fn interest_chunk_set_signature(coords: &[IVec2]) -> u64 {
    let mut h: u64 = coords.len() as u64;
    for c in coords {
        h = h.wrapping_mul(31).wrapping_add(c.x as u64);
        h = h.wrapping_mul(31).wrapping_add(c.y as u64);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::Vec3;
    use crate::gui::{LodZoneClass, LodZoneSource, WorldLodBand};

    #[test]
    fn merge_interest_orbs_deduped_keeps_highest_priority() {
        let orbs = vec![
            InterestOrb {
                center: IVec2::ZERO,
                radius_chunks: 1,
                priority: 10,
                kind: InterestOrbKind::LodZone,
            },
            InterestOrb {
                center: IVec2::ZERO,
                radius_chunks: 2,
                priority: 40,
                kind: InterestOrbKind::LodZone,
            },
        ];
        let merged = merge_interest_orbs_deduped(&orbs);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].priority, 40);
    }

    #[test]
    fn merge_interest_chunk_coords_unions_orb_windows() {
        let orbs = vec![
            InterestOrb {
                center: IVec2::ZERO,
                radius_chunks: 0,
                priority: 10,
                kind: InterestOrbKind::LodZone,
            },
            InterestOrb {
                center: IVec2::new(2, 0),
                radius_chunks: 0,
                priority: 20,
                kind: InterestOrbKind::LodZone,
            },
        ];
        let merged = merge_interest_chunk_coords(&orbs);
        assert_eq!(merged.len(), 2);
    }

    fn orb(center: IVec2, radius: i32, priority: u8) -> InterestOrb {
        InterestOrb {
            center,
            radius_chunks: radius,
            priority,
            kind: InterestOrbKind::LodZone,
        }
    }

    #[test]
    fn ecs11_uninterested_chunk_is_skipped() {
        let orbs = vec![orb(IVec2::ZERO, 1, 10)];
        let far = IVec2::new(9, 9);
        assert_eq!(registered_priority_for_chunk(far, &orbs), None);
        assert!(!chunk_has_registered_interest(far, &orbs));
        assert_eq!(priority_for_chunk(far, &orbs), 0);
        let kept = retain_registered_chunks(&[IVec2::ZERO, far, IVec2::new(1, 1)], &orbs);
        assert_eq!(kept, vec![IVec2::ZERO, IVec2::new(1, 1)]);
    }

    #[test]
    fn ecs11_no_orbs_means_no_fan_out() {
        let candidates = [IVec2::ZERO, IVec2::new(1, 0)];
        assert!(retain_registered_chunks(&candidates, &[]).is_empty());
        assert_eq!(registered_priority_for_chunk(IVec2::ZERO, &[]), None);
    }

    #[test]
    fn ecs11_negative_radius_registers_nothing() {
        let orbs = vec![orb(IVec2::ZERO, -1, 200)];
        assert_eq!(registered_priority_for_chunk(IVec2::ZERO, &orbs), None);
        assert!(merge_interest_chunk_coords(&orbs).is_empty());
    }

    #[test]
    fn ecs11_highest_registered_priority_wins_and_zero_priority_is_still_registered() {
        let orbs = vec![orb(IVec2::ZERO, 2, 5), orb(IVec2::new(1, 0), 0, 90), orb(IVec2::new(8, 8), 0, 0)];
        assert_eq!(registered_priority_for_chunk(IVec2::new(1, 0), &orbs), Some(90));
        assert_eq!(registered_priority_for_chunk(IVec2::new(-2, 2), &orbs), Some(5));
        // priority 0 is a registration, not absence — `priority_for_chunk` alone cannot tell.
        assert_eq!(registered_priority_for_chunk(IVec2::new(8, 8), &orbs), Some(0));
        assert_eq!(registered_priority_for_chunk(IVec2::new(4, 4), &orbs), None);
    }

    #[test]
    fn ecs11_gate_matches_window_membership_for_every_orb() {
        for radius in -1..=3 {
            let o = orb(IVec2::new(2, -3), radius, 7);
            let window: HashSet<IVec2> = chunk_window_coords(o.center, o.radius_chunks).into_iter().collect();
            for y in -8..=8 {
                for x in -8..=8 {
                    let c = IVec2::new(x, y);
                    assert_eq!(orb_registers_chunk(&o, c), window.contains(&c), "r={radius} c={c}");
                }
            }
        }
    }

    #[test]
    fn ecs11_merged_streaming_coords_are_all_registered() {
        let orbs = vec![orb(IVec2::ZERO, 2, 10), orb(IVec2::new(10, 10), 1, 20)];
        let merged = merge_interest_chunk_coords(&orbs);
        assert_eq!(retain_registered_chunks(&merged, &orbs), merged);
    }

    #[test]
    fn interest_chunk_set_signature_stable() {
        let a = interest_chunk_set_signature(&[IVec2::new(1, 2), IVec2::new(3, 4)]);
        let b = interest_chunk_set_signature(&[IVec2::new(1, 2), IVec2::new(3, 4)]);
        assert_eq!(a, b);
    }

    #[test]
    fn cap_chunk_coords_by_focus_limits_to_nearest() {
        let coords = vec![
            IVec2::new(100, 0),
            IVec2::new(2, 0),
            IVec2::new(1, 0),
            IVec2::new(50, 0),
        ];
        let capped = cap_chunk_coords_by_focus(coords, IVec2::ZERO, 2);
        assert_eq!(capped, vec![IVec2::new(1, 0), IVec2::new(2, 0)]);
    }

    #[test]
    fn lod_zone_orb_uses_world_center_in_chunk_space() {
        let zones = vec![OperationalLodZone {
            zone_id: 1,
            class: LodZoneClass::Mission,
            center: Vec3::new(64.0, 0.0, 32.0),
            radius: 32.0,
            band: WorldLodBand::Operational,
            priority: 0.5,
            source: LodZoneSource::Settlement,
        }];
        let orbs = interest_orbs_from_lod_zones(&zones, UVec2::new(32, 32));
        assert_eq!(orbs[0].center, IVec2::new(2, 1));
    }
}
