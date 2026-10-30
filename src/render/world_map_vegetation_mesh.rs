//! VEG-H02 — trees and cover on the TacticalFlat2D world map.
//!
//! Reads [`VegetationExtractFrame`] and nothing else about vegetation (no second extract,
//! no second LOD ladder). The extract carries one row per chunk, so each row is expanded
//! into deterministic symbols inside that chunk. All symbols for the visible chunks go into
//! one merged vertex-colored mesh on one entity.
//!
//! Plane and layering: same tile axes as the terrain sprite (`[0,w] x [0,h]`, z = 0).
//! The layer sits at [`VEG_Z`], above terrain and below the construction / road plane
//! (`world_map_tile_center` z = 1, network ribbons z = 1.04). It is never pickable.
//!
//! Zoom is the camera ([`MainWorldCameraOrthoTrace`]): pixels per tile pick one of four
//! [`VegLod`] bands. A band change, a visible-chunk-window change, or an extract content
//! change rebuilds the mesh. Nothing else does.
//!
//! Water: no symbol is placed where the footprint touches an open-water tile. The mask is the sim's
//! per-cell water class ([`ChunkCellMatrix::water`], decided once by world generation) at tile resolution,
//! and each symbol is tested at its own sub-cell position so shorelines stay dry on the land side.
//!
//! PixelWorld3D keeps its own per-chunk LG-5 atlas stamps, so this layer is hidden there.

use std::collections::HashSet;

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use crate::engine::states::BaseState;
use crate::gui::{MainWorldCameraOrthoTrace, WorldPresentationMode};
use crate::render::extraction::{VegExtractRow, VegetationExtractFrame, VegetationExtractFrameSet};
use crate::systems::ecology::SuccessionTopologyStage;
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::world_scale_contract::CHUNK_TILES_SIM;

/// Above terrain (0), below `world_map_tile_center` (1) and the network ribbons (1.04).
pub const VEG_Z: f32 = 0.5;

/// Symbol budget per build. A finer band is dropped to a coarser one past this.
const MAX_SYMBOLS: usize = 40_000;

/// Chunks beyond the view edge that still build, so a pan does not pop at the border.
const WINDOW_MARGIN_CHUNKS: i32 = 1;

#[derive(Component)]
pub(crate) struct WorldMapVegetationLayer;

/// Zoom band, chosen from screen pixels per tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum VegLod {
    /// One tinted quad per chunk. Whole-map overview.
    Wash,
    /// Stand blobs on a 4-tile grid.
    Blob,
    /// Single crowns on a 2-tile grid.
    Tree,
    /// Crowns with trunks on a 1-tile grid.
    Fine,
}

impl VegLod {
    #[must_use]
    pub(crate) fn for_px_per_tile(px: f32) -> Self {
        if px < 1.2 {
            Self::Wash
        } else if px < 4.0 {
            Self::Blob
        } else if px < 10.0 {
            Self::Tree
        } else {
            Self::Fine
        }
    }

    fn step_tiles(self) -> u32 {
        match self {
            Self::Wash => 0,
            Self::Blob => 4,
            Self::Tree => 2,
            Self::Fine => 1,
        }
    }

    fn coarser(self) -> Option<Self> {
        match self {
            Self::Fine => Some(Self::Tree),
            Self::Tree => Some(Self::Blob),
            Self::Blob => Some(Self::Wash),
            Self::Wash => None,
        }
    }
}

/// What the stand is, read from the extract row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Cover {
    Grass,
    Shrub,
    YoungForest,
    OldGrowth,
    Burning,
    Burnt,
    Regrowth,
}

/// How the stand lies inside the chunk (program topology kind).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Layout {
    Uniform,
    Patch,
    /// Strip across the chunk. Reads as a riparian / hedgerow band.
    Corridor,
    Ring,
    Cluster,
    Fringe,
}

