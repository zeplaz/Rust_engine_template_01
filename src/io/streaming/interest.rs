//! Multi-focus interest orbs merged into chunk streaming windows.

use std::collections::HashSet;

use bevy::prelude::{IVec2, UVec2};

use crate::gui::{OperationalLodZone, WorldRepresentationFrame};

use super::residency::chunk_window_coords;

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

#[must_use]
pub fn priority_for_chunk(coord: IVec2, orbs: &[InterestOrb]) -> u8 {
    orbs.iter()
        .filter(|orb| chunk_window_coords(orb.center, orb.radius_chunks).contains(&coord))
        .map(|orb| orb.priority)
        .max()
        .unwrap_or(0)
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
