//! Stateless, deterministic fire particle animation: flame flicker, ember flight, smoke plume.
//!
//! One [`FireEmitter`] (a burning cell from the authoritative `FireVisualFrame` rows) expands into
//! instanced-quad rows for the *current* time. Physics lives in
//! [`crate::systems::fire::ember_physics`] (shared with sim spot ignition); this module only maps
//! it onto [`GpuParticleInstance`] rows. Row packing consumed by `fire_particle.wgsl`:
//!
//! * `world_xyz_heat` = (map x, map y, plane z, glow temperature `u` — or smoke density for smoke);
//! * `ember_class_radius_smoke` = (alpha envelope, class id, **physical** half-edge in tiles, 0).
//!
//! Screen-space minimum size and the matching alpha compensation (no pop across zoom) are applied
//! in the shader, which knows the camera zoom.

use bevy::math::{Vec2, Vec4};

use crate::systems::fire::ember_physics::{
    ember_alpha, ember_kinematics, ember_lifetime_s, ember_temperature_k, flame_flicker, glow_u,
    hash01, wind_vector_from_field, EmberLaunch, FIRE_TILE_METERS, HEIGHT_TO_SCREEN,
};

use super::pack::{GpuParticleInstance, ParticleClass};

/// Map-plane spark height. Same value as private `PLANE_Z` in `world_map_network_mesh`.
pub const FIRE_SPARK_MAP_PLANE_Z: f32 = 1.04;

/// Primary flame loop period (s). Phase uses wall-clock time so LOD crossfades never reset it.
pub const FLAME_LOOP_PERIOD_S: f32 = 2.4;

/// Normalized loop phase in `[0, 1)` — `fract(global_time / period + hash(cell))` (ANIM-01).
#[inline]
#[must_use]
pub fn flame_loop_phase(global_time_s: f32, cell_xy: Vec2, period_s: f32) -> f32 {
    let period = period_s.max(1e-3);
    let hash = hash01(emitter_seed(cell_xy), 7);
    (global_time_s / period + hash).fract()
}

/// Smoke puff cycle (s) bounds and rise speed (m/s).
const SMOKE_CYCLE_S: (f32, f32) = (6.0, 10.0);
const SMOKE_RISE_MPS: f32 = 2.2;

/// One burning cell, ready to animate.
#[derive(Clone, Copy, Debug)]
pub struct FireEmitter {
    /// Cell centre on the map (tile units).
    pub xy: Vec2,
    /// Sim heat `[0, 1]` (flame size / ember temperature).
    pub heat: f32,
    /// Stage-scaled ember generation (`[0, 1.5]`, 0 when smouldering/out).
    pub ember_rate: f32,
    /// Stage-scaled smoke density `[0, 1]`.
    pub smoke: f32,
    /// LOD crossfade amplitudes `[0, 1]` (eased by the caller — never hard-cut).
    pub flame_amp: f32,
    pub ember_amp: f32,
    pub smoke_amp: f32,
    /// Ember slots budgeted for this cell (from heat and budget pressure).
    pub ember_slots: usize,
    /// Class used for flame/ember rows (`Spark` full detail, `Ember` low flame).
    pub class: ParticleClass,
    /// Local map-plane wind at the emitter (m/s); falls back to [`FireAnim::wind_mps`].
    pub wind_mps: Vec2,
}

/// Frame-global animation inputs.
#[derive(Clone, Copy, Debug)]
pub struct FireAnim {
    pub time_s: f32,
    /// Fallback / mean wind (m/s) when an emitter has no local sample.
    pub wind_mps: Vec2,
}

#[inline]
fn emitter_seed(xy: Vec2) -> u32 {
    (xy.x.floor() as i32 as u32).wrapping_mul(73_856_093) ^ (xy.y.floor() as i32 as u32).wrapping_mul(19_349_663)
}

#[inline]
fn pack(xy: Vec2, heat: f32, alpha: f32, class: ParticleClass, half_tiles: f32) -> GpuParticleInstance {
    GpuParticleInstance {
        world_xyz_heat: Vec4::new(xy.x, xy.y, FIRE_SPARK_MAP_PLANE_Z, heat),
        ember_class_radius_smoke: Vec4::new(alpha, class.as_f32(), half_tiles, 0.0),
    }
}

/// Screen position of a point `p_m` metres from the emitter (height drawn as an oblique offset).
#[inline]
fn project(origin: Vec2, off_m: Vec2, z_m: f32) -> Vec2 {
    origin + off_m / FIRE_TILE_METERS + Vec2::new(0.0, z_m * HEIGHT_TO_SCREEN / FIRE_TILE_METERS)
}

