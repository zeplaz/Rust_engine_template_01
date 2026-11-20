//! DW-2 — a destroyed structure becomes rubble in the ground, not a building.
//!
//! PLAN-DESTROY-PRESENTATION-v1 Tier 1 (sim truth) via the DW-1 ground delta: when the demolish
//! funnel announces a destroyed standing site ([`SiteDestroyed`], cause-gated: an undone
//! placement or a cancelled build leaves nothing) its footprint gets a rubble heap holding the
//! collapsed structure volume (footprint area x height x the material's solid fraction). A dam
//! column that fails (integrity at collapse) drops its own rubble at its foot. The DEBRIS-FX
//! particle burst covers the moment; the heap is what stays. Water routes around the heap through
//! the bed follow, and DW-5 can wash it downstream.
//!
//! A building that was burning when it fell leaves smouldering rubble:
//! [`ignite_rubble_of_burning_sites_system`] sets the rubble disk alight through the fire spine
//! (`SimEffectQueue` IgniteCells); a blast on a burning site also throws embers beyond it.

use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

use super::destruction::{DestroyCause, SiteDestroyed, SiteMaterial};
use crate::strategic::{ConstructionSite, SiteFootprint};
use crate::systems::fire::SiteFireState;
use crate::terrain::ground_deform::{GroundDeformOp, GroundDeformQueue};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Solid share of a collapsed brick structure's envelope (rooms and voids collapse; rubble bulks).
/// Other materials scale from [`SiteMaterial::rubble_solid_fraction`].
pub const RUBBLE_SOLID_FRACTION: f32 = 0.3;

/// Rubble disk of a footprint: centre (m), outer radius (m, spills a little past the footprint)
/// and footprint area (m^2).
fn rubble_disk(tiles: &[IVec2]) -> Option<(Vec2, f32, f32)> {
    if tiles.is_empty() {
        return None;
    }
    let n = tiles.len() as f32;
    let sum = tiles.iter().fold(Vec2::ZERO, |a, t| a + t.as_vec2() + Vec2::splat(0.5));
    let centre_m = sum / n * TILE_METRES_GAMEPLAY;
    let area_m2 = n * TILE_METRES_GAMEPLAY * TILE_METRES_GAMEPLAY;
    let outer_radius_m = (area_m2 / std::f32::consts::PI).sqrt() * 1.3 + TILE_METRES_GAMEPLAY * 0.5;
    Some((centre_m, outer_radius_m, area_m2))
}

/// Rubble heap for a footprint of `tiles` that stood `height_m` tall, built of `material`.
#[must_use]
pub fn rubble_heap_op(tiles: &[IVec2], height_m: f32, material: SiteMaterial) -> Option<GroundDeformOp> {
    if height_m <= 0.0 {
        return None;
    }
    let (centre_m, outer_radius_m, area_m2) = rubble_disk(tiles)?;
    Some(GroundDeformOp::Heap {
        centre_m,
        inner_radius_m: 0.0,
        outer_radius_m,
        volume_m3: area_m2 * height_m * material.rubble_solid_fraction(),
    })
}

/// Standing height (m) a site would collapse from; dams use their crest height.
fn standing_height_m(site: &ConstructionSite, fp: &SiteFootprint, dam: Option<&super::DamBody>) -> f32 {
    if let Some(d) = dam {
        return if d.barrier_installed { d.crest_m.max(0.0) } else { 0.0 };
    }
    super::water_obstacles::water_obstacle_height_m(site.archetype, site.phase, fp.tiles.len()).unwrap_or(0.0)
}

/// Remember what every standing site would collapse from; when the funnel announces one destroyed
/// with a wreckage-leaving cause, heap its rubble on its footprint. Dams keep their own account:
/// the remembered height already discounts the columns that failed (and heaped) earlier.
pub fn rubble_from_destroyed_sites_system(
    sites: Query<(Entity, &ConstructionSite, &SiteFootprint, Option<&super::DamBody>)>,
    mut destroyed: MessageReader<SiteDestroyed>,
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
    for ev in destroyed.read() {
        // Read the memory before pruning: the site is already despawned when its message lands.
        let remembered = standing.remove(&ev.entity).map(|(_, h)| h);
        if !ev.leaves_wreckage() {
            continue;
        }
        let height = remembered.unwrap_or(ev.height_m);
        if let Some(op) = rubble_heap_op(&ev.tiles, height, ev.material) {
            queue.push(op);
        }
    }
    standing.retain(|e, _| seen.contains(e));
}

/// Ember ring thrown by a blast on a burning site reaches this many disk radii out.
const EMBER_REACH: f32 = 2.5;

