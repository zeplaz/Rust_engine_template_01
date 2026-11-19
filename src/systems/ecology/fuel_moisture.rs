//! DF-5 — dead-fuel moisture with NFDRS-style timelag classes.

use bevy::prelude::*;

use crate::systems::ecology::cell_stand::ChunkCellStand;
use crate::systems::sim_control::{SimControlState, SimStepClock};
use crate::systems::water::SlowDomainTimeSkip;
use crate::systems::weather::ChunkWeather;
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;

/// 1-h fine fuels (litter, needles).
pub const TIMELAG_FINE_H: f32 = 1.0;
/// 10-h branchlets.
pub const TIMELAG_TEN_H: f32 = 10.0;
/// 100-h branch wood.
pub const TIMELAG_HUNDRED_H: f32 = 100.0;
/// 1000-h trunk / large woody.
pub const TIMELAG_THOUSAND_H: f32 = 1000.0;

/// Per-cell dead-fuel moisture classes (0..1). Ecology sole writer.
#[derive(Component, Debug, Clone)]
pub struct ChunkCellFuelMoisture {
    pub fine_01: Vec<f32>,
    pub ten_h_01: Vec<f32>,
    pub hundred_h_01: Vec<f32>,
    pub thousand_h_01: Vec<f32>,
}

impl ChunkCellFuelMoisture {
    #[must_use]
    pub fn new(n: usize, seed_m: f32) -> Self {
        let m = seed_m.clamp(0.05, 1.0);
        Self {
            fine_01: vec![m; n],
            ten_h_01: vec![m; n],
            hundred_h_01: vec![m; n],
            thousand_h_01: vec![m; n],
        }
    }

    pub fn resize_to(&mut self, n: usize, seed_m: f32) {
        if self.fine_01.len() == n {
            return;
        }
        *self = Self::new(n, seed_m);
    }
}

/// Equilibrium moisture from soil / rain; submerged saturates.
#[must_use]
pub fn equilibrium_moisture01(soil_m: f32, rain_mm_h: f32, submerged: bool) -> f32 {
    if submerged {
        1.0
    } else {
        (soil_m.clamp(0.0, 1.0) * 0.85 + (rain_mm_h / 10.0).clamp(0.0, 0.4)).clamp(0.05, 1.0)
    }
}

/// Exponential approach toward equilibrium with a given timelag (hours).
#[must_use]
pub fn step_timelag01(current: f32, eq: f32, dt_s: f32, timelag_h: f32) -> f32 {
    let dt_h = (dt_s.max(0.0) / 3600.0).min(timelag_h.max(1e-3) * 4.0);
    let a = 1.0 - (-dt_h / timelag_h.max(1e-3)).exp();
    (current + (eq - current) * a).clamp(0.0, 1.0)
}

/// Effective moisture for spread gates: submerged saturates; flood-dead uses fine timelag class.
#[must_use]
pub fn effective_fuel_moisture01(
    base_moisture: f32,
    flood_dead: bool,
    submerged: bool,
    fine_dead_m: Option<f32>,
) -> f32 {
    let base = base_moisture.clamp(0.0, 1.0);
    if submerged {
        1.0
    } else if flood_dead {
        fine_dead_m.unwrap_or_else(|| (base * 0.42).min(0.55))
    } else {
        base
    }
}

/// Attach moisture book next to new stands.
pub(crate) fn spawn_chunk_cell_fuel_moisture(
    mut commands: Commands,
    q: Query<(Entity, &ChunkCellMatrix), (With<ChunkCellStand>, Without<ChunkCellFuelMoisture>)>,
) {
    for (e, matrix) in &q {
        let n = (matrix.size.x * matrix.size.y) as usize;
        if n == 0 {
            continue;
        }
        let seed = matrix
            .moisture
            .first()
            .copied()
            .unwrap_or(0.45)
            .clamp(0.05, 1.0);
        commands.entity(e).insert(ChunkCellFuelMoisture::new(n, seed));
    }
}

