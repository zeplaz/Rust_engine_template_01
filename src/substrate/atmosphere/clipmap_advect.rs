//! AC-004 — L0 smoke advect + blend helpers (mass-preserving stub).
//! DEBT-010/010b/010c — FieldFill/WindAdvect write L0 channels as authority.
//! DEBT-011 — Field resource + Field↔L0 fold/mirror helpers **deleted**.

use bevy::math::Vec2;

use super::{AtmosphereClipLevel, AtmosphereClipmapStack};

/// Semi-Lagrangian-style neighbor blend that preserves total mass on L0 smoke.
pub fn advect_l0_preserving_mass(level: &mut AtmosphereClipLevel) {
    let n = level.smoke_density.len();
    if n < 2 {
        return;
    }
    let total_before: f32 = level.smoke_density.iter().sum();
    if total_before <= 0.0 {
        return;
    }

    let mut next = level.smoke_density.clone();
    for i in 1..n {
        let blended = (level.smoke_density[i] + level.smoke_density[i - 1]) * 0.5;
        next[i] = blended;
        next[i - 1] = blended;
    }

    let total_after: f32 = next.iter().sum();
    if total_after > 1e-6 {
        let scale = total_before / total_after;
        for v in &mut next {
            *v *= scale;
        }
    }
    level.smoke_density = next;
}

#[inline]
fn advect_channel_with_wind(
    channel: &[f32],
    resolution: bevy::math::UVec2,
    direction: Vec2,
    speed: f32,
    dt: f32,
    decay: f32,
) -> Option<Vec<f32>> {
    if dt <= 0.0 || speed <= 1e-6 {
        return None;
    }
    let dir = direction.normalize_or_zero();
    if dir.length_squared() <= 1e-8 {
        return None;
    }
    let sx = resolution.x as i32;
    let sy = resolution.y as i32;
    if sx <= 0 || sy <= 0 || channel.len() != (resolution.x * resolution.y) as usize {
        return None;
    }
    let mut next = vec![0.0_f32; channel.len()];
    for y in 0..resolution.y {
        for x in 0..resolution.x {
            let fx = x as f32 - dir.x * speed * dt * 2.0;
            let fy = y as f32 - dir.y * speed * dt * 2.0;
            let sx_i = fx.floor() as i32;
            let sy_i = fy.floor() as i32;
            let dst = (y * resolution.x + x) as usize;
            if sx_i < 0 || sy_i < 0 || sx_i >= sx || sy_i >= sy {
                next[dst] = 0.0;
                continue;
            }
            let src_i = (sy_i as u32 * resolution.x + sx_i as u32) as usize;
            next[dst] = (channel[src_i] * decay).clamp(0.0, 1.0);
        }
    }
    Some(next)
}

/// DEBT-010 / 010b / 010c — wind-driven drift on L0 smoke/fog/toxicity/ash/ember/heat.
pub fn advect_l0_with_wind(level: &mut AtmosphereClipLevel, direction: Vec2, speed: f32, dt: f32) {
    let res = level.resolution;
    if let Some(next) = advect_channel_with_wind(&level.smoke_density, res, direction, speed, dt, 0.985)
    {
        level.smoke_density = next;
    }
    if let Some(next) = advect_channel_with_wind(&level.fog_density, res, direction, speed, dt, 0.99)
    {
        level.fog_density = next;
    }
    if let Some(next) =
        advect_channel_with_wind(&level.toxicity, res, direction, speed, dt, 0.992)
    {
        level.toxicity = next;
    }
    if let Some(next) =
        advect_channel_with_wind(&level.ash_density, res, direction, speed, dt, 0.98)
    {
        level.ash_density = next;
    }
    if let Some(next) =
        advect_channel_with_wind(&level.ember_density, res, direction, speed, dt, 0.97)
    {
        level.ember_density = next;
    }
    if let Some(next) =
        advect_channel_with_wind(&level.heat_distortion, res, direction, speed, dt, 0.96)
    {
        level.heat_distortion = next;
    }
}

#[inline]
fn max_blend_l0_channel_at(
    channel: &mut [f32],
    resolution: bevy::math::UVec2,
    x: u32,
    y: u32,
    value: f32,
) {
    if value <= 0.0 || x >= resolution.x || y >= resolution.y {
        return;
    }
    let idx = (y * resolution.x + x) as usize;
    if let Some(cell) = channel.get_mut(idx) {
        *cell = (*cell).max(value.clamp(0.0, 1.0));
    }
}

