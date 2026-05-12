//! Unified scalar **fuel row** for smoke tint, flame proxy, and hazard presets.
//! Per-stratum detail stays in [`VegetationFuelLayer`]; this type is the shared shape for
//! wildland aggregates, industrial presets, and future structure overlays (`base_fire2_smoke.md` §19).

use super::fuel::fuel_material_def;
use super::vegetation_fuel::{layer_fuel_mass, VegetationFuelLayer};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FuelLayer {
    pub surface_fuel: f32,
    pub shrub_fuel: f32,
    pub canopy_fuel: f32,
    pub moisture: f32,
    pub volatility: f32,
    pub toxic_smoke: f32,
    pub burn_temperature: f32,
    pub ember_generation: f32,
}

impl Default for FuelLayer {
    fn default() -> Self {
        Self {
            surface_fuel: 0.0,
            shrub_fuel: 0.0,
            canopy_fuel: 0.0,
            moisture: 1.0,
            volatility: 0.0,
            toxic_smoke: 0.0,
            burn_temperature: 0.0,
            ember_generation: 0.0,
        }
    }
}

impl FuelLayer {
    /// Temperate closed-canopy wildland (design reference values).
    pub const fn forest() -> Self {
        Self {
            surface_fuel: 0.8,
            shrub_fuel: 0.6,
            canopy_fuel: 0.9,
            moisture: 0.4,
            volatility: 0.5,
            toxic_smoke: 0.1,
            burn_temperature: 0.7,
            ember_generation: 0.9,
        }
    }

    pub const fn fuel_dump() -> Self {
        Self {
            surface_fuel: 1.0,
            shrub_fuel: 0.0,
            canopy_fuel: 0.0,
            moisture: 0.0,
            volatility: 1.0,
            toxic_smoke: 0.9,
            burn_temperature: 1.0,
            ember_generation: 0.8,
        }
    }

    pub const fn battery_facility() -> Self {
        Self {
            surface_fuel: 0.2,
            shrub_fuel: 0.0,
            canopy_fuel: 0.0,
            moisture: 0.0,
            volatility: 0.95,
            toxic_smoke: 1.0,
            burn_temperature: 1.0,
            ember_generation: 0.2,
        }
    }

    pub const fn concrete_building() -> Self {
        Self {
            surface_fuel: 0.1,
            shrub_fuel: 0.0,
            canopy_fuel: 0.0,
            moisture: 0.1,
            volatility: 0.1,
            toxic_smoke: 0.3,
            burn_temperature: 0.2,
            ember_generation: 0.0,
        }
    }

    /// Aggregate grass / brush / canopy strata into one row (mass-weighted where noted).
    pub fn from_vegetation_strata(
        grass: &VegetationFuelLayer,
        brush: &VegetationFuelLayer,
        canopy: &VegetationFuelLayer,
    ) -> Self {
        let mg = layer_fuel_mass(grass).max(0.0);
        let mb = layer_fuel_mass(brush).max(0.0);
        let mc = layer_fuel_mass(canopy).max(0.0);
        let wsum = mg + mb + mc;
        let (wg, wb, wc) = if wsum <= 1e-6 {
            (1.0f32 / 3.0, 1.0 / 3.0, 1.0 / 3.0)
        } else {
            (mg / wsum, mb / wsum, mc / wsum)
        };

        let surface_fuel = mg.min(1.0);
        let shrub_fuel = mb.min(1.0);
        let canopy_fuel = mc.min(1.0);
        let moisture = (grass.moisture * wg + brush.moisture * wb + canopy.moisture * wc).clamp(0.0, 1.0);
        let volatility = (grass.ignition_bias * wg + brush.ignition_bias * wb + canopy.ignition_bias * wc)
            .clamp(0.0, 1.0);

        let da = fuel_material_def(grass.fuel_kind);
        let db = fuel_material_def(brush.fuel_kind);
        let dc = fuel_material_def(canopy.fuel_kind);
        let toxic_smoke = (da.toxic_output * wg + db.toxic_output * wb + dc.toxic_output * wc).clamp(0.0, 1.0);
        let burn_temperature =
            (da.burn_energy * wg + db.burn_energy * wb + dc.burn_energy * wc).clamp(0.0, 1.0);

        let dryness = (1.0 - moisture).clamp(0.0, 1.0);
        let ember_generation = (dryness * 0.55 + wc * canopy_fuel * 0.85 + volatility * 0.25).clamp(0.0, 1.0);

        Self {
            surface_fuel,
            shrub_fuel,
            canopy_fuel,
            moisture,
            volatility,
            toxic_smoke,
            burn_temperature,
            ember_generation,
        }
    }

