//! Combustion **multipliers** — `ChunkEcology::fire_risk` is one upstream term; layer fuels set burn/smoke/toxic character (`base_fire_sim.md`).
//!
//! **F1 fuel gate:** ambient spark and crown-class boost require wildland mass and (for crown) old-growth.

use crate::systems::fire::{chunk_fuel_profile::ChunkFuelProfile, FireFuelField};
use crate::terrain::fire::{fuel_material_def, layer_fuel_mass, VegetationFuelLayer};

/// Minimum grass+brush+canopy mass before ambient spark is non-zero.
pub const MIN_WILDLAND_FUEL_MASS: f32 = 0.18;

/// Old-growth scalar at or above this unlocks crown-class intensity boost.
pub const OLD_GROWTH_CROWN_THRESHOLD: f32 = 0.32;

/// Per-cell fuel reservoir below this blocks neighbor heat diffusion.
pub const MIN_CELL_FUEL_FOR_SPREAD: f32 = 0.06;

/// Default per-cell fuel seed when overlay spawns (not free-burn `1.0`).
pub const DEFAULT_CELL_FUEL_SEED: f32 = 0.06;

#[inline]
pub fn profile_total_fuel_mass(profile: &ChunkFuelProfile) -> f32 {
    layer_fuel_mass(&profile.grass)
        + layer_fuel_mass(&profile.brush)
        + layer_fuel_mass(&profile.canopy)
}

/// Maps profile mass into per-cell fuel reservoir cap `[0.02, 0.95]`.
#[inline]
pub fn profile_cell_fuel_seed(profile: &ChunkFuelProfile) -> f32 {
    let m = profile.wildland_fuel_mass.max(profile_total_fuel_mass(profile));
    (m * 0.72 + profile.canopy.dead_biomass * 0.12).clamp(0.02, 0.95)
}

/// `[0, 1]` multiplier — zero when wildland mass is below ignition threshold.
#[inline]
pub fn fuel_ignition_gate(wildland_mass: f32) -> f32 {
    if wildland_mass < MIN_WILDLAND_FUEL_MASS {
        return 0.0;
    }
    ((wildland_mass - MIN_WILDLAND_FUEL_MASS) / (1.0 - MIN_WILDLAND_FUEL_MASS)).clamp(0.0, 1.0)
}

/// Profile spark with F1 mass gate (old-growth affects profile layers, not this scalar directly).
#[inline]
pub fn profile_spark_multiplier_gated(profile: &ChunkFuelProfile) -> f32 {
    profile_spark_multiplier(profile) * fuel_ignition_gate(profile.wildland_fuel_mass)
}

/// Crown boost applies only when old-growth supports ladder + canopy involvement.
#[inline]
pub fn crown_boost_for_old_growth(old_growth: f32, fuel: &FireFuelField) -> f32 {
    if old_growth < OLD_GROWTH_CROWN_THRESHOLD {
        1.0
    } else {
        crown_fire_intensity_boost(fuel)
    }
}

/// Macro ecology `fire_risk` contribution to sparkle/spread (never the sole driver once fuels exist).
#[inline]
pub fn ecology_fire_risk_spark_factor(fire_risk: f32) -> f32 {
    0.55 + 0.45 * fire_risk.clamp(0.0, 1.0)
}

#[inline]
pub fn layer_ignite_scale(layer: &VegetationFuelLayer) -> f32 {
    let m = layer.moisture.clamp(0.05, 1.0);
    let bias = layer.ignition_bias.clamp(0.0, 1.0);
    (bias * 0.55 + (1.0 - m) * 0.45).clamp(0.08, 1.5)
}

/// Weighted `burn_energy` from grass / brush / canopy profile.
pub fn profile_weighted_burn_energy(profile: &ChunkFuelProfile) -> f32 {
    let wg = layer_fuel_mass(&profile.grass);
    let wb = layer_fuel_mass(&profile.brush);
    let wc = layer_fuel_mass(&profile.canopy);
    let wsum = (wg + wb + wc).max(1e-6);
    let eg = fuel_material_def(profile.grass.fuel_kind).burn_energy * wg;
    let eb = fuel_material_def(profile.brush.fuel_kind).burn_energy * wb;
    let ec = fuel_material_def(profile.canopy.fuel_kind).burn_energy * wc;
    ((eg + eb + ec) / wsum).clamp(0.05, 1.2)
}

pub fn profile_spark_multiplier(profile: &ChunkFuelProfile) -> f32 {
    let wg = layer_fuel_mass(&profile.grass);
    let wb = layer_fuel_mass(&profile.brush);
    let wc = layer_fuel_mass(&profile.canopy);
    let wsum = (wg + wb + wc).max(1e-6);
    let g = layer_ignite_scale(&profile.grass) * wg;
    let b = layer_ignite_scale(&profile.brush) * wb;
    let c = layer_ignite_scale(&profile.canopy) * wc;
    let ig = (g + b + c) / wsum;
    profile_weighted_burn_energy(profile) * ig
}

pub fn profile_weighted_smoke_toxic_explosion(profile: &ChunkFuelProfile) -> (f32, f32, f32) {
    let wg = layer_fuel_mass(&profile.grass);
    let wb = layer_fuel_mass(&profile.brush);
    let wc = layer_fuel_mass(&profile.canopy);
    let wsum = (wg + wb + wc).max(1e-6);

    let mut sm = 0f32;
    let mut tx = 0f32;
    let mut ex = 0f32;
    for (w, layer) in [
        (wg, &profile.grass),
        (wb, &profile.brush),
        (wc, &profile.canopy),
    ] {
        if w <= 1e-6 {
            continue;
        }
        let d = fuel_material_def(layer.fuel_kind);
        sm += d.smoke_density * w;
        tx += d.toxic_output * w;
        ex += d.explosive_force * w;
    }
    (sm / wsum, tx / wsum, ex / wsum)
}

/// Crown involvement proxy: ladder + canopy fuels increase intensity beyond surface-only.
#[inline]
pub fn crown_fire_intensity_boost(fuel: &FireFuelField) -> f32 {
    1.0 + fuel.ladder_fuel * 0.55 + fuel.canopy_fuel * 0.85
}

#[cfg(test)]
mod fuel_gate_tests {
    use super::*;
    use crate::systems::ecology::VegetationField;
    use crate::systems::fire::chunk_fuel_profile::chunk_fuel_profile_from_vegetation;

    #[test]
    fn near_empty_vegetation_blocks_ignition_gate() {
        let veg = VegetationField {
            ground_fuel: 0.05,
            canopy_density: 0.05,
            understory_density: 0.05,
            dryness: 0.9,
            old_growth: 0.02,
            fuel_load: 0.05,
            ..Default::default()
        };
        let p = chunk_fuel_profile_from_vegetation(&veg);
        assert_eq!(fuel_ignition_gate(p.wildland_fuel_mass), 0.0);
        assert_eq!(profile_spark_multiplier_gated(&p), 0.0);
    }

    #[test]
    fn old_growth_raises_wildland_mass_vs_sparse() {
        let sparse = chunk_fuel_profile_from_vegetation(&VegetationField {
            old_growth: 0.05,
            ..Default::default()
        });
        let og = chunk_fuel_profile_from_vegetation(&VegetationField {
            old_growth: 0.72,
            canopy_density: 0.65,
            fuel_load: 0.7,
            dryness: 0.55,
            ..Default::default()
        });
        assert!(og.wildland_fuel_mass > sparse.wildland_fuel_mass);
        assert!(fuel_ignition_gate(og.wildland_fuel_mass) > fuel_ignition_gate(sparse.wildland_fuel_mass));
    }
}
