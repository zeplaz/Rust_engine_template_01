//! D4a / spine P3 — drone cargo demand + closed-loop refill from front stock.

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::capability_toe::CapabilityToeRegistry;
use super::drone_missions::{DronePackage, DroneRole};
use super::formation::Formation;
use super::supply::FrontSupplyPoints;
use super::theater::TheaterBook;
use crate::economy::logistics::orders::{
    CargoKind, ConsumerKind, LogisticsOrderCommand, LogisticsOrderRequest, LogisticsOrderSiteId,
    LogisticsOrdersSnapshotRes, Priority, SiteId,
};
use crate::systems::sim_control::SimStepClock;
use crate::systems::weather::LinkModality;

/// Aggregated UAS BOM demand (dump + publish). Sole writer: [`publish_drone_cargo_demands_system`].
#[derive(Resource, Debug, Default, Clone)]
pub struct DroneCargoDemandBook {
    pub by_side_tag: BTreeMap<(u8, String), u32>,
    pub total_units: u32,
}

/// Aligned P3 airframe tag (all roles). Legacy role-specific tags remain in cargo registry as aliases.
#[must_use]
pub fn airframe_tag(_role: DroneRole) -> &'static str {
    "drone_airframe"
}

/// Package refill demand → front sites (same PublishDemand bus as FC-6 supply).
pub fn publish_drone_cargo_demands_system(
    clock: Res<SimStepClock>,
    theaters: Res<TheaterBook>,
    points: Res<FrontSupplyPoints>,
    toe: Res<CapabilityToeRegistry>,
    sites: Query<&LogisticsOrderSiteId>,
    packages: Query<(&Formation, &DronePackage)>,
    mut book: ResMut<DroneCargoDemandBook>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    if clock.due == 0 {
        return;
    }
    book.by_side_tag.clear();
    let mut total = 0u32;
    for (f, p) in packages.iter() {
        for role in DroneRole::ALL {
            let row = toe.drone(role);
            let have = p.count(role);
            if have >= row.toe_count {
                continue;
            }
            let need_units = row.toe_count - have;
            for line in &row.lines {
                let qty = need_units.saturating_mul(line.qty);
                *book
                    .by_side_tag
                    .entry((f.faction_slot, line.tag.clone()))
                    .or_insert(0) += qty;
                total = total.saturating_add(qty);
            }
        }
        if matches!(p.modality, LinkModality::Fiber) {
            *book
                .by_side_tag
                .entry((f.faction_slot, "fiber_spool_km".into()))
                .or_insert(0) += 1;
            total = total.saturating_add(1);
        }
    }
    book.total_units = total;

    for ((_theater, side), entity) in &points.entities {
        let Ok(site) = sites.get(*entity).map(|s| s.0) else {
            continue;
        };
        for ((slot, tag), qty) in &book.by_side_tag {
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
    let _ = theaters;
}

/// Consume front `drone_airframe` / battery / fiber stock to top up [`DronePackage`] toward TOE.
///
/// Sole package count writer besides sortie [`DronePackage::take`] (via mission step).
pub fn refill_drone_packages_from_front_system(
    clock: Res<SimStepClock>,
    points: Res<FrontSupplyPoints>,
    toe: Res<CapabilityToeRegistry>,
    snap: Option<Res<LogisticsOrdersSnapshotRes>>,
    sites: Query<&LogisticsOrderSiteId>,
    mut packages: Query<(Entity, &Formation, &mut DronePackage)>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    if clock.due == 0 {
        return;
    }
    let Some(snap) = snap else {
        return;
    };

    // Per side: best front site stock for each cargo tag.
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

    let mut order: Vec<(Entity, u8)> = packages
        .iter()
        .map(|(e, f, _)| (e, f.faction_slot))
        .collect();
    order.sort_by_key(|(e, slot)| (*slot, e.to_bits()));

    for (entity, slot) in order {
        let Ok((_, _, mut pkg)) = packages.get_mut(entity) else {
            continue;
        };
        for role in DroneRole::ALL {
            let row = toe.drone(role);
            while pkg.count(role) < row.toe_count {
                let mut ok = true;
                let mut planned: Vec<(String, SiteId)> = Vec::new();
                for line in &row.lines {
                    let key = (slot, line.tag.clone());
                    let Some((site, on_hand)) = available.get(&key).copied() else {
                        ok = false;
                        break;
                    };
                    if on_hand < line.qty {
                        ok = false;
                        break;
                    }
                    planned.push((line.tag.clone(), site));
                }
                if !ok {
                    break;
                }
                for (tag, site) in &planned {
                    let key = (slot, tag.clone());
                    if let Some(entry) = available.get_mut(&key) {
                        let take = row
                            .lines
                            .iter()
                            .find(|l| l.tag == *tag)
                            .map(|l| l.qty)
                            .unwrap_or(1);
                        entry.1 = entry.1.saturating_sub(take);
                        requests.write(LogisticsOrderRequest(LogisticsOrderCommand::ConsumeStock {
                            site: *site,
                            cargo: CargoKind::new(tag.clone()),
                            amount: take,
                        }));
                    }
                }
                let i = role.index();
                pkg.counts[i] = pkg.counts[i].saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::balance::UnitKind;
    use bevy::math::IVec2;

    #[test]
    fn airframe_tag_aligned() {
        assert_eq!(airframe_tag(DroneRole::FpvStrike), "drone_airframe");
        assert_eq!(airframe_tag(DroneRole::Interceptor), "drone_airframe");
        let p = DronePackage::with_count(DroneRole::FpvStrike, 1);
        let _ = Formation::new(0, UnitKind::Infantry, IVec2::ZERO);
        assert_eq!(p.count(DroneRole::FpvStrike), 1);
    }
}