#[must_use]
pub(crate) fn classify_row(row: &VegExtractRow) -> (Cover, Layout) {
    let key = row.variant_key.as_str();
    let cover = if row.burn_active {
        Cover::Burning
    } else if key == "veg_damaged" || row.succession_stage == SuccessionTopologyStage::BurnScar {
        Cover::Burnt
    } else if key.starts_with("veg_regrowth") {
        Cover::Regrowth
    } else {
        match row.succession_stage {
            SuccessionTopologyStage::Grass => Cover::Grass,
            SuccessionTopologyStage::Shrub => Cover::Shrub,
            SuccessionTopologyStage::YoungForest => Cover::YoungForest,
            SuccessionTopologyStage::OldGrowth => Cover::OldGrowth,
            SuccessionTopologyStage::BurnScar => Cover::Burnt,
        }
    };
    let layout = match key.strip_prefix("veg_topo_") {
        Some("patch") => Layout::Patch,
        Some("corridor") => Layout::Corridor,
        Some("ring") => Layout::Ring,
        Some("cluster") => Layout::Cluster,
        Some("fringe") => Layout::Fringe,
        _ => Layout::Uniform,
    };
    (cover, layout)
}

fn mix64(mut x: u64) -> u64 {
    x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^ (x >> 31)
}

/// Deterministic value in `[0, 1)` for a chunk site.
fn site_rand(coord: IVec2, a: u32, b: u32, salt: u64) -> f32 {
    let h = mix64(
        (coord.x as u32 as u64)
            ^ ((coord.y as u32 as u64) << 32)
            ^ salt.wrapping_mul(0xD6E8_FEB8_6659_FD93)
            ^ ((u64::from(a) << 20) ^ u64::from(b)).wrapping_mul(0xA24B_AED4_963E_E407),
    );
    (h >> 40) as f32 / (1u64 << 24) as f32
}

/// Layout weight at chunk-local `(u, v)` in `[0, 1]`, and that layout's mean weight.
fn layout_weight(layout: Layout, coord: IVec2, u: f32, v: f32) -> (f32, f32) {
    match layout {
        Layout::Uniform => (1.0, 1.0),
        Layout::Patch => {
            let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt();
            (if d < 0.42 { 1.0 } else { 0.25 }, 0.665)
        }
        Layout::Corridor => {
            let phase = site_rand(coord, 7, 7, 11) * std::f32::consts::TAU;
            let v0 = 0.5 + 0.18 * (u * std::f32::consts::TAU + phase).sin();
            (if (v - v0).abs() < 0.16 { 1.0 } else { 0.12 }, 0.40)
        }
        Layout::Ring => {
            let d = ((u - 0.5).powi(2) + (v - 0.5).powi(2)).sqrt();
            (if (0.28..0.46).contains(&d) { 1.0 } else { 0.15 }, 0.505)
        }
        Layout::Cluster => {
            let mut near = false;
            for i in 0..3u32 {
                let cx = 0.2 + 0.6 * site_rand(coord, i, 1, 21);
                let cy = 0.2 + 0.6 * site_rand(coord, i, 2, 21);
                if ((u - cx).powi(2) + (v - cy).powi(2)).sqrt() < 0.2 {
                    near = true;
                    break;
                }
            }
            (if near { 1.0 } else { 0.12 }, 0.45)
        }
        Layout::Fringe => {
            let edge = u.min(1.0 - u).min(v).min(1.0 - v);
            (if edge < 0.2 { 1.0 } else { 0.3 }, 0.75)
        }
    }
}

/// Chance a candidate site holds a symbol.
fn base_probability(cover: Cover, density: f32) -> f32 {
    let d = density.clamp(0.0, 1.0);
    match cover {
        Cover::OldGrowth | Cover::YoungForest | Cover::Shrub => d * 0.9,
        Cover::Grass => d * 0.6,
        Cover::Burning => d.max(0.4),
        Cover::Burnt => 0.3 + 0.4 * d,
        Cover::Regrowth => 0.2 + 0.4 * d,
    }
}

/// Water tiles in world tile coordinates.
#[derive(Default, Clone, Debug)]
pub(crate) struct WaterMask {
    tiles: HashSet<(i32, i32)>,
}

impl WaterMask {
    fn tile_wet(&self, p: Vec2) -> bool {
        self.tiles.contains(&(p.x.floor() as i32, p.y.floor() as i32))
    }

    /// True when the symbol center or any of four points at `reach` tiles falls on water.
    fn footprint_wet(&self, center: Vec2, reach: f32) -> bool {
        if self.tiles.is_empty() {
            return false;
        }
        self.tile_wet(center)
            || [Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y]
                .iter()
                .any(|d| self.tile_wet(center + *d * reach))
    }
}

