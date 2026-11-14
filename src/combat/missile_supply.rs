//! Spine P6 — missile / air magazine counts (MissionBook pattern; no airframe entities).
//!
//! [`MissilePackage`] on formations (esp. AirDefense) holds class counts. AirOpZones in
//! [`MissionBook`] fire weather-gated sorties that consume packages. Demand + refill reuse the
//! front PublishDemand / ConsumeStock bus (same as drones). MFG plants optional — cargo tags +
//! magazine + consume + demand are the closed minimum.

use std::collections::BTreeMap;

use bevy::prelude::*;

use super::capability_toe::{CapabilityToeRegistry, MissileClass};
use super::drone_missions::{AirOpZone, MissionBook};
use super::formation::Formation;
use super::supply::FrontSupplyPoints;
use crate::economy::logistics::orders::{
    CargoKind, ConsumerKind, LogisticsOrderCommand, LogisticsOrderRequest, LogisticsOrderSiteId,
    LogisticsOrdersSnapshotRes, Priority, SiteId,
};
use crate::systems::sim_control::{SimStepClock, SimTimeMicros};
use crate::systems::weather::{
    evaluate_grounding, AirOpsProfileId, AirOpsTuning, WeatherField, WeatherSample,
};
use crate::terrain::world_scale_contract::TILE_METRES_GAMEPLAY;

/// Formation-local missile magazine counts (not airframe entities).
#[derive(Component, Clone, Debug, PartialEq)]
pub struct MissilePackage {
    pub counts: [u32; 3],
}

impl Default for MissilePackage {
    fn default() -> Self {
        Self { counts: [0; 3] }
    }
}

impl MissilePackage {
    #[must_use]
    pub fn with_count(class: MissileClass, n: u32) -> Self {
        let mut p = Self::default();
        p.counts[class.index()] = n;
        p
    }

    #[must_use]
    pub fn count(&self, class: MissileClass) -> u32 {
        self.counts[class.index()]
    }

    pub fn take(&mut self, class: MissileClass, n: u32) -> u32 {
        let i = class.index();
        let got = self.counts[i].min(n);
        self.counts[i] -= got;
        got
    }
}

/// Aggregated missile BOM demand (dump + publish). Sole writer: [`publish_missile_cargo_demands_system`].
#[derive(Resource, Debug, Default, Clone)]
pub struct MissileCargoDemandBook {
    pub by_side_tag: BTreeMap<(u8, String), u32>,
    pub total_units: u32,
}

/// Dump counters for missile weather sorties (MissionBook writer updates via pulse).
#[derive(Resource, Debug, Default, Clone)]
pub struct MissileSortieDump {
    pub sorties_ok: u64,
    pub sorties_aborted_wx: u64,
    pub consumed: u64,
}

