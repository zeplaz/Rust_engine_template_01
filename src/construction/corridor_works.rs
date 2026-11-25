//! Corridor works (PLAN-ROADS-v1, RD-6 cost item): a road, railway or bridge the player places
//! has a **material bill** — the line's surface per metre by tier (`road_tiers.ron`
//! `materials_per_m`, rail `rail_materials_per_m`) plus each bridge's `materials_per_m` × length.
//! Trucks haul the bill: each unsupplied line has a **works site** in the logistics order sim (at
//! the middle of the line); one-off runs bring each cargo from the owner's nearest stocked site,
//! and what has unloaded at the works site is what is delivered. The line's construction progress
//! never runs ahead of the share delivered (a starved line stops, like a starved building site).
//! No money: materials only.
//!
//! - [`register_corridor_works_system`]: sole writer of [`CorridorWorksBook`] rows (new charged
//!   lines → bill; removed lines → dropped).
//! - [`sync_corridor_works_sites_system`]: one [`CorridorWorksSite`] endpoint per open line
//!   (spawned with a [`LogisticsOrderSite`], despawned when the line is supplied or removed).
//! - [`supply_corridor_works_system`]: reads the published orders snapshot, books what has unloaded
//!   at each works site (sole writer of `delivered` / `site` / `closed`) and sends one-off runs for
//!   what is neither delivered nor already on the way ([`plan_works_runs`]).
//! - [`publish_corridor_works_caps_system`]: per transport edge of an unfinished line, the progress
//!   cap = scarcest delivered share ([`CorridorWorksCaps`], read by the corridor construction tick).
//!
//! Lines the world starts with (scenario fixtures, settlement skeletons) are not charged.

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;