/// Tiles of the rubble disk of `tiles` (and, for `embers`, a sparse ring beyond it) with the
/// ignition strength each gets.
fn burning_rubble_tiles(tiles: &[IVec2], embers: bool) -> Vec<(IVec2, f32)> {
    let Some((centre_m, radius_m, _)) = rubble_disk(tiles) else {
        return Vec::new();
    };
    let reach = if embers { radius_m * EMBER_REACH } else { radius_m };
    let r_tiles = (reach / TILE_METRES_GAMEPLAY).ceil() as i32;
    let centre_tile = (centre_m / TILE_METRES_GAMEPLAY).floor().as_ivec2();
    let mut out = Vec::new();
    for dz in -r_tiles..=r_tiles {
        for dx in -r_tiles..=r_tiles {
            let t = centre_tile + IVec2::new(dx, dz);
            if t.x < 0 || t.y < 0 {
                continue;
            }
            let c = (t.as_vec2() + Vec2::splat(0.5)) * TILE_METRES_GAMEPLAY;
            let d = c.distance(centre_m);
            if d <= radius_m {
                out.push((t, 0.6));
            } else if embers && d <= reach && (t.x.wrapping_mul(31) ^ t.y.wrapping_mul(17)) % 3 == 0 {
                // Embers land on a deterministic third of the ring cells, weaker than the rubble.
                out.push((t, 0.3));
            }
        }
    }
    out
}