/// Open-water tiles read from the sim's per-cell water class.
fn build_water_mask<'a>(chunks: impl Iterator<Item = (&'a Chunk, &'a ChunkCellMatrix)>) -> WaterMask {
    let mut tiles = HashSet::new();
    for (chunk, m) in chunks {
        let ox = chunk.coord.x * m.size.x as i32;
        let oy = chunk.coord.y * m.size.y as i32;
        for y in 0..m.size.y {
            for x in 0..m.size.x {
                if m.water.get(m.idx(x, y)).is_some_and(|c| c.is_open_water()) {
                    tiles.insert((ox + x as i32, oy + y as i32));
                }
            }
        }
    }
    WaterMask { tiles }
}

/// Merged vertex-colored triangle soup in tile space.
#[derive(Default)]
pub(crate) struct VegGeometry {
    pub positions: Vec<[f32; 3]>,
    pub colors: Vec<[f32; 4]>,
    pub indices: Vec<u32>,
    pub symbols: usize,
}

impl VegGeometry {
    fn vertex(&mut self, x: f32, y: f32, c: [f32; 4]) -> u32 {
        let i = self.positions.len() as u32;
        self.positions.push([x, y, 0.0]);
        self.colors.push(c);
        i
    }

    fn polygon(&mut self, cx: f32, cy: f32, r: f32, sides: u32, c: [f32; 4]) {
        let first = self.positions.len() as u32;
        for i in 0..sides {
            let a = std::f32::consts::TAU * (i as f32 / sides as f32) + 0.4;
            self.vertex(cx + a.cos() * r, cy + a.sin() * r, c);
        }
        for i in 1..sides.saturating_sub(1) {
            self.indices.extend_from_slice(&[first, first + i, first + i + 1]);
        }
    }

    fn hex(&mut self, cx: f32, cy: f32, r: f32, c: [f32; 4]) {
        self.polygon(cx, cy, r, 6, c);
    }

    fn diamond(&mut self, cx: f32, cy: f32, r: f32, c: [f32; 4]) {
        self.polygon(cx, cy, r, 4, c);
    }

    fn tri(&mut self, cx: f32, cy: f32, r: f32, c: [f32; 4]) {
        let a = self.vertex(cx, cy + r, c);
        let b = self.vertex(cx - r * 0.87, cy - r * 0.5, c);
        let d = self.vertex(cx + r * 0.87, cy - r * 0.5, c);
        self.indices.extend_from_slice(&[a, b, d]);
    }

    fn rect(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, c: [f32; 4]) {
        let a = self.vertex(x0, y0, c);
        let b = self.vertex(x1, y0, c);
        let d = self.vertex(x1, y1, c);
        let e = self.vertex(x0, y1, c);
        self.indices.extend_from_slice(&[a, b, d, a, d, e]);
    }
}

/// Linear RGBA for the vertex-color attribute. `shade` is a signed brightness offset.
fn rgba(r: f32, g: f32, b: f32, a: f32, shade: f32) -> [f32; 4] {
    let s = 1.0 + shade;
    Color::srgba(
        (r * s).clamp(0.0, 1.0),
        (g * s).clamp(0.0, 1.0),
        (b * s).clamp(0.0, 1.0),
        a,
    )
    .to_linear()
    .to_f32_array()
}

fn wash_color(cover: Cover, density: f32) -> [f32; 4] {
    let d = density.clamp(0.0, 1.0);
    match cover {
        Cover::OldGrowth => rgba(0.07, 0.27, 0.13, 0.25 + 0.5 * d, 0.0),
        Cover::YoungForest => rgba(0.15, 0.42, 0.18, 0.2 + 0.45 * d, 0.0),
        Cover::Shrub => rgba(0.38, 0.48, 0.20, 0.15 + 0.35 * d, 0.0),
        Cover::Grass => rgba(0.55, 0.65, 0.30, 0.08 + 0.2 * d, 0.0),
        Cover::Burning => rgba(1.0, 0.45, 0.08, 0.65, 0.0),
        Cover::Burnt => rgba(0.10, 0.09, 0.08, 0.55, 0.0),
        Cover::Regrowth => rgba(0.45, 0.70, 0.30, 0.3, 0.0),
    }
}

