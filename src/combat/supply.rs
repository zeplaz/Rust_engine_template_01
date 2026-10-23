//! **Front supply demand** (PLAN-FRONT-COMBAT-v1 FC-6 groundwork): every theater side owns a front
//! supply-point entity registered as a `SiteRole::Front` logistics site, and publishes its sustainment
//! need through `LogisticsOrderCommand::PublishDemand` (upsert; re-sent only when the targets change).
//!
//! Demand is sized from the notional garrison the front length implies (`supply.garrison_per_km`) times
//! the infantry draw profile; formation-accurate draw replaces it with FC-5/FC-6 proper.

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::balance::{DrawRates, FrontBalance};
use super::formation::{Formation, Posture};
use super::theater::{Theater, TheaterBook, TheaterId};
use crate::economy::logistics::orders::{
    CargoKind, ConsumerKind, LogisticsOrderCommand, LogisticsOrderRequest, LogisticsOrderSite, LogisticsOrderSiteId,
    LogisticsOrdersSnapshotRes, OwnerId, Priority, SiteRole,
};

/// Marks the logistics endpoint of one side of one theater.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct FrontSupplyPoint {
    pub theater: TheaterId,
    pub side: u8,
}

/// Bookkeeping so the system only sends changes.
#[derive(Resource, Debug, Default)]
pub struct FrontSupplyPoints {
    pub entities: BTreeMap<(TheaterId, u8), Entity>,
    pub published: BTreeMap<(TheaterId, u8), Vec<(&'static str, u32)>>,
}

/// Delivered-stock coverage (0..1) of each front supply point, from the logistics snapshot.
#[derive(Resource, Clone, Debug, Default)]
pub struct FrontSupplyCoverage {
    pub by_point: BTreeMap<(TheaterId, u8), f32>,
}

/// Combat draw of the formations of `side` standing in `theater`'s area of responsibility; `None` when
/// there are none (the front then publishes the notional garrison demand instead).
#[must_use]
pub fn formation_draw<'a>(
    theater: &Theater,
    side: u8,
    depth_tiles: f32,
    balance: &FrontBalance,
    formations: impl Iterator<Item = &'a Formation>,
) -> Option<DrawRates> {
    let mut total = DrawRates::default();
    let mut any = false;
    for f in formations.filter(|f| f.faction_slot == side && theater.aor_contains(f.tile, depth_tiles)) {
        any = true;
        let d = balance.unit(f.kind).draw_kg_per_s;
        let w = f.strength
            * if f.engaged || f.posture == Posture::Attack { 1.0 } else { balance.combat.idle_draw_fraction };
        total.ammunition += d.ammunition * w;
        total.fuel += d.fuel * w;
        total.rations += d.rations * w;
        total.engineering += d.engineering * w;
    }
    any.then_some(total)
}

/// `(cargo tag, buffer target)` for a front held by `side` in `theater`; zero-draw classes are skipped.
/// `draw` is the standing formations' draw when any; otherwise a notional garrison sized from front length.
#[must_use]
pub fn front_demand_targets(theater: &Theater, balance: &FrontBalance, draw: Option<DrawRates>) -> Vec<(&'static str, u32)> {
    let p = &balance.supply;
    let d = draw.unwrap_or_else(|| {
        let units = (theater.length_m / 1000.0 * p.garrison_per_km).ceil().max(1.0);
        let i = balance.unit(super::balance::UnitKind::Infantry).draw_kg_per_s;
        DrawRates {
            ammunition: i.ammunition * units,
            fuel: i.fuel * units,
            rations: i.rations * units,
            engineering: i.engineering * units,
        }
    });
    [("ammo", d.ammunition), ("fuel", d.fuel), ("rations", d.rations), ("engineering", d.engineering)]
        .into_iter()
        .filter_map(|(tag, kg_s)| {
            let target = (kg_s * p.buffer_s).ceil();
            (target >= 1.0).then_some((tag, target as u32))
        })
        .collect()
}

/// Mean fill (on hand / buffer target) of each registered front supply point.
pub fn update_front_supply_coverage_system(
    points: Res<FrontSupplyPoints>,
    sites: Query<&LogisticsOrderSiteId>,
    snapshot: Option<Res<LogisticsOrdersSnapshotRes>>,
    mut coverage: ResMut<FrontSupplyCoverage>,
) {
    coverage.by_point.clear();
    let Some(snapshot) = snapshot else { return };
    for (key, e) in &points.entities {
        let (Ok(site), Some(targets)) = (sites.get(*e), points.published.get(key)) else { continue };
        let Some(row) = snapshot.0.site(site.0) else { continue };
        if targets.is_empty() {
            continue;
        }
        let fill: f32 = targets
            .iter()
            .map(|(tag, target)| {
                let have = row.stock.iter().find(|r| r.cargo == *tag).map_or(0, |r| r.on_hand);
                (have as f32 / (*target).max(1) as f32).min(1.0)
            })
            .sum::<f32>()
            / targets.len() as f32;
        coverage.by_point.insert(*key, fill);
    }
}

fn side_position_m(theater: &Theater, side: u8, tile_size_m: f32) -> Vec2 {
    let (sum, n) = theater
        .cells_of(side)
        .fold((Vec2::ZERO, 0.0), |(s, n), c| (s + c.tile.as_vec2() + Vec2::splat(0.5), n + 1.0));
    let centre = if n > 0.0 { sum / n } else { theater.centroid };
    centre * tile_size_m
}

