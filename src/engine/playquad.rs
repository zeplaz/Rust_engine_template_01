//! PLAYTEST-QUADRANT-WORLD-001 P0 — the one quadrant partition for `--test playquad` (pure data, no ECS writes).
//!
//! One Standard (512²) world split at the chunk-aligned center `(w/2, h/2)`. **+Y is north** (tile `y` grows
//! up on screen — `tile_world_fallback` flips the raster to match, `tile_to_world3` documents the same), so
//! `NW = x<w/2 ∧ y≥h/2`, `SW = x<w/2 ∧ y<h/2`. Fire seeding, the NE build seed, hydrology emphasis, tests and
//! the dump all read this module — no ad-hoc `y < h/2` anywhere else.
//!
//! | Quadrant | Role | Seeded by |
//! |:--|:--|:--|
//! | NW | fire | `test_harness` fire seeder (land chunks inside NW only) |
//! | NE | build order | `test_harness` NE seeder → `CommitConstructionSiteEvent` (generic pilot catalog) |
//! | SW | river-attached lake basin | world-gen hydrology (`LandFeatureRhythm::playquad_hydrology` + chosen seed) |
//! | SE | marsh belt | world-gen hydrology marsh belts (`marsh_emphasis`, `marsh_region` = SE from [`quadrant_aabb`]) |
//!
//! Rain is **not** a quadrant: one world-wide preset, dry first then wet (P5 draws it in PixelWorld3D).
//! `--test buildread` keeps no fire seeds and no weather chrome; weather never writes `massing_columns`.

use bevy::prelude::Vec2;

use crate::terrain::world_map_scale::TileExtentPreset;

/// P1: play-test world extent (512²; chunk matrix authoritative). FrameScreen stays TacticalSmall.
pub const PLAYTEST_EXTENT_PRESET: TileExtentPreset = TileExtentPreset::Standard;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlayQuadrant {
    Nw,
    Ne,
    Sw,
    Se,
}

impl PlayQuadrant {
    pub const ALL: [Self; 4] = [Self::Nw, Self::Ne, Self::Sw, Self::Se];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Nw => "NW fire",
            Self::Ne => "NE build order",
            Self::Sw => "SW lake basin",
            Self::Se => "SE marsh belt",
        }
    }
}

/// Half-open integer tile rectangle `[x0, x1) × [y0, y1)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileRect {
    pub x0: u32,
    pub y0: u32,
    pub x1: u32,
    pub y1: u32,
}

impl TileRect {
    #[must_use]
    pub const fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x0 && x < self.x1 && y >= self.y0 && y < self.y1
    }

    #[must_use]
    pub const fn area(&self) -> u64 {
        (self.x1 - self.x0) as u64 * (self.y1 - self.y0) as u64
    }
}

/// Half-open tile-space box `[min, max)` (float view of a [`TileRect`], for chunk-center tests).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TileAabb {
    pub min: Vec2,
    pub max: Vec2,
}

impl TileAabb {
    #[must_use]
    pub fn contains(&self, p: Vec2) -> bool {
        p.x >= self.min.x && p.x < self.max.x && p.y >= self.min.y && p.y < self.max.y
    }

    #[must_use]
    pub fn center(&self) -> Vec2 {
        (self.min + self.max) * 0.5
    }
}

/// **The** partition. Quadrant rectangle of a `width × height` tile world, split at `(w/2, h/2)`, +Y north.
#[must_use]
pub const fn quadrant_tile_rect(q: PlayQuadrant, width: u32, height: u32) -> TileRect {
    let (mx, my) = (width / 2, height / 2);
    let (x0, x1) = match q {
        PlayQuadrant::Nw | PlayQuadrant::Sw => (0, mx),
        PlayQuadrant::Ne | PlayQuadrant::Se => (mx, width),
    };
    let (y0, y1) = match q {
        PlayQuadrant::Sw | PlayQuadrant::Se => (0, my),
        PlayQuadrant::Nw | PlayQuadrant::Ne => (my, height),
    };
    TileRect { x0, y0, x1, y1 }
}

/// Float view of [`quadrant_tile_rect`] (same split — never a second partition).
#[must_use]
pub fn quadrant_aabb(q: PlayQuadrant, width: u32, height: u32) -> TileAabb {
    let r = quadrant_tile_rect(q, width, height);
    TileAabb {
        min: Vec2::new(r.x0 as f32, r.y0 as f32),
        max: Vec2::new(r.x1 as f32, r.y1 as f32),
    }
}

/// Which quadrant a tile cell is in, or `None` outside the world.
#[must_use]
pub fn quadrant_of_tile(x: u32, y: u32, width: u32, height: u32) -> Option<PlayQuadrant> {
    PlayQuadrant::ALL
        .into_iter()
        .find(|q| quadrant_tile_rect(*q, width, height).contains(x, y))
}

/// Which quadrant a tile-space point is in, or `None` outside the world.
#[must_use]
pub fn quadrant_of(p: Vec2, width: u32, height: u32) -> Option<PlayQuadrant> {
    PlayQuadrant::ALL
        .into_iter()
        .find(|q| quadrant_aabb(*q, width, height).contains(p))
}

