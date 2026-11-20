//! DW-5 — fast water washes loose material (rubble, crater rims, dam debris) downstream.
//!
//! Water domain reader of terrain truth: loose material is the positive part of the DW-1 ground
//! delta (heaps and rims laid on the DEM; bedrock is never eroded). Where the cell velocity
//! exceeds [`WASH_THRESHOLD_M_S`] the flow picks material up and moves it one tile along the
//! flow; it settles where the water slows, because nothing is picked up there. Moves go back
//! to terrain as `GroundDeformOp::Transfer` (volume conserved), so the voxel ground, the bed and
//! ecology all follow the one owner.

use bevy::prelude::*;

use super::surface_water::SurfaceWaterField;
use crate::terrain::ground_deform::{GroundDeformOp, GroundDeformQueue, GroundDeltaBook};

/// Velocity above which loose debris moves (m/s).
pub const WASH_THRESHOLD_M_S: f32 = 1.2;
/// Metres of loose material moved per second per m/s above the threshold.
pub const WASH_RATE_M_PER_S: f32 = 0.004;
/// Cap per cell per tick (m) so a time-skip step can't teleport a heap.
const WASH_MAX_PER_TICK_M: f32 = 0.5;
/// Cells processed per tick at most (bounded cost on a flood).
const WASH_MAX_CELLS: usize = 2048;

/// One wash step: `(from tile, to tile, metres)` for every cell fast enough to carry debris.
#[must_use]
pub fn wash_moves(field: &SurfaceWaterField, delta: &GroundDeltaBook, dt_s: f32) -> Vec<(IVec2, IVec2, f32)> {
    let mut out = Vec::new();
    if !field.wired || dt_s <= 0.0 || delta.delta_m.is_empty() {
        return out;
    }
    let (w, h) = (field.width as usize, field.height as usize);
    let dx = field.dx_m.max(1e-3);
    for z in 0..h {
        for x in 0..w {
            let i = z * w + x;
            let d = field.depth_m[i];
            if d < 0.05 {
                continue;
            }
            let t = IVec2::new(field.origin_x + x as i32, field.origin_z + z as i32);
            let loose = delta.delta_at(t.x, t.y);
            if loose <= 0.01 {
                continue;
            }
            // Cell-centred discharge from the two faces (east/north face flux of this cell and
            // of the west/south neighbour).
            let qx = 0.5 * (field.qx_m3s[i] + if x > 0 { field.qx_m3s[i - 1] } else { field.qx_m3s[i] });
            let qz = 0.5 * (field.qy_m3s[i] + if z > 0 { field.qy_m3s[i - w] } else { field.qy_m3s[i] });
            let v = (qx * qx + qz * qz).sqrt() / (d * dx);
            if v <= WASH_THRESHOLD_M_S {
                continue;
            }
            let take = ((v - WASH_THRESHOLD_M_S) * WASH_RATE_M_PER_S * dt_s).min(WASH_MAX_PER_TICK_M).min(loose);
            if take <= 1e-4 {
                continue;
            }
            let step = if qx.abs() >= qz.abs() {
                IVec2::new(qx.signum() as i32, 0)
            } else {
                IVec2::new(0, qz.signum() as i32)
            };
            out.push((t, t + step, take));
            if out.len() >= WASH_MAX_CELLS {
                return out;
            }
        }
    }
    out
}

/// Each sim step, queue the wash moves to terrain (the sole height writer).
pub fn wash_debris_downstream_system(
    field: Res<SurfaceWaterField>,
    delta: Option<Res<GroundDeltaBook>>,
    clock: Res<crate::systems::sim_control::SimStepClock>,
    mut queue: ResMut<GroundDeformQueue>,
) {
    let Some(delta) = delta else {
        return;
    };
    if clock.due == 0 {
        return;
    }
    let dt = clock.step_secs() * clock.due as f32;
    for (from, to, depth_m) in wash_moves(&field, &delta, dt) {
        queue.push(GroundDeformOp::Transfer { from, to, depth_m });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dw5_fast_water_moves_rubble_downstream_slow_water_leaves_it() {
        let mut f = SurfaceWaterField::default();
        f.allocate(0, 0, 4, 4, vec![100.0; 16], 0.0);
        let mut book = GroundDeltaBook { width: 4, height: 4, delta_m: vec![0.0; 16], revision: 0 };
        book.delta_m[1 * 4 + 1] = 2.0; // rubble heap at (1,1)
        let i = f.idx(1, 1);
        f.depth_m[i] = 1.0;
        // Strong flow toward -z (downstream): north-face fluxes negative.
        f.qy_m3s[i] = -60.0;
        f.qy_m3s[i - 4] = -60.0;
        let moves = wash_moves(&f, &book, 10.0);
        assert_eq!(moves.len(), 1);
        assert_eq!(moves[0].0, IVec2::new(1, 1));
        assert_eq!(moves[0].1, IVec2::new(1, 0), "carried downstream along the flow");
        assert!(moves[0].2 > 0.0 && moves[0].2 <= 0.5);
        f.qy_m3s[i] = -1.0;
        f.qy_m3s[i - 4] = -1.0;
        assert!(wash_moves(&f, &book, 10.0).is_empty(), "slow water deposits, never picks up");
    }
}