/// Package refill demand → front sites.
pub fn publish_missile_cargo_demands_system(
    clock: Res<SimStepClock>,
    points: Res<FrontSupplyPoints>,
    toe: Res<CapabilityToeRegistry>,
    sites: Query<&LogisticsOrderSiteId>,
    packages: Query<(&Formation, &MissilePackage)>,
    mut book: ResMut<MissileCargoDemandBook>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    if clock.due == 0 {
        return;
    }
    book.by_side_tag.clear();
    let mut total = 0u32;
    for (f, p) in packages.iter() {
        for class in MissileClass::ALL {
            let row = toe.missile(class);
            let have = p.count(class);
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
    }
    book.total_units = total;

    for ((_, side), entity) in &points.entities {
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
}

/// Consume front missile_* stock to top up [`MissilePackage`] toward TOE.
pub fn refill_missile_packages_from_front_system(
    clock: Res<SimStepClock>,
    points: Res<FrontSupplyPoints>,
    toe: Res<CapabilityToeRegistry>,
    snap: Option<Res<LogisticsOrdersSnapshotRes>>,
    sites: Query<&LogisticsOrderSiteId>,
    mut packages: Query<(Entity, &Formation, &mut MissilePackage)>,
    mut requests: MessageWriter<LogisticsOrderRequest>,
) {
    if clock.due == 0 {
        return;
    }
    let Some(snap) = snap else {
        return;
    };

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
        for class in MissileClass::ALL {
            let row = toe.missile(class);
            while pkg.count(class) < row.toe_count {
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
                let i = class.index();
                pkg.counts[i] = pkg.counts[i].saturating_add(1);
            }
        }
    }
}

/// One successful weather-gated missile launch (for ballistics / audio).
#[derive(Clone, Debug)]
pub struct MissileSortieLaunch {
    pub faction_slot: u8,
    pub origin_xz: Vec2,
    pub target_xz: Vec2,
}

/// Drained by [`apply_missile_sortie_launches_system`] (keeps mission step under SystemParam limits).
#[derive(Resource, Default, Debug)]
pub struct MissileSortieLaunchQueue {
    pub pending: Vec<MissileSortieLaunch>,
}

/// Weather-gated missile sorties from [`AirOpZone`]s; consumes nearest [`MissilePackage`].
///
/// Called from [`super::drone_missions::step_drone_missions_system`] (sole MissionBook writer).
/// `faction_allowed`: return false to skip a zone (FIDELITY-1 attention gate).
pub fn pulse_missile_sorties(
    book: &MissionBook,
    dump: &mut MissileSortieDump,
    packages: &mut [(Entity, &Formation, &mut MissilePackage)],
    tuning: &AirOpsTuning,
    field: Option<&WeatherField>,
    sim_time: SimTimeMicros,
    preferred: MissileClass,
    faction_allowed: &dyn Fn(u8) -> bool,
) -> Vec<MissileSortieLaunch> {
    let profile = tuning.profile_or_default(AirOpsProfileId::Missile);
    let mut zones: Vec<&AirOpZone> = book.air_op_zones.values().collect();
    zones.sort_by(|a, b| {
        b.priority
            .partial_cmp(&a.priority)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.id.cmp(&b.id))
    });

    let mut launches = Vec::new();
    for z in zones {
        if z.paused || !faction_allowed(z.faction_slot) {
            continue;
        }
        let c = z.centroid_tile();
        let mut best: Option<(usize, i32)> = None;
        for (i, (_, f, p)) in packages.iter().enumerate() {
            if f.faction_slot != z.faction_slot || p.count(preferred) == 0 {
                continue;
            }
            let d = (f.tile - c).abs().x + (f.tile - c).abs().y;
            if best.is_none_or(|(_, bd)| d < bd) {
                best = Some((i, d));
            }
        }
        let Some((idx, _)) = best else {
            continue;
        };

        let sample = if let Some(field) = field.filter(|f| f.is_ready()) {
            field.sample(c.as_vec2() * TILE_METRES_GAMEPLAY, sim_time)
        } else {
            WeatherSample::default()
        };
        if evaluate_grounding(&sample, profile, None).is_grounded() {
            dump.sorties_aborted_wx = dump.sorties_aborted_wx.wrapping_add(1);
            continue;
        }
        // Drift envelope: treat extreme wind as abort (reuse missile_weather max).
        if sample.wind_mps.length() > profile.max_missile_wind_mps {
            dump.sorties_aborted_wx = dump.sorties_aborted_wx.wrapping_add(1);
            continue;
        }

        let (_, form, pkg) = &mut packages[idx];
        if pkg.take(preferred, 1) == 0 {
            continue;
        }
        dump.sorties_ok = dump.sorties_ok.wrapping_add(1);
        dump.consumed = dump.consumed.wrapping_add(1);
        let origin_xz = form.tile.as_vec2() * TILE_METRES_GAMEPLAY;
        let target_xz = c.as_vec2() * TILE_METRES_GAMEPLAY;
        launches.push(MissileSortieLaunch {
            faction_slot: z.faction_slot,
            origin_xz,
            target_xz,
        });
    }
    launches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat::balance::UnitKind;
    use bevy::math::IVec2;

    #[test]
    fn package_take_and_class_parse() {
        let mut p = MissilePackage::with_count(MissileClass::SamShort, 3);
        assert_eq!(p.take(MissileClass::SamShort, 2), 2);
        assert_eq!(p.count(MissileClass::SamShort), 1);
        assert_eq!(MissileClass::parse("GenericCruise"), Some(MissileClass::GenericCruise));
        let _ = Formation::new(0, UnitKind::AirDefense, IVec2::ZERO);
    }
}
