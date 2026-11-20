//! D-3 / DEBRIS-FX — dust that lingers and fades: one book, two sources, one render path.
//!
//! * **Blast:** every detonation throws a dust column over its crater (sim record
//!   [`crate::construction::BlastCraterBook`] `recent`).
//! * **Collapse:** every site that leaves the world (all demolish paths: player, blast, fire,
//!   flood, structural) and every dam column that fails ([`crate::construction::DamBody`]
//!   column integrity at the collapse bar). Both are read from sim components; the old
//!   one-frame push inside the demolish funnel is gone (the particle frame is rebuilt every
//!   frame, so it lasted a single frame).
//!
//! Presentation appends smoke-class rows into the shared [`WorldFireParticleFrame`] after the
//! fire build, through the same `smoke_puff` animation the fire's lingering smoke uses. Rows
//! follow the particle convention of `particle_anim`: `xy` = tile position (tile units), height
//! drawn as an oblique lift carried in `ember_class_radius_smoke.w`, so the voxel and tactical
//! views draw it with no new pipeline.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;

use super::particle_anim::{smoke_puff, FireAnim};
use crate::render::WorldFireParticleFrame;

/// How long blast dust lingers (s, wall clock).
pub const BLAST_DUST_S: f32 = 12.0;
/// How long collapse dust lingers (s, wall clock): a few seconds, shorter than a blast column.
pub const COLLAPSE_DUST_S: f32 = 6.0;
/// Puffs per blast (spread over the crater disk).
const PUFFS_PER_BLAST: u32 = 18;
/// Puffs per collapsed site / failed dam column.
const PUFFS_PER_COLLAPSE: u32 = 12;
/// Collapses can come in bursts (a dam span, a chain of burning sites): wider live window.
const MAX_LIVE_DUST: usize = 48;

#[derive(Clone, Copy, Debug)]
struct DustBlast {
    centre_tiles: Vec2,
    radius_tiles: f32,
    born_s: f32,
    seed: u32,
    life_s: f32,
    puffs: u32,
}

/// Recent blasts still dusting (render-side memory).
#[derive(Resource, Default, Debug)]
pub struct BlastDustBook {
    live: Vec<DustBlast>,
    cursor: u64,
}

impl BlastDustBook {
    fn push(&mut self, d: DustBlast) {
        self.live.push(d);
        if self.live.len() > MAX_LIVE_DUST {
            self.live.remove(0);
        }
    }
}

/// Dust disk (centre in tile units, radius in tiles) over a footprint: the tile-centre centroid
/// and half the larger bounding extent. `None` for an empty footprint.
#[must_use]
pub fn footprint_dust_disk(tiles: &[IVec2]) -> Option<(Vec2, f32)> {
    let first = *tiles.first()?;
    let (mut lo, mut hi) = (first, first);
    for t in tiles {
        lo = lo.min(*t);
        hi = hi.max(*t);
    }
    let centre = (lo.as_vec2() + hi.as_vec2()) * 0.5 + Vec2::splat(0.5);
    let extent = (hi - lo + IVec2::ONE).max_element() as f32;
    Some((centre, (extent * 0.5).max(1.0)))
}

fn collapse_seed(centre: Vec2) -> u32 {
    (centre.x.floor() as i32 as u32).wrapping_mul(73_856_093)
        ^ (centre.y.floor() as i32 as u32).wrapping_mul(19_349_663)
}

/// Collapse dust source: footprints that vanish and dam columns that fail. Reads sim components
/// only; the next [`append_blast_dust_particles`] draws what lands in the book.
pub fn detect_collapse_dust(
    time: Res<Time>,
    born: Query<(Entity, &crate::strategic::SiteFootprint), Added<crate::strategic::SiteFootprint>>,
    mut gone: RemovedComponents<crate::strategic::SiteFootprint>,
    dams: Query<(Entity, &crate::construction::DamBody, &crate::strategic::SiteFootprint)>,
    mut book: ResMut<BlastDustBook>,
    mut known: Local<HashMap<Entity, Vec<IVec2>>>,
    mut failed: Local<HashSet<(Entity, usize)>>,
) {
    let now = time.elapsed_secs();
    for (e, fp) in &born {
        known.insert(e, fp.tiles.clone());
    }
    for e in gone.read() {
        let Some(tiles) = known.remove(&e) else { continue };
        if let Some((centre_tiles, radius_tiles)) = footprint_dust_disk(&tiles) {
            book.push(DustBlast {
                centre_tiles,
                radius_tiles,
                born_s: now,
                seed: collapse_seed(centre_tiles),
                life_s: COLLAPSE_DUST_S,
                puffs: PUFFS_PER_COLLAPSE,
            });
        }
    }
    failed.retain(|(e, _)| dams.contains(*e));
    let bar = crate::construction::DESTRUCTIBLE_COLLAPSE_INTEGRITY;
    for (e, body, fp) in &dams {
        for (i, (tile, integrity)) in fp.tiles.iter().zip(&body.column_integrity).enumerate() {
            if *integrity <= bar && failed.insert((e, i)) {
                let centre_tiles = tile.as_vec2() + Vec2::splat(0.5);
                book.push(DustBlast {
                    centre_tiles,
                    radius_tiles: 1.5,
                    born_s: now,
                    seed: collapse_seed(centre_tiles) ^ i as u32,
                    life_s: COLLAPSE_DUST_S,
                    puffs: PUFFS_PER_COLLAPSE,
                });
            }
        }
    }
}