use super::bridges::BridgeBook;
use super::corridor_network::{CorridorKind, CorridorNetwork};
use crate::economy::logistics::orders::model::{CargoKind, OwnerId, Priority, SiteId};
use crate::economy::logistics::orders::plugin::{
    LogisticsOrderRequest, LogisticsOrderSite, LogisticsOrderSiteId, LogisticsOrdersSnapshotRes,
};
use crate::economy::logistics::orders::sim::LogisticsOrderCommand;
use crate::economy::logistics::orders::snapshot::{LogisticsOrdersSnapshot, SiteRole};
use crate::systems::sim_control::SimStepClock;
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Largest single run sent to a works site (several trucks share it; keeps a source from being
/// emptied by one order while other lines wait).
const MAX_RUN_UNITS: u32 = 400;

/// One cargo of a bill.
#[derive(Clone, Debug, PartialEq)]
pub struct WorksMaterial {
    pub cargo: String,
    pub needed: f32,
    pub delivered: f32,
}

/// The works of one line.
#[derive(Clone, Debug, PartialEq)]
pub struct CorridorWorks {
    pub segment_id: u32,
    pub owner: OwnerId,
    /// Middle of the line (tile units): where the nearest source is measured from.
    pub mid: Vec2,
    pub length_m: f32,
    pub bill: Vec<WorksMaterial>,
    /// Site the last run was sent from (dump / readout).
    pub last_source: Option<SiteId>,
    /// This line's works site in the order sim (set once registered).
    pub site: Option<SiteId>,
    /// Supplied and closed: open runs cancelled, works site released.
    pub closed: bool,
}

impl CorridorWorks {
    /// Scarcest delivered share (1 = fully supplied, or nothing to supply).
    #[must_use]
    pub fn delivered_ratio(&self) -> f32 {
        self.bill
            .iter()
            .filter(|m| m.needed > 0.0)
            .map(|m| (m.delivered / m.needed).clamp(0.0, 1.0))
            .fold(1.0, f32::min)
    }

    #[must_use]
    pub fn supplied(&self) -> bool {
        self.delivered_ratio() >= 1.0 - 1e-4
    }

    /// Cargos still short, with how many units.
    #[must_use]
    pub fn short(&self) -> Vec<(String, f32)> {
        self.bill
            .iter()
            .filter(|m| m.delivered + 1e-3 < m.needed)
            .map(|m| (m.cargo.clone(), m.needed - m.delivered))
            .collect()
    }
}

/// Material bills of placed lines. Rows: [`register_corridor_works_system`]; deliveries:
/// [`supply_corridor_works_system`].
#[derive(Resource, Debug, Default, Clone)]
pub struct CorridorWorksBook {
    pub works: BTreeMap<u32, CorridorWorks>,
    pub revision: u64,
    /// Units ordered onto trucks so far (dump).
    pub ordered_total: f32,
}

/// Progress cap per transport edge of an unfinished line (missing ⇒ no cap).
#[derive(Resource, Debug, Default, Clone)]
pub struct CorridorWorksCaps {
    pub by_edge: HashMap<TransportEdgeId, f32>,
}

/// Materials per metre of line for a corridor kind (cargo, units).
#[must_use]
pub fn line_materials_per_m(kind: &CorridorKind) -> Vec<(String, f32)> {
    let table = crate::infrastructure::road_tier_table();
    match kind {
        CorridorKind::Road { tier, .. } => tier.spec().materials_per_m.clone(),
        CorridorKind::Rail { .. } => table.rail_materials_per_m.clone(),
    }
}

/// Adds `per_m × length_m` of each cargo to `bill`.
pub fn add_to_bill(bill: &mut Vec<(String, f32)>, per_m: &[(String, f32)], length_m: f32) {
    for (cargo, q) in per_m {
        let units = q * length_m;
        if units <= 0.0 {
            continue;
        }
        match bill.iter_mut().find(|(c, _)| c == cargo) {
            Some((_, n)) => *n += units,
            None => bill.push((cargo.clone(), units)),
        }
    }
}

fn centre(t: crate::strategic::BuildSiteTile) -> Vec2 {
    Vec2::new(t.x as f32 + 0.5, t.z as f32 + 0.5)
}

/// New charged lines get their bill (line surface + their bridges); removed lines drop theirs.
/// Runs after bridge registration so a line's bridges are known.
pub fn register_corridor_works_system(
    network: Res<CorridorNetwork>,
    bridges: Res<BridgeBook>,
    mut book: ResMut<CorridorWorksBook>,
) {
    if !network.is_changed() && !bridges.is_changed() {
        return;
    }
    let before = book.works.len();
    book.works.retain(|id, _| network.segment(*id).is_some());
    let mut changed = book.works.len() != before;
    let fresh: Vec<&super::corridor_network::CorridorSegment> =
        network.segments.iter().filter(|s| s.charged && !book.works.contains_key(&s.id)).collect();
    for seg in fresh {
        let pts: Vec<Vec2> = seg.nodes.iter().copied().map(centre).collect();
        let length_m: f32 = pts.windows(2).map(|w| w[0].distance(w[1])).sum::<f32>() * TILE_METRES_GAMEPLAY;
        let mut bill = Vec::new();
        add_to_bill(&mut bill, &line_materials_per_m(&seg.kind), length_m);
        for b in bridges.bridges.iter().filter(|b| b.segment_id == seg.id) {
            add_to_bill(&mut bill, &b.bridge_type.spec().materials_per_m, b.length_m());
        }
        let mid = pts.get(pts.len() / 2).copied().unwrap_or_default();
        book.works.insert(
            seg.id,
            CorridorWorks {
                segment_id: seg.id,
                owner: OwnerId::PLAYER,
                mid,
                length_m,
                bill: bill
                    .into_iter()
                    .map(|(cargo, needed)| WorksMaterial { cargo, needed: needed.ceil(), delivered: 0.0 })
                    .collect(),
                last_source: None,
                site: None,
                closed: false,
            },
        );
        changed = true;
    }
    if changed {
        book.revision = book.revision.wrapping_add(1);
    }
}

/// The order-sim endpoint of one line's works (where its material is unloaded).
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct CorridorWorksSite {
    pub segment_id: u32,
}

/// One works site per open line: spawned at the middle of the line, despawned when the line is
/// supplied (its material went into the line) or removed.
pub fn sync_corridor_works_sites_system(
    mut commands: Commands,
    book: Res<CorridorWorksBook>,
    sites: Query<(Entity, &CorridorWorksSite)>,
) {
    if !book.is_changed() {
        return;
    }
    let mut have = Vec::new();
    for (entity, site) in &sites {
        match book.works.get(&site.segment_id) {
            Some(w) if !w.closed => have.push(site.segment_id),
            _ => commands.entity(entity).despawn(),
        }
    }
    for w in book.works.values().filter(|w| !w.closed && !have.contains(&w.segment_id)) {
        let mid_m = w.mid * TILE_METRES_GAMEPLAY;
        commands.spawn((
            Name::new(format!("Corridor works {}", w.segment_id)),
            CorridorWorksSite { segment_id: w.segment_id },
            LogisticsOrderSite {
                name: format!("Works {}", w.segment_id),
                owner: w.owner,
                role: SiteRole::Construction,
                position_m: Some([mid_m.x, mid_m.y]),
            },
        ));
    }
}

/// A one-off run to send to a works site.
#[derive(Clone, Debug, PartialEq)]
pub struct WorksRun {
    pub source: SiteId,
    pub cargo: String,
    pub quantity: u32,
}

/// Units of `cargo` on hand at `site`.
#[must_use]
fn on_hand(snap: &LogisticsOrdersSnapshot, site: SiteId, cargo: &str) -> u32 {
    snap.site(site)
        .and_then(|s| s.stock.iter().find(|r| r.cargo == cargo))
        .map_or(0, |r| r.on_hand)
}

/// Units of `cargo` still on the way to `site` (open orders, loaded or not).
#[must_use]
fn on_the_way(snap: &LogisticsOrdersSnapshot, site: SiteId, cargo: &str) -> u32 {
    snap.orders
        .iter()
        .filter(|o| o.sink == site && o.cargo == cargo && !o.status.is_terminal())
        .map(|o| o.remaining)
        .sum()
}

/// Runs to send for one line: for each cargo, what is neither delivered, on the way, nor already
/// asked for since the snapshot refreshed, from the owner's nearest site with free stock (never
/// another works site). `asked` holds units requested per `(source, cargo)` and per
/// `(works site, cargo)` since the last snapshot.
#[must_use]
pub fn plan_works_runs(
    w: &CorridorWorks,
    site: SiteId,
    snap: &LogisticsOrdersSnapshot,
    works_sites: &[SiteId],
    asked: &HashMap<(SiteId, String), u32>,
) -> Vec<WorksRun> {
    let mid_m = w.mid * TILE_METRES_GAMEPLAY;
    let mut runs = Vec::new();
    for m in w.bill.iter().filter(|m| m.delivered + 1e-3 < m.needed) {
        let covered = m.delivered.ceil() as u32
            + on_the_way(snap, site, &m.cargo)
            + asked.get(&(site, m.cargo.clone())).copied().unwrap_or(0);
        let short = (m.needed.ceil() as u32).saturating_sub(covered);
        if short == 0 {
            continue;
        }
        let pick = snap
            .sites
            .iter()
            .filter(|s| s.owner == w.owner && !works_sites.contains(&s.id))
            .filter_map(|s| {
                let row = s.stock.iter().find(|r| r.cargo == m.cargo)?;
                let a = asked.get(&(s.id, m.cargo.clone())).copied().unwrap_or(0);
                let free = row.on_hand.saturating_sub(row.reserved_out).saturating_sub(a);
                (free > 0).then(|| (s.id, free, Vec2::from(s.world_xz).distance(mid_m)))
            })
            .min_by(|a, b| a.2.total_cmp(&b.2).then(a.0.cmp(&b.0)));
        if let Some((source, free, _)) = pick {
            runs.push(WorksRun { source, cargo: m.cargo.clone(), quantity: short.min(free).min(MAX_RUN_UNITS) });
        }
    }
    runs
}

/// Books what has unloaded at each works site and sends one-off runs for the rest. A supplied line
/// cancels its leftover runs and closes (its works site is then released).
pub fn supply_corridor_works_system(
    clock: Option<Res<SimStepClock>>,
    snapshot: Option<Res<LogisticsOrdersSnapshotRes>>,
    mut book: ResMut<CorridorWorksBook>,
    sites: Query<(&CorridorWorksSite, &LogisticsOrderSiteId)>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
    mut asked: Local<HashMap<(SiteId, String), u32>>,
) {
    let (Some(clock), Some(snapshot)) = (clock, snapshot) else {
        return;
    };
    // Runs sent since the snapshot last refreshed are not in it yet.
    if snapshot.is_changed() {
        asked.clear();
    }
    if clock.due == 0 || book.works.values().all(|w| w.closed) {
        return;
    }
    let snap = &snapshot.0;
    let site_of: HashMap<u32, SiteId> = sites.iter().map(|(w, id)| (w.segment_id, id.0)).collect();
    let works_sites: Vec<SiteId> = site_of.values().copied().collect();
    let mut ordered = 0.0;
    let mut changed = false;
    for w in book.works.values_mut().filter(|w| !w.closed) {
        let Some(&site) = site_of.get(&w.segment_id) else {
            continue;
        };
        if w.site != Some(site) {
            w.site = Some(site);
            changed = true;
        }
        // Delivered = unloaded at the works site (nothing is drawn from it until the line closes).
        for m in &mut w.bill {
            let here = (on_hand(snap, site, &m.cargo) as f32).min(m.needed);
            if here > m.delivered {
                m.delivered = here;
                changed = true;
            }
        }
        if w.supplied() {
            for o in snap.orders.iter().filter(|o| o.sink == site && !o.status.is_terminal()) {
                requests.write(LogisticsOrderRequest(LogisticsOrderCommand::Cancel { order: o.id }));
            }
            w.closed = true;
            changed = true;
            continue;
        }
        for run in plan_works_runs(w, site, snap, &works_sites, &asked) {
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::CreateOneOffRun {
                owner: w.owner,
                source: run.source,
                sink: site,
                cargo: CargoKind::new(run.cargo.clone()),
                quantity: run.quantity,
                priority: Priority::Normal,
                vehicle_cap: None,
            }));
            *asked.entry((run.source, run.cargo.clone())).or_insert(0) += run.quantity;
            *asked.entry((site, run.cargo)).or_insert(0) += run.quantity;
            w.last_source = Some(run.source);
            ordered += run.quantity as f32;
            changed = true;
        }
    }
    if changed {
        book.ordered_total += ordered;
        book.revision = book.revision.wrapping_add(1);
    }
}

