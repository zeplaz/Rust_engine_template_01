//! RD-10 road condition (PLAN-ROADS-v1): weather, water and blasts wear the roads short of
//! closing them (closing is the flood / debris books' job, D-4; bridge decks fall in `bridges`).
//!
//! - **Mud**: an unpaved road (track, dirt, gravel) in a chunk whose soil is wet past
//!   `offroad.mud_from_moisture` runs at its tier's `wet_speed_mul`, scaled by how wet it is.
//! - **Flood erosion**: water standing on an unpaved road washes it out (`damage` grows per
//!   flooded minute, faster on lower tiers).
//! - **Blast damage**: a blast near a road breaks its surface (`damage` by the blast's falloff at
//!   the road; paved roads take half). Craters deep enough to close it stay with the debris book.
//! - A damaged road is slower (`1 + damage_slow × damage`) until rebuilt: an upgrade order (player
//!   or AI, RD-8) changes the edge's profile and the damage resets. Repair proper stays with D-7.
//!
//! Sole writer of [`RoadConditionBook`]: [`publish_road_condition_system`]. The road traffic
//! publisher folds [`RoadConditionBook::time_mul`] into each edge's travel time.

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::infrastructure::RoadTier;
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId};
use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

/// Condition of one road edge.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoadCondition {
    /// Profile the condition belongs to; a new profile (rebuilt / upgraded) starts fresh.
    pub profile: String,
    /// Travel-time multiplier from mud (1 = dry).
    pub mud_mul: f32,
    /// Surface damage 0..1 (erosion + blasts).
    pub damage: f32,
}

/// Road condition by edge. Sole writer: [`publish_road_condition_system`].
#[derive(Resource, Debug, Default, Clone)]
pub struct RoadConditionBook {
    pub by_edge: BTreeMap<u64, RoadCondition>,
    /// Bumps when some edge's time multiplier moves by a tenth (routes re-plan on real changes).
    pub revision: u64,
    buckets: BTreeMap<u64, u16>,
    blast_seq: u64,
    last_step: u64,
}

impl RoadConditionBook {
    /// Travel-time multiplier on `id` from mud and damage (1 = good road, dry).
    #[must_use]
    pub fn time_mul(&self, id: TransportEdgeId) -> f32 {
        let t = &crate::infrastructure::road_tier_table().condition;
        self.by_edge.get(&id.0).map_or(1.0, |c| c.mud_mul * (1.0 + t.damage_slow * c.damage))
    }

    #[must_use]
    pub fn damage(&self, id: TransportEdgeId) -> f32 {
        self.by_edge.get(&id.0).map_or(0.0, |c| c.damage)
    }

    #[must_use]
    pub fn muddy(&self, id: TransportEdgeId) -> bool {
        self.by_edge.get(&id.0).is_some_and(|c| c.mud_mul > 1.01)
    }
}

/// Time multiplier from mud on `tier` at soil `moisture` (0..1).
#[must_use]
pub fn mud_time_mul(tier: RoadTier, moisture: f32) -> f32 {
    if !tier.is_unpaved() {
        return 1.0;
    }
    let from = crate::infrastructure::road_tier_table().offroad.mud_from_moisture;
    if moisture <= from {
        return 1.0;
    }
    let share = ((moisture - from) / (1.0 - from).max(1e-3)).clamp(0.0, 1.0);
    let speed = 1.0 - (1.0 - tier.spec().wet_speed_mul.clamp(0.05, 1.0)) * share;
    1.0 / speed
}

/// Damage per flooded minute on `tier` (lower tiers wash out faster; paved does not erode).
#[must_use]
pub fn erosion_per_min(tier: RoadTier) -> f32 {
    let base = crate::infrastructure::road_tier_table().condition.flood_erosion_per_min;
    match tier {
        RoadTier::Track => base * 2.0,
        RoadTier::Dirt => base * 1.5,
        RoadTier::Gravel => base,
        _ => 0.0,
    }
}

/// Damage a blast does to a road of `tier` whose nearest point is `dist_m` from the centre.
#[must_use]
pub fn blast_road_damage(tier: RoadTier, dist_m: f32, radius_m: f32, peak: f32) -> f32 {
    let t = &crate::infrastructure::road_tier_table().condition;
    let reach = radius_m * t.blast_reach_mul;
    if dist_m >= reach || reach <= 0.0 {
        return 0.0;
    }
    let falloff = 1.0 - dist_m / reach;
    let hardness = if tier.is_unpaved() { 1.0 } else { 0.5 };
    (falloff * (peak / t.blast_full_damage).min(1.0) * hardness).clamp(0.0, 1.0)
}

