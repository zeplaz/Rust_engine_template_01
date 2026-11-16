//! **CS-1** shared front stock pool — PLAN-COMBAT-SUSTAINMENT-v1 / PLAN-ANTI-CLONE-KERNEL-v1.
//!
//! One scan of [`FrontSupplyPoints`] × logistics snapshot per refill pass. Typed sinks
//! (drone / missile / personnel) allocate through this pool instead of cloning the scan.

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::balance::FrontSupplyParams;
use super::capability_toe::ToeLine;
use super::supply::FrontSupplyPoints;
use crate::economy::logistics::orders::{
    CargoKind, LogisticsOrderCommand, LogisticsOrderRequest, LogisticsOrderSiteId,
    LogisticsOrdersSnapshotRes, SiteId,
};
use crate::systems::sim_control::SimStepClock;

/// Per-site on-hand rows for one (faction_slot, cargo_tag), lowest site id first then richest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontStockSiteRow {
    pub site: SiteId,
    pub on_hand: u32,
}

/// Stock available per (faction_slot, cargo_tag) across front sites (CQ-08: per-site list + fallback).
#[derive(Debug, Default, Clone)]
pub struct FrontStockPool {
    /// `(side, tag) → sites with stock` (deterministic order).
    available: BTreeMap<(u8, String), Vec<FrontStockSiteRow>>,
}

/// Frame-scoped pool rebuilt once before chained TOE refills (**CS-5**).
#[derive(Resource, Debug, Default, Clone)]
pub struct FrontStockPoolFrame {
    pub pool: FrontStockPool,
}

impl FrontStockPool {
    /// Build once from the current front sites + orders snapshot.
    #[must_use]
    pub fn from_front_snapshot(
        points: &FrontSupplyPoints,
        snap: &LogisticsOrdersSnapshotRes,
        sites: &Query<&LogisticsOrderSiteId>,
    ) -> Self {
        let mut available: BTreeMap<(u8, String), Vec<FrontStockSiteRow>> = BTreeMap::new();
        for ((_, side), entity) in &points.entities {
            let Ok(site_id) = sites.get(*entity).map(|s| s.0) else {
                continue;
            };
            let Some(row) = snap.0.sites.iter().find(|s| s.id == site_id) else {
                continue;
            };
            for stock in &row.stock {
                if stock.on_hand == 0 {
                    continue;
                }
                let key = (*side, stock.cargo.clone());
                available.entry(key).or_default().push(FrontStockSiteRow {
                    site: site_id,
                    on_hand: stock.on_hand,
                });
            }
        }
        for rows in available.values_mut() {
            rows.sort_by(|a, b| b.on_hand.cmp(&a.on_hand).then(a.site.cmp(&b.site)));
        }
        Self { available }
    }

    #[must_use]
    pub fn sites_for(&self, side: u8, tag: &str) -> &[FrontStockSiteRow] {
        self.available
            .get(&(side, tag.to_string()))
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Try to consume one BOM unit (`lines` once). Walks per-tag site lists with fallback (CQ-08).
    pub fn try_consume_bom_unit(
        &mut self,
        side: u8,
        lines: &[ToeLine],
        requests: &mut MessageWriter<LogisticsOrderRequest>,
    ) -> bool {
        let mut planned: Vec<(String, SiteId, u32)> = Vec::with_capacity(lines.len());
        for line in lines {
            let key = (side, line.tag.clone());
            let Some(rows) = self.available.get(&key) else {
                return false;
            };
            let Some(row) = rows.iter().find(|r| r.on_hand >= line.qty) else {
                return false;
            };
            planned.push((line.tag.clone(), row.site, line.qty));
        }
        for (tag, site, qty) in planned {
            let key = (side, tag.clone());
            if let Some(rows) = self.available.get_mut(&key) {
                if let Some(entry) = rows.iter_mut().find(|r| r.site == site) {
                    entry.on_hand = entry.on_hand.saturating_sub(qty);
                }
                rows.retain(|r| r.on_hand > 0);
                rows.sort_by(|a, b| b.on_hand.cmp(&a.on_hand).then(a.site.cmp(&b.site)));
            }
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::ConsumeStock {
                site,
                cargo: CargoKind::new(tag),
                amount: qty,
            }));
        }
        true
    }

    /// Single-tag consume (personnel credits path). Returns amount actually taken.
    pub fn consume_tag_up_to(
        &mut self,
        side: u8,
        tag: &str,
        amount: u32,
        requests: &mut MessageWriter<LogisticsOrderRequest>,
    ) -> u32 {
        if amount == 0 {
            return 0;
        }
        let key = (side, tag.to_string());
        let Some(rows) = self.available.get_mut(&key) else {
            return 0;
        };
        let mut remaining = amount;
        let mut took_total = 0u32;
        let mut draws: Vec<(SiteId, u32)> = Vec::new();
        for row in rows.iter_mut() {
            if remaining == 0 {
                break;
            }
            let take = remaining.min(row.on_hand);
            if take == 0 {
                continue;
            }
            row.on_hand = row.on_hand.saturating_sub(take);
            remaining = remaining.saturating_sub(take);
            took_total = took_total.saturating_add(take);
            draws.push((row.site, take));
        }
        rows.retain(|r| r.on_hand > 0);
        rows.sort_by(|a, b| b.on_hand.cmp(&a.on_hand).then(a.site.cmp(&b.site)));
        for (site, take) in draws {
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::ConsumeStock {
                site,
                cargo: CargoKind::new(tag.to_string()),
                amount: take,
            }));
        }
        took_total
    }
}

