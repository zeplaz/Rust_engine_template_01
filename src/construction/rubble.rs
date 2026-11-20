//! DW-2 — a destroyed structure becomes rubble in the ground, not a building.
//!
//! PLAN-DESTROY-PRESENTATION-v1 Tier 1 (sim truth) via the DW-1 ground delta: when a standing
//! site leaves the world (every terminal path ends in `execute_demolish_at_tile`'s despawn) its
//! footprint gets a rubble heap holding the collapsed structure volume (footprint area x height
//! x bulking). A dam column that fails (integrity at collapse) drops its own rubble at its foot.
//! The DEBRIS-FX particle burst covers the moment; the heap is what stays. Water routes around
//! the heap through the bed follow, and DW-5 can wash it downstream.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::strategic::{ConstructionSite, SiteFootprint};
use crate::terrain::ground_deform::{GroundDeformOp, GroundDeformQueue};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Solid share of a collapsed structure's envelope (rooms and voids collapse; rubble bulks).
pub const RUBBLE_SOLID_FRACTION: f32 = 0.3;

/// Rubble heap for a footprint of `tiles` that stood `height_m` tall: centre (m), outer radius
/// (m, spills a little past the footprint) and volume (m^3).
#[must_use]
pub fn rubble_heap_op(tiles: &[IVec2], height_m: f32) -> Option<GroundDeformOp> {
    if tiles.is_empty() || height_m <= 0.0 {
        return None;
    }
    let n = tiles.len() as f32;
    let sum = tiles.iter().fold(Vec2::ZERO, |a, t| a + t.as_vec2() + Vec2::splat(0.5));
    let centre_m = sum / n * TILE_METRES_GAMEPLAY;
    let area_m2 = n * TILE_METRES_GAMEPLAY * TILE_METRES_GAMEPLAY;
    let outer_radius_m = (area_m2 / std::f32::consts::PI).sqrt() * 1.3 + TILE_METRES_GAMEPLAY * 0.5;
    Some(GroundDeformOp::Heap {
        centre_m,
        inner_radius_m: 0.0,
        outer_radius_m,
        volume_m3: area_m2 * height_m * RUBBLE_SOLID_FRACTION,
    })
}

/// Standing height (m) a site would collapse from; dams use their crest height.
fn standing_height_m(site: &ConstructionSite, fp: &SiteFootprint, dam: Option<&super::DamBody>) -> f32 {
    if let Some(d) = dam {
        return if d.barrier_installed { d.crest_m.max(0.0) } else { 0.0 };
    }
    super::water_obstacles::water_obstacle_height_m(site.archetype, site.phase, fp.tiles.len()).unwrap_or(0.0)
}

/// Remember every standing site; when one is gone, heap its rubble on its footprint.
pub fn rubble_from_removed_sites_system(
    sites: Query<(Entity, &ConstructionSite, &SiteFootprint, Option<&super::DamBody>)>,
    mut queue: ResMut<GroundDeformQueue>,
    mut standing: Local<HashMap<Entity, (Vec<IVec2>, f32)>>,
) {
    let mut seen = std::collections::HashSet::new();
    for (e, site, fp, dam) in &sites {
        seen.insert(e);
        let h = standing_height_m(site, fp, dam);
        if h > 0.0 {
            // A dam's failed columns already dropped their share; heap only what still stands.
            let h_left = dam.map_or(h, |d| {
                let n = d.column_integrity.len().max(1) as f32;
                let alive = d.column_integrity.iter().map(|i| super::dam_column_crest_fraction(*i)).sum::<f32>();
                h * if d.column_integrity.is_empty() { 1.0 } else { alive / n }
            });
            standing.insert(e, (fp.tiles.clone(), h_left));
        } else {
            standing.remove(&e);
        }
    }
    standing.retain(|e, (tiles, h)| {
        if seen.contains(e) {
            return true;
        }
        if let Some(op) = rubble_heap_op(tiles, *h) {
            queue.push(op);
        }
        false
    });
}

/// A dam column that fails drops its rubble just downstream of its tile (the face it fell off).
pub fn rubble_from_failed_dam_columns_system(
    dams: Query<(Entity, &super::DamBody, &SiteFootprint)>,
    mut queue: ResMut<GroundDeformQueue>,
    mut failed: Local<HashMap<Entity, Vec<bool>>>,
) {
    failed.retain(|e, _| dams.contains(*e));
    for (e, body, fp) in &dams {
        if !body.barrier_installed || body.column_integrity.len() != fp.tiles.len() {
            continue;
        }
        let seen = failed.entry(e).or_insert_with(|| vec![false; fp.tiles.len()]);
        if seen.len() != fp.tiles.len() {
            *seen = vec![false; fp.tiles.len()];
        }
        for ((t, i), was) in fp.tiles.iter().zip(&body.column_integrity).zip(seen.iter_mut()) {
            let gone = super::dam_column_crest_fraction(*i) <= 0.0;
            if gone && !*was {
                // Downstream = -z in the dam scene valley (river runs +z -> -z).
                let foot = [*t - IVec2::Y];
                if let Some(op) = rubble_heap_op(&foot, body.crest_m) {
                    queue.push(op);
                }
            }
            *was = gone;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dw2_rubble_heap_holds_the_collapsed_volume() {
        let tiles = [IVec2::new(10, 10), IVec2::new(11, 10)];
        let Some(GroundDeformOp::Heap { volume_m3, outer_radius_m, inner_radius_m, .. }) = rubble_heap_op(&tiles, 10.0) else {
            panic!("heap expected");
        };
        let area = 2.0 * TILE_METRES_GAMEPLAY * TILE_METRES_GAMEPLAY;
        assert!((volume_m3 - area * 10.0 * RUBBLE_SOLID_FRACTION).abs() < 1e-3);
        assert_eq!(inner_radius_m, 0.0);
        assert!(outer_radius_m > TILE_METRES_GAMEPLAY);
        assert!(rubble_heap_op(&tiles, 0.0).is_none());
    }
}