/// Append every particle row for `e` at `anim.time_s`.
pub fn animate_emitter(e: &FireEmitter, anim: &FireAnim, out: &mut Vec<GpuParticleInstance>) {
    let t = anim.time_s;
    let seed = emitter_seed(e.xy);
    let heat = e.heat.clamp(0.0, 1.0);

    // Flame core: global loop phase (LOD-safe) plus incommensurate flicker.
    if e.flame_amp > 0.01 && heat > 0.0 {
        let loop_ph = flame_loop_phase(t, e.xy, FLAME_LOOP_PERIOD_S);
        let loop_wobble = (loop_ph * std::f32::consts::TAU).sin();
        let fl = flame_flicker(t, seed) * 0.55 + loop_wobble * 0.45;
        let h = (heat * (1.0 + 0.22 * fl)).clamp(0.0, 1.0);
        let half = (0.10 + 0.22 * heat) * (1.0 + 0.14 * fl);
        let pos = project(e.xy, Vec2::ZERO, 0.8);
        out.push(pack(
            pos,
            0.50 + 0.45 * h,
            (0.30 + 0.70 * h) * e.flame_amp,
            e.class,
            half,
        ));
    }

    // Embers: staggered per-slot cycles (size fixed per slot, launch randomised per cycle).
    if e.ember_amp > 0.01 && e.ember_rate > 0.0 {
        for s in 0..e.ember_slots as u32 {
            let size01 = 0.1 + 0.9 * hash01(seed, 100 + s).powi(2);
            let life = ember_lifetime_s(size01);
            let cycle = life * (1.3 + 0.6 * hash01(seed, 200 + s));
            let ph = t / cycle + hash01(seed, 300 + s);
            let k = ph.floor();
            let age = (ph - k) * cycle;
            if age >= life {
                continue;
            }
            let kk = (k as i64 as u32).wrapping_mul(2_654_435_761) ^ s;
            let r = |n: u32| hash01(seed ^ kk, n);
            let launch = EmberLaunch::from_randoms(heat, size01, r(1), r(2), r(3));
            let base = if e.wind_mps.length_squared() > 1e-8 {
                e.wind_mps
            } else {
                anim.wind_mps
            };
            let wind = wind_vector_from_field(base, r(4) * 2.0 - 1.0);
            let (p, v) = ember_kinematics(&launch, wind, age);
            if p.z <= 0.0 {
                continue;
            }
            // Near the ground on the way down the glow is quenched, not cut.
            let ground = if v.z < 0.0 { (p.z / 0.3).clamp(0.0, 1.0) } else { 1.0 };
            let u = glow_u(ember_temperature_k(heat, size01, age));
            let start = Vec2::new(r(5) - 0.5, r(6) - 0.5) * (0.6 * FIRE_TILE_METERS);
            let pos = project(e.xy, start + p.truncate(), p.z);
            let radius_m = (0.10 + 0.25 * size01) * (0.6 + 0.4 * u.sqrt());
            let alpha = ember_alpha(age / life) * ground * e.ember_amp * e.ember_rate.min(1.0).max(0.3);
            out.push(pack(
                pos,
                u,
                alpha,
                ParticleClass::Ember,
                radius_m / FIRE_TILE_METERS,
            ));
        }
    }

    // Smoke plume: 0-2 puffs per cell, long staggered cycles, wind-sheared drift.
    let smoke_n = ((e.smoke * e.smoke_amp * 2.4).ceil() as u32).min(2);
    let mut smoke_anim = *anim;
    if e.wind_mps.length_squared() > 1e-8 {
        smoke_anim.wind_mps = e.wind_mps;
    }
    for s in 0..smoke_n {
        out.extend(smoke_puff(
            e.xy,
            1.0 + 4.0 * heat,
            e.smoke * e.smoke_amp,
            &smoke_anim,
            seed ^ 0x5A5A,
            s,
            1.0,
        ));
    }
}