/// Riparian strips lean blue-green.
fn riparian_tint(c: [f32; 4], on: bool) -> [f32; 4] {
    if on {
        [c[0] * 0.8, c[1] * 1.05, c[2] * 1.3, c[3]]
    } else {
        c
    }
}

struct Site {
    lod: VegLod,
    cover: Cover,
    riparian: bool,
    pos: Vec2,
    step: f32,
    rand: f32,
    burn_phase: u8,
}

fn push_symbol(g: &mut VegGeometry, s: &Site) {
    let (pos, step) = (s.pos, s.step);
    let shade = (s.rand - 0.5) * 0.12;
    let fine = s.lod == VegLod::Fine;
    let detailed = matches!(s.lod, VegLod::Tree | VegLod::Fine);
    let size_jitter = 0.9 + 0.3 * s.rand;
    match s.cover {
        Cover::OldGrowth => {
            let r = step * 0.55 * size_jitter;
            if fine {
                g.rect(
                    pos.x - r * 0.12,
                    pos.y - r * 0.9,
                    pos.x + r * 0.12,
                    pos.y - r * 0.2,
                    rgba(0.30, 0.20, 0.12, 1.0, shade),
                );
            }
            g.hex(pos.x, pos.y, r, riparian_tint(rgba(0.07, 0.27, 0.13, 0.95, shade), s.riparian));
            if detailed {
                g.hex(
                    pos.x - r * 0.15,
                    pos.y + r * 0.15,
                    r * 0.5,
                    riparian_tint(rgba(0.12, 0.38, 0.18, 0.9, shade), s.riparian),
                );
            }
        }
        Cover::YoungForest => {
            let r = step * 0.45 * size_jitter;
            if fine {
                g.rect(
                    pos.x - r * 0.12,
                    pos.y - r * 0.9,
                    pos.x + r * 0.12,
                    pos.y - r * 0.2,
                    rgba(0.32, 0.22, 0.13, 1.0, shade),
                );
            }
            g.hex(pos.x, pos.y, r, riparian_tint(rgba(0.15, 0.42, 0.18, 0.92, shade), s.riparian));
        }
        Cover::Shrub => {
            g.diamond(
                pos.x,
                pos.y,
                step * 0.32 * size_jitter,
                riparian_tint(rgba(0.38, 0.48, 0.20, 0.9, shade), s.riparian),
            );
        }
        Cover::Grass => {
            if detailed {
                g.tri(pos.x, pos.y, step * 0.25 * size_jitter, rgba(0.55, 0.65, 0.30, 0.8, shade));
            } else {
                g.diamond(pos.x, pos.y, step * 0.2, rgba(0.55, 0.65, 0.30, 0.5, shade));
            }
        }
        Cover::Burning => {
            let flicker = 0.9 + 0.1 * f32::from(s.burn_phase % 3);
            let r = step * 0.5 * size_jitter;
            g.hex(pos.x, pos.y, r, rgba(1.0, 0.40, 0.06, 0.95, shade * flicker));
            if detailed {
                g.hex(pos.x, pos.y, r * 0.5, rgba(1.0, 0.85, 0.30, 1.0, shade * flicker));
            }
        }
        Cover::Burnt => {
            g.diamond(pos.x, pos.y, step * 0.3 * size_jitter, rgba(0.10, 0.09, 0.08, 0.9, shade));
        }
        Cover::Regrowth => {
            g.tri(pos.x, pos.y, step * 0.28 * size_jitter, rgba(0.45, 0.70, 0.30, 0.9, shade));
        }
    }
    g.symbols += 1;
}

/// Wash block edge in tiles. Coarse, but fine enough that a coast is not painted over.
const WASH_BLOCK_TILES: u32 = 8;

/// Candidate sites per axis for a chunk at this band.
fn sites_per_axis(chunk: UVec2, lod: VegLod) -> (u32, u32) {
    let s = lod.step_tiles().max(1);
    (chunk.x.div_ceil(s), chunk.y.div_ceil(s))
}

