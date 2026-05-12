//! Layered vegetation fuel (ground / shrub / canopy) — see `base_fire_sim.md` §2.

use super::fuel::FuelMaterialKind;

#[derive(Clone, Copy, Debug)]
pub struct VegetationFuelLayer {
    pub live_biomass: f32,
    pub dead_biomass: f32,
    pub moisture: f32,
    pub ignition_bias: f32,
    pub fuel_kind: FuelMaterialKind,
}

impl Default for VegetationFuelLayer {
    fn default() -> Self {
        Self {
            live_biomass: 0.0,
            dead_biomass: 0.0,
            moisture: 0.45,
            ignition_bias: 0.5,
            fuel_kind: FuelMaterialKind::Grass,
        }
    }
}

#[inline]
pub fn layer_fuel_mass(layer: &VegetationFuelLayer) -> f32 {
    (layer.live_biomass + layer.dead_biomass).max(0.0)
}
