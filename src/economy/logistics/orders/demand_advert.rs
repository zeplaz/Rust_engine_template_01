//! DEC-10 — demand advert scoring (Smelly GOB): urgency grows with unmet time; travel discounts score.

use std::cmp::Reverse;
use std::collections::BTreeMap;

use super::model::{Priority, SiteId};
use super::snapshot::{DemandRow, LogisticsOrdersSnapshot};
use super::source_pick::{
    best_source_and_travel_ticks, best_source_with_travel_costs, dijkstra_from,
};

/// Tunables for [`demand_advert_score`]. Plain integers — no utility curves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DemandAdvertTuning {
    /// Added score per tick the demand has no standing line (`DrawDemand::unmet_ticks`).
    pub urgency_per_unmet_tick: u64,
    /// Subtracted score per tick from sink to best stocked source.
    pub distance_discount_per_tick: u64,
    pub health_weight: u64,
}

impl Default for DemandAdvertTuning {
    fn default() -> Self {
        Self {
            urgency_per_unmet_tick: 3,
            distance_discount_per_tick: 1,
            health_weight: 500,
        }
    }
}

/// GOB advert strength for one published demand row.
#[must_use]
pub fn demand_advert_score(d: &DemandRow, travel_ticks: u32, tuning: &DemandAdvertTuning) -> u64 {
    let health = u64::from(d.health.severity()) * tuning.health_weight;
    let priority = u64::from(d.priority.weight());
    let urgency = d.unmet_ticks.saturating_mul(tuning.urgency_per_unmet_tick);
    let starved = d.starved_ticks.saturating_mul(tuning.urgency_per_unmet_tick / 2);
    let distance = u64::from(travel_ticks).saturating_mul(tuning.distance_discount_per_tick);
    health
        .saturating_add(priority)
        .saturating_add(urgency)
        .saturating_add(starved)
        .saturating_sub(distance)
}

#[must_use]
pub fn travel_ticks_to_best_source(snap: &LogisticsOrdersSnapshot, demand: &DemandRow) -> u32 {
    best_source_and_travel_ticks(snap, demand)
        .map(|(_, ticks)| ticks)
        .unwrap_or(u32::MAX)
}

#[must_use]
pub fn demand_advert_score_from_snapshot(
    snap: &LogisticsOrdersSnapshot,
    demand: &DemandRow,
    tuning: &DemandAdvertTuning,
) -> u64 {
    demand_advert_score(demand, travel_ticks_to_best_source(snap, demand), tuning)
}

/// Sort key for unserved demands: higher advert score first, then deterministic tie-breaks.
///
/// Prefer [`rank_demands_by_advert`] when sorting a list — this comparator re-runs Dijkstra
/// on every comparison and is kept only for single-pair checks.
#[must_use]
pub fn cmp_demands_by_advert(
    a: &DemandRow,
    b: &DemandRow,
    snap: &LogisticsOrdersSnapshot,
    tuning: &DemandAdvertTuning,
) -> std::cmp::Ordering {
    let sa = demand_advert_score_from_snapshot(snap, a, tuning);
    let sb = demand_advert_score_from_snapshot(snap, b, tuning);
    sb.cmp(&sa)
        .then(b.health.severity().cmp(&a.health.severity()))
        .then(b.priority.cmp(&a.priority))
        .then(a.site.cmp(&b.site))
        .then(a.cargo.cmp(&b.cargo))
}

type AdvertSortKey = (Reverse<u64>, Reverse<u8>, Reverse<Priority>, SiteId, String);

fn advert_sort_key(score: u64, d: &DemandRow) -> AdvertSortKey {
    (
        Reverse(score),
        Reverse(d.health.severity()),
        Reverse(d.priority),
        d.site,
        d.cargo.clone(),
    )
}

