//! Corridor works (PLAN-ROADS-v1, RD-6 cost item): a road, railway or bridge the player places
//! has a **material bill** — the line's surface per metre by tier (`road_tiers.ron`
//! `materials_per_m`, rail `rail_materials_per_m`) plus each bridge's `materials_per_m` × length.
//! The bill is taken from the owner's nearest stocked site in the logistics order sim, a truck
//! load per cargo per sim second, and the line's construction progress never runs ahead of the
//! share delivered (a starved line stops, like a starved building site). No money: materials only.
//!
//! - [`register_corridor_works_system`]: sole writer of [`CorridorWorksBook`] rows (new charged
//!   lines → bill; removed lines → dropped).
//! - [`supply_corridor_works_system`]: reads the published orders snapshot, sends
//!   `ConsumeStock` requests to the order sim, books the delivered units (sole writer of
//!   `delivered`).
//! - [`publish_corridor_works_caps_system`]: per transport edge of an unfinished line, the progress
//!   cap = scarcest delivered share ([`CorridorWorksCaps`], read by the corridor construction tick).
//!
//! Lines the world starts with (scenario fixtures, settlement skeletons) are not charged.

use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;

use super::bridges::BridgeBook;
use super::corridor_network::{CorridorKind, CorridorNetwork};
use crate::economy::logistics::orders::model::{CargoKind, OwnerId, SiteId};
use crate::economy::logistics::orders::plugin::{LogisticsOrderRequest, LogisticsOrdersSnapshotRes};
use crate::economy::logistics::orders::sim::LogisticsOrderCommand;
use crate::systems::sim_control::SimStepClock;
use crate::systems::transport::{TransportEdgeDirectory, TransportEdgeId};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Units of one cargo a works site takes per sim second (about a truck load).
const SUPPLY_UNITS_PER_S: u32 = 40;

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
    /// Site the last delivery came from (dump / readout).
    pub last_source: Option<SiteId>,
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
    /// Units taken from stock so far (dump).
    pub taken_total: f32,
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
            },
        );
        changed = true;
    }
    if changed {
        book.revision = book.revision.wrapping_add(1);
    }
}

/// Takes the bills from the owner's nearest stocked sites, a truck load per cargo per sim second.
pub fn supply_corridor_works_system(
    clock: Option<Res<SimStepClock>>,
    snapshot: Option<Res<LogisticsOrdersSnapshotRes>>,
    mut book: ResMut<CorridorWorksBook>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
    mut requested: Local<HashMap<(SiteId, String), u32>>,
) {
    let (Some(clock), Some(snapshot)) = (clock, snapshot) else {
        return;
    };
    // Requests sent since the snapshot last refreshed are not in its stock yet.
    if snapshot.is_changed() {
        requested.clear();
    }
    let due = clock.due;
    if due == 0 || book.works.values().all(CorridorWorks::supplied) {
        return;
    }
    let budget = SUPPLY_UNITS_PER_S * due;
    let mut taken = 0.0;
    let mut changed = false;
    for w in book.works.values_mut().filter(|w| !w.supplied()) {
        let mid_m = w.mid * TILE_METRES_GAMEPLAY;
        for m in w.bill.iter_mut().filter(|m| m.delivered + 1e-3 < m.needed) {
            let want = ((m.needed - m.delivered).ceil() as u32).min(budget);
            // Nearest site of this owner with free stock of the cargo.
            let pick = snapshot
                .0
                .sites
                .iter()
                .filter(|s| s.owner == w.owner)
                .filter_map(|s| {
                    let row = s.stock.iter().find(|r| r.cargo == m.cargo)?;
                    let asked = requested.get(&(s.id, m.cargo.clone())).copied().unwrap_or(0);
                    let free = row.on_hand.saturating_sub(row.reserved_out).saturating_sub(asked);
                    (free > 0).then(|| (s.id, free, Vec2::from(s.world_xz).distance(mid_m)))
                })
                .min_by(|a, b| a.2.total_cmp(&b.2));
            let Some((site, free, _)) = pick else {
                continue;
            };
            let amount = want.min(free);
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::ConsumeStock {
                site,
                cargo: CargoKind::new(m.cargo.clone()),
                amount,
            }));
            *requested.entry((site, m.cargo.clone())).or_insert(0) += amount;
            m.delivered = (m.delivered + amount as f32).min(m.needed);
            w.last_source = Some(site);
            taken += amount as f32;
            changed = true;
        }
    }
    if changed {
        book.taken_total += taken;
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
        };
        assert!((w.delivered_ratio() - 0.2).abs() < 1e-5);
        assert!(!w.supplied());
        assert_eq!(w.short().len(), 2);
    }
}
