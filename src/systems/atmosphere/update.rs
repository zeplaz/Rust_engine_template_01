//! Fold chunk weather / ecology / surface fire into clipmap L0 (`base_fire2_smoke.md` §2).
//!
//! **DEBT-011:** writers are L0-only via [`atmos_chunk_to_tile`]; the fixed 128² Field
//! resource is gone. Smoke authority triangle is L0 → ChunkSmoke → GPU extract.

use bevy::prelude::*;

use crate::substrate::atmosphere::{
    max_blend_l0_ash_at, max_blend_l0_ember_at, max_blend_l0_fog_at, max_blend_l0_heat_at,
    max_blend_l0_smoke_at, max_blend_l0_toxicity_at, AtmosphereClipmapStack,
};
use crate::systems::ecology::ChunkEcology;
use crate::systems::fire::ChunkSurfaceFire;
use crate::systems::fire::{ChunkFireOverlay, ChunkSmokeField};
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

use super::diagnostics::AtmosphereDiagnostics;
use super::field::atmos_chunk_to_tile;

#[inline]
fn blend_l0_channels_for_chunk(
    stack: &mut AtmosphereClipmapStack,
    chunk: IVec2,
    smoke: f32,
    fog: f32,
    toxicity: f32,
    ash: f32,
    ember: f32,
    heat: f32,
) {
    let Some((x, y)) = atmos_chunk_to_tile(chunk) else {
        return;
    };
    max_blend_l0_smoke_at(stack, x, y, smoke);
    max_blend_l0_fog_at(stack, x, y, fog);
    max_blend_l0_toxicity_at(stack, x, y, toxicity);
    max_blend_l0_ash_at(stack, x, y, ash);
    max_blend_l0_ember_at(stack, x, y, ember);
    max_blend_l0_heat_at(stack, x, y, heat);
}

#[inline]
fn channels_from_chunk(
    wx: &ChunkWeather,
    eco: &ChunkEcology,
    fire_opt: Option<&ChunkSurfaceFire>,
) -> (f32, f32, f32, f32, f32, f32) {
    let heat = fire_opt.map(|f| f.heat).unwrap_or(0.0);
    let smoke_gen = (heat * eco.biomass * (1.0 + eco.fire_risk)).clamp(0.0, 1.0);
    let fog = (wx.fog_density + wx.rain_intensity * 0.2).clamp(0.0, 1.0);
    let toxicity = (smoke_gen * 0.45).clamp(0.0, 1.0);
    let ember = (heat * wx.wind_speed * eco.biomass).clamp(0.0, 1.0);
    let heat_d = (heat * 0.8).clamp(0.0, 1.0);
    let ash = (smoke_gen * 0.35).clamp(0.0, 1.0);
    (smoke_gen, fog, toxicity, ash, ember, heat_d)
}

