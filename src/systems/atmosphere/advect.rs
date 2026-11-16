//! Semi-Lagrangian drift for smoke / toxic / ash (`base_fire2_smoke.md` §3).
//!
//! **DEBT-011:** advect is clipmap L0 only — Field resource + Field-channel advect removed.
//! **W2:** per-texel wind from [`WeatherField::sample`] (replaces deleted `GlobalWind`).
//! **W4:** sole rain washout owner — uses previous-frame [`AtmosphereFrame`] rain (Frame runs after).

use bevy::prelude::*;

use crate::substrate::atmosphere::{
    advect_l0_with_wind_field, apply_l0_rain_washout, AtmosphereClipmapStack,
    L0_ADVECT_WRITER_WIND_ADVECT,
};
use crate::systems::sim_control::{SimControlState, SimTimeMicros};
use crate::systems::weather::WeatherField;
use crate::terrain::generation::chunk_world_center;
use crate::terrain::world_scale_contract::{CHUNK_TILES_SIM, TILE_METRES_GAMEPLAY};

use super::diagnostics::AtmosphereDiagnostics;
use super::field::{ATMO_GRID_ORIGIN, ATMO_GRID_SIZE};
use super::frame::AtmosphereFrame;

pub fn advect_atmosphere_field(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    sim_time: Res<SimTimeMicros>,
    // Bevy 0.20: missing Res panics — `If` skips this system when weather field absent (minimal test apps).
    If(field): If<Res<WeatherField>>,
    frame: Res<AtmosphereFrame>,
    mut stack: ResMut<AtmosphereClipmapStack>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    if !ctrl.should_tick() || !field.is_ready() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 {
        return;
    }
    diag.advect_runs = diag.advect_runs.wrapping_add(1);

    let t = *sim_time;
    let origin = ATMO_GRID_ORIGIN;
    let size = ATMO_GRID_SIZE;
    let tiles = UVec2::splat(CHUNK_TILES_SIM);
    let frame_ready = frame.is_built();
    let frame_tiles = frame.tiles();
    let frame_w = frame.size().x;

    let mut advected = false;
    let mut washout = 0.0f32;
    if let Some(level0) = stack.levels.first_mut() {
        advect_l0_with_wind_field(level0, dt, |x, y| {
            if x >= size.x || y >= size.y {
                return Vec2::ZERO;
            }
            let chunk = IVec2::new(origin.x + x as i32, origin.y + y as i32);
            let pos = chunk_world_center(chunk, tiles) * TILE_METRES_GAMEPLAY;
            field.sample(pos, t).wind_mps
        });
        // Previous-frame rain (AtmosphereFrame rebuilds after this set).
        washout = apply_l0_rain_washout(level0, dt, |x, y| {
            if !frame_ready || frame_w == 0 {
                return 0.0;
            }
            let idx = (y * frame_w + x) as usize;
            frame_tiles
                .get(idx)
                .filter(|tile| tile.weather_valid)
                .map(|tile| tile.weather.rain_mm_h)
                .unwrap_or(0.0)
        });
        advected = true;
    }
    if washout > 0.0 {
        diag.rain_washout_mass = diag.rain_washout_mass + washout;
    }
    if advected {
        // W1: sole L0 advect writer; the dump reads `l0_advect_writers` from this mask.
        stack.note_advect_writer(L0_ADVECT_WRITER_WIND_ADVECT);
    }
}

#[cfg(test)]
mod tests {
    use bevy::math::Vec2;

    use crate::substrate::atmosphere::{advect_l0_with_wind, AtmosphereClipLevel};

    #[test]
    fn calm_wind_leaves_l0_untouched_path() {
        let mut level = AtmosphereClipLevel {
            resolution: bevy::math::UVec2::new(4, 1),
            smoke_density: vec![0.5, 0.0, 0.0, 0.0],
            fog_density: vec![0.0; 4],
            toxicity: vec![0.0; 4],
            ash_density: vec![0.0; 4],
            ember_density: vec![0.0; 4],
            heat_distortion: vec![0.0; 4],
        };
        let before = level.smoke_density.clone();
        advect_l0_with_wind(&mut level, Vec2::X, 0.0, 1.0);
        assert_eq!(level.smoke_density, before);
    }
}
