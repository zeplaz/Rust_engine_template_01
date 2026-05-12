//! Burn profiles for **structures** and volatile sites (`base_fire_sim.md` §4).

use super::fuel::fuel_material_def;
use super::fuel_layer::FuelLayer;

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
}
