//! Meso-scale **vegetation structure** — continuous fields between terrain tags and individual trees.
//!
//! CPU-authoritative scalars per chunk (subcell SoA is a future step). Drives fire fuel synthesis,
//! preview tinting, logistics concealment hooks, and succession-ish recovery after disturbance.

use bevy::prelude::*;

use super::ChunkEcology;
use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::weather::ChunkWeather;
use crate::terrain::biome::BiomeWeights;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// High-level vegetation community — derived from structure fields + climate, **not** a terrain family.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum VegetationStructure {
    Grass,
    Shrubland,
    SparseForest,
    MixedForest,
    DenseForest,
    OldGrowth,
    SwampForest,
    BurnScar,
    Regrowth,
}

/// Post-disturbance recovery stage for economy / recon / GPU growth passes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EcologicalSuccessionStage {
    Bare,
    Grass,
    Shrub,
    YoungForest,
    MatureForest,
    OldGrowth,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct VegetationField {
    pub canopy_density: f32,
    pub understory_density: f32,
    pub ground_fuel: f32,
    pub dryness: f32,
    pub fuel_load: f32,
    pub old_growth: f32,
    pub fragmentation: f32,
    pub smoke_absorption: f32,
    pub concealment: f32,
    pub burn_severity: f32,
    pub regrowth_stage: f32,
}

impl Default for VegetationField {
    fn default() -> Self {
        Self {
            canopy_density: 0.28,
            understory_density: 0.22,
            ground_fuel: 0.25,
            dryness: 0.35,
            fuel_load: 0.3,
            old_growth: 0.12,
            fragmentation: 0.4,
            smoke_absorption: 0.18,
            concealment: 0.2,
            burn_severity: 0.0,
            regrowth_stage: 0.45,
        }
    }
}

fn mean_field(n: usize, v: &[f32]) -> Option<f32> {
    if v.len() != n || n == 0 {
        return None;
    }
    Some(v.iter().sum::<f32>() / n as f32)
}

fn mean_biome_weights(matrix: &ChunkCellMatrix) -> Option<BiomeWeights> {
    let n = (matrix.size.x * matrix.size.y) as usize;
    if matrix.weights.len() != n || n == 0 {
        return None;
    }
    let mut acc = BiomeWeights::default();
    for w in &matrix.weights {
        acc.marine += w.marine;
        acc.coastal += w.coastal;
        acc.arid += w.arid;
        acc.temperate += w.temperate;
        acc.boreal += w.boreal;
        acc.alpine += w.alpine;
        acc.wetland += w.wetland;
    }
    let c = n as f32;
    acc.marine /= c;
    acc.coastal /= c;
    acc.arid /= c;
    acc.temperate /= c;
    acc.boreal /= c;
    acc.alpine /= c;
    acc.wetland /= c;
    Some(acc.normalize())
}

/// Deterministic tick slice (tests + single integration site).
pub fn integrate_vegetation_field_step(
    dt_e: f32,
    matrix_opt: Option<&ChunkCellMatrix>,
    eco: &ChunkEcology,
    wx: &ChunkWeather,
    heat: f32,
    veg: &mut VegetationField,
) {
    if dt_e <= 0.0 {
        return;
    }

    let heat_c = heat.clamp(0.0, 1.0);
    let (mean_m, wetland_w) = match matrix_opt {
        Some(matrix) => {
            let n = (matrix.size.x * matrix.size.y) as usize;
            let m = mean_field(n, &matrix.moisture).unwrap_or(0.35);
            let w = mean_biome_weights(matrix).map(|b| b.wetland).unwrap_or(0.0);
            (m, w)
        }
        None => (0.35, 0.0),
    };

    let moisture_signal = mean_m * 0.55 + wx.soil_moisture * 0.45;
    let dryness_target = (0.62 - moisture_signal).max(0.0).min(1.0)
        * (1.0 - wx.rain_intensity * 0.75)
        * (0.85 + wx.wind_speed * 0.12);
    let dk = (0.22 * dt_e).min(0.22);
    veg.dryness = veg.dryness * (1.0 - dk) + dryness_target * dk;

    veg.burn_severity = (veg.burn_severity * (1.0 - 0.35 * dt_e) + heat_c * (0.55 + veg.dryness * 0.35) * dt_e * 1.25)
        .clamp(0.0, 1.0);

    let canopy_target = (eco.biomass * 0.92 + eco.shade_factor * 0.35).clamp(0.0, 1.0);
    let ck = (0.16 * dt_e).min(0.28);
    veg.canopy_density = veg.canopy_density * (1.0 - ck) + canopy_target * ck;
    veg.canopy_density = (veg.canopy_density - veg.burn_severity * 0.04 * dt_e * 18.0).clamp(0.0, 1.0);

    let understory_target = (veg.canopy_density * 0.65 * (1.0 - veg.burn_severity * 0.85) + eco.biomass * 0.15).clamp(0.0, 1.0);
    veg.understory_density = veg.understory_density * (1.0 - ck) + understory_target * ck;

    veg.ground_fuel = ((veg.understory_density * 0.5 + veg.dryness * 0.45) * (1.0 + wetland_w * 0.35))
        .clamp(0.0, 1.0);

    veg.fuel_load = (veg.ground_fuel * 0.55 + veg.canopy_density * 0.45 * (1.0 + veg.dryness * 0.4)).clamp(0.0, 1.0);

    let og_target = (eco.biomass * eco.regrowth_rate * 1.2).min(1.0);
    let ok = (0.05 * dt_e).min(0.12);
    veg.old_growth = veg.old_growth * (1.0 - ok) + og_target * ok;
    veg.old_growth *= (1.0 - veg.burn_severity * 0.06 * dt_e * 10.0).clamp(0.65, 1.0);

    veg.fragmentation = (0.35 + (1.0 - eco.root_strength) * 0.4).clamp(0.0, 1.0);

    veg.smoke_absorption = (veg.canopy_density * 0.75 + veg.understory_density * 0.35 + wx.fog_density * 0.4)
        .clamp(0.0, 1.0);

    veg.concealment = ((veg.canopy_density * 0.55 + veg.understory_density * 0.65) * (1.0 - wx.snow_depth * 0.35))
        .clamp(0.0, 1.0);

    let regrow_drive = (eco.regrowth_rate * moisture_signal * (1.0 - heat_c * 1.1).max(0.0)).clamp(0.0, 0.85);
    veg.regrowth_stage = (veg.regrowth_stage + regrow_drive * dt_e * 0.35 - veg.burn_severity * dt_e * 0.22)
        .clamp(0.0, 1.0);
}

