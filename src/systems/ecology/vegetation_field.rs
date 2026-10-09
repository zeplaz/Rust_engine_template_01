//! Meso-scale **vegetation structure** — continuous fields between terrain tags and individual trees.
//!
//! CPU-authoritative scalars per chunk (subcell SoA is a future step). Drives fire fuel synthesis,
//! preview tinting, logistics concealment hooks, and succession-ish recovery after disturbance.

use bevy::prelude::*;

use super::ChunkEcology;
use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::weather::ChunkWeather;
use crate::terrain::biome::BiomeWeights;
use crate::terrain::generation::{landform_means, Chunk, ChunkCellMatrix};

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

/// Steep ground and ridge-heavy chunks hold less canopy. Hollows (positive curvature) hold more.
///
/// Chunk-mean signed curvature cancels on a closed bowl, so cover uses the hollow mass and the
/// ridge mass separately, plus mean slope.
fn landform_cover_scale(matrix: &ChunkCellMatrix) -> f32 {
    let (slope, _curve) = landform_means(matrix);
    let curve = crate::terrain::generation::compute_profile_curvature(matrix);
    let n = curve.len();
    if n == 0 {
        return 1.0;
    }
    let inv = 1.0 / n as f32;
    let hollow = curve.iter().map(|c| c.max(0.0)).sum::<f32>() * inv;
    let ridge = curve.iter().map(|c| (-c).max(0.0)).sum::<f32>() * inv;
    let steep = (slope / 0.2).clamp(0.0, 1.0);
    (1.0 - steep * 0.5 - (ridge / 0.04).clamp(0.0, 1.0) * 0.12
        + (hollow / 0.04).clamp(0.0, 1.0) * 0.12)
        .clamp(0.4, 1.2)
}

/// Player-visible cover for one cell. Chunk canopy times the landform scale.
#[must_use]
pub fn veg_draw_density(chunk_canopy: f32, cell_scale: f32) -> f32 {
    veg_draw_density_after_impact(chunk_canopy, cell_scale, 0.0)
}

/// Impact lowers the cover the player sees. Impact 0 matches [`veg_draw_density`].
#[must_use]
pub fn veg_draw_density_after_impact(chunk_canopy: f32, cell_scale: f32, impact: f32) -> f32 {
    let base = (chunk_canopy.clamp(0.0, 1.0) * cell_scale.clamp(0.35, 1.35)).clamp(0.0, 1.0);
    (base * (1.0 - impact.clamp(0.0, 1.0) * 0.5)).clamp(0.0, 1.0)
}

/// Snow lies on the canopy. Depth 0 leaves the density unchanged.
#[must_use]
pub fn canopy_after_snow(canopy: f32, snow_depth: f32) -> f32 {
    let snow = snow_depth.clamp(0.0, 1.0);
    (canopy.clamp(0.0, 1.0) * (1.0 - snow * 0.45)).clamp(0.0, 1.0)
}

/// Snow thins the understory less than the canopy. Depth 0 leaves it unchanged.
#[must_use]
pub fn understory_after_snow(understory: f32, snow_depth: f32) -> f32 {
    let snow = snow_depth.clamp(0.0, 1.0);
    (understory.clamp(0.0, 1.0) * (1.0 - snow * 0.25)).clamp(0.0, 1.0)
}

/// Snow slows the regrowth step. Depth 0 leaves the drive unchanged.
#[must_use]
pub fn regrowth_drive_after_snow(drive: f32, snow_depth: f32) -> f32 {
    let snow = snow_depth.clamp(0.0, 1.0);
    drive.max(0.0) * (1.0 - snow * 0.6)
}

/// Burned ground breaks the stand more than the same roots on unburned ground.
#[must_use]
pub fn fragmentation_after_burn(root_strength: f32, burn_severity: f32) -> f32 {
    (0.35 + (1.0 - root_strength.clamp(0.0, 1.0)) * 0.4 + burn_severity.clamp(0.0, 1.0) * 0.2)
        .clamp(0.0, 1.0)
}

/// Heat breaks the stand further. The same amount of snow does not.
#[must_use]
pub fn fragmentation_after_heat(
    root_strength: f32,
    burn_severity: f32,
    heat: f32,
    snow_depth: f32,
) -> f32 {
    let base = fragmentation_after_burn(root_strength, burn_severity);
    let climate = heat.clamp(0.0, 1.0) * 0.12 - snow_depth.clamp(0.0, 1.0) * 0.12;
    (base + climate).clamp(0.0, 1.0)
}

