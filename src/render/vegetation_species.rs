//! VEG-SHAPE-001 — parametric vegetation species and crown shapes, shared by both presentation modes.
//!
//! ```text
//! ChunkCellMatrix (biome weights, moisture, temperature)      sim / terrain truth, read only
//!   └─ VegSpeciesTable::species_cell_codes ─▶ VegExtractRow::species   (extract, cached per chunk)
//! VegExtractRow (cells + species) ─▶ sample_row_instances ─▶ VegInstance { pos, cover, species, rand }
//!   └─ VegSpeciesTable::resolve_shape(species, maturity, rand) ─▶ TreeShape  (pure, Copy, no alloc)
//!        ├─ TacticalFlat2D  `world_map_vegetation_mesh`  : crown polygon by `CrownProfile`
//!        └─ PixelWorld3D    `pixel_world::layers`        : stacked voxel tiers
//! ```
//!
//! This module owns **no sim state** and writes nothing: species is a pure function of cell data and the
//! authoring table (`assets/configs/ecology/veg_species_v1.ron`), so it needs no second writer and stays
//! deterministic. It is not part of `VoxelMassing` (`massing_columns`); flora never enters that surface.
//! New species and shapes are RON rows, not code.

use std::path::PathBuf;
use std::sync::Arc;

use bevy::prelude::*;
use serde::Deserialize;

use crate::terrain::biome::BiomeWeights;
use crate::terrain::ecology::FloraType;
use crate::terrain::generation::ChunkCellMatrix;

pub const VEG_SPECIES_RON: &str = "assets/configs/ecology/veg_species_v1.ron";
const EMBEDDED_VEG_SPECIES: &str = include_str!("../../assets/configs/ecology/veg_species_v1.ron");

/// Species used when a row carries no species codes (no cell matrix yet). Always table index 0.
pub const SPECIES_DEFAULT: u8 = 0;
/// Most voxel tiers a crown can stack.
pub const MAX_TIERS: usize = 5;
/// Table size cap (species id is a `u8`).
pub const MAX_SPECIES: usize = 64;

/// Silhouette family. Both views read it: a 2D crown polygon in Tactical, tier width falloff in voxels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum CrownProfile {
    /// Domed broadleaf.
    Round,
    /// Pointed conifer, widest at the base.
    Cone,
    /// Narrow and tall.
    Column,
    /// Wide, flat crown on a long trunk.
    Spread,
}

/// Growth stage a shape is resolved at (from the sim's per-cell stand stage).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Maturity {
    Shrub,
    Young,
    Old,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct MaturityScale {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct MaturityScales {
    pub shrub: MaturityScale,
    pub young: MaturityScale,
    pub old: MaturityScale,
}

/// One species row: where it grows (biome / moisture / temperature affinity) and its shape parameters.
#[derive(Clone, Debug, Deserialize)]
pub struct SpeciesDef {
    pub id: String,
    /// Existing terrain taxonomy this species belongs to.
    pub flora: FloraType,
    pub profile: CrownProfile,
    pub biome: BiomeWeights,
    /// Preferred moisture / temperature (`0..=1`) and how far a cell may stray before the species drops out.
    pub moisture: f32,
    pub temperature: f32,
    pub breadth: f32,
    /// Mature height and crown width ranges (tile units); the per-site rand picks inside them.
    pub height: (f32, f32),
    pub crown_width: (f32, f32),
    /// Share of the height that is bare trunk.
    pub trunk_frac: f32,
    pub trunk_width: f32,
    /// How much each voxel tier narrows going up (`0` = straight, `1` = comes to a point).
    pub taper: f32,
    pub tiers: u8,
    pub crown_old: [f32; 3],
    pub crown_young: [f32; 3],
    pub crown_tip: [f32; 3],
    pub crown_shrub: [f32; 3],
}

/// One voxel tier, relative to the crown base (`lo`/`hi` in tile units above it).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tier {
    pub width: f32,
    pub lo: f32,
    pub hi: f32,
}

