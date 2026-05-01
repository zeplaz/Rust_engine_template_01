//! Pluggable height noise (fractal family + basis). Moisture/temperature use classic fBm·Perlin.
//! See `WorldGenParams` / editor UI for presets and post-shaping controls.

use noise::{Billow, Fbm, HybridMulti, NoiseFn, OpenSimplex, Perlin, RidgedMulti};
use serde::{Deserialize, Serialize};

/// fBm·Perlin for moisture, temperature, domain warp, and detail overlays.
pub fn build_fbm_perlin(
    scale: f32,
    octaves: u32,
    seed: u64,
    lacunarity: f64,
    persistence: f64,
) -> Fbm<Perlin> {
    let mut fbm = Fbm::<Perlin>::new(seed as u32);
    fbm.octaves = octaves as usize;
    fbm.frequency = scale as f64;
    fbm.lacunarity = lacunarity;
    fbm.persistence = persistence;
    fbm
}

/// Channel frequencies / seeds / warp kernels — **no magic numbers** in generators; tune here, sliders, or JSON.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct NoiseSamplingTuning {
    /// Scale multiplier for the domain-warp fBm (relative to world `noise_scale`).
    pub warp_noise_scale_mul: f32,
    pub warp_noise_octaves: u32,
    pub warp_seed_offset: u64,
    pub detail_noise_scale_mul: f32,
    pub detail_noise_octaves: u32,
    pub detail_seed_offset: u64,
    /// Multiplies world persistence for the detail channel.
    pub detail_persistence_mul: f64,
    pub moisture_noise_scale_mul: f32,
    pub temperature_noise_scale_mul: f32,
    /// Extra frequency multiplier inside `moisture_noise.get([nx * mul, ...])`.
    pub moisture_sample_freq_mul: f64,
    pub temperature_sample_freq_mul: f64,
    pub warp_coord_frequency_mul: f64,
    pub warp_coord_z: f64,
    pub warp_phase_offset_x: f64,
    pub warp_phase_offset_y: f64,
    /// Scales `domain_warp_strength` into world-space displacement (with the noise value).
    pub warp_displacement_scale: f64,
    pub detail_coord_frequency_mul: f64,
}

impl Default for NoiseSamplingTuning {
    fn default() -> Self {
        Self {
            warp_noise_scale_mul: 0.25,
            warp_noise_octaves: 5,
            warp_seed_offset: 911,
            detail_noise_scale_mul: 4.0,
            detail_noise_octaves: 4,
            detail_seed_offset: 1603,
            detail_persistence_mul: 0.85,
            moisture_noise_scale_mul: 1.5,
            temperature_noise_scale_mul: 0.8,
            moisture_sample_freq_mul: 1.5,
            temperature_sample_freq_mul: 0.8,
            warp_coord_frequency_mul: 0.07,
            warp_coord_z: 2.0,
            warp_phase_offset_x: 41.7,
            warp_phase_offset_y: 19.3,
            warp_displacement_scale: 12.0,
            detail_coord_frequency_mul: 4.2,
        }
    }
}

/// Base fractal used for **height** sampling (cliffs, continents, ranges, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum TerrainNoiseProfile {
    /// fBm over Perlin — smooth, homogeneous landforms (good baseline).
    #[default]
    FbmPerlin,
    /// Ridged multifractal — ridges and mountain chains.
    RidgedMulti,
    /// Billow — rounded, cloud-like landmasses.
    Billow,
    /// Hybrid multifractal — valleys stay smoother; mixed global character.
    HybridMulti,
    /// fBm over OpenSimplex — less axis-aligned than Perlin alone.
    FbmOpenSimplex,
}

#[derive(Clone)]
pub enum HeightNoise {
    FbmPerlin(Fbm<Perlin>),
    Ridged(RidgedMulti<Perlin>),
    Billow(Billow<Perlin>),
    Hybrid(HybridMulti<Perlin>),
    FbmOpenSimplex(Fbm<OpenSimplex>),
}