/// Fog hides the stand. Clear air leaves concealment unchanged.
#[must_use]
pub fn concealment_after_fog(concealment: f32, fog_density: f32) -> f32 {
    let fog = fog_density.clamp(0.0, 1.0);
    (concealment.clamp(0.0, 1.0) * (1.0 - fog * 0.35)).clamp(0.0, 1.0)
}

/// Drought thins concealment. Full rain leaves the incoming value unchanged.
#[must_use]
pub fn concealment_after_rain(concealment: f32, rain: f32) -> f32 {
    let rain = rain.clamp(0.0, 1.0);
    (concealment.clamp(0.0, 1.0) * (0.72 + 0.28 * rain)).clamp(0.0, 1.0)
}

/// Heat consumes the surface fuel the stand is showing. Cold ground is unchanged.
#[must_use]
pub fn ground_fuel_after_heat(fuel: f32, heat: f32) -> f32 {
    (fuel.clamp(0.0, 1.0) * (1.0 - heat.clamp(0.0, 1.0) * 0.25)).clamp(0.0, 1.0)
}

/// Rain washes the smoke the stand is holding. Dry air leaves it unchanged.
#[must_use]
pub fn smoke_held_after_rain(smoke: f32, rain: f32) -> f32 {
    (smoke.clamp(0.0, 1.0) * (1.0 - rain.clamp(0.0, 1.0) * 0.5)).clamp(0.0, 1.0)
}

/// Rain lowers the fuel load the stand would pass to a fire. Dry air leaves it unchanged.
#[must_use]
pub fn fuel_load_after_rain(load: f32, rain: f32) -> f32 {
    (load.clamp(0.0, 1.0) * (1.0 - rain.clamp(0.0, 1.0) * 0.4)).clamp(0.0, 1.0)
}

