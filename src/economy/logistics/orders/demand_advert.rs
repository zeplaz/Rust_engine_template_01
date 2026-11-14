//! DEC-10 — demand advert scoring (Smelly GOB): urgency grows with unmet time; travel discounts score.

use super::model::Priority;
use super::snapshot::{DemandRow, LogisticsOrdersSnapshot};
use super::source_pick::{best_source_open_goal, dijkstra_from};

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
    let Some(source) = best_source_open_goal(snap, demand) else {
        return u32::MAX;
    };
    dijkstra_from(demand.site, &snap.site_adjacency)
        .get(&source)
        .copied()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::logistics::orders::model::{ConsumerKind, DemandHealth, OwnerId, SiteId};

    fn row(site: u32, unmet: u64, travel: u32) -> DemandRow {
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
    fn nearer_demand_beats_equal_unmet_when_closer() {
        let tuning = DemandAdvertTuning::default();
        let near = row(1, 10, 20);
        let far = row(2, 10, 200);
        assert!(demand_advert_score(&near, 20, &tuning) > demand_advert_score(&far, 200, &tuning));
    }
}