/// Rebuild [`FrontStockPoolFrame`] once per due sim step (before chained refills).
pub fn rebuild_front_stock_pool_system(
    clock: Res<SimStepClock>,
    points: Res<FrontSupplyPoints>,
    snap: Option<Res<LogisticsOrdersSnapshotRes>>,
    sites: Query<&LogisticsOrderSiteId>,
    mut frame: ResMut<FrontStockPoolFrame>,
) {
    if clock.due == 0 {
        return;
    }
    let Some(snap) = snap else {
        frame.pool = FrontStockPool::default();
        return;
    };
    frame.pool = FrontStockPool::from_front_snapshot(&points, &snap, &sites);
}

/// Split side-wide `(side, tag) → qty` across that side's front sites (CQ-08 — no N× inflate).
#[must_use]
pub fn split_side_tag_demand_across_sites(
    side_sites: &[SiteId],
    qty: u32,
) -> Vec<(SiteId, u32)> {
    if side_sites.is_empty() || qty == 0 {
        return Vec::new();
    }
    let n = side_sites.len() as u32;
    let base = qty / n;
    let rem = qty % n;
    side_sites
        .iter()
        .enumerate()
        .filter_map(|(i, site)| {
            let share = base + u32::from((i as u32) < rem);
            (share > 0).then_some((*site, share))
        })
        .collect()
}

/// Publish `(side, tag)` demand split per front site for that side (CQ-08).
pub fn publish_side_tag_demands(
    points: &FrontSupplyPoints,
    sites: &Query<&LogisticsOrderSiteId>,
    by_side_tag: &BTreeMap<(u8, String), u32>,
    supply: &FrontSupplyParams,
    requests: &mut MessageWriter<LogisticsOrderRequest>,
) {
    use crate::economy::logistics::orders::{ConsumerKind, Priority};

    let reorder = supply.reorder_fraction.clamp(0.0, 1.0);
    let draw_rate = supply.toe_draw_rate_per_min.max(1);

    let mut sites_by_side: BTreeMap<u8, Vec<SiteId>> = BTreeMap::new();
    for ((_, side), entity) in &points.entities {
        let Ok(site) = sites.get(*entity).map(|s| s.0) else {
            continue;
        };
        sites_by_side.entry(*side).or_default().push(site);
    }
    for list in sites_by_side.values_mut() {
        list.sort_unstable();
        list.dedup();
    }

    for ((slot, tag), qty) in by_side_tag {
        if *qty == 0 {
            continue;
        }
        let Some(side_sites) = sites_by_side.get(slot) else {
            continue;
        };
        for (site, share) in split_side_tag_demand_across_sites(side_sites, *qty) {
            let buffer_target = share.max(1);
            let reorder_point = ((buffer_target as f32) * reorder).ceil() as u32;
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::PublishDemand {
                site,
                cargo: CargoKind::new(tag.clone()),
                consumer: ConsumerKind::Front,
                buffer_target,
                reorder_point,
                draw_rate_per_min: draw_rate,
                priority: Priority::Normal,
            }));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::capability_toe::ToeLine;
    use crate::economy::logistics::orders::SiteId;

    #[test]
    fn cq08_split_does_not_inflate_total() {
        let sites = [SiteId(1), SiteId(2), SiteId(3)];
        let parts = split_side_tag_demand_across_sites(&sites, 10);
        assert_eq!(parts.len(), 3);
        assert_eq!(parts.iter().map(|(_, q)| *q).sum::<u32>(), 10);
        assert_eq!(parts[0], (SiteId(1), 4));
        assert_eq!(parts[1], (SiteId(2), 3));
        assert_eq!(parts[2], (SiteId(3), 3));
    }

    #[test]
    fn cq08_pool_falls_back_to_second_site() {
        let mut pool = FrontStockPool::default();
        pool.available.insert(
            (0, "drone_airframe".into()),
            vec![
                FrontStockSiteRow {
                    site: SiteId(1),
                    on_hand: 0,
                },
                FrontStockSiteRow {
                    site: SiteId(2),
                    on_hand: 2,
                },
            ],
        );
        // Keep only positive rows as from_front_snapshot would.
        pool.available
            .get_mut(&(0, "drone_airframe".into()))
            .unwrap()
            .retain(|r| r.on_hand > 0);
        assert_eq!(pool.sites_for(0, "drone_airframe")[0].site, SiteId(2));
    }

    #[test]
    fn cs1_pool_try_consume_decrements_chosen_site() {
        use bevy::ecs::system::SystemState;
        use crate::economy::logistics::orders::LogisticsOrderRequest;

        let mut app = bevy::app::App::new();
        app.add_message::<LogisticsOrderRequest>();

        let mut pool = FrontStockPool::default();
        pool.available.insert(
            (0, "drone_airframe".into()),
            vec![FrontStockSiteRow {
                site: SiteId(1),
                on_hand: 4,
            }],
        );
        pool.available.insert(
            (0, "battery_pack".into()),
            vec![FrontStockSiteRow {
                site: SiteId(1),
                on_hand: 4,
            }],
        );
        let lines = [
            ToeLine {
                tag: "drone_airframe".into(),
                qty: 1,
            },
            ToeLine {
                tag: "battery_pack".into(),
                qty: 1,
            },
        ];

        {
            let mut state: SystemState<bevy::ecs::system::MessageWriter<LogisticsOrderRequest>> =
                SystemState::new(app.world_mut());
            let mut requests = state.get_mut(app.world_mut());
            assert!(pool.try_consume_bom_unit(0, &lines, &mut requests));
            state.apply(app.world_mut());
        }
        assert_eq!(pool.sites_for(0, "drone_airframe")[0].on_hand, 3);
        assert_eq!(pool.sites_for(0, "battery_pack")[0].on_hand, 3);
    }
}