/// Keep supply points in step with the theaters and publish their demand.
pub fn publish_front_supply_demands_system(
    theaters: Res<TheaterBook>,
    balance: Res<FrontBalance>,
    mut points: ResMut<FrontSupplyPoints>,
    sites: Query<&LogisticsOrderSiteId>,
    formations: Query<&Formation>,
    mut commands: Commands,
    requests: Option<ResMut<Messages<LogisticsOrderRequest>>>,
) {
    let Some(mut requests) = requests else { return };
    let mut live: Vec<(TheaterId, u8)> = Vec::new();
    for t in &theaters.theaters {
        for side in [t.pair.0, t.pair.1] {
            let key = (t.id, side);
            live.push(key);
            let pos = side_position_m(t, side, balance.tile_size_m);
            match points.entities.get(&key).copied() {
                None => {
                    let e = commands
                        .spawn((
                            FrontSupplyPoint { theater: t.id, side },
                            Transform::from_xyz(pos.x, 0.0, pos.y),
                            LogisticsOrderSite {
                                name: format!("Front {} / slot {}", t.id.0, side),
                                owner: OwnerId(side),
                                role: SiteRole::Front,
                                position_m: Some([pos.x, pos.y]),
                            },
                        ))
                        .id();
                    points.entities.insert(key, e);
                }
                Some(e) => {
                    if let Ok(mut ec) = commands.get_entity(e) {
                        ec.insert(Transform::from_xyz(pos.x, 0.0, pos.y));
                    }
                }
            }
            let Some(site) = points.entities.get(&key).and_then(|e| sites.get(*e).ok()).map(|s| s.0) else {
                continue;
            };
            let draw = formation_draw(t, side, theaters.depth_tiles, &balance, formations.iter());
            let targets = front_demand_targets(t, &balance, draw);
            if points.published.get(&key) == Some(&targets) {
                continue;
            }
            for &(tag, target) in &targets {
                requests.write(LogisticsOrderRequest(LogisticsOrderCommand::PublishDemand {
                    site,
                    cargo: CargoKind::new(tag),
                    consumer: ConsumerKind::Front,
                    buffer_target: target,
                    reorder_point: ((target as f32) * balance.supply.reorder_fraction).ceil() as u32,
                    draw_rate_per_min: ((target as f32) / balance.supply.buffer_s * 60.0).ceil() as u32,
                    priority: Priority::Normal,
                }));
            }
            points.published.insert(key, targets);
        }
    }
    let stale: Vec<(TheaterId, u8)> = points.entities.keys().copied().filter(|k| !live.contains(k)).collect();
    for key in stale {
        let Some(e) = points.entities.remove(&key) else { continue };
        if let (Ok(site), Some(targets)) = (sites.get(e), points.published.remove(&key)) {
            for (tag, _) in targets {
                requests.write(LogisticsOrderRequest(LogisticsOrderCommand::RetractDemand {
                    site: site.0,
                    cargo: CargoKind::new(tag),
                }));
            }
        }
        points.published.remove(&key);
        if let Ok(mut ec) = commands.get_entity(e) {
            ec.despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::front::{FrontCell, FrontSegment};

    fn theater(len: i32) -> Theater {
        let cells = (0..len)
            .map(|y| FrontCell { tile: IVec2::new(0, y), owner: 0, facing: 1, contested: false, margin: 0.5 })
            .collect();
        let seg = FrontSegment { pair: (0, 1), cells, polylines: vec![], contested_cells: 0, length_m: 0.0 };
        let mut book = TheaterBook::default();
        book.update(&[seg], &FrontBalance::default().theater, 25.0, 1);
        book.theaters.remove(0)
    }

    #[test]
    fn longer_front_demands_more() {
        let b = FrontBalance::default();
        let short = front_demand_targets(&theater(20), &b, None);
        let long = front_demand_targets(&theater(70), &b, None);
        assert!(!short.is_empty());
        assert!(short.iter().zip(&long).all(|(s, l)| l.1 > s.1));
    }

    #[test]
    fn system_spawns_front_site_per_side_and_publishes_once_site_is_registered() {
        let mut app = App::new();
        app.insert_resource(FrontBalance::default())
            .init_resource::<FrontSupplyPoints>()
            .add_message::<LogisticsOrderRequest>()
            .add_systems(Update, publish_front_supply_demands_system);
        let b = FrontBalance::default();
        let seg_cells = theater(20).cells;
        let seg = FrontSegment { pair: (0, 1), cells: seg_cells, polylines: vec![], contested_cells: 0, length_m: 0.0 };
        let mut book = TheaterBook::default();
        book.update(&[seg], &b.theater, b.tile_size_m, 1);
        app.insert_resource(book);
        app.update();
        let entities: Vec<Entity> = app.world().resource::<FrontSupplyPoints>().entities.values().copied().collect();
        assert_eq!(entities.len(), 2);
        let site = app.world().entity(entities[0]).get::<LogisticsOrderSite>().unwrap();
        assert_eq!(site.role, SiteRole::Front);
        // Adapter registers the site: next pass publishes demand for it.
        let sid = crate::economy::logistics::orders::SiteId(1);
        app.world_mut().entity_mut(entities[0]).insert(LogisticsOrderSiteId(sid));
        app.update();
        let pts = app.world().resource::<FrontSupplyPoints>();
        let published = pts.published.values().next().expect("demand published once the site is registered");
        assert!(!published.is_empty());
        assert_eq!(pts.published.len(), 1, "only the registered side publishes");
    }
}