/// Progress caps for the transport edges of unfinished lines.
pub fn publish_corridor_works_caps_system(
    book: Res<CorridorWorksBook>,
    network: Res<CorridorNetwork>,
    directory: Res<TransportEdgeDirectory>,
    mut caps: ResMut<CorridorWorksCaps>,
) {
    if !book.is_changed() && !directory.is_changed() {
        return;
    }
    let mut by_edge = HashMap::new();
    for w in book.works.values().filter(|w| !w.supplied()) {
        let Some(seg) = network.segment(w.segment_id) else {
            continue;
        };
        let ratio = w.delivered_ratio();
        for (a, b) in seg.spans() {
            if let Some(e) = super::corridor_transport::find_edge_id_for_segment(&directory, a, b) {
                by_edge.insert(e, ratio);
            }
        }
    }
    if caps.by_edge != by_edge {
        caps.by_edge = by_edge;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::RoadTier;

    #[test]
    fn a_line_bills_its_surface_and_its_bridges() {
        let mut bill = Vec::new();
        let gravel = line_materials_per_m(&CorridorKind::Road { tier: RoadTier::Gravel, lanes: 2 });
        assert!(gravel.iter().any(|(c, q)| c == "Gravel" && *q > 0.0), "{gravel:?}");
        add_to_bill(&mut bill, &gravel, 500.0);
        let bailey = crate::infrastructure::bridge_types::BridgeType::Bailey.spec().materials_per_m.clone();
        add_to_bill(&mut bill, &bailey, 60.0);
        assert!(bill.iter().any(|(c, q)| c == "Steel" && *q > 0.0));
        let track = line_materials_per_m(&CorridorKind::Road { tier: RoadTier::Track, lanes: 1 });
        assert!(track.is_empty(), "a track is just driven in");
    }

    #[test]
    fn a_line_is_capped_at_its_scarcest_delivered_share() {
        let w = CorridorWorks {
            segment_id: 1,
            owner: OwnerId::PLAYER,
            mid: Vec2::ZERO,
            length_m: 100.0,
            bill: vec![
                WorksMaterial { cargo: "Gravel".into(), needed: 100.0, delivered: 80.0 },
                WorksMaterial { cargo: "Steel".into(), needed: 10.0, delivered: 2.0 },
            ],
            last_source: None,
            site: None,
            closed: false,
        };
        assert!((w.delivered_ratio() - 0.2).abs() < 1e-5);
        assert!(!w.supplied());
        assert_eq!(w.short().len(), 2);
    }

    use crate::economy::logistics::orders::snapshot::{SiteRow, StockRow};

    fn site(id: u32, x_tiles: f32, gravel: u32) -> SiteRow {
        SiteRow {
            id: SiteId(id),
            name: format!("s{id}"),
            owner: OwnerId::PLAYER,
            role: SiteRole::Depot,
            world_xz: [x_tiles * TILE_METRES_GAMEPLAY, 0.0],
            tile: [x_tiles as i32, 0],
            stock: vec![StockRow { cargo: "Gravel".into(), unit: "t", on_hand: gravel, reserved_out: 0, inbound: 0 }],
        }
    }

    fn gravel_works(needed: f32) -> CorridorWorks {
        CorridorWorks {
            segment_id: 7,
            owner: OwnerId::PLAYER,
            mid: Vec2::new(10.0, 0.0),
            length_m: 100.0,
            bill: vec![WorksMaterial { cargo: "Gravel".into(), needed, delivered: 0.0 }],
            last_source: None,
            site: None,
            closed: false,
        }
    }

    #[test]
    fn works_runs_come_from_the_nearest_stock_never_another_works_site() {
        let snap = LogisticsOrdersSnapshot {
            sites: vec![site(1, 1000.0, 1000), site(2, 12.0, 50), site(99, 10.0, 500)],
            ..Default::default()
        };
        let works = SiteId(50);
        let runs = plan_works_runs(&gravel_works(120.0), works, &snap, &[SiteId(99), works], &HashMap::new());
        assert_eq!(runs, vec![WorksRun { source: SiteId(2), cargo: "Gravel".into(), quantity: 50 }]);
    }

    #[test]
    fn works_runs_skip_what_is_already_asked_for() {
        let snap = LogisticsOrdersSnapshot { sites: vec![site(1, 1000.0, 1000), site(2, 12.0, 50)], ..Default::default() };
        let works = SiteId(50);
        let mut asked = HashMap::new();
        asked.insert((works, "Gravel".to_string()), 50);
        asked.insert((SiteId(2), "Gravel".to_string()), 50);
        let runs = plan_works_runs(&gravel_works(120.0), works, &snap, &[works], &asked);
        assert_eq!(runs, vec![WorksRun { source: SiteId(1), cargo: "Gravel".into(), quantity: 70 }]);
        asked.insert((works, "Gravel".to_string()), 120);
        assert!(plan_works_runs(&gravel_works(120.0), works, &snap, &[works], &asked).is_empty());
    }

    /// A charged gravel road is supplied by trucks from the HQ: nothing counts as delivered before
    /// a truck unloads at the works site, and the line closes once its bill has arrived.
    #[test]
    fn trucks_haul_a_roads_bill_to_its_works_site() {
        use bevy::asset::AssetPlugin;
        use crate::economy::logistics::orders::{LogisticsOrdersPlugin, LogisticsOrdersSnapshotRes};
        use crate::infrastructure::RoadTier;
        use crate::strategic::BuildSiteTile;
        use crate::systems::sim_control::{SimControlState, SimTick, SimTimeMicros};

        fn frame(app: &mut App) {
            app.world_mut().resource_mut::<SimTick>().0 += 1;
            app.world_mut().resource_mut::<SimTimeMicros>().0 += 1_000_000;
            app.update();
        }

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_resource::<crate::terrain::generation::world_generator_enhanced::WorldGenParams>()
            .add_plugins(crate::systems::terrain::MaterialUnificationPlugin)
            .init_resource::<SimControlState>()
            .init_resource::<SimTick>()
            .init_resource::<SimTimeMicros>()
            .add_plugins(LogisticsOrdersPlugin)
            .init_resource::<CorridorNetwork>()
            .init_resource::<BridgeBook>()
            .init_resource::<CorridorWorksBook>()
            .add_systems(
                Update,
                (register_corridor_works_system, sync_corridor_works_sites_system, supply_corridor_works_system).chain(),
            );
        let mut start = crate::ai::faction::FactionStartConfig::load_default().expect("shipped start config");
        start.factions.retain(|f| f.slot == 0);
        start.factions[0].hq = crate::ai::faction::start::HqPlacement::Tile { x: 0, z: 0 };
        app.insert_resource(start);
        app.add_plugins(crate::ai::faction::FactionStartPlugin);
        for _ in 0..3 {
            frame(&mut app);
        }
        // A short gravel road a few tiles from the HQ, placed to order.
        {
            let mut net = app.world_mut().resource_mut::<CorridorNetwork>();
            let id = net
                .add_segment(
                    CorridorKind::Road { tier: RoadTier::Gravel, lanes: 2 },
                    BuildSiteTile { x: 6, z: 2 },
                    BuildSiteTile { x: 14, z: 2 },
                )
                .expect("segment");
            net.segments.iter_mut().find(|s| s.id == id).expect("seg").charged = true;
        }
        frame(&mut app);
        frame(&mut app);
        let (needed, early) = {
            let book = app.world().resource::<CorridorWorksBook>();
            let w = book.works.values().next().expect("bill registered");
            (w.bill.iter().map(|m| m.needed).sum::<f32>(), w.bill.iter().map(|m| m.delivered).sum::<f32>())
        };
        assert!(needed > 0.0 && needed < 700.0, "a short gravel road fits the HQ stock: {needed}");
        assert_eq!(early, 0.0, "nothing is delivered before a truck arrives");
        let mut closed = false;
        for _ in 0..600 {
            frame(&mut app);
            if app.world().resource::<CorridorWorksBook>().works.values().all(|w| w.closed) {
                closed = true;
                break;
            }
        }
        let book = app.world().resource::<CorridorWorksBook>();
        let w = book.works.values().next().expect("works");
        assert!(closed && w.supplied(), "the bill arrives by truck: {:?}", w.bill);
        assert!(book.ordered_total >= needed, "runs were sent for the bill");
        let site = w.site.expect("works site registered");
        let snap = &app.world().resource::<LogisticsOrdersSnapshotRes>().0;
        let hauled: u32 = snap.orders.iter().filter(|o| o.sink == site).map(|o| o.delivered).sum();
        assert!(hauled as f32 >= needed - 1.0, "it came by truck runs to the works site: {hauled} of {needed}");
    }
}
