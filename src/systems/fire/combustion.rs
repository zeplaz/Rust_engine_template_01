//! Combustion **multipliers** — `ChunkEcology::fire_risk` is one upstream term; layer fuels set burn/smoke/toxic character (`base_fire_sim.md`).

use crate::systems::fire::{chunk_fuel_profile::ChunkFuelProfile, FireFuelField};
use crate::terrain::fire::{fuel_material_def, layer_fuel_mass, VegetationFuelLayer};

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