fn parse_tile(key: &str) -> Option<IVec2> {
    let (x, z) = key.strip_prefix('t')?.split_once('_')?;
    Some(IVec2::new(x.parse().ok()?, z.parse().ok()?))
}

fn dist_to_segment(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    let d = b - a;
    let t = ((p - a).dot(d) / d.length_squared().max(1e-6)).clamp(0.0, 1.0);
    (a + d * t).distance(p)
}

/// Weather the publisher reads (soil moisture per chunk).
type ChunkMoisture<'w, 's> = Query<'w, 's, (&'static crate::terrain::generation::Chunk, &'static crate::systems::weather::ChunkWeather)>;

/// Recompute mud, erosion and blast damage per road edge every `condition.interval_steps` sim steps.
pub fn publish_road_condition_system(
    clock: Option<Res<crate::systems::sim_control::SimStepClock>>,
    directory: Option<Res<TransportEdgeDirectory>>,
    flood: Option<Res<crate::systems::water::FloodRouteBlockBook>>,
    blasts: Option<Res<crate::construction::BlastCraterBook>>,
    weather: ChunkMoisture,
    mut book: ResMut<RoadConditionBook>,
) {
    let (Some(clock), Some(directory)) = (clock, directory) else {
        return;
    };
    let t = &crate::infrastructure::road_tier_table().condition;
    if clock.steps < book.last_step + t.interval_steps {
        return;
    }
    let minutes = (clock.steps - book.last_step) as f32 * clock.step_micros as f32 / 60e6;
    book.last_step = clock.steps;
    let moisture: BTreeMap<(i32, i32), f32> = weather.iter().map(|(c, w)| ((c.coord.x, c.coord.y), w.soil_moisture)).collect();
    let new_blasts: Vec<(Vec2, f32, f32)> = blasts
        .as_deref()
        .map(|b| b.recent.iter().filter(|(seq, ..)| *seq > book.blast_seq).map(|(_, c, r, d)| (*c, *r, *d)).collect())
        .unwrap_or_default();
    if let Some(b) = blasts.as_deref() {
        book.blast_seq = b.recent.iter().map(|(s, ..)| *s).max().unwrap_or(book.blast_seq).max(book.blast_seq);
    }
    let mut edges: Vec<(&TransportEdgeId, &_)> = directory.by_edge.iter().collect();
    edges.sort_by_key(|(id, _)| id.0);
    let mut next: BTreeMap<u64, RoadCondition> = BTreeMap::new();
    for (id, meta) in edges {
        let (Some(tier), Some(a), Some(b)) = (RoadTier::from_profile_id(&meta.profile), parse_tile(&meta.head_key), parse_tile(&meta.tail_key)) else {
            continue;
        };
        let mut c = match book.by_edge.get(&id.0) {
            Some(old) if old.profile == meta.profile => old.clone(),
            _ => RoadCondition { profile: meta.profile.clone(), mud_mul: 1.0, damage: 0.0 },
        };
        let mid = (a + b) / 2;
        let chunk = mid.div_euclid(IVec2::splat(CHUNK_TILES_SIM as i32));
        c.mud_mul = mud_time_mul(tier, moisture.get(&(chunk.x, chunk.y)).copied().unwrap_or(0.0));
        let flooded = flood.as_deref().is_some_and(|f| f.blocks_key(&meta.head_key) || f.blocks_key(&meta.tail_key));
        if flooded {
            c.damage += erosion_per_min(tier) * minutes;
        }
        let (pa, pb) = ((a.as_vec2() + 0.5) * TILE_METRES_GAMEPLAY, (b.as_vec2() + 0.5) * TILE_METRES_GAMEPLAY);
        for (centre, radius, peak) in &new_blasts {
            c.damage += blast_road_damage(tier, dist_to_segment(pa, pb, *centre), *radius, *peak);
        }
        c.damage = c.damage.clamp(0.0, 1.0);
        if c.mud_mul > 1.0 || c.damage > 0.0 {
            next.insert(id.0, c);
        } else if let Some(old) = book.by_edge.get(&id.0).filter(|o| o.profile == meta.profile && o.damage > 0.0) {
            next.insert(id.0, old.clone());
        }
    }
    let buckets: BTreeMap<u64, u16> = next
        .iter()
        .map(|(id, c)| (*id, ((c.mud_mul * (1.0 + t.damage_slow * c.damage)) * 10.0).round() as u16))
        .filter(|(_, b)| *b > 10)
        .collect();
    if buckets != book.buckets {
        book.buckets = buckets;
        book.revision = book.revision.wrapping_add(1);
    }
    book.by_edge = next;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mud_slows_unpaved_roads_only_and_more_when_wetter() {
        let from = crate::infrastructure::road_tier_table().offroad.mud_from_moisture;
        assert_eq!(mud_time_mul(RoadTier::Dirt, from * 0.5), 1.0, "dry");
        let damp = mud_time_mul(RoadTier::Dirt, (from + 1.0) * 0.5);
        let soaked = mud_time_mul(RoadTier::Dirt, 1.0);
        assert!(damp > 1.0 && soaked > damp, "{damp} {soaked}");
        assert!((soaked - 1.0 / RoadTier::Dirt.spec().wet_speed_mul).abs() < 1e-3, "full mud = the tier's wet speed");
        assert_eq!(mud_time_mul(RoadTier::Paved, 1.0), 1.0, "paved stays fast");
    }

    #[test]
    fn floods_wash_out_unpaved_roads_and_blasts_break_all_roads_less_so_paved() {
        assert!(erosion_per_min(RoadTier::Track) > erosion_per_min(RoadTier::Gravel));
        assert_eq!(erosion_per_min(RoadTier::Paved), 0.0);
        let full = crate::infrastructure::road_tier_table().condition.blast_full_damage;
        let near = blast_road_damage(RoadTier::Gravel, 0.0, 20.0, full);
        assert!((near - 1.0).abs() < 1e-3);
        assert!((blast_road_damage(RoadTier::Paved, 0.0, 20.0, full) - 0.5).abs() < 1e-3);
        assert_eq!(blast_road_damage(RoadTier::Gravel, 1000.0, 20.0, full), 0.0, "out of reach");
    }

    #[test]
    fn a_flooded_dirt_road_erodes_and_an_upgrade_rebuilds_it() {
        use crate::systems::transport::TransportEdgeMeta;
        let mut app = App::new();
        let mut dir = TransportEdgeDirectory::default();
        dir.by_edge.insert(
            TransportEdgeId(7),
            TransportEdgeMeta { profile: RoadTier::Dirt.profile_id().into(), head_key: "t0_0".into(), tail_key: "t4_0".into(), ..Default::default() },
        );
        let mut flood = crate::systems::water::FloodRouteBlockBook::default();
        flood.blocked.insert("t0_0".into());
        app.insert_resource(dir)
            .insert_resource(flood)
            .init_resource::<RoadConditionBook>()
            .insert_resource(crate::systems::sim_control::SimStepClock::default())
            .add_systems(Update, publish_road_condition_system);
        let step_to = |app: &mut App, steps: u64| {
            app.world_mut().resource_mut::<crate::systems::sim_control::SimStepClock>().steps = steps;
            app.update();
        };
        let per_min = 60e6 / crate::systems::sim_control::SimStepClock::default().step_micros as f64;
        step_to(&mut app, (per_min * 10.0) as u64);
        let book = app.world().resource::<RoadConditionBook>();
        let d = book.damage(TransportEdgeId(7));
        assert!(d > 0.0, "ten flooded minutes erode: {d}");
        assert!(book.time_mul(TransportEdgeId(7)) > 1.0);
        // Upgraded to gravel: a new road, no damage; and the water has gone.
        {
            let mut dir = app.world_mut().resource_mut::<TransportEdgeDirectory>();
            dir.by_edge.get_mut(&TransportEdgeId(7)).unwrap().profile = RoadTier::Gravel.profile_id().into();
        }
        app.world_mut().resource_mut::<crate::systems::water::FloodRouteBlockBook>().blocked.clear();
        step_to(&mut app, (per_min * 20.0) as u64);
        assert_eq!(app.world().resource::<RoadConditionBook>().damage(TransportEdgeId(7)), 0.0);
    }
}
