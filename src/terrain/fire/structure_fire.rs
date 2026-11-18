//! Burn profiles for **structures** and volatile sites (`base_fire_sim.md` §4).
//! FIRE-ENTITY-1: archetype → profile + unit/truck susceptibility coeffs.

use super::fuel::fuel_material_def;
use super::fuel_layer::FuelLayer;

use super::fuel::FuelMaterialKind;
use crate::combat::balance::UnitKind;
use crate::strategic::SiteArchetype;

#[derive(Clone, Copy, Debug)]
pub struct StructureFireProfile {
    pub primary_material: FuelMaterialKind,
    pub fuel_load: f32,
    pub ignition_resistance: f32,
    pub collapse_threshold: f32,
    pub internal_pressure_risk: f32,
    pub emits_toxic_smoke: bool,
    pub explosion_chain: bool,
}

impl StructureFireProfile {
    /// Row for [`crate::systems::fire::ChunkFuelProfile::structure_overlay`] / atmosphere coupling.
    pub fn to_fuel_layer_overlay(self) -> FuelLayer {
        let d = fuel_material_def(self.primary_material);
        let load = self.fuel_load.clamp(0.0, 1.0);
        let tox = if self.emits_toxic_smoke {
            (d.toxic_output + 0.25).min(1.0)
        } else {
            (d.toxic_output * 0.55).min(1.0)
        };
        let volatility = (d.explosive_force * 0.72
            + load * 0.28
            + self.internal_pressure_risk * 0.22
            + if self.explosion_chain { 0.22 } else { 0.0 })
        .min(1.0);
        let ember = if self.explosion_chain {
            0.72f32
        } else if d.thermal_runaway {
            0.55
        } else {
            (load * 0.35 + d.structural_damage_rate * 0.4).min(1.0)
        };
        FuelLayer {
            surface_fuel: (load * 0.9 + d.smoke_density * 0.12).min(1.0),
            shrub_fuel: load * 0.22,
            canopy_fuel: load * 0.12,
            moisture: (1.0 - self.ignition_resistance.clamp(0.0, 1.0)).max(0.04),
            volatility,
            toxic_smoke: tox,
            burn_temperature: d.burn_energy.clamp(0.0, 1.0),
            ember_generation: ember,
        }
    }
}

pub fn fuel_depot_profile() -> StructureFireProfile {
    StructureFireProfile {
        primary_material: FuelMaterialKind::Gasoline,
        fuel_load: 1.0,
        ignition_resistance: 0.1,
        collapse_threshold: 0.2,
        internal_pressure_risk: 1.0,
        emits_toxic_smoke: true,
        explosion_chain: true,
    }
}

pub fn ammo_dump_profile() -> StructureFireProfile {
    StructureFireProfile {
        primary_material: FuelMaterialKind::Ammunition,
        fuel_load: 0.9,
        ignition_resistance: 0.2,
        collapse_threshold: 0.1,
        internal_pressure_risk: 1.0,
        emits_toxic_smoke: false,
        explosion_chain: true,
    }
}

pub fn lithium_battery_warehouse() -> StructureFireProfile {
    StructureFireProfile {
        primary_material: FuelMaterialKind::BatteryLithium,
        fuel_load: 1.0,
        ignition_resistance: 0.05,
        collapse_threshold: 0.15,
        internal_pressure_risk: 0.4,
        emits_toxic_smoke: true,
        explosion_chain: false,
    }
}

pub fn wood_structure_profile() -> StructureFireProfile {
    StructureFireProfile {
        primary_material: FuelMaterialKind::WoodStructure,
        fuel_load: 0.75,
        ignition_resistance: 0.25,
        collapse_threshold: 0.35,
        internal_pressure_risk: 0.1,
        emits_toxic_smoke: false,
        explosion_chain: false,
    }
}

pub fn concrete_structure_profile() -> StructureFireProfile {
    StructureFireProfile {
        primary_material: FuelMaterialKind::ConcreteStructure,
        fuel_load: 0.35,
        ignition_resistance: 0.72,
        collapse_threshold: 0.12,
        internal_pressure_risk: 0.05,
        emits_toxic_smoke: false,
        explosion_chain: false,
    }
}

pub fn steel_structure_profile() -> StructureFireProfile {
    StructureFireProfile {
        primary_material: FuelMaterialKind::SteelStructure,
        fuel_load: 0.45,
        ignition_resistance: 0.55,
        collapse_threshold: 0.18,
        internal_pressure_risk: 0.15,
        emits_toxic_smoke: true,
        explosion_chain: false,
    }
}

/// Map construction archetype → burn profile (catalog tags can refine later).
#[must_use]
pub fn profile_for_site_archetype(arch: SiteArchetype) -> StructureFireProfile {
    match arch {
        SiteArchetype::FuelDepot => fuel_depot_profile(),
        SiteArchetype::MilitaryBase | SiteArchetype::BunkerComplex | SiteArchetype::DefensiveWall => {
            concrete_structure_profile()
        }
        SiteArchetype::Factory
        | SiteArchetype::PowerPlant
        | SiteArchetype::RailDepot
        | SiteArchetype::Dam => steel_structure_profile(),
        SiteArchetype::CivilHousing
        | SiteArchetype::WaterPlant
        | SiteArchetype::RadarSite
        | SiteArchetype::SensorPost
        | SiteArchetype::TrenchLine
        | SiteArchetype::DragonTeeth
        | SiteArchetype::Minefield => wood_structure_profile(),
    }
}

/// How hard fire attrits this unit kind (0 = nearly immune, 1 = soft target).
#[must_use]
pub fn unit_fire_susceptibility(kind: UnitKind) -> f32 {
    match kind {
        UnitKind::Infantry => 0.85,
        UnitKind::Recon => 0.75,
        UnitKind::Engineer => 0.7,
        UnitKind::Mechanized => 0.45,
        UnitKind::Artillery => 0.4,
        UnitKind::AirDefense => 0.4,
        UnitKind::Armor => 0.22,
    }
}

/// Haul truck fire susceptibility (default class).
pub const TRUCK_FIRE_SUSCEPTIBILITY: f32 = 0.55;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn depot_overlay_is_hot_and_toxic() {
        let row = fuel_depot_profile().to_fuel_layer_overlay();
        assert!(row.toxic_smoke > 0.4);
        assert!(row.volatility > 0.5);
    }

    #[test]
    fn battery_overlay_prefers_thermal_tone() {
        let row = lithium_battery_warehouse().to_fuel_layer_overlay();
        assert!(row.toxic_smoke > 0.2);
    }

    #[test]
    fn concrete_hq_resists_more_than_housing() {
        let hq = profile_for_site_archetype(SiteArchetype::MilitaryBase);
        let house = profile_for_site_archetype(SiteArchetype::CivilHousing);
        assert!(hq.ignition_resistance > house.ignition_resistance);
        assert!(hq.collapse_threshold < house.collapse_threshold);
    }

    #[test]
    fn armor_less_susceptible_than_infantry() {
        assert!(unit_fire_susceptibility(UnitKind::Armor) < unit_fire_susceptibility(UnitKind::Infantry));
    }
}