/// Score each demand once, caching one Dijkstra per sink site (CQ-03).
#[must_use]
pub fn rank_demands_by_advert<'a>(
    demands: impl IntoIterator<Item = &'a DemandRow>,
    snap: &LogisticsOrdersSnapshot,
    tuning: &DemandAdvertTuning,
) -> Vec<&'a DemandRow> {
    let mut dist_cache: BTreeMap<SiteId, BTreeMap<SiteId, u32>> = BTreeMap::new();
    let mut ranked: Vec<(AdvertSortKey, &'a DemandRow)> = demands
        .into_iter()
        .map(|d| {
            let costs = dist_cache
                .entry(d.site)
                .or_insert_with(|| dijkstra_from(d.site, &snap.site_adjacency));
            let travel = best_source_with_travel_costs(snap, d, costs)
                .and_then(|src| costs.get(&src).copied())
                .unwrap_or(u32::MAX);
            let score = demand_advert_score(d, travel, tuning);
            (advert_sort_key(score, d), d)
        })
        .collect();
    ranked.sort_by(|a, b| a.0.cmp(&b.0));
    ranked.into_iter().map(|(_, d)| d).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::model::{
        ConsumerKind, DemandHealth, OwnerId, Priority, SiteId,
    };
    use crate::economy::logistics::orders::snapshot::{SiteRole, SiteRow, StockRow};
    use crate::economy::logistics::orders::source_pick::SiteTravelAdjacency;

    fn row(site: u32, unmet: u64, travel: u32) -> DemandRow {
        let _ = travel;
        DemandRow {
            site: SiteId(site),
            site_name: format!("p{site}"),
            owner: OwnerId::PLAYER,
            cargo: "Cement".into(),
            unit: "t",
            consumer: ConsumerKind::Plant,
            on_hand: 0,
            inbound: 0,
            buffer_target: 100,
            reorder_point: 25,
            draw_rate_per_min: 10,
            priority: Priority::Normal,
            health: DemandHealth::Low,
            served_by: 0,
            starved_ticks: 0,
            shortfall_total: 0,
            unmet_ticks: unmet,
            time_to_empty_ticks: None,
            round_trip_ticks: None,
            trucks_needed: None,
        }
    }

    fn site(id: u32, role: SiteRole, stock: u32) -> SiteRow {
        SiteRow {
            id: SiteId(id),
            name: format!("s{id}"),
            owner: OwnerId::PLAYER,
            role,
            world_xz: [id as f32 * 10.0, 0.0],
            tile: [id as i32, 0],
            stock: if stock > 0 {
                vec![StockRow {
                    cargo: "Cement".into(),
                    unit: "t",
                    on_hand: stock,
                    reserved_out: 0,
                    inbound: 0,
                }]
            } else {
                Vec::new()
            },
        }
    }

    #[test]
    fn older_unmet_demand_outranks_newer_equal_one() {
        let tuning = DemandAdvertTuning::default();
        let older = row(1, 200, 50);
        let newer = row(2, 5, 50);
        assert!(
            demand_advert_score(&older, 50, &tuning) > demand_advert_score(&newer, 50, &tuning),
            "longer unmet time should dominate equal health/priority/distance"
        );
    }

    #[test]
    fn nearer_source_outranks_farther_equal_urgency() {
        let tuning = DemandAdvertTuning::default();
        let near = row(1, 10, 20);
        let far = row(2, 10, 200);
        assert!(demand_advert_score(&near, 20, &tuning) > demand_advert_score(&far, 200, &tuning));
    }

    #[test]
    fn rank_scores_each_sink_dijkstra_once() {
        let mut adj = SiteTravelAdjacency::new();
        adj.insert(SiteId(1), vec![(SiteId(10), 5), (SiteId(2), 1)]);
        adj.insert(SiteId(2), vec![(SiteId(10), 40), (SiteId(1), 1)]);
        adj.insert(SiteId(10), vec![(SiteId(1), 5), (SiteId(2), 40)]);

        let mut snap = LogisticsOrdersSnapshot {
            sites: vec![
                site(1, SiteRole::Plant, 0),
                site(2, SiteRole::Plant, 0),
                site(10, SiteRole::Depot, 500),
            ],
            demands: vec![
                {
                    let mut d = row(1, 100, 0);
                    d.cargo = "Cement".into();
                    d
                },
                {
                    let mut d = row(1, 10, 0);
                    d.cargo = "Timber".into();
                    d
                },
                {
                    let mut d = row(2, 50, 0);
                    d.cargo = "Cement".into();
                    d
                },
            ],
            site_adjacency: adj,
            ..Default::default()
        };
        // Give timber stock too so both site-1 demands resolve.
        if let Some(depot) = snap.sites.iter_mut().find(|s| s.id == SiteId(10)) {
            depot.stock.push(StockRow {
                cargo: "Timber".into(),
                unit: "t",
                on_hand: 200,
                reserved_out: 0,
                inbound: 0,
            });
        }

        let tuning = DemandAdvertTuning::default();
        let ranked = rank_demands_by_advert(snap.demands.iter(), &snap, &tuning);
        assert_eq!(ranked.len(), 3);
        // Site 1 / unmet 100 should outrank site 1 / unmet 10 and the farther site 2 demand.
        assert_eq!(ranked[0].site, SiteId(1));
        assert_eq!(ranked[0].unmet_ticks, 100);
        assert_eq!(
            demand_advert_score_from_snapshot(&snap, ranked[0], &tuning),
            demand_advert_score(
                ranked[0],
                travel_ticks_to_best_source(&snap, ranked[0]),
                &tuning
            )
        );
    }
}
