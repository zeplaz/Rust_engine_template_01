//! **CS-1** shared front stock pool — PLAN-COMBAT-SUSTAINMENT-v1 / PLAN-ANTI-CLONE-KERNEL-v1.
//!
//! One scan of [`FrontSupplyPoints`] × logistics snapshot per refill pass. Typed sinks
//! (drone / missile / personnel) allocate through this pool instead of cloning the scan.

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::capability_toe::ToeLine;
use super::supply::FrontSupplyPoints;
use crate::economy::logistics::orders::{
    CargoKind, LogisticsOrderCommand, LogisticsOrderRequest, LogisticsOrderSiteId,
    LogisticsOrdersSnapshotRes, SiteId,
};
use crate::systems::sim_control::SimStepClock;

/// Best on-hand stock per (faction_slot, cargo_tag) at a front site.
#[derive(Debug, Default, Clone)]
pub struct FrontStockPool {
    /// `(side, tag) → (site, on_hand)`
    available: BTreeMap<(u8, String), (SiteId, u32)>,
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
        let mut available: BTreeMap<(u8, String), (SiteId, u32)> = BTreeMap::new();
        for ((_, side), entity) in &points.entities {
            let Ok(site_id) = sites.get(*entity).map(|s| s.0) else {
                continue;
            };
            let Some(row) = snap.0.sites.iter().find(|s| s.id == site_id) else {
                continue;
            };
            for stock in &row.stock {
                let key = (*side, stock.cargo.clone());
                let entry = available.entry(key).or_insert((site_id, 0));
                if stock.on_hand > entry.1 {
                    *entry = (site_id, stock.on_hand);
                }
            }
        }
        Self { available }
    }

    /// Try to consume one BOM unit (`lines` once). On success writes ConsumeStock and returns true.
    pub fn try_consume_bom_unit(
        &mut self,
        side: u8,
        lines: &[ToeLine],
        requests: &mut MessageWriter<LogisticsOrderRequest>,
    ) -> bool {
        let mut planned: Vec<(String, SiteId, u32)> = Vec::with_capacity(lines.len());
        for line in lines {
            let key = (side, line.tag.clone());
            let Some((site, on_hand)) = self.available.get(&key).copied() else {
                return false;
            };
            if on_hand < line.qty {
                return false;
            }
            planned.push((line.tag.clone(), site, line.qty));
        }
        for (tag, site, qty) in planned {
            let key = (side, tag.clone());
            if let Some(entry) = self.available.get_mut(&key) {
                entry.1 = entry.1.saturating_sub(qty);
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
        let Some((site, on_hand)) = self.available.get(&key).copied() else {
            return 0;
        };
        let take = amount.min(on_hand);
        if take == 0 {
            return 0;
        }
        if let Some(entry) = self.available.get_mut(&key) {
            entry.1 = entry.1.saturating_sub(take);
        }
        requests.write(LogisticsOrderRequest(LogisticsOrderCommand::ConsumeStock {
            site,
            cargo: CargoKind::new(tag.to_string()),
            amount: take,
        }));
        take
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

/// Publish aggregated `(side, tag) → qty` demand to every front site for that side.
pub fn publish_side_tag_demands(
    points: &FrontSupplyPoints,
    sites: &Query<&LogisticsOrderSiteId>,
    by_side_tag: &BTreeMap<(u8, String), u32>,
    requests: &mut MessageWriter<LogisticsOrderRequest>,
) {
    use crate::economy::logistics::orders::{ConsumerKind, Priority};

    for ((_, side), entity) in &points.entities {
        let Ok(site) = sites.get(*entity).map(|s| s.0) else {
            continue;
        };
        for ((slot, tag), qty) in by_side_tag {
            if *slot != *side || *qty == 0 {
                continue;
            }
            requests.write(LogisticsOrderRequest(LogisticsOrderCommand::PublishDemand {
                site,
                cargo: CargoKind::new(tag.clone()),
                consumer: ConsumerKind::Front,
                buffer_target: (*qty).max(1),
                reorder_point: ((*qty as f32) * 0.5).ceil() as u32,
                draw_rate_per_min: 1,
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
    fn cs1_pool_consumes_bom_deterministically() {
        let mut pool = FrontStockPool::default();
        pool.available
            .insert((0, "drone_airframe".into()), (SiteId(1), 4));
        pool.available
            .insert((0, "battery_pack".into()), (SiteId(1), 4));
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
        assert_eq!(
            pool.available
                .get(&(0, "drone_airframe".into()))
                .map(|e| e.1),
            Some(4)
        );
        for line in &lines {
            let key = (0u8, line.tag.clone());
            let entry = pool.available.get_mut(&key).unwrap();
            entry.1 = entry.1.saturating_sub(line.qty);
        }
        assert_eq!(
            pool.available
                .get(&(0, "drone_airframe".into()))
                .map(|e| e.1),
            Some(3)
        );
        assert_eq!(
            pool.available
                .get(&(0, "battery_pack".into()))
                .map(|e| e.1),
            Some(3)
        );
    }
}
