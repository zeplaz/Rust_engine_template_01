//! Strategic **smoke / visibility** scalars per chunk (`base_fire_sim.md` §5).
//! After global advection, [`chunk_smoke_field_pull_from_advected_atmosphere`] nudges these toward
//! [`crate::systems::atmosphere::AtmosphereField`] so chunk reads track transported smoke (`sim-smoke-1`).
//! **Future:** optional per-cell grid + richer bidirectional coupling.

use bevy::prelude::*;

use crate::systems::atmosphere::{AtmosphereCell, AtmosphereField};
use super::chunk_fuel_profile::ChunkFuelProfile;
use super::chunk_surface_fire::ChunkSurfaceFire;
use super::combustion::profile_weighted_smoke_toxic_explosion;
use crate::systems::chunk_sim_lod::ChunkSimLod;
use crate::systems::sim_control::SimControlState;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::Chunk;

#[derive(Component, Clone, Copy, Debug)]
pub struct ChunkSmokeField {
    pub density: f32,
    pub toxicity: f32,
    pub visibility_penalty: f32,
}

impl Default for ChunkSmokeField {
    fn default() -> Self {
        Self {
            density: 0.0,
            toxicity: 0.0,
            visibility_penalty: 0.0,
        }
    }
}

pub(crate) fn spawn_chunk_smoke_field_on_new_chunk(
    mut commands: Commands,
    q: Query<Entity, (Added<Chunk>, Without<ChunkSmokeField>)>,
) {
    for e in &q {
        commands.entity(e).insert(ChunkSmokeField::default());
    }
}

pub fn chunk_smoke_field_tick(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    mut q: Query<(
        &ChunkSurfaceFire,
        Option<&ChunkFuelProfile>,
        Option<&ChunkSimLod>,
        &ChunkWeather,
        &mut ChunkSmokeField,
    )>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }

    for (fire, prof_opt, lod, wx, mut smoke) in &mut q {
        let lod_s = lod.map(|l| l.dt_scale()).unwrap_or(1.0);
        let dt_e = dt * lod_s;

        let heat = fire.heat.clamp(0.0, 1.0);
        let (smoke_r, toxic_prof, _) = prof_opt
            .map(profile_weighted_smoke_toxic_explosion)
            .unwrap_or((0.5, 0.12, 0.0));
        let toxic_fuel = prof_opt.map(|p| p.to_fuel_layer().toxic_smoke).unwrap_or(toxic_prof);
        let toxic_rate = (0.5 * toxic_prof + 0.5 * toxic_fuel).clamp(0.0, 1.0);

        let emit = heat * (0.55 + smoke_r * 1.1) * (1.0 + wx.wind_speed * 0.35);
        let rain_cleanse = wx.rain_intensity * 0.45;

        smoke.density = (smoke.density + emit * dt_e * 2.8 - smoke.density * 0.28 * dt_e - rain_cleanse * dt_e)
            .clamp(0.0, 1.0);
        smoke.toxicity =
            (smoke.toxicity + heat * toxic_rate * dt_e * 1.9 - smoke.toxicity * 0.22 * dt_e - rain_cleanse * 0.35 * dt_e)
                .clamp(0.0, 1.0);

        smoke.visibility_penalty =
            (smoke.density * 0.75 + wx.fog_density * 0.35 + smoke.toxicity * 0.25).clamp(0.0, 0.98);
    }
}

/// Blend weight for folding advected [`AtmosphereField`] smoke back into chunk scalars.
pub const ATMOSPHERE_TO_CHUNK_SMOKE_BLEND: f32 = 0.16;

#[inline]
fn blend_chunk_smoke_toward_atmosphere_cell(
    smoke: &mut ChunkSmokeField,
    cell: &AtmosphereCell,
    wx: &ChunkWeather,
    w_atm: f32,
) {
    let w = w_atm.clamp(0.0, 0.55);
    smoke.density = (smoke.density * (1.0 - w) + cell.smoke_density * w).clamp(0.0, 1.0);
    smoke.toxicity = (smoke.toxicity * (1.0 - w) + cell.toxicity * w).clamp(0.0, 1.0);
    smoke.visibility_penalty =
        (smoke.density * 0.75 + wx.fog_density * 0.35 + smoke.toxicity * 0.25).clamp(0.0, 0.98);
}

/// Runs in [`crate::systems::atmosphere::AtmospherePipelineSet::WindAdvect`] after advection so chunk
/// smoke matches the transported plume for logistics / extract.
pub fn chunk_smoke_field_pull_from_advected_atmosphere(
    ctrl: Res<SimControlState>,
    atm: Res<AtmosphereField>,
    mut q: Query<(&Chunk, &ChunkWeather, &mut ChunkSmokeField)>,
) {
    if !ctrl.should_tick() {
        return;
    }
    for (chunk, wx, mut smoke) in &mut q {
        let Some(cell) = atm.cell_at_chunk(chunk.coord) else {
            continue;
        };
        blend_chunk_smoke_toward_atmosphere_cell(&mut smoke, &cell, wx, ATMOSPHERE_TO_CHUNK_SMOKE_BLEND);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::ecology::VegetationField;
    use crate::systems::fire::chunk_fuel_profile::chunk_fuel_profile_from_vegetation;

    #[test]
    fn active_fire_builds_smoke() {
        let mut smoke = ChunkSmokeField::default();
        let fire = ChunkSurfaceFire {
            heat: 0.6,
            fuel: 0.8,
        };
        let veg = VegetationField {
            canopy_density: 0.6,
            dryness: 0.7,
            fuel_load: 0.6,
            ..Default::default()
        };
        let prof = chunk_fuel_profile_from_vegetation(&veg);
        let wx = ChunkWeather::default();
        let heat = fire.heat.clamp(0.0, 1.0);
        let (smoke_r, toxic_prof, _) = profile_weighted_smoke_toxic_explosion(&prof);
        let toxic_fuel = prof.to_fuel_layer().toxic_smoke;
        let toxic_rate = (0.5 * toxic_prof + 0.5 * toxic_fuel).clamp(0.0, 1.0);

        let emit = heat * (0.55 + smoke_r * 1.1);
        smoke.density = (smoke.density + emit * 0.05 * 2.8).clamp(0.0, 1.0);
        smoke.toxicity = (smoke.toxicity + heat * toxic_rate * 0.05 * 1.9).clamp(0.0, 1.0);
        smoke.visibility_penalty =
            (smoke.density * 0.75 + wx.fog_density * 0.35 + smoke.toxicity * 0.25).clamp(0.0, 0.98);
        assert!(smoke.density > 0.05);
    }

    #[test]
    fn pull_shifts_density_toward_atmosphere_cell() {
        let mut field = AtmosphereField::default();
        let coord = IVec2::new(0, 0);
        if let Some(c) = field.cell_mut_at_chunk(coord) {
            c.smoke_density = 1.0;
            c.toxicity = 0.9;
        }
        let cell = field.cell_at_chunk(coord).unwrap();
        let wx = ChunkWeather::default();
        let mut smoke = ChunkSmokeField {
            density: 0.1,
            toxicity: 0.1,
            visibility_penalty: 0.0,
        };
        super::blend_chunk_smoke_toward_atmosphere_cell(
            &mut smoke,
            &cell,
            &wx,
            ATMOSPHERE_TO_CHUNK_SMOKE_BLEND,
        );
        assert!(smoke.density > 0.1 && smoke.density < 1.0);
        assert!(smoke.toxicity > 0.1);
    }
}
