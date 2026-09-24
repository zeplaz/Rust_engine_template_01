//! Semi-Lagrangian drift for smoke / toxic / ash (`base_fire2_smoke.md` §3).
//!
//! **DEBT-011:** advect is clipmap L0 only — Field resource + Field-channel advect removed.

use bevy::prelude::*;

use crate::substrate::atmosphere::{advect_l0_with_wind, AtmosphereClipmapStack};
use crate::systems::sim_control::SimControlState;

use super::diagnostics::AtmosphereDiagnostics;
use super::field::GlobalWind;

pub fn advect_atmosphere_field(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    wind: Res<GlobalWind>,
    mut stack: ResMut<AtmosphereClipmapStack>,
    mut diag: ResMut<AtmosphereDiagnostics>,
) {
    if !ctrl.should_tick() {
        return;
    }
    let dt = time.delta_secs() * ctrl.dt_scale();
    if dt <= 0.0 || wind.speed <= 1e-6 {
        return;
    }
    diag.advect_runs = diag.advect_runs.wrapping_add(1);

    let dir = wind.direction.normalize_or_zero();
    if dir.length_squared() <= 1e-8 {
        return;
    }

    if let Some(level0) = stack.levels.first_mut() {
        advect_l0_with_wind(level0, wind.direction, wind.speed, dt);
    }
}

#[cfg(test)]
mod tests {
    use super::GlobalWind;

    #[test]
    fn global_wind_default_is_calm() {
        let w = GlobalWind::default();
        assert!(w.speed <= 1e-5);
    }
}
