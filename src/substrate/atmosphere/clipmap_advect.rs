//! AC-004 — L0 smoke advect + blend helpers.
//! DEBT-010/010b/010c — FieldFill/WindAdvect write L0 channels as authority.
//! DEBT-011 — Field resource + Field↔L0 fold/mirror helpers **deleted**.
//! W7 — `advect_l0_preserving_mass` deleted (obsolete duplicate advect); L0 fog fill/advect
//! retired (weather fog aliases [`crate::systems::atmosphere::AtmosphereFrame`]).

use bevy::math::Vec2;

use super::{AtmosphereClipLevel, AtmosphereClipmapStack};

/// Convert map-plane wind (m/s) into L0 texel drift. One L0 texel = one chunk
/// (`CHUNK_TILES_SIM * TILE_METRES_GAMEPLAY` metres); scale matches the legacy `* 2` kernel.
const L0_ADVECT_TEXEL_PER_MPS: f32 = 2.0 / (32.0 * 16.0);

#[inline]
fn advect_channel_with_wind_field<F>(
    channel: &[f32],
    resolution: bevy::math::UVec2,
    dt: f32,
    decay: f32,
    mut wind_at: F,
) -> Option<Vec<f32>>
where
    F: FnMut(u32, u32) -> Vec2,
{
    if dt <= 0.0 {
        return None;
    }
    let sx = resolution.x as i32;
    let sy = resolution.y as i32;
    if sx <= 0 || sy <= 0 || channel.len() != (resolution.x * resolution.y) as usize {
        return None;
    }
    let mut next = vec![0.0_f32; channel.len()];
    let mut any = false;
    for y in 0..resolution.y {
        for x in 0..resolution.x {
            let wind = wind_at(x, y);
            let speed = wind.length();
            let dst = (y * resolution.x + x) as usize;
            if speed <= 1e-6 {
                next[dst] = (channel[dst] * decay).clamp(0.0, 1.0);
                continue;
            }
            any = true;
            let drift = wind * (dt * L0_ADVECT_TEXEL_PER_MPS);
            let fx = x as f32 - drift.x;
            let fy = y as f32 - drift.y;
            let sx_i = fx.floor() as i32;
            let sy_i = fy.floor() as i32;
            if sx_i < 0 || sy_i < 0 || sx_i >= sx || sy_i >= sy {
                next[dst] = 0.0;
                continue;
            }
            let src_i = (sy_i as u32 * resolution.x + sx_i as u32) as usize;
            next[dst] = (channel[src_i] * decay).clamp(0.0, 1.0);
        }
    }
    any.then_some(next)
}

/// Uniform wind (legacy tests / harness). Prefer [`advect_l0_with_wind_field`].
pub fn advect_l0_with_wind(level: &mut AtmosphereClipLevel, direction: Vec2, speed: f32, dt: f32) {
    let wind = direction.normalize_or_zero() * speed;
    advect_l0_with_wind_field(level, dt, |_, _| wind);
}

/// Per-texel wind (m/s) from `WeatherField::sample` — W2 smoke/ash/ember advection authority.
pub fn advect_l0_with_wind_field(
    level: &mut AtmosphereClipLevel,
    dt: f32,
    mut wind_at: impl FnMut(u32, u32) -> Vec2,
) {
    let res = level.resolution;
    if let Some(next) =
        advect_channel_with_wind_field(&level.smoke_density, res, dt, 0.985, &mut wind_at)
    {
        level.smoke_density = next;
    }
    // W7: do not advect L0 fog — lane is retired weather mirror (zeros / unused).
    if let Some(next) = advect_channel_with_wind_field(&level.toxicity, res, dt, 0.992, &mut wind_at)
    {
        level.toxicity = next;
    }
    if let Some(next) =
        advect_channel_with_wind_field(&level.ash_density, res, dt, 0.98, &mut wind_at)
    {
        level.ash_density = next;
    }
    if let Some(next) =
        advect_channel_with_wind_field(&level.ember_density, res, dt, 0.97, &mut wind_at)
    {
        level.ember_density = next;
    }
    if let Some(next) =
        advect_channel_with_wind_field(&level.heat_distortion, res, dt, 0.96, &mut wind_at)
    {
        level.heat_distortion = next;
    }
}

/// W4 — sole rain washout owner (L0 advect). Matches retired chunk `rain_cleanse` scale:
/// `rain01 = mm_h / 10`, cleanse = `rain01 * 0.45` per second on smoke; toxicity ×0.35.
/// Returns total smoke mass removed.
pub fn apply_l0_rain_washout(
    level: &mut AtmosphereClipLevel,
    dt: f32,
    mut rain_mm_h_at: impl FnMut(u32, u32) -> f32,
) -> f32 {
    if dt <= 0.0 {
        return 0.0;
    }
    let res = level.resolution;
    let n = (res.x * res.y) as usize;
    if level.smoke_density.len() != n {
        return 0.0;
    }
    let mut washed = 0.0f32;
    for y in 0..res.y {
        for x in 0..res.x {
            let rain_mm_h = rain_mm_h_at(x, y);
            if rain_mm_h <= 1e-4 {
                continue;
            }
            let rain01 = (rain_mm_h / 10.0).clamp(0.0, 1.0);
            let cleanse = rain01 * 0.45 * dt;
            let i = (y * res.x + x) as usize;
            let before = level.smoke_density[i];
            let after = (before - cleanse).clamp(0.0, 1.0);
            washed += before - after;
            level.smoke_density[i] = after;
            if let Some(tox) = level.toxicity.get_mut(i) {
                *tox = (*tox - cleanse * 0.35).clamp(0.0, 1.0);
            }
            if let Some(ash) = level.ash_density.get_mut(i) {
                *ash = (*ash - cleanse * 0.5).clamp(0.0, 1.0);
            }
        }
    }
    washed
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
    fn rain_washout_reduces_smoke_mass() {
        let mut level = super::super::AtmosphereClipLevel {
            resolution: UVec2::new(2, 1),
            smoke_density: vec![0.8, 0.5],
            fog_density: vec![0.0; 2],
            toxicity: vec![0.4, 0.2],
            ash_density: vec![0.3, 0.1],
            ember_density: vec![0.0; 2],
            heat_distortion: vec![0.0; 2],
        };
        let before: f32 = level.smoke_density.iter().sum();
        let washed = apply_l0_rain_washout(&mut level, 1.0, |x, _| {
            if x == 0 {
                10.0
            } else {
                0.0
            }
        });
        let after: f32 = level.smoke_density.iter().sum();
        assert!(washed > 0.0);
        assert!(after < before);
        assert!(level.smoke_density[0] < 0.8);
        assert!((level.smoke_density[1] - 0.5).abs() < 1e-5);
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
