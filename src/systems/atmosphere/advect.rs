//! Semi-Lagrangian drift for smoke / toxic / ash (`base_fire2_smoke.md` §3).

use bevy::prelude::*;

use crate::systems::sim_control::SimControlState;

use super::diagnostics::AtmosphereDiagnostics;
use super::field::{AtmosphereCell, AtmosphereField, GlobalWind};

pub fn advect_atmosphere_field(
    ctrl: Res<SimControlState>,
    time: Res<Time>,
    wind: Res<GlobalWind>,
    mut field: ResMut<AtmosphereField>,
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

    let size = field.size;
    let sx = size.x as i32;
    let sy = size.y as i32;
    let old = field.cells.clone();
    let dir = wind.direction.normalize_or_zero();
    if dir.length_squared() <= 1e-8 {
        return;
    }

    for y in 0..size.y {
        for x in 0..size.x {
            let fx = x as f32 - dir.x * wind.speed * dt * 2.0;
            let fy = y as f32 - dir.y * wind.speed * dt * 2.0;
            let sx_i = fx.floor() as i32;
            let sy_i = fy.floor() as i32;
            let dst = field.idx(x, y);
            if sx_i < 0 || sy_i < 0 || sx_i >= sx || sy_i >= sy {
                field.cells[dst] = AtmosphereCell::default();
                continue;
            }
            let src_i = (sy_i as u32 * size.x + sx_i as u32) as usize;
            let src = old[src_i];
            field.cells[dst].smoke_density = (src.smoke_density * 0.985).clamp(0.0, 1.0);
            field.cells[dst].toxicity = (src.toxicity * 0.992).clamp(0.0, 1.0);
            field.cells[dst].ash_density = (src.ash_density * 0.98).clamp(0.0, 1.0);
            // fog / visibility / ember / heat: keep from fill pass (re-fill next frame) — copy from src lightly
            field.cells[dst].fog_density = src.fog_density * 0.99;
            field.cells[dst].ember_density = src.ember_density * 0.97;
            field.cells[dst].heat_distortion = src.heat_distortion * 0.96;
            field.cells[dst].visibility = src.visibility;
        }
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