/// Rough symbol count for these rows, used to pick a band inside the budget.
fn estimate_symbols(rows: &[&VegExtractRow], chunk: UVec2, lod: VegLod) -> usize {
    if lod == VegLod::Wash {
        return rows.len() * (chunk.x.div_ceil(WASH_BLOCK_TILES) * chunk.y.div_ceil(WASH_BLOCK_TILES)) as usize;
    }
    let (nx, ny) = sites_per_axis(chunk, lod);
    let sites = (nx * ny) as f32;
    rows.iter()
        .map(|r| {
            let (cover, _) = classify_row(r);
            (sites * base_probability(cover, r.modifiers.mean_density)) as usize
        })
        .sum()
}

/// Expand extract rows into symbols. `window` is an inclusive chunk range, `None` for all rows.
/// Returns the geometry and the band actually used (coarser than asked past the budget).
pub(crate) fn build_vegetation_geometry(
    rows: &[VegExtractRow],
    chunk: UVec2,
    lod: VegLod,
    window: Option<(IVec2, IVec2)>,
    water: &WaterMask,
) -> (VegGeometry, VegLod) {
    let mut visible: Vec<&VegExtractRow> = rows
        .iter()
        .filter(|r| {
            window.is_none_or(|(lo, hi)| {
                r.coord.x >= lo.x && r.coord.x <= hi.x && r.coord.y >= lo.y && r.coord.y <= hi.y
            })
        })
        .collect();
    // Far rows first (higher world y is higher on screen) so nearer rows overdraw.
    visible.sort_by_key(|r| (std::cmp::Reverse(r.coord.y), r.coord.x));

    let mut lod = lod;
    while estimate_symbols(&visible, chunk, lod) > MAX_SYMBOLS {
        match lod.coarser() {
            Some(c) => lod = c,
            None => break,
        }
    }

    let mut g = VegGeometry::default();
    let step = lod.step_tiles() as f32;
    for row in visible {
        let (cover, layout) = classify_row(row);
        let density = row.modifiers.mean_density;
        let origin = Vec2::new(
            row.coord.x as f32 * chunk.x as f32,
            row.coord.y as f32 * chunk.y as f32,
        );
        let base = base_probability(cover, density);
        if lod == VegLod::Wash {
            if base < 0.03 {
                continue;
            }
            let color = wash_color(cover, density);
            let blk = WASH_BLOCK_TILES as f32;
            for by in 0..chunk.y.div_ceil(WASH_BLOCK_TILES) {
                for bx in 0..chunk.x.div_ceil(WASH_BLOCK_TILES) {
                    let x0 = origin.x + bx as f32 * blk;
                    let y0 = origin.y + by as f32 * blk;
                    let x1 = (x0 + blk).min(origin.x + chunk.x as f32);
                    let y1 = (y0 + blk).min(origin.y + chunk.y as f32);
                    if water.footprint_wet(Vec2::new((x0 + x1) * 0.5, (y0 + y1) * 0.5), blk * 0.4) {
                        continue;
                    }
                    g.rect(x0 + 0.3, y0 + 0.3, x1 - 0.3, y1 - 0.3, color);
                    g.symbols += 1;
                }
            }
            continue;
        }
        if base < 0.02 {
            continue;
        }
        let riparian = layout == Layout::Corridor;
        let (nx, ny) = sites_per_axis(chunk, lod);
        for iy in (0..ny).rev() {
            for ix in 0..nx {
                let jx = 0.2 + 0.6 * site_rand(row.coord, ix, iy, 2);
                let jy = 0.2 + 0.6 * site_rand(row.coord, ix, iy, 3);
                let lx = (ix as f32 + jx) * step;
                let ly = (iy as f32 + jy) * step;
                if lx >= chunk.x as f32 || ly >= chunk.y as f32 {
                    continue;
                }
                let (w, mean) =
                    layout_weight(layout, row.coord, lx / chunk.x as f32, ly / chunk.y as f32);
                let p = (base * w / mean).min(0.97);
                if site_rand(row.coord, ix, iy, 1) >= p {
                    continue;
                }
                let pos = origin + Vec2::new(lx, ly);
                if water.footprint_wet(pos, step * 0.4) {
                    continue;
                }
                push_symbol(
                    &mut g,
                    &Site {
                        lod,
                        cover,
                        riparian,
                        pos,
                        step,
                        rand: site_rand(row.coord, ix, iy, 4),
                        burn_phase: row.frame_index / 3,
                    },
                );
            }
        }
    }
    (g, lod)
}

