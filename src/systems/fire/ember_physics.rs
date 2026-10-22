//! Shared **ember / spark / flame** physics — single authority for the visual particles
//! ([`crate::render::fire_vfx`]) and the sim's long-range spot ignition
//! ([`super::ember_spot_ignition`]).
//!
//! Units are SI. One sim tile = [`FIRE_TILE_METERS`] metres **for fire physics**; this is a
//! deliberately local scale (a 3x3-tile building is ~12 m), not the 100 m lore scale in
//! `world_scale_contract::TILE_METERS_SYMBOLIC`.
//!
//! Ember model (closed form, stateless, deterministic):
//! * air velocity = wind (horizontal) + plume updraft `W0 * exp(-t / TAU_BUOYANCY)` (vertical);
//! * linear drag toward the air velocity with rate `k = g / v_terminal(size)`;
//! * gravity always acts; once the plume updraft has decayed the ember falls at `v_terminal`;
//! * temperature cools exponentially and reaches the just-visible 700 K at the ember lifetime.
//!
//! Wind: the sim only carries a **scalar** `ChunkWeather::wind_speed` in `[0, 1]`; direction is the
//! shared [`PREVAILING_WIND_DIR`] plus a per-ember angular scatter. A real wind-direction field
//! would replace [`wind_vector_mps`] only.

use bevy::math::{Vec2, Vec3};

/// Metres per sim tile for fire / ember physics.
pub const FIRE_TILE_METERS: f32 = 4.0;
/// `ChunkWeather::wind_speed == 1.0` in m/s.
pub const WIND_FULL_SCALE_MPS: f32 = 15.0;
pub const GRAVITY_MPS2: f32 = 9.81;
/// Prevailing wind heading in the map plane (unit vector), 30 degrees off +x.
pub const PREVAILING_WIND_DIR: Vec2 = Vec2::new(0.866_025_4, 0.5);
/// Time constant (s) with which the plume updraft decays as an ember leaves the column.
pub const TAU_BUOYANCY_S: f32 = 0.9;
/// Ember lifetime bounds (s): glow lifetime grows with ember size.
pub const EMBER_LIFETIME_MIN_S: f32 = 0.5;
pub const EMBER_LIFETIME_MAX_S: f32 = 4.0;
/// Ambient / visibility temperatures (K).
pub const T_AMBIENT_K: f32 = 300.0;
pub const T_DARK_K: f32 = 700.0;
pub const T_WHITE_K: f32 = 1500.0;
/// Heat-to-screen: metres of height drawn as this fraction of a tile of screen-up offset
/// (oblique projection of the top-down map).
pub const HEIGHT_TO_SCREEN: f32 = 0.55;

/// Flame flicker band (Hz) — base components, before per-emitter detune.
pub const FLICKER_HZ: [f32; 3] = [2.6, 5.4, 9.3];
const FLICKER_WEIGHT: [f32; 3] = [0.5, 0.3, 0.2];
/// Per-emitter detune of [`FLICKER_HZ`] (±15 %) keeps every component in 2.2–10.7 Hz.
pub const FLICKER_DETUNE: f32 = 0.15;

#[inline]
#[must_use]
pub fn hash01(a: u32, b: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA6B).rotate_left(13);
    h ^= h >> 16;
    h = h.wrapping_mul(0x7FEB_352D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846C_A68B);
    h ^= h >> 16;
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// `ChunkWeather::wind_speed` (normalised) → m/s.
#[inline]
#[must_use]
pub fn wind_speed_mps(wind01: f32) -> f32 {
    wind01.clamp(0.0, 1.0) * WIND_FULL_SCALE_MPS
}

/// Wind vector (m/s) in the map plane. `scatter` in `[-1, 1]` rotates the heading: up to ±40° in
/// calm air (turbulent), ±10° in strong wind (steady).
#[inline]
#[must_use]
pub fn wind_vector_mps(wind01: f32, scatter: f32) -> Vec2 {
    let w = wind01.clamp(0.0, 1.0);
    let max_angle = (40.0 - 30.0 * w).to_radians();
    let a = scatter.clamp(-1.0, 1.0) * max_angle;
    let (s, c) = a.sin_cos();
    let d = PREVAILING_WIND_DIR;
    Vec2::new(d.x * c - d.y * s, d.x * s + d.y * c) * wind_speed_mps(w)
}

