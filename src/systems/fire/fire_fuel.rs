//! **Fire fuel** meso-layer — bridges [`VegetationField`](crate::systems::ecology::VegetationField), weather,
//! and macro [`ChunkEcology`](crate::systems::ecology::ChunkEcology) to spread-ready scalars (CPU authority).
//!
//! Surface fire systems may consume these fields in a later pass; GPU fire growth preview aligns to the same schema.

use bevy::prelude::*;

use crate::systems::ecology::{ChunkEcology, VegetationField};
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::Chunk;

#[derive(Component, Clone, Copy, Debug)]
pub struct FireFuelField {
    pub surface_fuel: f32,
    pub ladder_fuel: f32,
    pub canopy_fuel: f32,
    pub ignition_resistance: f32,
    pub moisture_retention: f32,
    pub ember_spread_factor: f32,
}

impl Default for FireFuelField {
    fn default() -> Self {
        Self {
            surface_fuel: 0.3,
            ladder_fuel: 0.18,
            canopy_fuel: 0.22,
            ignition_resistance: 0.35,
            moisture_retention: 0.42,
            ember_spread_factor: 0.25,
        }
    }
}

/// Recompute fuel scalars from vegetation + ecology + weather (pure; also used by [`fire_fuel_field_tick`]).
pub fn derive_fire_fuel_from_vegetation(
    veg: &VegetationField,
    wx: &ChunkWeather,
    eco: &ChunkEcology,
) -> FireFuelField {
    let soil = wx.soil_moisture.clamp(0.0, 1.0);
    let og = veg.old_growth.clamp(0.0, 1.0);
    let surface_fuel =
        (veg.ground_fuel * (0.55 + og * 0.3) + veg.understory_density * 0.22).clamp(0.0, 1.0);
    let ladder_fuel =
        (veg.understory_density * veg.canopy_density * (0.85 + og * 0.45)).clamp(0.0, 1.0);
    let canopy_fuel = (veg.canopy_density
        * veg.fuel_load
        * (0.35 + og * 0.55 + veg.dryness * 0.25))
        .clamp(0.0, 1.0);

    let moisture_retention =
        (soil * 0.55 + (1.0 - veg.dryness) * 0.45 + wx.rain_intensity * 0.35).clamp(0.0, 1.0);
    let ignition_resistance = ((moisture_retention * 0.55 + eco.root_strength * 0.25
        + (1.0 - eco.fire_risk) * 0.25)
        * (1.0 + wx.snow_depth * 0.25))
    .clamp(0.0, 1.0);

    let ember_spread_factor = (veg.dryness * 0.45 + wx.wind_speed * 0.4 + ladder_fuel * 0.35 + eco.fire_risk * 0.2)
        .clamp(0.0, 1.0);

    FireFuelField {
        surface_fuel,
        ladder_fuel,
        canopy_fuel,
        ignition_resistance,
        moisture_retention,
        ember_spread_factor,
    }
}

pub(crate) fn spawn_fire_fuel_field_on_new_chunk(
    mut commands: Commands,
    q: Query<Entity, (Added<Chunk>, Without<FireFuelField>)>,
) {
    for e in &q {
        commands.entity(e).insert(FireFuelField::default());
    }
}

pub(crate) fn fire_fuel_field_tick(mut q: Query<(&VegetationField, &ChunkWeather, &ChunkEcology, &mut FireFuelField)>) {
    for (veg, wx, eco, mut fuel) in &mut q {
        *fuel = derive_fire_fuel_from_vegetation(veg, wx, eco);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dryness_increases_ember_spread() {
        let eco = ChunkEcology::default();
        let wx = ChunkWeather {
            wind_speed: 0.4,
            rain_intensity: 0.0,
            soil_moisture: 0.3,
            ..Default::default()
        };
        let veg_dry = VegetationField {
            dryness: 0.85,
            canopy_density: 0.5,
            understory_density: 0.4,
            ground_fuel: 0.5,
            fuel_load: 0.55,
            ..Default::default()
        };
        let veg_wet = VegetationField {
            dryness: 0.15,
            ..veg_dry
        };
        let f_dry = derive_fire_fuel_from_vegetation(&veg_dry, &wx, &eco);
        let f_wet = derive_fire_fuel_from_vegetation(&veg_wet, &wx, &eco);
        assert!(f_dry.ember_spread_factor > f_wet.ember_spread_factor);
    }
}