/// A burning building that collapses leaves smouldering rubble. Narrow: reads the destroyed-site
/// message plus a memory of which sites were burning, writes the fire spine's effect queue only.
pub fn ignite_rubble_of_burning_sites_system(
    sites: Query<(Entity, &SiteFireState)>,
    mut destroyed: MessageReader<SiteDestroyed>,
    raster: Option<Res<crate::strategic::StrategicRasterConfig>>,
    mut effects: Option<ResMut<crate::sim::effects::SimEffectQueue>>,
    mut burning: Local<HashSet<Entity>>,
) {
    for (e, state) in &sites {
        if state.burning() {
            burning.insert(e);
        } else {
            burning.remove(&e);
        }
    }
    let cells_per_chunk = raster
        .as_ref()
        .map(|r| r.cells_per_chunk)
        .unwrap_or_else(|| crate::strategic::StrategicRasterConfig::default().cells_per_chunk);
    for ev in destroyed.read() {
        let was_burning = burning.remove(&ev.entity);
        if !was_burning || !ev.leaves_wreckage() {
            continue;
        }
        let Some(effects) = effects.as_mut() else {
            continue;
        };
        let embers = matches!(ev.cause, DestroyCause::Blast { .. });
        let cells: Vec<_> = burning_rubble_tiles(&ev.tiles, embers)
            .into_iter()
            .map(|(t, spark)| {
                crate::scenario::ignite::ignite_cell_from_world_tile(
                    crate::scenario::scenario_steps::ScenarioWorldTile {
                        tile_x: t.x as u32,
                        tile_z: t.y as u32,
                    },
                    cells_per_chunk,
                    spark,
                )
            })
            .collect();
        if cells.is_empty() {
            continue;
        }
        crate::scenario::ignite::enqueue_scenario_ignite_sim_effect(
            effects,
            crate::sim::effects::SimEffectSource::Construction,
            format!("rubble-burn-{},{}", ev.origin.x, ev.origin.z),
            None,
            cells,
        );
    }
    burning.retain(|e| sites.contains(*e));
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
                if let Some(op) = rubble_heap_op(&foot, body.crest_m, SiteMaterial::Earth) {
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
        let Some(GroundDeformOp::Heap { volume_m3, outer_radius_m, inner_radius_m, .. }) =
            rubble_heap_op(&tiles, 10.0, SiteMaterial::Brick)
        else {
            panic!("heap expected");
        };
        let area = 2.0 * TILE_METRES_GAMEPLAY * TILE_METRES_GAMEPLAY;
        assert!((volume_m3 - area * 10.0 * RUBBLE_SOLID_FRACTION).abs() < 1e-3);
        assert_eq!(inner_radius_m, 0.0);
        assert!(outer_radius_m > TILE_METRES_GAMEPLAY);
        assert!(rubble_heap_op(&tiles, 0.0, SiteMaterial::Brick).is_none());
    }

    #[test]
    fn rubble_bulk_follows_the_material() {
        let tiles = [IVec2::new(3, 3)];
        let vol = |m| match rubble_heap_op(&tiles, 8.0, m) {
            Some(GroundDeformOp::Heap { volume_m3, .. }) => volume_m3,
            _ => panic!("heap expected"),
        };
        assert!(vol(SiteMaterial::Concrete) > vol(SiteMaterial::Brick));
        assert!(vol(SiteMaterial::Brick) > vol(SiteMaterial::Timber));
    }

    fn rubble_app() -> App {
        let mut app = App::new();
        app.add_message::<SiteDestroyed>()
            .init_resource::<GroundDeformQueue>()
            .init_resource::<crate::sim::effects::SimEffectQueue>()
            .add_systems(
                Update,
                (rubble_from_destroyed_sites_system, ignite_rubble_of_burning_sites_system),
            );
        app
    }

    fn spawn_site(app: &mut App, tile: IVec2, fire: Option<SiteFireState>) -> Entity {
        use crate::strategic::{SiteArchetype, SiteConstructionPhase};
        let mut e = app.world_mut().spawn((
            ConstructionSite {
                site_id: 1,
                owner: Entity::PLACEHOLDER,
                archetype: SiteArchetype::CivilHousing,
                phase: SiteConstructionPhase::Operational,
                operational_readiness: 1.0,
            },
            SiteFootprint { tiles: vec![tile, tile + IVec2::X], layer: crate::strategic::LayerType::Surface },
        ));
        if let Some(f) = fire {
            e.insert(f);
        }
        e.id()
    }

    fn announce(app: &mut App, entity: Entity, cause: DestroyCause) {
        app.world_mut().resource_mut::<Messages<SiteDestroyed>>().write(SiteDestroyed {
            entity,
            origin: crate::strategic::BuildSiteTile { x: 10, z: 10 },
            tiles: vec![IVec2::new(10, 10), IVec2::new(11, 10)],
            height_m: 6.0,
            material: SiteMaterial::Timber,
            cause,
            phase: crate::strategic::SiteConstructionPhase::Operational,
        });
    }

    fn heaps(app: &App) -> usize {
        app.world().resource::<GroundDeformQueue>().len()
    }

    /// Undo of a placement (and a scenario teardown) is a plain despawn: no message, no rubble.
    #[test]
    fn undo_of_a_placement_leaves_no_rubble() {
        let mut app = rubble_app();
        let e = spawn_site(&mut app, IVec2::new(10, 10), None);
        app.update();
        app.world_mut().entity_mut(e).despawn();
        app.update();
        app.update();
        assert_eq!(heaps(&app), 0);
        assert!(app.world().resource::<crate::sim::effects::SimEffectQueue>().pending.is_empty());
    }

    #[test]
    fn destroyed_standing_site_heaps_rubble_once() {
        let mut app = rubble_app();
        let e = spawn_site(&mut app, IVec2::new(10, 10), None);
        app.update();
        app.world_mut().entity_mut(e).despawn();
        announce(&mut app, e, DestroyCause::Fire);
        app.update();
        app.update();
        assert_eq!(heaps(&app), 1);
    }

    /// A building that was burning when it fell leaves rubble that burns; one that was not does not.
    #[test]
    fn a_burning_buildings_rubble_burns() {
        use crate::systems::fire::SiteFireStage;
        let mut app = rubble_app();
        let burning = SiteFireState { stage: SiteFireStage::Deteriorating, ..Default::default() };
        let hot = spawn_site(&mut app, IVec2::new(10, 10), Some(burning));
        let cold = spawn_site(&mut app, IVec2::new(30, 30), Some(SiteFireState::default()));
        app.update();
        app.world_mut().entity_mut(cold).despawn();
        announce(&mut app, cold, DestroyCause::Structural);
        app.update();
        assert!(app.world().resource::<crate::sim::effects::SimEffectQueue>().pending.is_empty());
        app.world_mut().entity_mut(hot).despawn();
        announce(&mut app, hot, DestroyCause::Fire);
        app.update();
        let q = app.world().resource::<crate::sim::effects::SimEffectQueue>();
        assert_eq!(q.pending.len(), 1);
        match &q.pending[0].kind {
            crate::sim::effects::SimEffectKind::IgniteCells { cells } => assert!(cells.len() >= 4),
            k => panic!("unexpected {k:?}"),
        }
    }

    /// A blast on a burning site also throws embers past the rubble disk.
    #[test]
    fn blast_on_a_burning_site_spreads_embers() {
        let tiles = [IVec2::new(10, 10), IVec2::new(11, 10)];
        let disk = burning_rubble_tiles(&tiles, false);
        let with_embers = burning_rubble_tiles(&tiles, true);
        assert!(with_embers.len() > disk.len());
        assert!(with_embers.iter().any(|(_, s)| *s < 0.5), "embers are weaker than the rubble fire");
    }
}
