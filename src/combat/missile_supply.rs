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
use crate::economy::logistics::orders::LogisticsOrderRequest;
use crate::combat::ballistics::MissileBallisticWind;
use crate::systems::weather::{
    evaluate_grounding, AirOpsProfileId, AirOpsTuning, WeatherPollingStations,
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

/// Aggregated missile BOM demand (dump). Sole writer: [`super::toe_demand::publish_toe_cargo_demands_system`].
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

/// Typed sink: top up [`MissilePackage`] toward TOE from a shared pool (**CS-6**).
/// Scheduled only via [`super::toe_refill::refill_toe_from_front_system`].
pub fn refill_missile_packages_sink(
    toe: &CapabilityToeRegistry,
    pool: &mut super::front_stock_pool::FrontStockPool,
    packages: &mut Query<(Entity, &Formation, &mut MissilePackage)>,
    requests: &mut MessageWriter<LogisticsOrderRequest>,
) {
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
                if !pool.try_consume_bom_unit(slot, &row.lines, requests) {
                    break;
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
    /// Station wind at launch (MOVE-09); `None` when weather stations are not ready.
    pub wind: Option<MissileBallisticWind>,
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
    stations: Option<&WeatherPollingStations>,
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

        let sample = stations
            .filter(|s| s.ready)
            .map(|s| s.sample_at_tile(c))
            .unwrap_or_default();
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
        let wind = stations.filter(|s| s.ready).map(|_| MissileBallisticWind {
            sample,
            drift_coef: profile.missile_drift_coef,
            max_wind_mps: profile.max_missile_wind_mps,
        });
        launches.push(MissileSortieLaunch {
            faction_slot: z.faction_slot,
            origin_xz,
            target_xz,
            wind,
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