    /// Merge wildland aggregate fuel with a **structure / site** overlay (urban, depot, battery hall).
    /// Uses elementwise max for load and hazard scalars; moisture uses the **drier** of the two rows.
    #[inline]
    pub fn merge_structure_overlay(base: Self, structure: Self) -> Self {
        Self {
            surface_fuel: base.surface_fuel.max(structure.surface_fuel),
            shrub_fuel: base.shrub_fuel.max(structure.shrub_fuel),
            canopy_fuel: base.canopy_fuel.max(structure.canopy_fuel),
            moisture: base.moisture.min(structure.moisture),
            volatility: base.volatility.max(structure.volatility),
            toxic_smoke: base.toxic_smoke.max(structure.toxic_smoke),
            burn_temperature: base.burn_temperature.max(structure.burn_temperature),
            ember_generation: base.ember_generation.max(structure.ember_generation),
        }
    }

    /// Hint for volumetric flame column scale (normalized units).
    #[inline]
    pub fn visual_fire_height(&self) -> f32 {
        (self.canopy_fuel * 8.0 + self.volatility * 4.0).min(24.0)
    }

    /// Scalar ember rate proxy before wind (caller multiplies wind).
    #[inline]
    pub fn ember_rate_base(&self) -> f32 {
        (self.ember_generation * (0.35 + self.surface_fuel * 0.65)).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::fire::FuelMaterialKind;

    #[test]
    fn presets_match_design_examples() {
        let f = FuelLayer::forest();
        assert!((f.surface_fuel - 0.8).abs() < 1e-6);
        assert!((f.toxic_smoke - 0.1).abs() < 1e-6);

        let d = FuelLayer::fuel_dump();
        assert!((d.surface_fuel - 1.0).abs() < 1e-6);
        assert!((d.toxic_smoke - 0.9).abs() < 1e-6);

        let bat = FuelLayer::battery_facility();
        assert!(bat.volatility > 0.9 && bat.toxic_smoke >= 0.99);

        let c = FuelLayer::concrete_building();
        assert!(c.ember_generation < 0.05 && c.canopy_fuel < 0.2);
    }

    #[test]
    fn from_vegetation_respects_mass_weights() {
        let dry_grass = VegetationFuelLayer {
            live_biomass: 0.4,
            dead_biomass: 0.35,
            moisture: 0.15,
            ignition_bias: 0.85,
            fuel_kind: FuelMaterialKind::Grass,
        };
        let wet_brush = VegetationFuelLayer {
            live_biomass: 0.05,
            dead_biomass: 0.02,
            moisture: 0.75,
            ignition_bias: 0.35,
            fuel_kind: FuelMaterialKind::Brush,
        };
        let canopy = VegetationFuelLayer::default();
        let row = FuelLayer::from_vegetation_strata(&dry_grass, &wet_brush, &canopy);
        assert!(row.surface_fuel > row.shrub_fuel);
        assert!(row.ember_generation > 0.2);
    }

    #[test]
    fn merge_structure_overlay_takes_hazard_max() {
        let wild = FuelLayer::forest();
        let urban = FuelLayer::concrete_building();
        let m = FuelLayer::merge_structure_overlay(wild, urban);
        assert!(m.toxic_smoke >= wild.toxic_smoke.max(urban.toxic_smoke) - 1e-5);
        assert!(m.moisture <= wild.moisture.min(urban.moisture) + 1e-5);
    }
}
