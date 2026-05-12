//! Fold chunk weather / ecology / surface fire into [`super::field::AtmosphereField`] (`base_fire2_smoke.md` §2).

use bevy::prelude::*;

use crate::systems::ecology::ChunkEcology;
use crate::systems::fire::ChunkSurfaceFire;
use crate::systems::fire::{ChunkFireOverlay, ChunkSmokeField};
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};

use super::diagnostics::AtmosphereDiagnostics;
use super::field::{AtmosphereCell, AtmosphereField};

pub fn atmosphere_field_fill_from_chunks(
    ctrl: Res<SimControlState>,
    mut field: ResMut<AtmosphereField>,
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
    for c in &mut field.cells {
        *c = AtmosphereCell::default();
    }

    for (chunk, wx, eco, fire_opt) in &q {
        let Some(cell) = field.cell_mut_at_chunk(chunk.coord) else {
            continue;
        };
        let heat = fire_opt.map(|f| f.heat).unwrap_or(0.0);
        let smoke_gen = heat * eco.biomass * (1.0 + eco.fire_risk);
        let fog = wx.fog_density + wx.rain_intensity * 0.2;
        let toxicity = smoke_gen * 0.45;
        let ember_density = heat * wx.wind_speed * eco.biomass;
        let visibility = (1.0 - smoke_gen * 0.7 - fog * 0.45).clamp(0.05, 1.0);

        cell.smoke_density = smoke_gen.clamp(0.0, 1.0);
        cell.fog_density = fog.clamp(0.0, 1.0);
        cell.toxicity = toxicity.clamp(0.0, 1.0);
        cell.ember_density = ember_density.clamp(0.0, 1.0);
        cell.visibility = visibility;
        cell.heat_distortion = (heat * 0.8).clamp(0.0, 1.0);
        cell.ash_density = (smoke_gen * 0.35).clamp(0.0, 1.0);
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
/// [`ChunkSmokeField`] chunk scalars into the chunk’s atmosphere tile (`atm-update-1a`).
pub fn atmosphere_field_blend_fire_overlay_sources(
    ctrl: Res<SimControlState>,
    mut field: ResMut<AtmosphereField>,
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
        let Some(cell) = field.cell_mut_at_chunk(chunk.coord) else {
            continue;
        };
        let n = (matrix.size.x * matrix.size.y) as usize;
        if n == 0 {
            continue;
        }

        if let Some(ovl) = ovl_opt {
            if ovl.heat.len() == n && ovl.smoke.len() == n && ovl.toxic.len() == n {
                let mh = mean_f32_slice(&ovl.heat).clamp(0.0, 1.0);
                let ms = mean_f32_slice(&ovl.smoke).clamp(0.0, 1.0);
                let mt = mean_f32_slice(&ovl.toxic).clamp(0.0, 1.0);
                cell.heat_distortion = cell.heat_distortion.max((mh * 0.88).min(1.0));
                cell.smoke_density = cell.smoke_density.max(ms);
                cell.toxicity = cell.toxicity.max(mt);
                cell.ember_density = cell.ember_density.max((mh * ms).sqrt().min(1.0));
                cell.ash_density = cell.ash_density.max((ms * 0.42).min(1.0));
            }
        }

        if let Some(smoke) = smoke_opt {
            cell.smoke_density = cell.smoke_density.max(smoke.density.clamp(0.0, 1.0));
            cell.toxicity = cell.toxicity.max(smoke.toxicity.clamp(0.0, 1.0));
            cell.smoke_density = cell.smoke_density.max(smoke.visibility_penalty.clamp(0.0, 0.98) * 0.9);
        }

        cell.visibility =
            (1.0 - cell.smoke_density * 0.72 - cell.fog_density * 0.45).clamp(0.05, 1.0);
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