fn geometry_to_mesh(g: VegGeometry) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, g.positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, g.colors);
    mesh.insert_indices(Indices::U32(g.indices));
    mesh
}

/// What the extract looks like to this layer. `revision` bumps every sim step, so the
/// layer hashes only the fields it draws.
fn extract_content_hash(rows: &[VegExtractRow], chunk: UVec2) -> u64 {
    let mut h = mix64(rows.len() as u64 ^ (u64::from(chunk.x) << 32) ^ u64::from(chunk.y));
    for r in rows {
        let (cover, layout) = classify_row(r);
        let q = (r.modifiers.mean_density.clamp(0.0, 1.0) * 16.0).round() as u64;
        let burn = if r.burn_active { u64::from(r.frame_index / 3) + 1 } else { 0 };
        h = mix64(
            h ^ (r.coord.x as u32 as u64)
                ^ ((r.coord.y as u32 as u64) << 32)
                ^ ((cover as u64) << 8)
                ^ ((layout as u64) << 16)
                ^ (q << 24)
                ^ (burn << 40),
        );
    }
    h
}

#[derive(Default)]
struct LayerState {
    built_key: Option<u64>,
    content_hash: u64,
    mesh: Option<Handle<Mesh>>,
    material: Option<Handle<ColorMaterial>>,
    last_empty: bool,
    water: WaterMask,
    water_rev: u64,
}

/// Visible chunk range plus a margin, or `None` when no camera fit exists yet.
fn chunk_window(trace: &MainWorldCameraOrthoTrace, chunk: UVec2) -> Option<(IVec2, IVec2)> {
    if trace.fixed_width <= 0.0 || trace.fixed_height <= 0.0 {
        return None;
    }
    let half = Vec2::new(trace.fixed_width, trace.fixed_height) * 0.5;
    let lo = trace.camera_center - half;
    let hi = trace.camera_center + half;
    let cs = Vec2::new(chunk.x.max(1) as f32, chunk.y.max(1) as f32);
    let m = WINDOW_MARGIN_CHUNKS;
    Some((
        IVec2::new((lo.x / cs.x).floor() as i32 - m, (lo.y / cs.y).floor() as i32 - m),
        IVec2::new((hi.x / cs.x).floor() as i32 + m, (hi.y / cs.y).floor() as i32 + m),
    ))
}

fn px_per_tile(trace: &MainWorldCameraOrthoTrace) -> f32 {
    if trace.fixed_width > 0.0 && trace.view_pixels.x > 0.0 {
        trace.view_pixels.x / trace.fixed_width
    } else {
        // No camera fit yet: overview band.
        1.0
    }
}