/// Snow lies on the drawn stand. Depth 0 matches [`veg_draw_density`].
#[must_use]
pub fn veg_draw_density_under_snow(chunk_canopy: f32, cell_scale: f32, snow_depth: f32) -> f32 {
    canopy_after_snow(veg_draw_density(chunk_canopy, cell_scale), snow_depth)
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

    let landform = matrix_opt
        .map(landform_cover_scale)
        .unwrap_or(1.0);
    let canopy_target = ((eco.biomass * 0.92 + eco.shade_factor * 0.35) * landform).clamp(0.0, 1.0);
    let ck = (0.16 * dt_e).min(0.28);
    veg.canopy_density = veg.canopy_density * (1.0 - ck) + canopy_target * ck;
    veg.canopy_density = (veg.canopy_density - veg.burn_severity * 0.04 * dt_e * 18.0).clamp(0.0, 1.0);
    veg.canopy_density = canopy_after_snow(veg.canopy_density, wx.snow_depth);

    let understory_target = (veg.canopy_density * 0.65 * (1.0 - veg.burn_severity * 0.85) + eco.biomass * 0.15).clamp(0.0, 1.0);
    veg.understory_density = veg.understory_density * (1.0 - ck) + understory_target * ck;
    veg.understory_density = understory_after_snow(veg.understory_density, wx.snow_depth);

    veg.ground_fuel = ((veg.understory_density * 0.5 + veg.dryness * 0.45) * (1.0 + wetland_w * 0.35))
        .clamp(0.0, 1.0);
    veg.ground_fuel = ground_fuel_after_heat(veg.ground_fuel, heat_c);

    veg.fuel_load = fuel_load_after_rain(
        (veg.ground_fuel * 0.55 + veg.canopy_density * 0.45 * (1.0 + veg.dryness * 0.4)).clamp(0.0, 1.0),
        wx.rain_intensity,
    );

    let og_target = (eco.biomass * eco.regrowth_rate * 1.2).min(1.0);
    let ok = (0.05 * dt_e).min(0.12);
    veg.old_growth = veg.old_growth * (1.0 - ok) + og_target * ok;
    veg.old_growth *= (1.0 - veg.burn_severity * 0.06 * dt_e * 10.0).clamp(0.65, 1.0);

    veg.fragmentation =
        fragmentation_after_heat(eco.root_strength, veg.burn_severity, heat_c, wx.snow_depth);

    veg.smoke_absorption = smoke_held_after_rain(
        (veg.canopy_density * 0.75 + veg.understory_density * 0.35 + wx.fog_density * 0.4).clamp(0.0, 1.0),
        wx.rain_intensity,
    );

    veg.concealment = concealment_after_rain(
        concealment_after_fog(
            ((veg.canopy_density * 0.55 + veg.understory_density * 0.65)
                * (1.0 - wx.snow_depth * 0.35))
                .clamp(0.0, 1.0),
            wx.fog_density,
        ),
        wx.rain_intensity,
    );

    let regrow_drive = regrowth_drive_after_snow(
        (eco.regrowth_rate * moisture_signal * (1.0 - heat_c * 1.1).max(0.0)).clamp(0.0, 0.85),
        wx.snow_depth,
    );
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
    fn steep_slope_thins_canopy_versus_flat() {
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
            wind_speed: 0.1,
            lightning_risk: 0.0,
            visibility_factor: 0.95,
            soil_moisture: 0.5,
        };
        let size = bevy::math::UVec2::new(8, 8);
        let mut flat = ChunkCellMatrix::new(size);
        for v in &mut flat.elevation {
            *v = 0.4;
        }
        let mut ramp = ChunkCellMatrix::new(size);
        for y in 0..size.y {
            for x in 0..size.x {
                let i = ramp.idx(x, y);
                ramp.elevation[i] = x as f32 / 7.0;
            }
        }
        let mut flat_veg = VegetationField::default();
        let mut ramp_veg = VegetationField::default();
        for _ in 0..80 {
            integrate_vegetation_field_step(0.2, Some(&flat), &eco, &wx, 0.0, &mut flat_veg);
            integrate_vegetation_field_step(0.2, Some(&ramp), &eco, &wx, 0.0, &mut ramp_veg);
        }
        assert!(
            ramp_veg.canopy_density < flat_veg.canopy_density - 0.02,
            "steep {} vs flat {}",
            ramp_veg.canopy_density,
            flat_veg.canopy_density
        );
    }

    #[test]
    fn veg_draw_follows_center_curvature() {
        use crate::terrain::generation::{cell_canopy_scale, cell_canopy_scales, ChunkCellMatrix};
        let size = bevy::math::UVec2::new(7, 7);
        let mut m = ChunkCellMatrix::new(size);
        for v in &mut m.elevation {
            *v = 0.4;
        }
        let hollow = m.idx(1, 1);
        let ridge = m.idx(5, 5);
        m.elevation[hollow] = 0.05;
        m.elevation[ridge] = 0.95;
        let cover = cell_canopy_scales(&m);
        let hollow_draw = veg_draw_density(0.6, cover[hollow]);
        let ridge_draw = veg_draw_density(0.6, cover[ridge]);
        assert!(hollow_draw > ridge_draw, "{hollow_draw} vs {ridge_draw}");
        let _ = cell_canopy_scale(0.0, 0.0);
    }

    #[test]
    fn impact_lowers_veg_draw_density() {
        let clear = veg_draw_density_after_impact(0.8, 1.0, 0.0);
        let hit = veg_draw_density_after_impact(0.8, 1.0, 1.0);
        assert!(hit < clear, "hit {hit} clear {clear}");
        assert!((clear - veg_draw_density(0.8, 1.0)).abs() < 1e-5);
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

    #[test]
    fn drain_snow_thins_canopy_against_bare_ground() {
        let bare = canopy_after_snow(0.8, 0.0);
        let snowy = canopy_after_snow(0.8, 1.0);
        assert!((bare - 0.8).abs() < 1e-5);
        assert!(snowy < bare, "{snowy} vs {bare}");
        assert!((snowy - 0.8 * 0.55).abs() < 1e-5);
    }

    #[test]
    fn live_understory_thins_in_snow() {
        let bare = understory_after_snow(0.8, 0.0);
        let snowy = understory_after_snow(0.8, 1.0);
        assert!((bare - 0.8).abs() < 1e-5);
        assert!(snowy < bare);
        assert!(snowy > canopy_after_snow(0.8, 1.0));
        assert!((snowy - 0.8 * 0.75).abs() < 1e-5);
    }

    #[test]
    fn push_snow_slows_regrowth_drive() {
        let open = regrowth_drive_after_snow(0.4, 0.0);
        let winter = regrowth_drive_after_snow(0.4, 1.0);
        assert!((open - 0.4).abs() < 1e-5);
        assert!(winter < open);
        assert!((winter - 0.4 * 0.4).abs() < 1e-5);
    }

    #[test]
    fn push_burn_raises_fragmentation() {
        let calm = fragmentation_after_burn(0.8, 0.0);
        let burned = fragmentation_after_burn(0.8, 1.0);
        assert!(burned > calm);
        assert!(burned <= 1.0);
    }

    #[test]
    fn next_fog_lowers_concealment() {
        let clear = concealment_after_fog(0.8, 0.0);
        let fogged = concealment_after_fog(0.8, 1.0);
        assert!((clear - 0.8).abs() < 1e-5);
        assert!(fogged < clear);
        assert!((fogged - 0.8 * 0.65).abs() < 1e-5);
    }

    #[test]
    fn next_heat_lowers_ground_fuel() {
        let cold = ground_fuel_after_heat(0.8, 0.0);
        let hot = ground_fuel_after_heat(0.8, 1.0);
        assert!((cold - 0.8).abs() < 1e-5);
        assert!(hot < cold);
        assert!((hot - 0.8 * 0.75).abs() < 1e-5);
    }

    #[test]
    fn cont_rain_lowers_held_smoke() {
        let dry = smoke_held_after_rain(0.8, 0.0);
        let wet = smoke_held_after_rain(0.8, 1.0);
        assert!((dry - 0.8).abs() < 1e-5);
        assert!((wet - 0.4).abs() < 1e-5);
    }

    #[test]
    fn cont_rain_lowers_the_stand_fuel_load() {
        let dry = fuel_load_after_rain(0.8, 0.0);
        let wet = fuel_load_after_rain(0.8, 1.0);
        assert!((dry - 0.8).abs() < 1e-5);
        assert!((wet - 0.8 * 0.6).abs() < 1e-5);
    }

    #[test]
    fn fold_drought_thins_concealment_against_rain() {
        let drought = concealment_after_rain(0.8, 0.0);
        let rain = concealment_after_rain(0.8, 1.0);
        assert!((rain - 0.8).abs() < 1e-5);
        assert!(drought < rain);
        assert!((drought - 0.8 * 0.72).abs() < 1e-5);
        let clamped = concealment_after_rain(2.0, 1.0);
        assert!((clamped - 1.0).abs() < 1e-5);
    }

    #[test]
    fn fold_heat_fragments_more_than_snow() {
        let hot = fragmentation_after_heat(0.6, 0.2, 1.0, 0.0);
        let cold = fragmentation_after_heat(0.6, 0.2, 0.0, 1.0);
        let even = fragmentation_after_heat(0.6, 0.2, 1.0, 1.0);
        let base = fragmentation_after_burn(0.6, 0.2);
        assert!(hot > cold);
        assert!(hot > base);
        assert!(cold < base);
        assert!((even - base).abs() < 1e-5);
        assert!(hot <= 1.0 && cold >= 0.0);
    }

    #[test]
    fn fold_zero_canopy_draws_nothing() {
        assert_eq!(veg_draw_density(0.0, 1.2), 0.0);
        assert_eq!(veg_draw_density(-0.4, 1.0), 0.0);
        assert_eq!(veg_draw_density_after_impact(0.0, 1.0, 0.3), 0.0);
    }

    #[test]
    fn fold_draw_density_clamps_to_one() {
        let full = veg_draw_density(1.0, 1.35);
        let over = veg_draw_density(4.0, 4.0);
        assert!((full - 1.0).abs() < 1e-5);
        assert!((over - 1.0).abs() < 1e-5);
        assert!(veg_draw_density_after_impact(1.0, 1.35, 0.0) <= 1.0);
    }

    #[test]
    fn fold_full_canopy_snow_clamps() {
        let bare = canopy_after_snow(2.0, 0.0);
        let deep = canopy_after_snow(2.0, 1.0);
        assert!((bare - 1.0).abs() < 1e-5);
        assert!(deep < bare);
        assert!(deep >= 0.0);
        assert!((deep - 0.55).abs() < 1e-5);
    }
}