/// One smoke puff row (or none while in the dead part of its cycle). Rises with decaying
/// buoyancy, drifts with the wind, grows, and fades in/out smoothly.
pub fn smoke_puff(
    origin: Vec2,
    start_height_m: f32,
    density: f32,
    anim: &FireAnim,
    seed: u32,
    slot: u32,
    size_scale: f32,
) -> Option<GpuParticleInstance> {
    let cyc = SMOKE_CYCLE_S.0 + (SMOKE_CYCLE_S.1 - SMOKE_CYCLE_S.0) * hash01(seed, 400 + slot);
    let ph = anim.time_s / cyc + hash01(seed, 500 + slot);
    let f = ph - ph.floor();
    let age = f * cyc;
    let sm = |x: f32| x * x * (3.0 - 2.0 * x);
    let env = sm((f / 0.12).clamp(0.0, 1.0)) * (1.0 - f).powf(1.6);
    if env < 0.004 || density <= 0.0 {
        return None;
    }
    // Rise speed saturates as the plume cools (tau 3 s); drift approaches 70 % of the wind.
    let rise_m = SMOKE_RISE_MPS * 1.6 * (age - 3.0 * (1.0 - (-age / 3.0).exp()));
    let wind = wind_vector_from_field(anim.wind_mps, 0.0);
    let drift_scale = 0.7 * (1.0 - (-age / 2.5).exp());
    let lateral = (hash01(seed, 600 + slot) - 0.5) * 0.4 * FIRE_TILE_METERS * f;
    let perp = if wind.length_squared() > 1e-8 {
        Vec2::new(-wind.y, wind.x).normalize_or_zero()
    } else {
        Vec2::Y
    };
    let off = wind * age * drift_scale + perp * lateral;
    let pos = project(origin, off, start_height_m + rise_m);
    let radius_m = (0.6 + 2.4 * (age / cyc)) * size_scale;
    Some(pack(
        pos,
        density.clamp(0.0, 1.0),
        (0.5 + 0.5 * density.clamp(0.0, 1.0)) * env,
        ParticleClass::Smoke,
        radius_m / FIRE_TILE_METERS,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emitter(heat: f32, ember_rate: f32, smoke: f32) -> FireEmitter {
        FireEmitter {
            xy: Vec2::new(10.5, 7.5),
            heat,
            ember_rate,
            smoke,
            flame_amp: 1.0,
            ember_amp: 1.0,
            smoke_amp: 1.0,
            ember_slots: 8,
            class: ParticleClass::Spark,
            wind_mps: Vec2::new(8.0, 0.0),
        }
    }

    #[test]
    fn no_fire_emits_no_particles() {
        let mut out = Vec::new();
        animate_emitter(
            &emitter(0.0, 0.0, 0.0),
            &FireAnim {
                time_s: 3.0,
                wind_mps: Vec2::new(8.0, 0.0),
            },
            &mut out,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn flame_loop_phase_is_global_and_cell_offset() {
        let xy = Vec2::new(10.0, 7.0);
        let t = 3.7;
        let a = flame_loop_phase(t, xy, FLAME_LOOP_PERIOD_S);
        let b = flame_loop_phase(t, xy, FLAME_LOOP_PERIOD_S);
        assert_eq!(a, b);
        assert!((0.0..1.0).contains(&a));
        let wrapped = flame_loop_phase(t + FLAME_LOOP_PERIOD_S, xy, FLAME_LOOP_PERIOD_S);
        assert!((wrapped - a).abs() < 1e-5, "period wraps with fixed cell offset");
        let other = flame_loop_phase(t, Vec2::new(12.0, 8.0), FLAME_LOOP_PERIOD_S);
        assert_ne!(a, other);
    }

    #[test]
    fn flame_size_unchanged_when_only_flame_amp_crossfades() {
        let e = emitter(0.9, 0.0, 0.0);
        let t = 1.25;
        let anim = FireAnim {
            time_s: t,
            wind_mps: Vec2::ZERO,
        };
        let mut full = Vec::new();
        let mut low = Vec::new();
        let mut e_full = e;
        e_full.flame_amp = 1.0;
        let mut e_low = e;
        e_low.flame_amp = 0.2;
        animate_emitter(&e_full, &anim, &mut full);
        animate_emitter(&e_low, &anim, &mut low);
        assert_eq!(full.len(), 1);
        assert_eq!(low.len(), 1);
        assert!(
            (full[0].ember_class_radius_smoke.z - low[0].ember_class_radius_smoke.z).abs() < 1e-5,
            "loop phase drives size; LOD only scales alpha"
        );
    }

    #[test]
    fn hot_cell_embers_stay_staggered_and_finite() {
        let mut counts = Vec::new();
        for i in 0..40 {
            let mut out = Vec::new();
            animate_emitter(
                &emitter(0.9, 1.0, 0.8),
                &FireAnim {
                    time_s: i as f32 * 0.17,
                    wind_mps: Vec2::new(6.0, 0.0),
                },
                &mut out,
            );
            assert!(out.iter().all(|r| r.world_xyz_heat.is_finite() && r.ember_class_radius_smoke.is_finite()));
            counts.push(out.len());
        }
        assert!(counts.iter().all(|&n| n >= 2), "flame core always present");
        assert!(counts.iter().min() != counts.iter().max(), "no synchronized pulsing");
    }
}