/// Ember launch descriptor. `size01`: 0 = fine spark, 1 = large glowing brand.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EmberLaunch {
    /// Initial velocity (m/s), z up.
    pub v0: Vec3,
    /// Fire intensity `[0, 1]` (drives plume updraft and initial temperature).
    pub intensity: f32,
    pub size01: f32,
}

impl EmberLaunch {
    /// Deterministic launch from three uniform randoms in `[0, 1)`.
    #[must_use]
    pub fn from_randoms(intensity: f32, size01: f32, r_angle: f32, r_speed: f32, r_up: f32) -> Self {
        let a = r_angle * std::f32::consts::TAU;
        let h = (0.3 + 1.7 * r_speed) * (0.5 + 0.5 * intensity);
        let up = plume_updraft_mps(intensity) * (0.35 + 0.3 * r_up);
        Self {
            v0: Vec3::new(a.cos() * h, a.sin() * h, up),
            intensity: intensity.clamp(0.0, 1.0),
            size01: size01.clamp(0.0, 1.0),
        }
    }
}

/// Plume updraft speed at the fire (m/s): 2 m/s smouldering to 12 m/s intense.
#[inline]
#[must_use]
pub fn plume_updraft_mps(intensity: f32) -> f32 {
    2.0 + 10.0 * intensity.clamp(0.0, 1.0)
}

/// Ember glow lifetime (s): 0.5 s fine spark to 4 s large brand.
#[inline]
#[must_use]
pub fn ember_lifetime_s(size01: f32) -> f32 {
    EMBER_LIFETIME_MIN_S + (EMBER_LIFETIME_MAX_S - EMBER_LIFETIME_MIN_S) * size01.clamp(0.0, 1.0)
}

/// Still-air terminal fall speed (m/s): 1.5 fine to 4.0 large.
#[inline]
#[must_use]
pub fn terminal_speed_mps(size01: f32) -> f32 {
    1.5 + 2.5 * size01.clamp(0.0, 1.0)
}

/// Position (m, z up, relative to the launch point) and velocity (m/s) at age `t` seconds.
#[must_use]
pub fn ember_kinematics(l: &EmberLaunch, wind_mps: Vec2, t: f32) -> (Vec3, Vec3) {
    let t = t.max(0.0);
    let k = GRAVITY_MPS2 / terminal_speed_mps(l.size01);
    let tau = TAU_BUOYANCY_S;
    let w0 = plume_updraft_mps(l.intensity);
    let a = k * w0 / (k - 1.0 / tau);
    let c = l.v0.z - a + GRAVITY_MPS2 / k;
    let e_t = (-t / tau).exp();
    let e_k = (-k * t).exp();
    let vz = a * e_t - GRAVITY_MPS2 / k + c * e_k;
    let z = a * tau * (1.0 - e_t) - GRAVITY_MPS2 / k * t + c / k * (1.0 - e_k);
    let rel = l.v0.truncate() - wind_mps;
    let xy = wind_mps * t + rel * (1.0 - e_k) / k;
    let vxy = wind_mps + rel * e_k;
    (Vec3::new(xy.x, xy.y, z), Vec3::new(vxy.x, vxy.y, vz))
}

/// Ember temperature (K) at age `t`: exponential cooling that reaches [`T_DARK_K`] at the
/// lifetime. Initial temperature 1100–1500 K with intensity.
#[must_use]
pub fn ember_temperature_k(intensity: f32, size01: f32, t: f32) -> f32 {
    let t0 = 1100.0 + 400.0 * intensity.clamp(0.0, 1.0);
    let life = ember_lifetime_s(size01);
    let tau_c = life / ((t0 - T_AMBIENT_K) / (T_DARK_K - T_AMBIENT_K)).ln();
    T_AMBIENT_K + (t0 - T_AMBIENT_K) * (-t.max(0.0) / tau_c).exp()
}

/// Normalised glow temperature `[0, 1]`: 0 at 700 K (dark), 1 at 1500 K (white-yellow).
#[inline]
#[must_use]
pub fn glow_u(temp_k: f32) -> f32 {
    ((temp_k - T_DARK_K) / (T_WHITE_K - T_DARK_K)).clamp(0.0, 1.0)
}