/// Sole writer of [`ChunkCellFuelMoisture`] (ecology domain).
pub(crate) fn cell_fuel_moisture_tick(
    ctrl: Res<SimControlState>,
    clock: Res<SimStepClock>,
    skip: Option<Res<SlowDomainTimeSkip>>,
    mut q: Query<(
        &ChunkCellMatrix,
        &ChunkCellStand,
        Option<&ChunkWeather>,
        &mut ChunkCellFuelMoisture,
    )>,
) {
    let _perf = crate::render::PerfScope::new("upd_eco_cell_fuel_moisture_tick");
    if !ctrl.should_tick() {
        return;
    }
    let mut dt = clock.step_secs() * clock.due.max(1) as f32;
    // DF-1 D4: when time-skip is armed, dry/wet clocks use the coarse dt.
    if let Some(skip) = skip.filter(|s| s.enabled) {
        dt = skip.dt_slow_secs.max(1) as f32 * clock.due.max(1) as f32;
    }
    if dt <= 0.0 {
        return;
    }
    for (matrix, stand, wx, mut fuel_m) in &mut q {
        let n = stand.kind.len();
        if n == 0 {
            continue;
        }
        let seed = matrix.moisture.first().copied().unwrap_or(0.45);
        fuel_m.resize_to(n, seed);
        let soil = wx.map(|w| w.soil_moisture).unwrap_or(0.45);
        let rain = wx.map(|w| w.rain_mm_h).unwrap_or(0.0);
        for i in 0..n {
            let submerged = stand.submerged_since.get(i).copied().unwrap_or(-1.0) >= 0.0;
            let mut eq = equilibrium_moisture01(soil, rain, submerged);
            // Live canopy holds more moisture than flood-dead woody fuel.
            if !stand.flood_dead.get(i).copied().unwrap_or(false) && !submerged {
                eq = eq.max(matrix.moisture.get(i).copied().unwrap_or(eq));
            }
            fuel_m.fine_01[i] = step_timelag01(fuel_m.fine_01[i], eq, dt, TIMELAG_FINE_H);
            fuel_m.ten_h_01[i] = step_timelag01(fuel_m.ten_h_01[i], eq, dt, TIMELAG_TEN_H);
            fuel_m.hundred_h_01[i] = step_timelag01(fuel_m.hundred_h_01[i], eq, dt, TIMELAG_HUNDRED_H);
            fuel_m.thousand_h_01[i] =
                step_timelag01(fuel_m.thousand_h_01[i], eq, dt, TIMELAG_THOUSAND_H);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_spreads_faster_in_dry_dead_than_live() {
        let live = effective_fuel_moisture01(0.65, false, false, None);
        let dead = effective_fuel_moisture01(0.65, true, false, Some(0.22));
        assert!(dead < live, "dead wood must read drier than live at same ambient");
        assert!(dead < 0.72, "dead should clear MOISTURE_BLOCKS_SPREAD more often");
        let wet = effective_fuel_moisture01(0.2, true, true, Some(0.2));
        assert!((wet - 1.0).abs() < 1e-5);
    }

    #[test]
    fn fuel_moisture_dries_with_timelag() {
        let eq = 0.15;
        let mut fine = 0.90_f32;
        let mut trunk = 0.90_f32;
        // 6 hours of dry weather in 10-minute steps.
        for _ in 0..(6 * 6) {
            fine = step_timelag01(fine, eq, 600.0, TIMELAG_FINE_H);
            trunk = step_timelag01(trunk, eq, 600.0, TIMELAG_THOUSAND_H);
        }
        assert!(fine < 0.35, "1-h class should nearly equilibrate in 6 h (got {fine})");
        assert!(trunk > fine + 0.2, "1000-h class must lag fine drying (trunk={trunk} fine={fine})");
        assert!(
            (equilibrium_moisture01(0.2, 0.0, true) - 1.0).abs() < 1e-5,
            "submerged saturates"
        );
    }
}
