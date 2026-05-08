//! Chunk-scale **ecology** scalars (biomass / fire risk / regrowth) — CPU authority, fields-first.
//!
//! Pairs with [`crate::terrain::ecology::estimate_ecological_suitability`] and runbooks:
//! [`prompts/guides/flora_ecology_runbook_v1.md`](../../../prompts/guides/flora_ecology_runbook_v1.md).

use bevy::prelude::*;

use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::fire::ChunkSurfaceFire;
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use crate::terrain::biome::{BiomeWeights, TerrainSurfaceMix};
use crate::terrain::ecology::estimate_ecological_suitability;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

/// Strategic ecology overlay for one [`Chunk`] (not per-tree).
#[derive(Component, Clone, Copy, Debug)]
pub struct ChunkEcology {
    /// Carrying-capacity proxy in `[0, 1]`.
    pub biomass: f32,
    /// Chunk-level ignition / spread pressure proxy.
    pub fire_risk: f32,
    /// Recovery rate toward biomass target after disturbance.
    pub regrowth_rate: f32,
}

impl Default for ChunkEcology {
    fn default() -> Self {
        Self {
            biomass: 0.35,
            fire_risk: 0.0,
            regrowth_rate: 0.12,
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

pub(crate) fn spawn_chunk_ecology_on_new_chunk(
    mut commands: Commands,
    q: Query<Entity, (Added<Chunk>, Without<ChunkEcology>)>,
) {
    for e in &q {
        commands.entity(e).insert(ChunkEcology::default());
    }
}

/// Deterministic slice of [`chunk_ecology_tick`] for fixed `dt_e` (tests + single place for integration math).
pub(crate) fn integrate_chunk_ecology_step(
    dt_e: f32,
    matrix_opt: Option<&ChunkCellMatrix>,
    wx: &ChunkWeather,
    lod_s: f32,
    heat: f32,
    eco: &mut ChunkEcology,
) {
    if dt_e <= 0.0 {
        return;
    }

    let (mean_m, mean_t, bw) = match matrix_opt {
        Some(matrix) => {
            let n = (matrix.size.x * matrix.size.y) as usize;
            let Some(m) = mean_field(n, &matrix.moisture) else {
                return;
            };
            let Some(t) = mean_field(n, &matrix.temperature) else {
                return;
            };
            let bw = mean_biome_weights(matrix).unwrap_or_default();
            (m, t, bw)
        }
        None => (0.35, 0.5, BiomeWeights::default()),
    };

    let mix = TerrainSurfaceMix::default();
    let suit = estimate_ecological_suitability(bw, mix, mean_m, mean_t);
    let target_biomass = (suit.flora_density * 0.55 + suit.flower_density * 0.15
        + suit.crop_yield_factor * 0.15)
        .clamp(0.0, 1.0);

    let k = (0.14 * dt_e).clamp(0.0, 0.35);
    eco.biomass = eco.biomass * (1.0 - k) + target_biomass * k;

    eco.biomass = (eco.biomass - heat * 0.022 * dt_e).clamp(0.0, 1.0);

    let dryness = (0.48 - mean_m).max(0.0);
    eco.fire_risk = (dryness * (mean_t + 0.12).max(0.0) * eco.biomass.powf(0.85)
        * (1.0 - wx.rain_intensity * 0.88)
        * (1.0 + wx.wind_speed * 0.35))
    .clamp(0.0, 1.0);

    let rain_floor = wx.rain_intensity.max(0.06);
    eco.regrowth_rate = ((1.0 - heat * 1.4).clamp(0.0, 1.0)
        * rain_floor
        * suit.flora_density
        * (lod_s * 0.35 + 0.65))
    .clamp(0.0, 1.0);
}

/// Integrate biomass toward a **terrain + weather** carrying capacity; fire stress suppresses regrowth.
pub fn chunk_ecology_tick(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    mut q: Query<(
        Option<&ChunkCellMatrix>,
        &ChunkWeather,
        &ChunkSimLod,
        Option<&ChunkSurfaceFire>,
        &mut ChunkEcology,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }

    for (matrix_opt, wx, lod, fire_opt, mut eco) in &mut q {
        let lod_s = lod.dt_scale();
        let dt_e = dt * lod_s;
        let heat = fire_opt.map(|f| f.heat).unwrap_or(0.0);
        integrate_chunk_ecology_step(dt_e, matrix_opt, wx, lod_s, heat, &mut eco);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::math::UVec2;

    #[test]
    fn ecology_follows_moist_chunk() {
        let mut matrix = ChunkCellMatrix::new(UVec2::new(2, 2));
        for m in matrix.moisture.iter_mut() {
            *m = 0.75;
        }
        for t in matrix.temperature.iter_mut() {
            *t = 0.25;
        }
        let template = {
            let mut w = BiomeWeights::default();
            w.temperate = 0.85;
            w.wetland = 0.1;
            w
        };
        for ww in matrix.weights.iter_mut() {
            *ww = template;
        }

        let wx = ChunkWeather {
            rain_intensity: 0.2,
            fog_density: 0.1,
            snow_depth: 0.0,
            wind_speed: 0.1,
            lightning_risk: 0.0,
            visibility_factor: 1.0,
        };
        let lod_s = ChunkSimLod::Normal.dt_scale();

        let mut eco = ChunkEcology {
            biomass: 0.05,
            fire_risk: 0.0,
            regrowth_rate: 0.0,
        };
        let dt = 1.0 / 60.0_f32;
        for _ in 0..240 {
            integrate_chunk_ecology_step(dt * lod_s, Some(&matrix), &wx, lod_s, 0.0, &mut eco);
        }

        assert!(
            eco.biomass > 0.12,
            "biomass should rise toward wet temperate carrying capacity, got {}",
            eco.biomass
        );
        assert!(eco.fire_risk < 0.35);
    }
}