/// DEBT-010 — max-blend a smoke sample into L0 at grid `(x, y)`.
pub fn max_blend_l0_smoke_at(stack: &mut AtmosphereClipmapStack, x: u32, y: u32, smoke: f32) {
    let Some(level0) = stack.levels.first_mut() else {
        return;
    };
    let res = level0.resolution;
    max_blend_l0_channel_at(&mut level0.smoke_density, res, x, y, smoke);
}

/// DEBT-010b — max-blend fog into L0.
pub fn max_blend_l0_fog_at(stack: &mut AtmosphereClipmapStack, x: u32, y: u32, fog: f32) {
    let Some(level0) = stack.levels.first_mut() else {
        return;
    };
    let res = level0.resolution;
    max_blend_l0_channel_at(&mut level0.fog_density, res, x, y, fog);
}

/// DEBT-010b — max-blend toxicity into L0.
pub fn max_blend_l0_toxicity_at(stack: &mut AtmosphereClipmapStack, x: u32, y: u32, toxicity: f32) {
    let Some(level0) = stack.levels.first_mut() else {
        return;
    };
    let res = level0.resolution;
    max_blend_l0_channel_at(&mut level0.toxicity, res, x, y, toxicity);
}

/// DEBT-010c — max-blend ash into L0.
pub fn max_blend_l0_ash_at(stack: &mut AtmosphereClipmapStack, x: u32, y: u32, ash: f32) {
    let Some(level0) = stack.levels.first_mut() else {
        return;
    };
    let res = level0.resolution;
    max_blend_l0_channel_at(&mut level0.ash_density, res, x, y, ash);
}

/// DEBT-010c — max-blend ember into L0.
pub fn max_blend_l0_ember_at(stack: &mut AtmosphereClipmapStack, x: u32, y: u32, ember: f32) {
    let Some(level0) = stack.levels.first_mut() else {
        return;
    };
    let res = level0.resolution;
    max_blend_l0_channel_at(&mut level0.ember_density, res, x, y, ember);
}

/// DEBT-010c — max-blend heat distortion into L0.
pub fn max_blend_l0_heat_at(stack: &mut AtmosphereClipmapStack, x: u32, y: u32, heat: f32) {
    let Some(level0) = stack.levels.first_mut() else {
        return;
    };
    let res = level0.resolution;
    max_blend_l0_channel_at(&mut level0.heat_distortion, res, x, y, heat);
}

pub fn fold_registry_smoke_into_l0(stack: &mut AtmosphereClipmapStack, smoke_seed: f32) {
    let Some(level0) = stack.levels.first_mut() else {
        return;
    };
    if let Some(cell0) = level0.smoke_density.first_mut() {
        *cell0 = cell0.max(smoke_seed);
    }
}

#[inline]
fn sample_l0_channel(channel: &[f32], resolution: bevy::math::UVec2, x: u32, y: u32) -> Option<f32> {
    if x >= resolution.x || y >= resolution.y {
        return None;
    }
    let idx = (y * resolution.x + x) as usize;
    channel.get(idx).copied()
}

/// ES-6-2 — tactical smoke sample from clipmap L0 (authoritative for clipmap consumers).
#[must_use]
pub fn sample_tactical_smoke_from_l0(
    stack: &AtmosphereClipmapStack,
    x: u32,
    y: u32,
) -> Option<f32> {
    let level0 = stack.levels.first()?;
    sample_l0_channel(&level0.smoke_density, level0.resolution, x, y)
}

/// DEBT-010b — tactical fog sample from clipmap L0.
#[must_use]
pub fn sample_tactical_fog_from_l0(
    stack: &AtmosphereClipmapStack,
    x: u32,
    y: u32,
) -> Option<f32> {
    let level0 = stack.levels.first()?;
    sample_l0_channel(&level0.fog_density, level0.resolution, x, y)
}

/// DEBT-010b — tactical toxicity sample from clipmap L0.
#[must_use]
pub fn sample_tactical_toxicity_from_l0(
    stack: &AtmosphereClipmapStack,
    x: u32,
    y: u32,
) -> Option<f32> {
    let level0 = stack.levels.first()?;
    sample_l0_channel(&level0.toxicity, level0.resolution, x, y)
}

/// DEBT-010c — tactical ash sample from clipmap L0.
#[must_use]
pub fn sample_tactical_ash_from_l0(
    stack: &AtmosphereClipmapStack,
    x: u32,
    y: u32,
) -> Option<f32> {
    let level0 = stack.levels.first()?;
    sample_l0_channel(&level0.ash_density, level0.resolution, x, y)
}