/// Rank candidate chunks: keep land fraction ≥ `min_land`, inside `region` when given, nearest `anchor` first.
///
/// Shared by the NW fire pick (anchor = quadrant center) and the NE build-yard pick. With `region = None`
/// this is the legacy world-center pick used by the other test scenes. Stable sort → deterministic.
#[must_use]
pub fn rank_land_chunks<K: Copy>(
    chunks: impl IntoIterator<Item = (K, Vec2, f32)>,
    region: Option<&TileAabb>,
    anchor: Vec2,
    min_land: f32,
) -> Vec<(f32, K)> {
    let mut ranked: Vec<(f32, K)> = chunks
        .into_iter()
        .filter(|(_, center, land)| {
            *land >= min_land && region.map_or(true, |r| r.contains(*center))
        })
        .map(|(key, center, _)| (anchor.distance_squared(center), key))
        .collect();
    ranked.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::world_scale_contract::CHUNK_TILES_SIM;

    #[test]
    fn quadrants_tile_the_world_without_overlap() {
        let (w, h) = (512, 512);
        let total: u64 = PlayQuadrant::ALL
            .into_iter()
            .map(|q| quadrant_tile_rect(q, w, h).area())
            .sum();
        assert_eq!(total, u64::from(w) * u64::from(h), "cover the world exactly");
        for (x, y) in [(0, 0), (255, 255), (256, 256), (511, 0), (0, 511), (511, 511)] {
            let hits = PlayQuadrant::ALL
                .into_iter()
                .filter(|q| quadrant_tile_rect(*q, w, h).contains(x, y))
                .count();
            assert_eq!(hits, 1, "({x},{y})");
        }
        assert_eq!(quadrant_of(Vec2::new(10.0, 500.0), w, h), Some(PlayQuadrant::Nw));
        assert_eq!(quadrant_of(Vec2::new(500.0, 500.0), w, h), Some(PlayQuadrant::Ne));
        assert_eq!(quadrant_of(Vec2::new(10.0, 10.0), w, h), Some(PlayQuadrant::Sw));
        assert_eq!(quadrant_of(Vec2::new(500.0, 10.0), w, h), Some(PlayQuadrant::Se));
        assert_eq!(quadrant_of(Vec2::new(512.0, 10.0), w, h), None);
        assert_eq!(quadrant_of_tile(300, 40, w, h), Some(PlayQuadrant::Se));
    }

    #[test]
    fn nw_is_the_high_y_half_and_sw_the_low_y_half() {
        let nw = quadrant_tile_rect(PlayQuadrant::Nw, 512, 512);
        assert_eq!((nw.x0, nw.y0, nw.x1, nw.y1), (0, 256, 256, 512));
        let sw = quadrant_tile_rect(PlayQuadrant::Sw, 512, 512);
        assert_eq!((sw.x0, sw.y0, sw.x1, sw.y1), (0, 0, 256, 256));
        // A cell at y >= h/2 is NW (fire), never SW (lake basin) — the WIP marsh bug.
        assert_eq!(quadrant_of_tile(10, 400, 512, 512), Some(PlayQuadrant::Nw));
        let aabb = quadrant_aabb(PlayQuadrant::Nw, 512, 512);
        assert_eq!(aabb.center(), Vec2::new(128.0, 384.0));
    }

    #[test]
    fn split_is_chunk_aligned_for_multiples_of_64() {
        for extent in [128u32, 192, 256, 320, 512, 640, 1024] {
            assert_eq!(extent % 64, 0);
            for q in PlayQuadrant::ALL {
                let r = quadrant_tile_rect(q, extent, extent);
                for edge in [r.x0, r.y0, r.x1, r.y1] {
                    assert_eq!(edge % CHUNK_TILES_SIM, 0, "{q:?} edge {edge} @ {extent}");
                }
            }
        }
    }

    #[test]
    fn rank_filters_region_and_land_and_sorts_by_anchor() {
        let nw = quadrant_aabb(PlayQuadrant::Nw, 512, 512);
        let chunks = vec![
            (1u32, Vec2::new(16.0, 400.0), 0.9), // NW land, near anchor
            (2, Vec2::new(16.0, 270.0), 0.9),    // NW land, far
            (3, Vec2::new(16.0, 390.0), 0.2),    // NW water — dropped
            (4, Vec2::new(400.0, 400.0), 1.0),   // NE — dropped
        ];
        let ranked = rank_land_chunks(chunks.clone(), Some(&nw), nw.center(), 0.5);
        let keys: Vec<u32> = ranked.iter().map(|(_, k)| *k).collect();
        assert_eq!(keys, vec![1, 2]);
        // No region: legacy world-wide pick keeps NE land.
        let world = rank_land_chunks(chunks, None, Vec2::new(256.0, 256.0), 0.5);
        assert_eq!(world.len(), 3);
    }

    #[test]
    fn playtest_extent_is_standard_512() {
        assert_eq!(PLAYTEST_EXTENT_PRESET.tiles_per_axis(), 512);
    }
}