/// A resolved tree: `Copy`, fixed size, no allocation. Plain data, so a per-chunk list of
/// `(position, species, rand)` can later be uploaded as an instance buffer without reshaping it.
#[derive(Clone, Copy, Debug)]
pub struct TreeShape {
    pub profile: CrownProfile,
    pub maturity: Maturity,
    pub height: f32,
    pub crown_width: f32,
    /// Bare trunk share of `height` (0 for shrubs).
    pub trunk_frac: f32,
    pub trunk_width: f32,
    pub tier_count: u8,
    pub tiers: [Tier; MAX_TIERS],
    pub crown: [f32; 3],
    pub tip: [f32; 3],
}

impl TreeShape {
    #[inline]
    #[must_use]
    pub fn tiers(&self) -> &[Tier] {
        &self.tiers[..self.tier_count as usize]
    }

    /// Height of the crown above the trunk.
    #[inline]
    #[must_use]
    pub fn crown_height(&self) -> f32 {
        self.height * (1.0 - self.trunk_frac)
    }

    /// Colour of tier `i` (crown base to tip).
    #[must_use]
    pub fn tier_color(&self, i: usize) -> [f32; 3] {
        let n = self.tier_count.max(1) as usize;
        let t = if n <= 1 { 0.0 } else { i as f32 / (n - 1) as f32 };
        std::array::from_fn(|c| self.crown[c] + (self.tip[c] - self.crown[c]) * t)
    }
}

#[derive(Clone, Debug, Deserialize)]
struct VegSpeciesRon {
    seed: u32,
    patch_tiles: f32,
    maturity: MaturityScales,
    species: Vec<SpeciesDef>,
}

/// The authoring table. Inserted once by the extract plugin; every consumer reads it.
#[derive(Resource, Clone, Debug)]
pub struct VegSpeciesTable {
    pub seed: u32,
    pub patch_tiles: f32,
    pub maturity: MaturityScales,
    pub species: Vec<SpeciesDef>,
}

fn mix32(mut x: u32) -> u32 {
    x = x.wrapping_mul(0x9E37_79B9);
    x ^= x >> 16;
    x = x.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 13;
    x = x.wrapping_mul(0xC2B2_AE35);
    x ^ (x >> 16)
}

fn lattice(ix: i32, iy: i32, salt: u32) -> f32 {
    let h = mix32((ix as u32).wrapping_mul(374_761_393) ^ (iy as u32).wrapping_mul(668_265_263) ^ salt);
    (h >> 8) as f32 / (1u32 << 24) as f32
}

/// Bilinear value noise in `[0, 1)`.
fn value_noise(x: f32, y: f32, salt: u32) -> f32 {
    let (x0, y0) = (x.floor(), y.floor());
    let (fx, fy) = (x - x0, y - y0);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (ix, iy) = (x0 as i32, y0 as i32);
    let a = lattice(ix, iy, salt) + (lattice(ix + 1, iy, salt) - lattice(ix, iy, salt)) * sx;
    let b = lattice(ix, iy + 1, salt) + (lattice(ix + 1, iy + 1, salt) - lattice(ix, iy + 1, salt)) * sx;
    a + (b - a) * sy
}

#[inline]
fn lerp(r: (f32, f32), t: f32) -> f32 {
    r.0 + (r.1 - r.0) * t.clamp(0.0, 1.0)
}

impl Default for VegSpeciesTable {
    fn default() -> Self {
        Self::from_ron(EMBEDDED_VEG_SPECIES).expect("embedded veg_species_v1.ron must parse and validate")
    }
}

impl VegSpeciesTable {
    /// Parse and validate an authoring table.
    pub fn from_ron(text: &str) -> Result<Self, String> {
        let ron: VegSpeciesRon = ron::from_str(text).map_err(|e| format!("veg species RON parse: {e}"))?;
        let table = Self {
            seed: ron.seed,
            patch_tiles: ron.patch_tiles,
            maturity: ron.maturity,
            species: ron.species,
        };
        table.validate()?;
        Ok(table)
    }

