//! **CS-4** — one TOE cargo demand publisher (drone + missile + personnel).
//!
//! Books stay as dump/read surfaces; this system is their sole writer for publish.

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::balance::FrontBalance;
use super::capability_toe::CapabilityToeRegistry;
use super::drone_missions::{DronePackage, DroneRole};
use super::drone_supply::DroneCargoDemandBook;
use super::formation::Formation;
use super::front_stock_pool::publish_side_tag_demands;
use super::missile_supply::{MissileCargoDemandBook, MissilePackage};
use super::personnel_supply::{side_personnel_deficit, PersonnelRestockBook};
use super::capability_toe::MissileClass;
use super::supply::FrontSupplyPoints;
use super::theater::TheaterBook;
use crate::economy::logistics::orders::{LogisticsOrderRequest, LogisticsOrderSiteId};
use crate::systems::sim_control::SimStepClock;
use crate::systems::weather::LinkModality;

/// Sole publisher for drone/missile/personnel TOE deficits onto the Front PublishDemand bus.
pub fn publish_toe_cargo_demands_system(
    clock: Res<SimStepClock>,
    theaters: Res<TheaterBook>,
    balance: Res<FrontBalance>,
    points: Res<FrontSupplyPoints>,
    toe: Res<CapabilityToeRegistry>,
    sites: Query<&LogisticsOrderSiteId>,
    drones: Query<(&Formation, &DronePackage)>,
    missiles: Query<(&Formation, &MissilePackage)>,
    formations: Query<&Formation>,
    mut drone_book: ResMut<DroneCargoDemandBook>,
    mut missile_book: ResMut<MissileCargoDemandBook>,
    mut personnel: ResMut<PersonnelRestockBook>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    if clock.due == 0 {
        return;
    }

    // --- drone ---
    drone_book.by_side_tag.clear();
    let mut drone_total = 0u32;
    for (f, p) in drones.iter() {
        for role in DroneRole::ALL {
            let row = toe.drone(role);
            let have = p.count(role);
            if have >= row.toe_count {
                continue;
            }
            let need_units = row.toe_count - have;
            for line in &row.lines {
                let qty = need_units.saturating_mul(line.qty);
                *drone_book
                    .by_side_tag
                    .entry((f.faction_slot, line.tag.clone()))
                    .or_insert(0) += qty;
                drone_total = drone_total.saturating_add(qty);
            }
        }
        if matches!(p.modality, LinkModality::Fiber) {
            *drone_book
                .by_side_tag
                .entry((f.faction_slot, "fiber_spool_km".into()))
                .or_insert(0) += 1;
            drone_total = drone_total.saturating_add(1);
        }
    }
    drone_book.total_units = drone_total;

    // --- missile ---
    missile_book.by_side_tag.clear();
    let mut missile_total = 0u32;
    for (f, p) in missiles.iter() {
        for class in MissileClass::ALL {
            let row = toe.missile(class);
            let have = p.count(class);
            if have >= row.toe_count {
                continue;
            }
            let need_units = row.toe_count - have;
            for line in &row.lines {
                let qty = need_units.saturating_mul(line.qty);
                *missile_book
                    .by_side_tag
                    .entry((f.faction_slot, line.tag.clone()))
                    .or_insert(0) += qty;
                missile_total = missile_total.saturating_add(qty);
            }
        }
    }
    missile_book.total_units = missile_total;

    // --- personnel ---
    personnel.published_deficit.clear();
    let mut sides: Vec<u8> = points.entities.keys().map(|(_, s)| *s).collect();
    sides.sort_unstable();
    sides.dedup();
    let mut personnel_tags: BTreeMap<(u8, String), u32> = BTreeMap::new();
    for side in sides {
        let deficit = side_personnel_deficit(side, &balance, formations.iter());
        personnel.published_deficit.insert(side, deficit);
        if deficit > 0 {
            personnel_tags.insert((side, "personnel".into()), deficit);
        }
    }

    // One publish pass for all TOE tags (drone + missile + personnel).
    let mut combined = drone_book.by_side_tag.clone();
    for (k, v) in &missile_book.by_side_tag {
        *combined.entry(k.clone()).or_insert(0) += *v;
    }
    for (k, v) in &personnel_tags {
        *combined.entry(k.clone()).or_insert(0) += *v;
    }
    publish_side_tag_demands(&points, &sites, &combined, &mut requests);
    let _ = theaters;
}

#[cfg(test)]
mod tests {
    #[test]
    fn cs4_toe_publisher_module_links() {
        // Compile/link tripwire — behavioral cover lives in drone/missile/personnel books.
        assert!(true);
    }
}