/// Blackbody-like glow colour (linear RGB, premultiplied radiance) for [`glow_u`].
/// Mirrored by `glow_rgb` in `assets/shaders/fire/fire_particle_draw.wgsl`.
#[must_use]
pub fn glow_rgb(u: f32) -> Vec3 {
    const KEYS: [(f32, Vec3); 4] = [
        (0.0, Vec3::new(0.060, 0.006, 0.003)),
        (0.25, Vec3::new(0.30, 0.040, 0.011)),
        (0.625, Vec3::new(0.85, 0.44, 0.085)),
        (1.0, Vec3::new(1.00, 0.93, 0.72)),
    ];
    let u = u.clamp(0.0, 1.0);
    for w in KEYS.windows(2) {
        let (u0, c0) = w[0];
        let (u1, c1) = w[1];
        if u <= u1 {
            return c0.lerp(c1, (u - u0) / (u1 - u0));
        }
    }
    KEYS[3].1
}

/// Smooth glow envelope over normalised age: quick fade-in, long fade-out (never a hard cut).
#[inline]
#[must_use]
pub fn ember_alpha(age01: f32) -> f32 {
    let a = age01.clamp(0.0, 1.0);
    let fin = (a / 0.06).clamp(0.0, 1.0);
    let fout = 1.0 - ((a - 0.7) / 0.3).clamp(0.0, 1.0);
    let sm = |x: f32| x * x * (3.0 - 2.0 * x);
    sm(fin) * sm(fout)
}

/// Landing of an ember: `(time_s, horizontal offset in m)` or `None` if it does not return to
/// launch height within 12 s.
#[must_use]
pub fn ember_landing(l: &EmberLaunch, wind_mps: Vec2) -> Option<(f32, Vec2)> {
    let dt = 0.1;
    let mut t_prev = 0.0;
    let mut t = dt;
    let mut peaked = false;
    while t <= 12.0 {
        let (p, v) = ember_kinematics(l, wind_mps, t);
        peaked |= v.z <= 0.0;
        if peaked && p.z <= 0.0 {
            let (mut lo, mut hi) = (t_prev, t);
            for _ in 0..12 {
                let mid = 0.5 * (lo + hi);
                if ember_kinematics(l, wind_mps, mid).0.z > 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            let tl = 0.5 * (lo + hi);
            let (pl, _) = ember_kinematics(l, wind_mps, tl);
            return Some((tl, pl.truncate()));
        }
        t_prev = t;
        t += dt;
    }
    None
}

/// Sim spot-ignition landing offset in **tiles** for one ember, or `None` if the ember is dark
/// (cooled below visibility) before it lands — it then cannot ignite anything. Same model, wind
/// and tile scale as the visual embers.
#[must_use]
pub fn spot_landing_offset_tiles(
    intensity: f32,
    wind01: f32,
    size01: f32,
    r_angle: f32,
    r_speed: f32,
    r_up: f32,
    r_wind_scatter: f32,
) -> Option<Vec2> {
    let wind = wind_vector_mps(wind01, r_wind_scatter * 2.0 - 1.0);
    let launch = EmberLaunch::from_randoms(intensity, size01, r_angle, r_speed, r_up);
    let (t_land, off) = ember_landing(&launch, wind)?;
    (t_land <= ember_lifetime_s(size01)).then_some(off / FIRE_TILE_METERS)
}

/// Flame flicker in `[-1, 1]`: three incommensurate components in the 2.2–10.7 Hz band, each
/// detuned and phase-shifted per emitter (`seed`), so neighbours never pulse in sync.
#[must_use]
pub fn flame_flicker(t_secs: f32, seed: u32) -> f32 {
    let mut sum = 0.0;
    for i in 0..3 {
        let detune = 1.0 + FLICKER_DETUNE * (hash01(seed, 11 + i as u32) * 2.0 - 1.0);
        let phase = hash01(seed, 31 + i as u32) * std::f32::consts::TAU;
        sum += FLICKER_WEIGHT[i] * (std::f32::consts::TAU * FLICKER_HZ[i] * detune * t_secs + phase).sin();
    }
    sum
}

/// Burn stage of one cell, from the overlay's heat / fuel / burn-hold.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BurnStage {
    Ignition,
    Flaming,
    Smouldering,
    Out,
}