/// Pick up new detonations and append their dust rows (after the fire particle build).
pub fn append_blast_dust_particles(
    time: Res<Time>,
    craters: Option<Res<crate::construction::BlastCraterBook>>,
    mut book: ResMut<BlastDustBook>,
    mut frame: ResMut<WorldFireParticleFrame>,
) {
    let now = time.elapsed_secs();
    if let Some(c) = craters.as_deref() {
        let from = book.cursor;
        for &(seq, centre_m, radius_m, _) in c.recent.iter().filter(|(s, ..)| *s > from) {
            book.cursor = book.cursor.max(seq);
            let m = crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;
            book.push(DustBlast {
                centre_tiles: centre_m / m,
                radius_tiles: (radius_m / m).max(1.0),
                born_s: now,
                seed: (seq as u32).wrapping_mul(2_654_435_761),
                life_s: BLAST_DUST_S,
                puffs: PUFFS_PER_BLAST,
            });
        }
    }
    book.live.retain(|b| now - b.born_s < b.life_s);
    if book.live.is_empty() {
        return;
    }
    let anim = FireAnim { time_s: now, wind_mps: Vec2::ZERO };
    let cap = frame.gpu_capacity.max(frame.instances.len());
    let mut pushed = false;
    for b in &book.live {
        let age = (now - b.born_s) / b.life_s;
        // Thick at once, thinning out; the column climbs as it ages.
        let density = (1.0 - age).powf(1.5) * 1.4;
        for k in 0..b.puffs {
            if frame.instances.len() >= cap {
                break;
            }
            let a = (k as f32 * 2.399_963) + (b.seed % 360) as f32;
            let r = b.radius_tiles * ((k as f32 + 0.5) / b.puffs as f32).sqrt();
            let xy = b.centre_tiles + Vec2::new(a.cos(), a.sin()) * r;
            let lift = 2.0 + 30.0 * age;
            if let Some(row) = smoke_puff(xy, lift, density, &anim, b.seed ^ k, k, 1.0 + 1.5 * age) {
                frame.instances.push(row);
                pushed = true;
            }
        }
    }
    if pushed {
        frame.snapshot_stamp = frame.snapshot_stamp.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::fire_vfx::particle_anim::particle_ground_and_height_m;
    use crate::strategic::{LayerType, SiteFootprint};
    use std::time::Duration;

    fn app() -> App {
        let mut app = App::new();
        app.init_resource::<Time>()
            .init_resource::<BlastDustBook>()
            .init_resource::<WorldFireParticleFrame>()
            .add_systems(
                Update,
                (detect_collapse_dust, append_blast_dust_particles).chain(),
            );
        app
    }

    fn at(app: &mut App, secs: f32) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_to(Duration::from_secs_f32(secs));
        app.world_mut()
            .resource_mut::<WorldFireParticleFrame>()
            .instances
            .clear();
        app.update();
    }

    fn footprint() -> SiteFootprint {
        SiteFootprint {
            tiles: vec![IVec2::new(40, 60), IVec2::new(41, 60), IVec2::new(40, 61), IVec2::new(41, 61)],
            layer: LayerType::Surface,
        }
    }

    #[test]
    fn footprint_disk_is_centroid_and_half_extent() {
        let (c, r) = footprint_dust_disk(&footprint().tiles).unwrap();
        assert_eq!(c, Vec2::new(41.0, 61.0));
        assert_eq!(r, 1.0);
        assert!(footprint_dust_disk(&[]).is_none());
    }

    #[test]
    fn demolished_site_dust_lands_on_its_tiles_and_lingers_seconds() {
        let mut app = app();
        let e = app.world_mut().spawn(footprint()).id();
        at(&mut app, 1.0);
        assert!(app.world().resource::<BlastDustBook>().live.is_empty(), "no dust while standing");
        app.world_mut().entity_mut(e).despawn();
        at(&mut app, 1.1);
        assert_eq!(app.world().resource::<BlastDustBook>().live.len(), 1);
        // Persists across later frames (the frame is rebuilt each frame; the book re-appends).
        for t in [2.0, 4.0] {
            at(&mut app, t);
            let frame = app.world().resource::<WorldFireParticleFrame>();
            assert!(!frame.instances.is_empty(), "dust still drawn at t={t}");
            for row in &frame.instances {
                let (ground, h) = particle_ground_and_height_m(row);
                assert!(h >= 0.0);
                // tile position, not (x, 1.2, -y): within the footprint's neighbourhood.
                assert!((38.0..44.0).contains(&ground.x) && (58.0..64.0).contains(&ground.y), "{ground:?}");
                assert!(row.ember_class_radius_smoke.w > 0.0, "height rides in the lift field");
            }
        }
        at(&mut app, 1.1 + COLLAPSE_DUST_S + 0.5);
        assert!(app.world().resource::<BlastDustBook>().live.is_empty());
        assert!(app.world().resource::<WorldFireParticleFrame>().instances.is_empty());
    }

    #[test]
    fn failed_dam_column_dusts_once() {
        let mut app = app();
        let mut body = crate::construction::DamBody::default();
        body.column_integrity = vec![1.0, 1.0, 1.0, 1.0];
        let e = app.world_mut().spawn((footprint(), body)).id();
        at(&mut app, 1.0);
        assert!(app.world().resource::<BlastDustBook>().live.is_empty());
        app.world_mut()
            .entity_mut(e)
            .get_mut::<crate::construction::DamBody>()
            .unwrap()
            .column_integrity[2] = 0.1;
        at(&mut app, 1.1);
        at(&mut app, 1.2);
        let book = app.world().resource::<BlastDustBook>();
        assert_eq!(book.live.len(), 1, "one column failed, one dust source, not re-fired per frame");
        assert_eq!(book.live[0].centre_tiles, Vec2::new(40.5, 61.5));
    }
}
