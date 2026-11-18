//! DEBRIS-FX — material-tint dust burst on collapse (reuses fire_vfx particle spine).

use bevy::prelude::*;

use crate::render::{GpuParticleInstance, ParticleClass, WorldFireParticleFrame};

/// Max quads per demolish event (covers remesh lag without flooding the fire lane).
pub const DEBRIS_BURST_MAX: usize = 24;

/// Push a short dust/concrete burst at footprint tiles into the shared particle frame.
pub fn emit_collapse_debris_burst(
    frame: &mut WorldFireParticleFrame,
    tiles: impl IntoIterator<Item = IVec2>,
) {
    let class = ParticleClass::Smoke.as_f32();
    let mut n = 0usize;
    for t in tiles {
        if n >= DEBRIS_BURST_MAX || frame.instances.len() >= frame.gpu_capacity {
            break;
        }
        // PixelWorld: X = tile x, Y up, Z = -tile y. Low heat = dust, not flame.
        let x = t.x as f32 + 0.5;
        let z = -(t.y as f32 + 0.5);
        frame.instances.push(GpuParticleInstance {
            world_xyz_heat: Vec4::new(x, 1.2, z, 0.12),
            ember_class_radius_smoke: Vec4::new(0.25, class, 0.55, 0.85),
        });
        n += 1;
    }
    if n > 0 {
        frame.snapshot_stamp = frame.snapshot_stamp.wrapping_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debris_fx_burst_pushes_smoke_class_rows() {
        let mut frame = WorldFireParticleFrame::default();
        emit_collapse_debris_burst(&mut frame, [IVec2::new(2, 3), IVec2::new(3, 3)]);
        assert_eq!(frame.instances.len(), 2);
        assert!((frame.instances[0].ember_class_radius_smoke.y - ParticleClass::Smoke.as_f32()).abs() < 1e-5);
        assert!(frame.instances[0].world_xyz_heat.w < 0.3);
    }
}
