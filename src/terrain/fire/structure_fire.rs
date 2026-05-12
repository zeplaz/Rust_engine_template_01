//! Burn profiles for **structures** and volatile sites (`base_fire_sim.md` §4).

use super::fuel::FuelMaterialKind;

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