impl HeightNoise {
    pub fn get(&self, x: f64, y: f64, z: f64) -> f64 {
        let p = [x, y, z];
        match self {
            HeightNoise::FbmPerlin(n) => n.get(p),
            HeightNoise::Ridged(n) => n.get(p),
            HeightNoise::Billow(n) => n.get(p),
            HeightNoise::Hybrid(n) => n.get(p),
            HeightNoise::FbmOpenSimplex(n) => n.get(p),
        }
    }
}

pub fn build_height_noise(
    profile: TerrainNoiseProfile,
    scale: f32,
    octaves: u32,
    seed: u64,
    lacunarity: f64,
    persistence: f64,
) -> HeightNoise {
    let oct = octaves as usize;
    let freq = scale as f64;
    let seed_u = seed as u32;
    match profile {
        TerrainNoiseProfile::FbmPerlin => {
            let mut n = Fbm::<Perlin>::new(seed_u);
            n.octaves = oct;
            n.frequency = freq;
            n.lacunarity = lacunarity;
            n.persistence = persistence;
            HeightNoise::FbmPerlin(n)
        }
        TerrainNoiseProfile::RidgedMulti => {
            let mut n = RidgedMulti::<Perlin>::new(seed_u);
            n.octaves = oct;
            n.frequency = freq;
            n.lacunarity = lacunarity;
            n.persistence = persistence;
            HeightNoise::Ridged(n)
        }
        TerrainNoiseProfile::Billow => {
            let mut n = Billow::<Perlin>::new(seed_u);
            n.octaves = oct;
            n.frequency = freq;
            n.lacunarity = lacunarity;
            n.persistence = persistence;
            HeightNoise::Billow(n)
        }
        TerrainNoiseProfile::HybridMulti => {
            let mut n = HybridMulti::<Perlin>::new(seed_u);
            n.octaves = oct;
            n.frequency = freq;
            n.lacunarity = lacunarity;
            n.persistence = persistence;
            HeightNoise::Hybrid(n)
        }
        TerrainNoiseProfile::FbmOpenSimplex => {
            let mut n = Fbm::<OpenSimplex>::new(seed_u);
            n.octaves = oct;
            n.frequency = freq;
            n.lacunarity = lacunarity;
            n.persistence = persistence;
            HeightNoise::FbmOpenSimplex(n)
        }
    }
}

/// Normalized height in \[0, 1\]: domain warp, response curve, optional detail mix (post base fractal).
pub fn sample_height_field(
    height_noise: &HeightNoise,
    warp_noise: &Fbm<Perlin>,
    detail_noise: &Fbm<Perlin>,
    noise_x: f64,
    noise_y: f64,
    height_curve_exponent: f32,
    domain_warp_strength: f32,
    terrain_detail_mix: f32,
    tuning: &NoiseSamplingTuning,
) -> f32 {
    let (mut nx, mut ny) = (noise_x, noise_y);
    if domain_warp_strength > 0.0 {
        let f = tuning.warp_coord_frequency_mul;
        let w1 = warp_noise.get([
            noise_x * f,
            noise_y * f,
            tuning.warp_coord_z,
        ]);
        let w2 = warp_noise.get([
            noise_x * f + tuning.warp_phase_offset_x,
            noise_y * f + tuning.warp_phase_offset_y,
            tuning.warp_coord_z,
        ]);
        let s = domain_warp_strength as f64 * tuning.warp_displacement_scale;
        nx += w1 * s;
        ny += w2 * s;
    }

    let raw = height_noise.get(nx, ny, 0.0);
    let mut h = (raw * 0.5 + 0.5) as f32;

    if height_curve_exponent != 1.0 {
        h = h.clamp(0.0, 1.0).powf(height_curve_exponent);
    }

    if terrain_detail_mix > 0.0 {
        let df = tuning.detail_coord_frequency_mul;
        let d = (detail_noise.get([noise_x * df, noise_y * df, 0.0]) * 0.5 + 0.5) as f32;
        h = (h * (1.0 - terrain_detail_mix) + d * terrain_detail_mix).clamp(0.0, 1.0);
    }

    h.clamp(0.0, 1.0)
}