/// DEBT-010c — tactical ember sample from clipmap L0.
#[must_use]
pub fn sample_tactical_ember_from_l0(
    stack: &AtmosphereClipmapStack,
    x: u32,
    y: u32,
) -> Option<f32> {
    let level0 = stack.levels.first()?;
    sample_l0_channel(&level0.ember_density, level0.resolution, x, y)
}

/// DEBT-010c — tactical heat-distortion sample from clipmap L0.
#[must_use]
pub fn sample_tactical_heat_from_l0(
    stack: &AtmosphereClipmapStack,
    x: u32,
    y: u32,
) -> Option<f32> {
    let level0 = stack.levels.first()?;
    sample_l0_channel(&level0.heat_distortion, level0.resolution, x, y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::UVec2;

    #[test]
    fn clipmap_advect_preserves_mass_approximately() {
        let n = 4;
        let mut level = super::super::AtmosphereClipLevel {
            resolution: UVec2::new(4, 1),
            smoke_density: vec![0.1, 0.2, 0.3, 0.4],
            fog_density: vec![0.0; n],
            toxicity: vec![0.0; n],
            ash_density: vec![0.0; n],
            ember_density: vec![0.0; n],
            heat_distortion: vec![0.0; n],
        };
        let before: f32 = level.smoke_density.iter().sum();
        advect_l0_preserving_mass(&mut level);
        let after: f32 = level.smoke_density.iter().sum();
        assert!((before - after).abs() < 1e-4);
        assert!(level.smoke_density.iter().any(|v| *v > 0.0));
    }

    #[test]
    fn l0_max_blend_all_channels() {
        use super::super::AtmosphereClipmapStack;

        let mut stack = AtmosphereClipmapStack::default();
        max_blend_l0_smoke_at(&mut stack, 0, 0, 0.42);
        max_blend_l0_fog_at(&mut stack, 1, 0, 0.8);
        max_blend_l0_toxicity_at(&mut stack, 2, 0, 0.55);
        max_blend_l0_ash_at(&mut stack, 3, 0, 0.31);
        max_blend_l0_ember_at(&mut stack, 4, 0, 0.22);
        max_blend_l0_heat_at(&mut stack, 5, 0, 0.18);
        assert_eq!(sample_tactical_smoke_from_l0(&stack, 0, 0), Some(0.42));
        assert_eq!(sample_tactical_fog_from_l0(&stack, 1, 0), Some(0.8));
        assert_eq!(sample_tactical_toxicity_from_l0(&stack, 2, 0), Some(0.55));
        assert_eq!(sample_tactical_ash_from_l0(&stack, 3, 0), Some(0.31));
        assert_eq!(sample_tactical_ember_from_l0(&stack, 4, 0), Some(0.22));
        assert_eq!(sample_tactical_heat_from_l0(&stack, 5, 0), Some(0.18));
        assert!((stack.levels[0].smoke_density[1]).abs() < 1e-5);
    }

    #[test]
    fn debt_010c_l0_write_advect_roundtrip() {
        use bevy::math::Vec2;
        use super::super::AtmosphereClipmapStack;

        let mut stack = AtmosphereClipmapStack::default();
        max_blend_l0_smoke_at(&mut stack, 0, 0, 0.55);
        max_blend_l0_fog_at(&mut stack, 0, 0, 0.33);
        max_blend_l0_toxicity_at(&mut stack, 0, 0, 0.44);
        max_blend_l0_ash_at(&mut stack, 0, 0, 0.21);
        max_blend_l0_ember_at(&mut stack, 0, 0, 0.17);
        max_blend_l0_heat_at(&mut stack, 0, 0, 0.19);
        assert!((stack.levels[0].ash_density[0] - 0.21).abs() < 1e-5);

        if let Some(level0) = stack.levels.first_mut() {
            advect_l0_with_wind(level0, Vec2::X, 0.0, 0.016);
        }
        // speed 0 → no-op
        assert!((stack.levels[0].smoke_density[0] - 0.55).abs() < 1e-5);
        assert!((stack.levels[0].ash_density[0] - 0.21).abs() < 1e-5);
        assert_eq!(sample_tactical_fog_from_l0(&stack, 0, 0), Some(0.33));
        assert_eq!(sample_tactical_toxicity_from_l0(&stack, 0, 0), Some(0.44));
        assert_eq!(sample_tactical_ember_from_l0(&stack, 0, 0), Some(0.17));
        assert_eq!(sample_tactical_heat_from_l0(&stack, 0, 0), Some(0.19));
    }
}