pub fn succession_stage_from_vegetation(veg: &VegetationField, eco: &ChunkEcology) -> EcologicalSuccessionStage {
    if veg.burn_severity > 0.55 && veg.canopy_density < 0.22 {
        return EcologicalSuccessionStage::Bare;
    }
    if veg.regrowth_stage < 0.28 || (veg.burn_severity > 0.2 && veg.canopy_density < 0.35) {
        return EcologicalSuccessionStage::Grass;
    }
    if veg.regrowth_stage < 0.48 && eco.biomass < 0.42 {
        return EcologicalSuccessionStage::Shrub;
    }
    if eco.biomass < 0.58 || veg.old_growth < 0.22 {
        return EcologicalSuccessionStage::YoungForest;
    }
    if veg.old_growth < 0.55 {
        return EcologicalSuccessionStage::MatureForest;
    }
    EcologicalSuccessionStage::OldGrowth
}

pub fn derive_vegetation_structure(
    veg: &VegetationField,
    eco: &ChunkEcology,
    wetland_weight: f32,
    mean_moisture: f32,
) -> VegetationStructure {
    let w = wetland_weight.clamp(0.0, 1.0);
    if veg.burn_severity > 0.45 && veg.canopy_density < 0.3 {
        return VegetationStructure::BurnScar;
    }
    if veg.regrowth_stage < 0.38 && veg.burn_severity > 0.12 {
        return VegetationStructure::Regrowth;
    }
    if w > 0.38 && mean_moisture > 0.45 {
        return VegetationStructure::SwampForest;
    }
    if veg.old_growth > 0.52 && eco.biomass > 0.55 {
        return VegetationStructure::OldGrowth;
    }
    if veg.canopy_density > 0.72 {
        return VegetationStructure::DenseForest;
    }
    if veg.canopy_density > 0.48 {
        return VegetationStructure::MixedForest;
    }
    if veg.canopy_density > 0.22 {
        return VegetationStructure::SparseForest;
    }
    if veg.understory_density > 0.35 {
        return VegetationStructure::Shrubland;
    }
    VegetationStructure::Grass
}

pub(crate) fn spawn_vegetation_field_on_new_chunk(
    mut commands: Commands,
    q: Query<Entity, (Added<Chunk>, Without<VegetationField>)>,
) {
    for e in &q {
        commands.entity(e).insert(VegetationField::default());
    }
}

pub(crate) fn vegetation_field_tick(
    ctrl: Res<crate::systems::sim_control::SimControlState>,
    time: Res<Time>,
    mut q: Query<(
        Option<&ChunkCellMatrix>,
        &ChunkEcology,
        &ChunkWeather,
        &ChunkSimLod,
        Option<&crate::systems::fire::ChunkSurfaceFire>,
        &mut VegetationField,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }

    for (matrix_opt, eco, wx, lod, fire_opt, mut veg) in &mut q {
        let lod_s = lod.dt_scale();
        let dt_e = dt * lod_s;
        let heat = fire_opt.map(|f| f.heat).unwrap_or(0.0);
        integrate_vegetation_field_step(dt_e, matrix_opt, eco, wx, heat, &mut veg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_heat_raises_burn_and_drops_canopy() {
        let eco = ChunkEcology {
            biomass: 0.65,
            fire_risk: 0.2,
            regrowth_rate: 0.15,
            moisture_need: 0.4,
            root_strength: 0.4,
            shade_factor: 0.45,
            harvest_value: 0.2,
            disease_resistance: 0.5,
        };
        let wx = ChunkWeather {
            rain_intensity: 0.05,
            fog_density: 0.02,
            snow_depth: 0.0,
            wind_speed: 0.35,
            lightning_risk: 0.0,
            visibility_factor: 0.95,
            soil_moisture: 0.35,
        };
        let mut veg = VegetationField {
            canopy_density: 0.7,
            ..Default::default()
        };
        for _ in 0..120 {
            integrate_vegetation_field_step(0.05, None, &eco, &wx, 0.9, &mut veg);
        }
        assert!(veg.burn_severity > 0.2, "burn {}", veg.burn_severity);
        assert!(veg.canopy_density < 0.65);
    }

    #[test]
    fn burn_scar_structure_when_severity_high() {
        let eco = ChunkEcology::default();
        let veg = VegetationField {
            burn_severity: 0.72,
            canopy_density: 0.12,
            regrowth_stage: 0.1,
            ..Default::default()
        };
        assert_eq!(
            derive_vegetation_structure(&veg, &eco, 0.0, 0.3),
            VegetationStructure::BurnScar
        );
    }
}
