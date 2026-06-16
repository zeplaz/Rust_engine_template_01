//! Ecology-driven **world preview** tinting and future GPU compositing hooks.
//!
//! Base color is computed from [`ChunkEcology`](crate::systems::ecology::ChunkEcology) + meso
//! [`VegetationField`](crate::systems::ecology::VegetationField) + [`ChunkWeather`](crate::systems::weather::ChunkWeather),
//! optionally modulated by [`FireFuelField`](crate::systems::fire::FireFuelField), [`ChunkSmokeField`](crate::systems::fire::ChunkSmokeField),
//! and surface heat — not terrain name strings (`base_fire_sim.md` §7).

use crate::systems::ecology::{ChunkEcology, VegetationField};
use crate::systems::fire::{ChunkSmokeField, FireFuelField};
use crate::systems::weather::ChunkWeather;
use bevy::math::{IVec2, UVec2};

/// Chunk-anchored ecology row for world-preview raster lookup (uniform per chunk until subcell veg lands).
pub type EcologyRasterChunkRow = (
    IVec2,
    UVec2,
    Option<ChunkEcology>,
    Option<VegetationField>,
    Option<ChunkWeather>,
    Option<FireFuelField>,
    f32,
    Option<ChunkSmokeField>,
    u8,
);

/// Scalar bundle for one raster sample (typically one chunk’s fields projected onto all its tiles).
#[derive(Clone, Copy, Debug)]
pub struct EcologyPreviewSample {
    pub eco: ChunkEcology,
    pub veg: VegetationField,
    pub wx: ChunkWeather,
    pub fuel: FireFuelField,
    pub surface_heat: f32,
    pub smoke: ChunkSmokeField,
    /// Precomputed topology heterogeneity bias (VEG-PREVIEW-TOPOLOGY-001).
    pub topology_tint_bias: f32,
    pub topology_kind_count: u8,
}

impl EcologyPreviewSample {
    pub fn from_chunk_components(
        eco: Option<&ChunkEcology>,
        veg: Option<&VegetationField>,
        wx: Option<&ChunkWeather>,
        fuel: Option<&FireFuelField>,
        surface_heat: f32,
        smoke: Option<&ChunkSmokeField>,
    ) -> Self {
        Self {
            eco: eco.copied().unwrap_or_default(),
            veg: veg.copied().unwrap_or_default(),
            wx: wx.copied().unwrap_or_default(),
            fuel: fuel.copied().unwrap_or_default(),
            surface_heat,
            smoke: smoke.copied().unwrap_or_default(),
            topology_tint_bias: 0.0,
            topology_kind_count: 0,
        }
    }

    pub fn with_topology_kind_count(mut self, count: u8) -> Self {
        if count == 0 {
            return self;
        }
        let kinds: Vec<String> = (0..count).map(|i| format!("Topo{i}")).collect();
        self.with_topology_kinds(&kinds)
    }

    pub fn with_topology_kinds(mut self, kinds: &[String]) -> Self {
        self.topology_kind_count = kinds.len().min(255) as u8;
        let [r, g, b] = topology_kind_tint_modulator(kinds);
        self.topology_tint_bias = (r + g + b) / 255.0;
        self
    }
}

/// Blend active flame + smoke into a base RGBA (`base_fire_sim.md` §7).
pub fn blend_fire_overlay(base: [u8; 4], heat: f32, smoke: f32) -> [u8; 4] {
    let heat = heat.clamp(0.0, 1.0);
    let smoke = smoke.clamp(0.0, 1.0);
    let mut r = base[0] as f32;
    let mut g = base[1] as f32;
    let mut b = base[2] as f32;
    let a = base[3];

    r += heat * 180.0;
    g *= 1.0 - smoke * 0.7;
    b *= 1.0 - smoke * 0.85;

    [
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
        a,
    ]
}

/// Non-zero when topology kinds produce preview tint bias (VEG-PREVIEW-TOPOLOGY-001 runtime proof).
#[must_use]
pub fn topology_tint_bias_for_kinds(topology_kinds: &[String]) -> f32 {
    if topology_kinds.is_empty() {
        return 0.0;
    }
    let [r, g, b] = topology_kind_tint_modulator(topology_kinds);
    (r + g + b) / 255.0
}

/// Count chunks whose program topology kinds would tint the ecology preview raster.
#[must_use]
pub fn count_chunks_with_topology_tint_bias<'a>(
    program_topology_kinds: impl Iterator<Item = &'a [String]>,
) -> u32 {
    program_topology_kinds
        .filter(|kinds| topology_tint_bias_for_kinds(kinds) > 0.0)
        .count() as u32
}