/// Per-stage multipliers on flame / ember / smoke emission.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StageEmission {
    pub ember: f32,
    pub smoke: f32,
}

impl BurnStage {
    #[must_use]
    pub fn classify(heat: f32, fuel: f32, burn_hold: u16) -> Self {
        use super::combustion::MIN_CELL_FUEL_FOR_SPREAD;
        if heat < 0.02 || (fuel <= 0.0 && burn_hold == 0) {
            Self::Out
        } else if burn_hold > 0 || fuel < MIN_CELL_FUEL_FOR_SPREAD {
            Self::Smouldering
        } else if heat < 0.25 {
            Self::Ignition
        } else {
            Self::Flaming
        }
    }

    #[must_use]
    pub fn emission(self) -> StageEmission {
        match self {
            Self::Ignition => StageEmission { ember: 0.25, smoke: 0.6 },
            Self::Flaming => StageEmission { ember: 1.0, smoke: 1.0 },
            Self::Smouldering => StageEmission { ember: 0.12, smoke: 1.4 },
            Self::Out => StageEmission { ember: 0.0, smoke: 0.0 },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colour_ramp_cools_monotonically_to_dark() {
        let luma = |c: Vec3| 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
        for &(intensity, size) in &[(0.2, 0.1), (0.7, 0.5), (1.0, 1.0)] {
            let life = ember_lifetime_s(size);
            let mut prev_t = f32::MAX;
            let mut prev_l = f32::MAX;
            for i in 0..=40 {
                let t = life * i as f32 / 40.0;
                let temp = ember_temperature_k(intensity, size, t);
                let c = glow_rgb(glow_u(temp));
                assert!(temp <= prev_t + 1e-3 && luma(c) <= prev_l + 1e-5);
                prev_t = temp;
                prev_l = luma(c);
            }
            assert!(glow_u(ember_temperature_k(intensity, size, life)) < 1e-3, "dark at end of life");
        }
        let hot = glow_rgb(1.0);
        assert!(hot.y > 0.9 && hot.x > hot.z, "white-yellow at the top");
    }

    #[test]
    fn embers_rise_then_fall_and_drift_downwind() {
        let wind = wind_vector_mps(0.5, 0.0);
        let l = EmberLaunch::from_randoms(0.9, 0.5, 0.0, 0.5, 0.5);
        let (p_early, v_early) = ember_kinematics(&l, wind, 0.3);
        assert!(p_early.z > 0.0 && v_early.z > 0.0, "buoyant rise first");
        let (_, v_late) = ember_kinematics(&l, wind, 3.0);
        assert!(v_late.z < 0.0, "falls once the updraft decays");
        let (t_land, off) = ember_landing(&l, wind).expect("lands");
        assert!(t_land > 0.5 && t_land < 12.0);
        assert!(off.normalize().dot(PREVAILING_WIND_DIR) > 0.9, "drift follows the wind");
        let calm = ember_landing(&l, Vec2::ZERO).expect("lands").1.length();
        assert!(off.length() > 2.0 * calm.max(1.0), "wind carries it further than calm air");
    }

    #[test]
    fn dead_cells_emit_nothing_and_lifetimes_are_bounded() {
        assert_eq!(BurnStage::classify(0.0, 0.0, 0), BurnStage::Out);
        assert_eq!(BurnStage::classify(0.5, 0.0, 0), BurnStage::Out);
        assert_eq!(BurnStage::Out.emission().ember, 0.0);
        assert_eq!(BurnStage::Out.emission().smoke, 0.0);
        assert!(BurnStage::classify(0.05, 0.0, 3) == BurnStage::Smouldering);
        assert!(BurnStage::Smouldering.emission().smoke > BurnStage::Flaming.emission().smoke);
        for i in 0..=10 {
            let life = ember_lifetime_s(i as f32 / 10.0);
            assert!((0.5..=4.0).contains(&life));
        }
        // Flicker stays bounded and decorrelates neighbours.
        let a: f32 = (0..200).map(|i| flame_flicker(i as f32 * 0.05, 1)).sum();
        assert!(flame_flicker(1.0, 1) != flame_flicker(1.0, 2) && a.is_finite());
    }
}
