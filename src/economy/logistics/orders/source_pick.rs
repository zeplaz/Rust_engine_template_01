//! Open-goal source pick (NAV-09): one shortest-path expansion from the sink, then best eligible site.
//!
//! Replaces per-candidate path/distance queries with a single Dijkstra from the demand site.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, BTreeMap};

use super::model::SiteId;
use super::sim::TravelModel;
use super::snapshot::{DemandRow, LogisticsOrdersSnapshot, SiteRole, SiteRow};

/// Undirected travel edges between registered sites (ticks at 1 m/s via [`TravelModel::travel_ticks`]).
pub type SiteTravelAdjacency = BTreeMap<SiteId, Vec<(SiteId, u32)>>;

#[must_use]
pub fn build_site_adjacency(site_ids: &[SiteId], travel: &dyn TravelModel) -> SiteTravelAdjacency {
    let mut adj = SiteTravelAdjacency::new();
    for &a in site_ids {
        for &b in site_ids {
            if a == b {
                continue;
            }
            let Some(t) = travel.travel_ticks(a, b, 1.0) else {
                continue;
            };
            if t == 0 {
                continue;
            }
            adj.entry(a).or_default().push((b, t));
        }
    }
    adj
}

/// Shortest travel ticks from `start` along [`SiteTravelAdjacency`] (missing site = unreachable).
#[must_use]
pub fn dijkstra_from(start: SiteId, adj: &SiteTravelAdjacency) -> BTreeMap<SiteId, u32> {
    let mut dist = BTreeMap::new();
    if !adj.contains_key(&start) && !adj.values().any(|n| n.iter().any(|(v, _)| *v == start)) {
        dist.insert(start, 0);
        return dist;
    }
    let mut heap = BinaryHeap::new();
    dist.insert(start, 0);
    heap.push(Reverse((0_u32, start)));
    while let Some(Reverse((d, u))) = heap.pop() {
        if dist.get(&u) != Some(&d) {
            continue;
        }
        for &(v, w) in adj.get(&u).into_iter().flatten() {
            let nd = d.saturating_add(w);
            if dist.get(&v).is_none_or(|&cur| nd < cur) {
                dist.insert(v, nd);
                heap.push(Reverse((nd, v)));
            }
        }
    }
    dist
}

fn available_stock(site: &SiteRow, cargo: &str) -> u32 {
    site.stock
        .iter()
        .find(|r| r.cargo == cargo)
        .map(|r| r.on_hand.saturating_sub(r.reserved_out))
        .unwrap_or(0)
}

fn dist2_fallback(sink: &SiteRow, source: &SiteRow) -> u32 {
    let dx = source.world_xz[0] - sink.world_xz[0];
    let dz = source.world_xz[1] - sink.world_xz[1];
    let m = (dx * dx + dz * dz).sqrt();
    if m.is_finite() && m > 0.0 {
        m.ceil() as u32
    } else {
        0
    }
}

fn is_eligible_source(s: &SiteRow, demand: &DemandRow) -> bool {
    s.owner == demand.owner
        && s.id != demand.site
        && matches!(s.role, SiteRole::Depot | SiteRole::Plant)
}

fn pick_key(
    demand: &DemandRow,
    sink: &SiteRow,
    source: &SiteRow,
    travel_costs: &BTreeMap<SiteId, u32>,
) -> Option<(bool, u32, SiteId)> {
    let available = available_stock(source, &demand.cargo);
    if available == 0 {
        return None;
    }
    let partial = available < demand.buffer_target;
    let cost = travel_costs
        .get(&source.id)
        .copied()
        .unwrap_or_else(|| dist2_fallback(sink, source).max(1));
    Some((partial, cost, source.id))
}

/// One Dijkstra from the sink, then min (partial stock, travel ticks, site id).
#[must_use]
pub fn best_source_open_goal(snap: &LogisticsOrdersSnapshot, demand: &DemandRow) -> Option<SiteId> {
    snap.site(demand.site)?;
    let travel_costs = dijkstra_from(demand.site, &snap.site_adjacency);
    let sink = snap.site(demand.site)?;
    snap.sites
        .iter()
        .filter(|s| is_eligible_source(s, demand))
        .filter_map(|s| pick_key(demand, sink, s, &travel_costs))
        .min_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)))
        .map(|(_, _, id)| id)
}

/// Per-candidate shortest-path expansion — test reference only (N NAV-09 searches).
#[must_use]
#[cfg(test)]
pub fn best_source_brute_force(snap: &LogisticsOrdersSnapshot, demand: &DemandRow) -> Option<SiteId> {
    snap.site(demand.site)?;
    snap.sites
        .iter()
        .filter(|s| is_eligible_source(s, demand))
        .filter_map(|s| {
            let available = available_stock(s, &demand.cargo);
            if available == 0 {
                return None;
            }
            let dist = dijkstra_from(s.id, &snap.site_adjacency);
            let cost = *dist.get(&demand.site)?;
            if cost == 0 && s.id != demand.site {
                return None;
            }
            let partial = available < demand.buffer_target;
            Some((partial, cost, s.id))
        })
        .min_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)))
        .map(|(_, _, id)| id)
}

#[cfg(test)]
mod unit {
    use super::*;
    use crate::economy::logistics::orders::model::OwnerId;
    use crate::economy::logistics::orders::snapshot::{SiteRow, StockRow};

    fn row(id: u32, role: SiteRole, x: f32, stock: u32) -> SiteRow {
        SiteRow {
            id: SiteId(id),
            name: format!("s{id}"),
            owner: OwnerId::PLAYER,
            role,
            world_xz: [x, 0.0],
            tile: [0, 0],
            stock: vec![StockRow {
                cargo: "ammo".into(),
                unit: "t",
                on_hand: stock,
                reserved_out: 0,
                inbound: 0,
            }],
        }
    }

    #[test]
    fn dijkstra_matches_line_graph() {
        let mut adj = SiteTravelAdjacency::new();
        adj.insert(SiteId(1), vec![(SiteId(2), 10)]);
        adj.insert(SiteId(2), vec![(SiteId(1), 10), (SiteId(3), 5)]);
        adj.insert(SiteId(3), vec![(SiteId(2), 5)]);
        let dist = dijkstra_from(SiteId(3), &adj);
        assert_eq!(dist.get(&SiteId(1)), Some(&15));
    }
}