pub(crate) fn sync_world_map_vegetation(
    mut commands: Commands,
    base: Res<State<BaseState>>,
    mode: Option<Res<WorldPresentationMode>>,
    frame: Option<Res<VegetationExtractFrame>>,
    trace: Option<Res<MainWorldCameraOrthoTrace>>,
    matrices: Query<(&Chunk, &ChunkCellMatrix)>,
    matrices_changed: Query<(), Changed<ChunkCellMatrix>>,
    mut layer: Query<(Entity, &mut Visibility), With<WorldMapVegetationLayer>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: Local<LayerState>,
) {
    if !matches!(base.get(), BaseState::Simulation) {
        for (e, _) in &layer {
            commands.entity(e).despawn();
        }
        state.built_key = None;
        return;
    }
    // PixelWorld3D has its own LG-5 atlas stamps. Hide, keep the built mesh for the way back.
    if mode.as_deref().is_some_and(|m| !m.is_tactical()) {
        for (_, mut vis) in &mut layer {
            vis.set_if_neq(Visibility::Hidden);
        }
        return;
    }
    let Some(frame) = frame else {
        return;
    };
    let chunk = matrices
        .iter()
        .next()
        .map(|(_, m)| m.size)
        .filter(|s| s.x > 0 && s.y > 0)
        .unwrap_or(UVec2::splat(CHUNK_TILES_SIM));
    if frame.is_changed() || state.built_key.is_none() {
        state.content_hash = extract_content_hash(&frame.rows, chunk);
    }
    // Water changes with a world regen (chunk matrices restamped), not per tick.
    if state.water_rev == 0 || !matrices_changed.is_empty() {
        state.water = build_water_mask(matrices.iter());
        state.water_rev = state.water_rev.wrapping_add(1);
    }
    let default_trace = MainWorldCameraOrthoTrace::default();
    let trace = trace.as_deref().unwrap_or(&default_trace);
    let lod = VegLod::for_px_per_tile(px_per_tile(trace));
    let window = if lod == VegLod::Wash { None } else { chunk_window(trace, chunk) };
    let window_key = window.map_or(0, |(lo, hi)| {
        mix64(
            (lo.x as u32 as u64)
                ^ ((lo.y as u32 as u64) << 16)
                ^ ((hi.x as u32 as u64) << 32)
                ^ ((hi.y as u32 as u64) << 48),
        )
    });
    let key = mix64(state.content_hash ^ (lod as u64) ^ window_key ^ mix64(state.water_rev));

    let mesh_handle = state
        .mesh
        .get_or_insert_with(|| meshes.add(geometry_to_mesh(VegGeometry::default())))
        .clone();
    let material = state
        .material
        .get_or_insert_with(|| materials.add(ColorMaterial::from_color(Color::WHITE)))
        .clone();

    if state.built_key != Some(key) {
        let (geo, _) = build_vegetation_geometry(&frame.rows, chunk, lod, window, &state.water);
        state.last_empty = geo.symbols == 0;
        meshes.insert(&mesh_handle, geometry_to_mesh(geo));
        state.built_key = Some(key);
    }
    let want = if state.last_empty { Visibility::Hidden } else { Visibility::Visible };

    if layer.is_empty() {
        commands.spawn((
            WorldMapVegetationLayer,
            Mesh2d(mesh_handle),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, VEG_Z),
            want,
            Pickable::IGNORE,
        ));
    } else {
        for (_, mut vis) in &mut layer {
            vis.set_if_neq(want);
        }
    }
}

pub struct WorldMapVegetationMeshPlugin;