/// Pixel-level heterogeneity: distinct RGB triples from ecology preview raster samples (VEG-HARD-PREVIEW-PIXEL-001).
#[must_use]
pub fn count_distinct_topology_visible_rgba(samples: &[EcologyPreviewSample]) -> u32 {
    use std::collections::HashSet;
    let mut distinct = HashSet::new();
    for s in samples {
        if s.topology_tint_bias <= 0.0 {
            continue;
        }
        let rgba = ecology_preview_rgba(s);
        distinct.insert((rgba[0], rgba[1], rgba[2]));
    }
    distinct.len() as u32
}

/// Build preview samples from program topology kind slices (sim harness raster proxy).
#[must_use]
pub fn preview_samples_from_topology_kinds(
    kind_slices: impl IntoIterator<Item = Vec<String>>,
) -> Vec<EcologyPreviewSample> {
    kind_slices
        .into_iter()
        .filter(|kinds| !kinds.is_empty())
        .map(|kinds| {
            EcologyPreviewSample::from_chunk_components(None, None, None, None, 0.0, None)
                .with_topology_kinds(&kinds)
        })
        .collect()
}

/// Stable ecology-only tint (macro + meso + weather + fuel + topology heterogeneity). No terrain family labels.
pub fn ecology_preview_rgba(s: &EcologyPreviewSample) -> [u8; 4] {
    let base = vegetation_preview_rgba(
        &s.eco,
        &s.veg,
        &s.wx,
        Some(&s.fuel),
        s.surface_heat,
        s.topology_tint_bias,
    );
    blend_fire_overlay(base, s.surface_heat, s.smoke.density)
}

/// Topology-kind RGB bias for operator-visible patch heterogeneity (VEG-PREVIEW-TOPOLOGY-001).
#[must_use]
pub fn topology_kind_tint_modulator(topology_kinds: &[String]) -> [f32; 3] {
    let mut r = 0.0f32;
    let mut g = 0.0f32;
    let mut b = 0.0f32;
    for k in topology_kinds {
        match k.as_str() {
            "Corridor" => {
                g += 12.0;
                b += 8.0;
            }
            "Ring" => {
                r += 6.0;
                g += 4.0;
            }
            "Patch" => g += 18.0,
            "Cluster" => {
                g += 22.0;
                b += 5.0;
            }
            "Fringe" => {
                r += 10.0;
                g += 5.0;
            }
            "Network" => b += 6.0,
            _ => {}
        }
    }
    [r, g, b]
}

/// Lower-level entry: macro ecology + vegetation fields + weather; optional fuel/heat for charring / crown stress.
pub fn vegetation_preview_rgba(
    eco: &ChunkEcology,
    veg: &VegetationField,
    wx: &ChunkWeather,
    fuel: Option<&FireFuelField>,
    surface_heat: f32,
    topology_tint_bias: f32,
) -> [u8; 4] {
    let biomass = eco.biomass.clamp(0.0, 1.0);
    let dryness = veg.dryness.clamp(0.0, 1.0);
    let burn = veg.burn_severity.clamp(0.0, 1.0);
    let reg = veg.regrowth_stage.clamp(0.0, 1.0);
    let crown = veg.canopy_density.clamp(0.0, 1.0);

    let heat = surface_heat.clamp(0.0, 1.0);
    let ladder = fuel.map(|f| f.ladder_fuel).unwrap_or(0.0).clamp(0.0, 1.0);
    let fuel_surface = fuel.map(|f| f.surface_fuel).unwrap_or(veg.ground_fuel).clamp(0.0, 1.0);

    let moisture = wx.soil_moisture.clamp(0.0, 1.0);
    let snow = wx.snow_depth.clamp(0.0, 1.0);
    let fog = wx.fog_density.clamp(0.0, 1.0);

    let mut r: f32 = 40.0 + dryness * 90.0 + burn * 80.0 + heat * 55.0 + ladder * 25.0;
    let mut g: f32 =
        80.0 + biomass * 140.0 - burn * 120.0 + reg * 55.0 + fuel_surface * 20.0 - dryness * 40.0;
    let mut b: f32 = 30.0 + moisture * 40.0 + crown * 35.0 - burn * 40.0 + fog * 25.0;

    g += reg * (1.0 - burn) * 35.0;
    r -= reg * 15.0;

    let tint = topology_tint_bias.clamp(0.0, 1.0) * 48.0;
    g += tint;
    b += tint * 0.35;

    let snow_m = snow * 70.0;
    r = r * (1.0 - snow * 0.35) + snow_m * 0.25;
    g = g * (1.0 - snow * 0.28) + snow_m * 0.28;
    b = b * (1.0 - snow * 0.22) + snow_m * 0.42;

    [
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
        255,
    ]
}