pub fn atmosphere_field_fill_from_chunks(
    ctrl: Res<SimControlState>,
    mut stack: ResMut<AtmosphereClipmapStack>,
    mut diag: ResMut<AtmosphereDiagnostics>,
    q: Query<(
        &Chunk,
        &ChunkWeather,
        &ChunkEcology,
        Option<&ChunkSurfaceFire>,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    diag.field_fill_runs = diag.field_fill_runs.wrapping_add(1);
    // L0 is max-blend only (same as pre–DEBT-011 Field→L0 fold). Decay lives in WindAdvect.

    diag.last_fill_contiguous = false;
    if let Ok(batches) = q.contiguous_iter() {
        diag.last_fill_contiguous = true;
        for (chunks, weathers, ecos, fires) in batches {
            let n = chunks.len();
            for i in 0..n {
                let fire_opt = fires.as_ref().map(|slice| &slice[i]);
                let (smoke, fog, toxicity, ash, ember, heat) =
                    channels_from_chunk(&weathers[i], &ecos[i], fire_opt);
                blend_l0_channels_for_chunk(
                    &mut stack,
                    chunks[i].coord,
                    smoke,
                    fog,
                    toxicity,
                    ash,
                    ember,
                    heat,
                );
            }
        }
    } else {
        for (chunk, wx, eco, fire_opt) in &q {
            let (smoke, fog, toxicity, ash, ember, heat) = channels_from_chunk(wx, eco, fire_opt);
            blend_l0_channels_for_chunk(
                &mut stack,
                chunk.coord,
                smoke,
                fog,
                toxicity,
                ash,
                ember,
                heat,
            );
        }
    }
}

/// Mean of a per-cell vector (SoA slice). Returns `0` when empty.
#[inline]
pub(crate) fn mean_f32_slice(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f32>() / values.len() as f32
}

/// After macro-scale fill, **max-blend** per-cell [`ChunkFireOverlay`] smoke / heat / toxic and
/// [`ChunkSmokeField`] chunk scalars into the chunk’s L0 tile (`atm-update-1a`).
pub fn atmosphere_field_blend_fire_overlay_sources(
    ctrl: Res<SimControlState>,
    mut stack: ResMut<AtmosphereClipmapStack>,
    q: Query<(
        &Chunk,
        &ChunkCellMatrix,
        Option<&ChunkFireOverlay>,
        Option<&ChunkSmokeField>,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }

    for (chunk, matrix, ovl_opt, smoke_opt) in &q {
        let n = (matrix.size.x * matrix.size.y) as usize;
        if n == 0 {
            continue;
        }

        let mut smoke = 0.0f32;
        let fog = 0.0f32;
        let mut toxicity = 0.0f32;
        let mut ash = 0.0f32;
        let mut ember = 0.0f32;
        let mut heat = 0.0f32;

        if let Some(ovl) = ovl_opt {
            if ovl.heat.len() == n && ovl.smoke.len() == n && ovl.toxic.len() == n {
                let mh = mean_f32_slice(&ovl.heat).clamp(0.0, 1.0);
                let ms = mean_f32_slice(&ovl.smoke).clamp(0.0, 1.0);
                let mt = mean_f32_slice(&ovl.toxic).clamp(0.0, 1.0);
                heat = heat.max((mh * 0.88).min(1.0));
                smoke = smoke.max(ms);
                toxicity = toxicity.max(mt);
                ember = ember.max((mh * ms).sqrt().min(1.0));
                ash = ash.max((ms * 0.42).min(1.0));
            }
        }

        if let Some(smoke_c) = smoke_opt {
            smoke = smoke.max(smoke_c.density.clamp(0.0, 1.0));
            toxicity = toxicity.max(smoke_c.toxicity.clamp(0.0, 1.0));
            smoke = smoke.max(smoke_c.visibility_penalty.clamp(0.0, 0.98) * 0.9);
        }

        if smoke <= 0.0 && fog <= 0.0 && toxicity <= 0.0 && ash <= 0.0 && ember <= 0.0 && heat <= 0.0
        {
            continue;
        }
        blend_l0_channels_for_chunk(
            &mut stack,
            chunk.coord,
            smoke,
            fog,
            toxicity,
            ash,
            ember,
            heat,
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::systems::ecology::ChunkEcology;
    use crate::systems::fire::ChunkSurfaceFire;
    use crate::systems::weather::ChunkWeather;

    #[test]
    fn mean_slice_handles_empty() {
        assert_eq!(super::mean_f32_slice(&[]), 0.0);
        assert!((super::mean_f32_slice(&[0.2, 0.4, 0.6]) - 0.4).abs() < 1e-5);
    }

    #[test]
    fn fill_formula_smoke_and_visibility_in_range() {
        let wx = ChunkWeather {
            fog_density: 0.1,
            rain_intensity: 0.0,
            wind_speed: 0.5,
            ..Default::default()
        };
        let eco = ChunkEcology {
            biomass: 0.8,
            fire_risk: 0.2,
            ..Default::default()
        };
        let fire = ChunkSurfaceFire {
            heat: 0.5,
            fuel: 1.0,
        };
        let heat = fire.heat;
        let smoke_gen = heat * eco.biomass * (1.0 + eco.fire_risk);
        let fog = wx.fog_density + wx.rain_intensity * 0.2;
        let visibility = (1.0 - smoke_gen * 0.7 - fog * 0.45).clamp(0.05, 1.0);
        assert!(smoke_gen > 0.0 && smoke_gen <= 1.0);
        assert!(visibility <= 1.0 && visibility >= 0.05);
    }
}