impl Plugin for WorldMapVegetationMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            sync_world_map_vegetation.after(VegetationExtractFrameSet::BuildProfiles),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gui::WorldPresentationKind;
    use crate::render::extraction::VegExtractModifiers;

    fn row(
        coord: IVec2,
        stage: SuccessionTopologyStage,
        key: &str,
        density: f32,
        burn: bool,
    ) -> VegExtractRow {
        VegExtractRow {
            coord,
            planning_glyph: '#',
            extract_glyph: '#',
            modifiers: VegExtractModifiers {
                burn_active: burn,
                mean_density: density,
            },
            variant_key: key.to_owned(),
            succession_stage: stage,
            burn_active: burn,
            frame_index: 0,
        }
    }

    fn known_rows() -> Vec<VegExtractRow> {
        vec![
            row(IVec2::new(0, 0), SuccessionTopologyStage::OldGrowth, "veg_topo_patch", 0.9, false),
            row(IVec2::new(1, 0), SuccessionTopologyStage::Shrub, "veg_topo_corridor", 0.5, false),
            row(IVec2::new(0, 1), SuccessionTopologyStage::YoungForest, "veg_burn_02", 0.6, true),
        ]
    }

    fn is_ember(c: &[f32; 4]) -> bool {
        c[0] > 0.8 && c[2] < 0.1
    }

    #[test]
    fn rows_build_symbols_inside_their_chunks_and_zoom_changes_density() {
        let chunk = UVec2::splat(32);
        let rows = known_rows();
        let dry = WaterMask::default();
        let (fine, fine_lod) = build_vegetation_geometry(&rows, chunk, VegLod::Fine, None, &dry);
        let (blob, _) = build_vegetation_geometry(&rows, chunk, VegLod::Blob, None, &dry);
        let (wash, _) = build_vegetation_geometry(&rows, chunk, VegLod::Wash, None, &dry);
        assert_eq!(fine_lod, VegLod::Fine);
        assert!(fine.symbols > blob.symbols, "{} vs {}", fine.symbols, blob.symbols);
        assert_eq!(wash.symbols, 3 * 16, "8-tile wash blocks per chunk");
        assert_eq!(fine.positions.len(), fine.colors.len());
        assert_eq!(fine.indices.len() % 3, 0);
        for p in &fine.positions {
            assert!(p[0] > -1.5 && p[0] < 65.5 && p[1] > -1.5 && p[1] < 65.5, "{p:?}");
        }
        // The burning chunk draws ember, and only that chunk does.
        assert!(fine.colors.iter().any(is_ember));
        let (calm, _) = build_vegetation_geometry(&rows[..2], chunk, VegLod::Fine, None, &dry);
        assert!(!calm.colors.iter().any(is_ember));
    }

    #[test]
    fn no_symbols_on_water_tiles() {
        let chunk = UVec2::splat(32);
        let rows = known_rows();
        // The whole left half of chunk (0,0) is water.
        let mut water = WaterMask::default();
        for x in 0..16 {
            for y in 0..32 {
                water.tiles.insert((x, y));
            }
        }
        for lod in [VegLod::Fine, VegLod::Tree, VegLod::Blob, VegLod::Wash] {
            let (g, _) = build_vegetation_geometry(
                &rows[..1],
                chunk,
                lod,
                None,
                &water,
            );
            assert!(g.symbols > 0, "{lod:?}");
            // Symbol centers are not inspectable after merge; vertices stay out of the wet half
            // by at most one crown radius.
            for p in &g.positions {
                assert!(p[0] > 16.0 - 2.5, "{lod:?} {p:?}");
            }
        }
    }

    #[test]
    fn window_culls_chunks_and_classify_reads_stand_type() {
        let chunk = UVec2::splat(32);
        let rows = known_rows();
        let (g, _) = build_vegetation_geometry(
            &rows,
            chunk,
            VegLod::Tree,
            Some((IVec2::new(1, 0), IVec2::new(1, 0))),
            &WaterMask::default(),
        );
        assert!(g.symbols > 0);
        for p in &g.positions {
            assert!(p[0] > 30.0 && p[0] < 66.0 && p[1] > -2.0 && p[1] < 34.0, "{p:?}");
        }
        assert_eq!(classify_row(&rows[0]), (Cover::OldGrowth, Layout::Patch));
        assert_eq!(classify_row(&rows[1]), (Cover::Shrub, Layout::Corridor));
        assert_eq!(classify_row(&rows[2]).0, Cover::Burning);
        let scar = row(IVec2::ZERO, SuccessionTopologyStage::BurnScar, "veg_damaged", 0.1, false);
        assert_eq!(classify_row(&scar).0, Cover::Burnt);
    }

    #[test]
    fn layer_draws_in_tactical_and_hides_in_pixel_world() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::state::app::StatesPlugin,
            AssetPlugin::default(),
        ));
        app.init_asset::<Mesh>();
        app.init_asset::<ColorMaterial>();
        app.init_state::<BaseState>();
        app.insert_resource(WorldPresentationMode::default());
        let frame = VegetationExtractFrame {
            rows: known_rows(),
            ..Default::default()
        };
        app.insert_resource(frame);
        app.add_systems(Update, sync_world_map_vegetation);
        app.world_mut()
            .resource_mut::<NextState<BaseState>>()
            .set(BaseState::Simulation);
        app.update();
        app.update();

        let vis = |app: &mut App| {
            let mut q = app
                .world_mut()
                .query_filtered::<&Visibility, With<WorldMapVegetationLayer>>();
            q.iter(app.world()).copied().collect::<Vec<_>>()
        };
        assert_eq!(vis(&mut app), vec![Visibility::Visible]);
        let handle = {
            let mut q = app
                .world_mut()
                .query_filtered::<&Mesh2d, With<WorldMapVegetationLayer>>();
            q.single(app.world()).unwrap().0.clone()
        };
        let verts = app
            .world()
            .resource::<Assets<Mesh>>()
            .get(&handle)
            .and_then(|m| m.attribute(Mesh::ATTRIBUTE_POSITION).map(|a| a.len()))
            .unwrap_or(0);
        assert!(verts > 0);

        app.world_mut().resource_mut::<WorldPresentationMode>().kind =
            WorldPresentationKind::PixelWorld3D;
        app.update();
        assert_eq!(vis(&mut app), vec![Visibility::Hidden]);

        app.world_mut().resource_mut::<WorldPresentationMode>().kind =
            WorldPresentationKind::TacticalFlat2D;
        app.update();
        assert_eq!(vis(&mut app), vec![Visibility::Visible]);
    }
}