/// Resolve chunk-level ecology data for a world tile (uniform across chunk until subcell fields land).
pub fn ecology_sample_for_world_tile(tx: u32, ty: u32, entries: &[EcologyRasterChunkRow]) -> EcologyPreviewSample {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size, eco, veg, wx, fuel, heat, smoke, topo_count) in entries {
        let sx = size.x as i32;
        let sy = size.y as i32;
        let wx0 = coord.x * sx;
        let wy0 = coord.y * sy;
        if tx_i < wx0 || ty_i < wy0 {
            continue;
        }
        let lx = tx_i - wx0;
        let ly = ty_i - wy0;
        if lx < 0 || ly < 0 || lx >= sx || ly >= sy {
            continue;
        }
        return EcologyPreviewSample::from_chunk_components(
            eco.as_ref(),
            veg.as_ref(),
            wx.as_ref(),
            fuel.as_ref(),
            *heat,
            smoke.as_ref(),
        )
        .with_topology_kind_count(*topo_count);
    }
    EcologyPreviewSample::from_chunk_components(None, None, None, None, 0.0, None)
}

// --- GPU roadmap (pass ids / buffer layout hooks; no wgpu here) ---

/// Tag for a future compute pass scheduling vegetation growth / fire spread on field atlases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EcologyGpuPassKind(pub u8);

impl EcologyGpuPassKind {
    pub const VEG_GROWTH: EcologyGpuPassKind = EcologyGpuPassKind(1);
    pub const FIRE_SPREAD: EcologyGpuPassKind = EcologyGpuPassKind(2);
    pub const PREVIEW_COMPOSITE: EcologyGpuPassKind = EcologyGpuPassKind(3);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drought_browns_preview() {
        let eco = ChunkEcology {
            biomass: 0.55,
            ..Default::default()
        };
        let veg_dry = VegetationField {
            dryness: 0.92,
            burn_severity: 0.05,
            canopy_density: 0.5,
            regrowth_stage: 0.5,
            ground_fuel: 0.4,
            ..Default::default()
        };
        let veg_wet = VegetationField {
            dryness: 0.08,
            ..veg_dry
        };
        let wx = ChunkWeather::default();
        let c_dry = vegetation_preview_rgba(&eco, &veg_dry, &wx, None, 0.0, 0.0);
        let c_wet = vegetation_preview_rgba(&eco, &veg_wet, &wx, None, 0.0, 0.0);
        assert!(c_dry[0] > c_wet[0] || c_dry[1] < c_wet[1], "dry {:?} wet {:?}", c_dry, c_wet);
    }

    #[test]
    fn burn_darkens_green() {
        let eco = ChunkEcology {
            biomass: 0.7,
            ..Default::default()
        };
        let calm = VegetationField {
            burn_severity: 0.05,
            canopy_density: 0.65,
            regrowth_stage: 0.5,
            ..Default::default()
        };
        let burnt = VegetationField {
            burn_severity: 0.85,
            canopy_density: 0.15,
            regrowth_stage: 0.1,
            ..Default::default()
        };
        let wx = ChunkWeather::default();
        let c0 = vegetation_preview_rgba(&eco, &calm, &wx, None, 0.0, 0.0);
        let c1 = vegetation_preview_rgba(&eco, &burnt, &wx, None, 0.0, 0.0);
        assert!(c1[1] < c0[1], "g {} {}", c1[1], c0[1]);
    }

    #[test]
    fn smoke_dampens_green_channel() {
        let base = [60u8, 140u8, 50u8, 255];
        let out = blend_fire_overlay(base, 0.0, 0.85);
        assert!(out[1] < base[1]);
    }

    #[test]
    fn topology_kinds_shift_green_channel() {
        let eco = ChunkEcology::default();
        let veg = VegetationField::default();
        let wx = ChunkWeather::default();
        let plain = vegetation_preview_rgba(&eco, &veg, &wx, None, 0.0, 0.0);
        let [r, g, b] = topology_kind_tint_modulator(&[
            "Patch".into(),
            "Cluster".into(),
            "Corridor".into(),
        ]);
        let rich = vegetation_preview_rgba(&eco, &veg, &wx, None, 0.0, (r + g + b) / 255.0);
        assert!(rich[1] > plain[1], "plain {:?} rich {:?}", plain, rich);
    }
}