    /// Disk table when it parses and validates (dev edit path), else the compiled-in default. A bad disk
    /// table is reported, not silently used.
    #[must_use]
    pub fn load() -> Self {
        let path = std::env::var_os("CARGO_MANIFEST_DIR")
            .map(PathBuf::from)
            .map(|root| root.join(VEG_SPECIES_RON))
            .unwrap_or_else(|| PathBuf::from(VEG_SPECIES_RON));
        match std::fs::read_to_string(&path) {
            Ok(text) => Self::from_ron(&text).unwrap_or_else(|e| {
                warn!("{}: {e}; using the compiled-in species table", path.display());
                Self::default()
            }),
            Err(_) => Self::default(),
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.species.is_empty() || self.species.len() > MAX_SPECIES {
            return Err(format!("species count {} not in 1..={MAX_SPECIES}", self.species.len()));
        }
        if !(self.patch_tiles >= 1.0) {
            return Err("patch_tiles must be >= 1".into());
        }
        for (i, s) in self.species.iter().enumerate() {
            let bad = |what: &str| Err(format!("species {i} `{}`: {what}", s.id));
            if self.species[..i].iter().any(|o| o.id == s.id) {
                return bad("duplicate id");
            }
            if !(s.height.0 > 0.0 && s.height.0 <= s.height.1) {
                return bad("height range");
            }
            if !(s.crown_width.0 > 0.0 && s.crown_width.0 <= s.crown_width.1) {
                return bad("crown_width range");
            }
            if !(0.0..1.0).contains(&s.trunk_frac) || !(0.0..=1.0).contains(&s.taper) {
                return bad("trunk_frac in 0..1 and taper in 0..=1");
            }
            if s.trunk_width <= 0.0 || s.breadth <= 0.0 {
                return bad("trunk_width and breadth must be > 0");
            }
            if s.tiers == 0 || s.tiers as usize > MAX_TIERS {
                return bad("tiers out of range");
            }
        }
        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn def(&self, species: u8) -> &SpeciesDef {
        self.species.get(species as usize).unwrap_or(&self.species[SPECIES_DEFAULT as usize])
    }

    #[inline]
    fn scale(&self, m: Maturity) -> MaturityScale {
        match m {
            Maturity::Shrub => self.maturity.shrub,
            Maturity::Young => self.maturity.young,
            Maturity::Old => self.maturity.old,
        }
    }

    /// Species for one cell. Affinity (biome weights x moisture / temperature fit) times a low-frequency
    /// per-species patch noise, so a biome yields mixed stands rather than salt-and-pepper. Pure and
    /// deterministic in `(cell, cell data, table)`.
    #[must_use]
    pub fn species_for_cell(&self, cell: IVec2, w: &BiomeWeights, moisture: f32, temperature: f32) -> u8 {
        let (px, py) = (cell.x as f32 / self.patch_tiles, cell.y as f32 / self.patch_tiles);
        let mut best = (SPECIES_DEFAULT, f32::NEG_INFINITY);
        for (i, s) in self.species.iter().enumerate() {
            let b = &s.biome;
            let biome = w.marine * b.marine
                + w.coastal * b.coastal
                + w.arid * b.arid
                + w.temperate * b.temperate
                + w.boreal * b.boreal
                + w.alpine * b.alpine
                + w.wetland * b.wetland;
            let stray = ((moisture - s.moisture).abs() + (temperature - s.temperature).abs()) * 0.5;
            let fit = (1.0 - stray / s.breadth).max(0.05);
            let salt = self.seed ^ (i as u32).wrapping_mul(0x9E37_79B1);
            // Wide enough that a runner-up species (affinity down to ~0.4x) wins whole patches.
            let patch = 0.1 + 2.0 * value_noise(px, py, salt);
            let score = (biome * fit + 1e-3) * patch;
            if score > best.1 {
                best = (i as u8, score);
            }
        }
        best.0
    }

    /// Species code per cell of one chunk, row-major like the matrix.
    #[must_use]
    pub fn species_cell_codes(&self, matrix: &ChunkCellMatrix, chunk_coord: IVec2) -> Arc<[u8]> {
        let origin = IVec2::new(chunk_coord.x * matrix.size.x as i32, chunk_coord.y * matrix.size.y as i32);
        let n = (matrix.size.x * matrix.size.y) as usize;
        (0..n)
            .map(|i| {
                let cell = origin + IVec2::new((i as u32 % matrix.size.x) as i32, (i as u32 / matrix.size.x) as i32);
                self.species_for_cell(cell, &matrix.weights[i], matrix.moisture[i], matrix.temperature[i])
            })
            .collect()
    }

    /// Resolve one tree. `rand` is the site's `[0, 1)` value; two independent jitters are derived from it.
    #[must_use]
    pub fn resolve_shape(&self, species: u8, maturity: Maturity, rand: f32) -> TreeShape {
        let d = self.def(species);
        let sc = self.scale(maturity);
        let r1 = rand.clamp(0.0, 0.999_999);
        let r2 = (r1 * 97.0).fract();
        let height = lerp(d.height, r1) * sc.height;
        let crown_width = lerp(d.crown_width, r2) * sc.width;
        let (profile, trunk_frac, tier_count, crown, tip) = match maturity {
            Maturity::Shrub => (CrownProfile::Round, 0.0, 1u8, d.crown_shrub, d.crown_shrub),
            Maturity::Young => (
                d.profile,
                d.trunk_frac * 0.85,
                d.tiers.saturating_sub(1).max(1),
                d.crown_young,
                d.crown_young.map(|c| (c * 1.12).min(1.0)),
            ),
            Maturity::Old => (d.profile, d.trunk_frac, d.tiers, d.crown_old, d.crown_tip),
        };
        let crown_h = height * (1.0 - trunk_frac);
        let n = tier_count as usize;
        let mut tiers = [Tier::default(); MAX_TIERS];
        for (i, t) in tiers.iter_mut().enumerate().take(n) {
            let u = i as f32 / n as f32;
            *t = Tier {
                width: crown_width * (1.0 - d.taper * u),
                lo: crown_h * u,
                hi: crown_h * (i + 1) as f32 / n as f32,
            };
        }
        TreeShape {
            profile,
            maturity,
            height,
            crown_width,
            trunk_frac,
            trunk_width: d.trunk_width * sc.width.max(0.5),
            tier_count,
            tiers,
            crown,
            tip,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn weights(f: impl FnOnce(&mut BiomeWeights)) -> BiomeWeights {
        let mut w = BiomeWeights {
            marine: 0.0,
            coastal: 0.0,
            arid: 0.0,
            temperate: 0.0,
            boreal: 0.0,
            alpine: 0.0,
            wetland: 0.0,
        };
        f(&mut w);
        w
    }

    /// Species histogram over a 64 x 64 world patch of one uniform cell kind.
    fn histogram(t: &VegSpeciesTable, w: &BiomeWeights, moisture: f32, temp: f32) -> Vec<u32> {
        let mut h = vec![0u32; t.species.len()];
        for y in 0..64 {
            for x in 0..64 {
                h[t.species_for_cell(IVec2::new(x, y), w, moisture, temp) as usize] += 1;
            }
        }
        h
    }

    fn idx(t: &VegSpeciesTable, id: &str) -> usize {
        t.species.iter().position(|s| s.id == id).unwrap()
    }

    #[test]
    fn embedded_table_parses_validates_and_has_every_profile() {
        let t = VegSpeciesTable::default();
        assert!(t.species.len() >= 6);
        for p in [CrownProfile::Round, CrownProfile::Cone, CrownProfile::Column, CrownProfile::Spread] {
            assert!(t.species.iter().any(|s| s.profile == p), "{p:?}");
        }
        assert!(t.species.iter().any(|s| s.flora == FloraType::ConiferousTrees));
        assert!(t.species.iter().any(|s| s.flora == FloraType::BroadleafTrees));
    }

    #[test]
    fn invalid_tables_are_rejected() {
        let ok = EMBEDDED_VEG_SPECIES;
        assert!(VegSpeciesTable::from_ron(&ok.replace("tiers: 3,", "tiers: 9,")).is_err());
        assert!(VegSpeciesTable::from_ron(&ok.replace("id: \"birch\"", "id: \"oak\"")).is_err());
        assert!(VegSpeciesTable::from_ron("(").is_err());
    }

    #[test]
    fn species_follow_biome_and_stands_are_mixed() {
        let t = VegSpeciesTable::default();
        let spruce = idx(&t, "spruce");
        let willow = idx(&t, "willow");
        let juniper = idx(&t, "juniper");
        let boreal = histogram(&t, &weights(|w| w.boreal = 1.0), 0.6, 0.3);
        let wet = histogram(&t, &weights(|w| w.wetland = 1.0), 0.85, 0.55);
        let arid = histogram(&t, &weights(|w| w.arid = 1.0), 0.2, 0.6);
        let top = |h: &[u32]| h.iter().enumerate().max_by_key(|(_, c)| **c).unwrap().0;
        assert_eq!(top(&boreal), spruce, "{boreal:?}");
        assert_eq!(top(&wet), willow, "{wet:?}");
        assert_eq!(top(&arid), juniper, "{arid:?}");
        // A temperate stand is not one species.
        let temperate = histogram(&t, &weights(|w| w.temperate = 1.0), 0.55, 0.55);
        assert!(temperate.iter().filter(|c| **c > 64).count() >= 2, "{temperate:?}");
    }

    #[test]
    fn species_cell_codes_are_deterministic_and_chunk_aligned() {
        let t = VegSpeciesTable::default();
        let mut m = ChunkCellMatrix::new(UVec2::splat(8));
        for w in &mut m.weights {
            *w = weights(|w| w.temperate = 1.0);
        }
        let a = t.species_cell_codes(&m, IVec2::new(2, -1));
        let b = t.species_cell_codes(&m, IVec2::new(2, -1));
        assert_eq!(a.len(), 64);
        assert_eq!(a, b);
        // Cell (3,5) of chunk (2,-1) is world cell (19,-3): chunk coordinates feed the patch noise.
        let w = m.weights[5 * 8 + 3];
        assert_eq!(
            a[5 * 8 + 3],
            t.species_for_cell(IVec2::new(19, -3), &w, m.moisture[5 * 8 + 3], m.temperature[5 * 8 + 3])
        );
    }

    #[test]
    fn shapes_differ_by_species_and_grow_with_maturity() {
        let t = VegSpeciesTable::default();
        let spruce = idx(&t, "spruce") as u8;
        let oak = idx(&t, "oak") as u8;
        let birch = idx(&t, "birch") as u8;
        let pine = idx(&t, "pine") as u8;
        let s_old = t.resolve_shape(spruce, Maturity::Old, 0.5);
        let o_old = t.resolve_shape(oak, Maturity::Old, 0.5);
        let b_old = t.resolve_shape(birch, Maturity::Old, 0.5);
        let p_old = t.resolve_shape(pine, Maturity::Old, 0.5);
        // Cone narrows tier by tier, a pine canopy is wide on a long trunk, a birch is slender.
        let w: Vec<f32> = s_old.tiers().iter().map(|x| x.width).collect();
        assert!(w.len() >= 3 && w.windows(2).all(|p| p[1] < p[0]), "{w:?}");
        assert!(p_old.crown_width > o_old.crown_width && p_old.trunk_frac > o_old.trunk_frac);
        assert!(b_old.crown_width < o_old.crown_width * 0.8);
        assert!(s_old.height > o_old.height);
        for sp in [spruce, oak, birch, pine] {
            let shrub = t.resolve_shape(sp, Maturity::Shrub, 0.5);
            let young = t.resolve_shape(sp, Maturity::Young, 0.5);
            let old = t.resolve_shape(sp, Maturity::Old, 0.5);
            assert!(shrub.height < young.height && young.height < old.height);
            assert!(shrub.crown_width < young.crown_width && young.crown_width < old.crown_width);
            assert_eq!(shrub.trunk_frac, 0.0);
            // Tiers tile the crown with no gap.
            let top = old.tiers().last().unwrap();
            assert!((top.hi - old.crown_height()).abs() < 1e-5);
        }
    }

    #[test]
    fn site_rand_varies_one_species_and_is_deterministic() {
        let t = VegSpeciesTable::default();
        let a = t.resolve_shape(0, Maturity::Old, 0.1);
        let b = t.resolve_shape(0, Maturity::Old, 0.9);
        let a2 = t.resolve_shape(0, Maturity::Old, 0.1);
        assert_ne!(a.height, b.height);
        assert_eq!(a.height, a2.height);
        assert_eq!(a.crown_width, a2.crown_width);
    }

    #[test]
    fn unknown_species_resolves_to_default_not_panic() {
        let t = VegSpeciesTable::default();
        let s = t.resolve_shape(250, Maturity::Old, 0.3);
        let d = t.resolve_shape(SPECIES_DEFAULT, Maturity::Old, 0.3);
        assert_eq!(s.height, d.height);
    }
}
