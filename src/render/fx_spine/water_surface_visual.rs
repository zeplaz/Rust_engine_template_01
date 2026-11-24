//! Water surface presentation catalog (FX-WATER-SHADER-001 / W1).
//!
//! Built from authoritative [`HydrologyResult`] at world-gen — **no second terrain extract**.

use std::collections::HashSet;

use bevy::math::Vec4;
use bevy::prelude::*;
use bevy::render::RenderApp;
use bevy::render::extract_resource::ExtractResource;
use bytemuck::{Pod, Zeroable};

use crate::terrain::generation::hydrology::HydrologyResult;
use crate::terrain::generation::water_class::{WaterClass, WorldWaterGrid};
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Presentation-only water classification (D-W01…D-W04).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
#[repr(u32)]
pub enum WaterSurfaceKind {
    #[default]
    None = 0,
    Lake = 1,
    River = 2,
    Ocean = 3,
}

impl WaterSurfaceKind {
    #[inline]
    const fn as_f32(self) -> f32 {
        self as u32 as f32
    }
}

/// One river path segment for GPU ribbon overlay (D-W01 A, D-W03 A).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RiverPolylineSegment {
    pub path_id: u32,
    pub start: Vec2,
    pub end: Vec2,
    pub flow_dir: Vec2,
    pub half_width: f32,
}

/// Sparse standing-water motion anchor (lake ripple / ocean swell — D-W02, D-W04).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WaterMotionAnchor {
    pub kind: WaterSurfaceKind,
    pub world: Vec2,
    pub extent: f32,
}

/// Authoritative presentation catalog from hydrology (single build at world-gen).
#[derive(Resource, Clone, Debug, Default)]
pub struct WaterSurfaceVisualCatalog {
    pub stamp: u64,
    pub grid_width: u32,
    pub grid_height: u32,
    pub river_segments: Vec<RiverPolylineSegment>,
    pub motion_anchors: Vec<WaterMotionAnchor>,
    pub river_tiles: HashSet<(u32, u32)>,
    pub lake_tiles: HashSet<(u32, u32)>,
    pub ocean_tiles: HashSet<(u32, u32)>,
}

impl WaterSurfaceVisualCatalog {
    /// Build straight from a hydrology solve (classifies it with the sim authority first).
    #[must_use]
    pub fn from_hydrology(hydro: &HydrologyResult, params: &WorldGenParams) -> Self {
        let grid = WorldWaterGrid::from_hydrology(
            hydro,
            params.width,
            params.height,
            params.seed ^ 0xA7E2_0001,
        );
        Self::from_water_grid(&grid, hydro, params)
    }

    /// Presentation projection of the sim water authority ([`WorldWaterGrid`]): river / lake / ocean tiles are
    /// the classes, nothing here decides what is water. River polylines and motion anchors come from the
    /// hydrology paths and regions.
    #[must_use]
    pub fn from_water_grid(
        grid: &WorldWaterGrid,
        hydro: &HydrologyResult,
        params: &WorldGenParams,
    ) -> Self {
        let w = grid.width as usize;
        let mut river_tiles = HashSet::new();
        let mut lake_tiles = HashSet::new();
        let mut ocean_tiles = HashSet::new();
        let mut ocean_anchor_tiles: Vec<(u32, u32)> = Vec::new();
        for (i, class) in grid.classes.iter().enumerate() {
            let tile = ((i % w.max(1)) as u32, (i / w.max(1)) as u32);
            match class {
                WaterClass::River => {
                    river_tiles.insert(tile);
                }
                WaterClass::Lake => {
                    lake_tiles.insert(tile);
                }
                WaterClass::Ocean => {
                    ocean_tiles.insert(tile);
                    if ocean_anchor_tiles.len() < 512 {
                        ocean_anchor_tiles.push(tile);
                    }
                }
                WaterClass::Wetland | WaterClass::Dry => {}
            }
        }

        // Exclusive look: standing/ocean wins over channel paint (no river-on-lake stack).
        river_tiles.retain(|t| !lake_tiles.contains(t) && !ocean_tiles.contains(t));

        // One fluid, one kind per tile: river ribbons stop at standing water.
        // A path that enters a lake continues as lake water — not a river drawn on the lake.
        let mut river_segments = Vec::new();
        for (path_id, path) in hydro.rivers.iter().enumerate() {
            for window in path.windows(2) {
                let (ax, ay) = window[0];
                let (bx, by) = window[1];
                let a_ok = river_tiles.contains(&(ax, ay));
                let b_ok = river_tiles.contains(&(bx, by));
                if !a_ok || !b_ok {
                    continue;
                }
                let start = Vec2::new(ax as f32, ay as f32);
                let end = Vec2::new(bx as f32, by as f32);
                let delta = end - start;
                let len = delta.length();
                if len < 1e-4 {
                    continue;
                }
                let flow_dir = delta / len;
                river_segments.push(RiverPolylineSegment {
                    path_id: path_id as u32,
                    start,
                    end,
                    flow_dir,
                    half_width: 1.15,
                });
            }
        }

        let mut motion_anchors = Vec::new();
        for lake in &hydro.lakes {
            let Some(&(fx, fy)) = lake.cells.first() else {
                continue;
            };
            // Ocean regions get ocean anchors below; only inland lakes ripple as lakes.
            if grid.class_at(fx as i32, fy as i32) != Some(WaterClass::Lake) {
                continue;
            }
            let (sx, sy) = lake.cells.iter().fold((0u64, 0u64), |acc, &(x, y)| {
                (acc.0 + x as u64, acc.1 + y as u64)
            });
            let n_cells = lake.cells.len() as f32;
            motion_anchors.push(WaterMotionAnchor {
                kind: WaterSurfaceKind::Lake,
                world: Vec2::new(sx as f32 / n_cells, sy as f32 / n_cells),
                extent: 1.25,
            });
        }
        let gw = params.width;
        let gh = params.height;
        for &(tx, ty) in &ocean_anchor_tiles {
            // Rim "ocean" (edge-touching depressions) flickers hard — skip anchors there.
            if tx < 2 || ty < 2 || tx + 2 >= gw || ty + 2 >= gh {
                continue;
            }
            motion_anchors.push(WaterMotionAnchor {
                kind: WaterSurfaceKind::Ocean,
                world: Vec2::new(tx as f32, ty as f32),
                extent: 1.6,
            });
        }

        Self {
            stamp: grid.stamp,
            grid_width: params.width,
            grid_height: params.height,
            river_segments,
            motion_anchors,
            river_tiles,
            lake_tiles,
            ocean_tiles,
        }
    }

    #[inline]
    #[must_use]
    pub fn tile_kind(&self, x: u32, y: u32) -> WaterSurfaceKind {
        if self.river_tiles.contains(&(x, y)) {
            WaterSurfaceKind::River
        } else if self.lake_tiles.contains(&(x, y)) {
            WaterSurfaceKind::Lake
        } else if self.ocean_tiles.contains(&(x, y)) {
            WaterSurfaceKind::Ocean
        } else {
            WaterSurfaceKind::None
        }
    }

    #[must_use]
    pub fn has_motion(&self) -> bool {
        !self.river_tiles.is_empty() || !self.lake_tiles.is_empty() || !self.ocean_tiles.is_empty()
    }

    #[must_use]
    pub fn w1_green(&self) -> bool {
        !self.river_segments.is_empty() || !self.river_tiles.is_empty()
    }

    /// Authored N→S river corridor (DamFlood fixture). Overwrites lake/ocean class in-band
    /// so the dam site has a proper channel on the presentation map — seed alone is noise.
    pub fn inject_ns_river_corridor(
        &mut self,
        center_x: u32,
        z0: u32,
        z1: u32,
        half_width: u32,
    ) {
        if self.grid_width == 0 || self.grid_height == 0 || z1 <= z0 {
            return;
        }
        let hw = half_width.max(1);
        let z0 = z0.min(self.grid_height.saturating_sub(1));
        let z1 = z1.min(self.grid_height.saturating_sub(1)).max(z0);
        let cx = center_x.min(self.grid_width.saturating_sub(1));
        for z in z0..=z1 {
            for dx in 0..=hw {
                for &sx in &[cx.saturating_sub(dx), (cx + dx).min(self.grid_width.saturating_sub(1))]
                {
                    self.lake_tiles.remove(&(sx, z));
                    self.ocean_tiles.remove(&(sx, z));
                    self.river_tiles.insert((sx, z));
                }
            }
        }
        // Flow south (−Y): upstream north → downstream south (fixture gauges).
        let mut z = z0;
        while z < z1 {
            let zend = (z + 10).min(z1);
            self.river_segments.push(RiverPolylineSegment {
                path_id: 0xD00D,
                start: Vec2::new(cx as f32 + 0.5, z as f32),
                end: Vec2::new(cx as f32 + 0.5, zend as f32),
                flow_dir: Vec2::new(0.0, -1.0),
                half_width: (hw as f32 * 0.9).max(1.15),
            });
            z = zend;
        }
        self.stamp = self.stamp.wrapping_add(0xD00D_0001);
    }

    /// WATER-W1-OCEAN-001 — open-ocean presentation band has tiles (D-W04).
    #[must_use]
    pub fn w1_ocean_green(&self) -> bool {
        !self.ocean_tiles.is_empty()
    }

    /// WATER-W1-RIVER-001 — river geometry present for overlay / ribbon pass.
    #[must_use]
    pub fn w1_river_green(&self) -> bool {
        self.w1_green()
    }

    /// RGB channel delta between river and lake overlay at a tile (witness / tests).
    #[must_use]
    pub fn river_lake_rgb_delta_at_zoom(
        &self,
        tx: u32,
        ty: u32,
        time_secs: f32,
        zoom_alpha: f32,
    ) -> u32 {
        let base = [0u8, 0u8, 200, 255];
        let river = river_overlay_pixel(base, tx, ty, self.flow_at(tx, ty), time_secs, zoom_alpha);
        let lake = lake_overlay_pixel(base, tx, ty, time_secs, zoom_alpha);
        river
            .iter()
            .zip(lake.iter())
            .take(3)
            .map(|(a, b)| a.abs_diff(*b) as u32)
            .sum()
    }

    /// River ribbon must read distinct from lake teal at strategic zoom (D-W01 / D-W03).
    #[must_use]
    pub fn w1_river_read_green_at_zoom(&self, zoom_alpha: f32) -> bool {
        if !self.w1_river_green() {
            return false;
        }
        if let Some(&(tx, ty)) = self.river_tiles.iter().next() {
            return self.river_lake_rgb_delta_at_zoom(tx, ty, 0.0, zoom_alpha) >= 12;
        }
        // Polyline-only maps: GPU ribbon path carries directional read.
        !self.river_segments.is_empty()
    }
}

/// WGSL overlay reference path (GPU pass W1 — constants mirrored in CPU raster).
pub const WATER_SURFACE_OVERLAY_WGSL: &str = "shaders/water/water_surface_overlay.wgsl";

/// Strategic zoom band (D-W09) — keep in sync with [`crate::render::pipelines::gpu_water_particles::WATER_PARTICLE_STRATEGIC_ZOOM_ALPHA`].
pub const WATER_STRATEGIC_ZOOM_ALPHA: f32 = 0.35;

#[inline]
fn water_at_strategic_zoom(zoom_alpha: f32) -> bool {
    zoom_alpha < WATER_STRATEGIC_ZOOM_ALPHA
}

// §6 palette — design plan tokens (W1 CPU mirror of WGSL constants).
const WATER_RIVER_DEEP: [u8; 3] = [30, 69, 68];
const WATER_TEAL_EDGE: [u8; 3] = [74, 120, 120];
const WATER_TEAL: [u8; 3] = [42, 90, 88];
const WATER_OCEAN_DEEP: [u8; 3] = [15, 40, 40];

#[inline]
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t.clamp(0.0, 1.0)).round() as u8
}

#[inline]
fn blend_rgb(base: [u8; 4], tint: [u8; 3], strength: f32) -> [u8; 4] {
    let t = strength.clamp(0.0, 1.0);
    [
        lerp_u8(base[0], tint[0], t),
        lerp_u8(base[1], tint[1], t),
        lerp_u8(base[2], tint[2], t),
        base[3],
    ]
}

/// Apply W1 water motion overlay to an RGBA8 tile subregion (overworld raster path).
pub fn apply_water_surface_overlay_subregion(
    data: &mut [u8],
    tex_w: usize,
    x0: usize,
    y0: usize,
    x1: usize,
    y1: usize,
    catalog: &WaterSurfaceVisualCatalog,
    time_secs: f32,
    zoom_alpha: f32,
) {
    if !catalog.has_motion() {
        return;
    }
    // `flow_at` scans river segments per river pixel; only segments that can reach this
    // subregion matter (order preserved so the first-match flow is identical).
    let margin = 2.0_f32;
    let (rx0, ry0, rx1, ry1) = (
        x0 as f32 - margin,
        y0 as f32 - margin,
        x1 as f32 + margin,
        y1 as f32 + margin,
    );
    let local_segments: Vec<&RiverPolylineSegment> = catalog
        .river_segments
        .iter()
        .filter(|seg| {
            let reach = seg.half_width + 0.6;
            seg.start.x.max(seg.end.x) + reach >= rx0
                && seg.start.x.min(seg.end.x) - reach <= rx1
                && seg.start.y.max(seg.end.y) + reach >= ry0
                && seg.start.y.min(seg.end.y) - reach <= ry1
        })
        .collect();
    for y in y0..y1 {
        for x in x0..x1 {
            let tx = x as u32;
            let ty = y as u32;
            if tx >= catalog.grid_width || ty >= catalog.grid_height {
                continue;
            }
            let i = 4 * (y * tex_w + x);
            if i + 3 >= data.len() {
                continue;
            }
            let base = [data[i], data[i + 1], data[i + 2], data[i + 3]];
            let out = match catalog.tile_kind(tx, ty) {
                WaterSurfaceKind::None => continue,
                WaterSurfaceKind::River => {
                    let flow = flow_from_segments(local_segments.iter().copied(), tx, ty);
                    river_overlay_pixel(base, tx, ty, flow, time_secs, zoom_alpha)
                }
                WaterSurfaceKind::Lake => lake_overlay_pixel(base, tx, ty, time_secs, zoom_alpha),
                WaterSurfaceKind::Ocean => ocean_overlay_pixel(base, tx, ty, time_secs),
            };
            data[i..i + 4].copy_from_slice(&out);
        }
    }
}

impl WaterSurfaceVisualCatalog {
    #[inline]
    fn flow_at(&self, x: u32, y: u32) -> Vec2 {
        flow_from_segments(self.river_segments.iter(), x, y)
    }
}

#[inline]
fn flow_from_segments<'a>(
    segments: impl Iterator<Item = &'a RiverPolylineSegment>,
    x: u32,
    y: u32,
) -> Vec2 {
    let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
    for seg in segments {
        if point_near_segment(p, seg) {
            return seg.flow_dir;
        }
    }
    Vec2::X
}

fn point_near_segment(p: Vec2, seg: &RiverPolylineSegment) -> bool {
    let ab = seg.end - seg.start;
    let len_sq = ab.length_squared();
    if len_sq < 1e-6 {
        return p.distance(seg.start) <= seg.half_width + 0.6;
    }
    let t = ((p - seg.start).dot(ab) / len_sq).clamp(0.0, 1.0);
    let closest = seg.start + ab * t;
    p.distance(closest) <= seg.half_width + 0.55
}

fn river_overlay_pixel(
    base: [u8; 4],
    x: u32,
    y: u32,
    flow: Vec2,
    time: f32,
    zoom_alpha: f32,
) -> [u8; 4] {
    let strategic = water_at_strategic_zoom(zoom_alpha);
    let flow_n = if flow.length_squared() > 1e-6 {
        flow.normalize()
    } else {
        Vec2::X
    };
    let cross = Vec2::new(-flow_n.y, flow_n.x);
    let wx = x as f32;
    let wy = y as f32;
    let along = wx * flow_n.x + wy * flow_n.y;
    let across = wx * cross.x + wy * cross.y;
    let ribbon = (1.0 - ((across % 1.0) - 0.5).abs() * 2.0).clamp(0.0, 1.0);
    let scroll_speed = if strategic { 1.55 } else { 1.0 };
    let scroll = ((along * 0.35 * scroll_speed - time * 0.8 * scroll_speed).sin() * 0.5 + 0.5)
        .clamp(0.0, 1.0);
    let tint = [
        lerp_u8(WATER_RIVER_DEEP[0], WATER_TEAL_EDGE[0], ribbon),
        lerp_u8(WATER_RIVER_DEEP[1], WATER_TEAL_EDGE[1], ribbon),
        lerp_u8(WATER_RIVER_DEEP[2], WATER_TEAL_EDGE[2], ribbon),
    ];
    let mut strength = 0.42 + ribbon * 0.28 + scroll * 0.22;
    if strategic {
        strength += 0.2;
    }
    blend_rgb(base, tint, strength.clamp(0.0, 0.92))
}

fn lake_overlay_pixel(base: [u8; 4], x: u32, y: u32, time: f32, zoom_alpha: f32) -> [u8; 4] {
    let wx = x as f32;
    let wy = y as f32;
    let ripple = ((wx * 0.21 + wy * 0.17 + time * 0.03).sin()
        * (wx * 0.13 - wy * 0.19 + time * 0.025).cos()
        * 0.5
        + 0.5)
        .clamp(0.0, 1.0);
    let mut strength = 0.12 + ripple * 0.18;
    if water_at_strategic_zoom(zoom_alpha) {
        strength *= 0.48;
    }
    blend_rgb(base, WATER_TEAL, strength)
}

fn ocean_overlay_pixel(base: [u8; 4], x: u32, y: u32, time: f32) -> [u8; 4] {
    let wx = x as f32;
    let wy = y as f32;
    let swell = ((wx * 0.08 + time * 0.02).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let haze = ((wy * 0.05 - time * 0.015).cos() * 0.5 + 0.5).clamp(0.0, 1.0);
    blend_rgb(base, WATER_OCEAN_DEEP, (0.25 + swell * 0.2 + haze * 0.12).clamp(0.0, 0.75))
}

#[derive(Resource, Default)]
struct LastWaterCatalogRevision(u64);

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct WaterSurfaceVisualSet;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct WaterOverlayGpuInstance {
    pub world_kind: Vec4,
    pub flow_extent: Vec4,
    pub segment_b: Vec4,
}

impl WaterOverlayGpuInstance {
    fn from_river(seg: &RiverPolylineSegment) -> Self {
        Self {
            world_kind: Vec4::new(
                seg.start.x,
                seg.start.y,
                WaterSurfaceKind::River.as_f32(),
                seg.path_id as f32,
            ),
            flow_extent: Vec4::new(
                seg.flow_dir.x,
                seg.flow_dir.y,
                seg.half_width,
                (seg.end - seg.start).length(),
            ),
            segment_b: Vec4::new(seg.end.x, seg.end.y, 0.0, 0.0),
        }
    }

    fn from_motion(anchor: &WaterMotionAnchor) -> Self {
        Self {
            world_kind: Vec4::new(
                anchor.world.x,
                anchor.world.y,
                anchor.kind.as_f32(),
                0.0,
            ),
            flow_extent: Vec4::new(0.0, 0.0, anchor.extent * 0.5, anchor.extent),
            segment_b: Vec4::ZERO,
        }
    }
}

/// Extracted draw payload for the water overlay pass.
#[derive(Resource, Clone, Debug, Default, ExtractResource)]
#[extract_app(RenderApp)]
pub struct WaterOverlayDrawFrame {
    pub anim_time_secs: f32,
    pub zoom_alpha: f32,
    pub instances: Vec<WaterOverlayGpuInstance>,
}

/// Field flow below this (m/s) is not drawn as an arrow (the field shader already shows the water).
const FIELD_FLOW_ARROW_MIN_MPS: f32 = 0.25;
/// Flow arrows per frame (the visual frame subsamples the map; this bounds the instance buffer).
const FIELD_FLOW_ARROWS_MAX: usize = 1024;

pub fn sync_water_overlay_draw_frame(
    catalog: Option<Res<WaterSurfaceVisualCatalog>>,
    sim_water: Option<Res<crate::systems::water::SurfaceWaterVisualFrame>>,
    gpu_field: Option<Res<crate::systems::water::SurfaceWaterGpuFieldMeta>>,
    wet_mask: Option<Res<crate::systems::water::WaterFieldWetMask>>,
    cam_scale: Option<Res<crate::render::pipelines::gpu_particles::FireParticleCameraScale>>,
    time: Res<Time>,
    mut frame: ResMut<WaterOverlayDrawFrame>,
) {
    // Presentation contract: one fluid -> at most one kind per tile. WM-5: once the map-wide field
    // is seeded it is the water everywhere -- its shader draws depth / flow / foam, so catalog
    // river ribbons (a second flow description) are dropped and motion comes from the field's own
    // flux as short arrows. The catalog keeps only what the field has no data for: everything
    // before the field is seeded, and still-water ripples (lake / ocean anchors) on tiles the
    // field says are wet.
    frame.instances.clear();
    frame.anim_time_secs = time.elapsed_secs();
    frame.zoom_alpha = cam_scale
        .as_deref()
        .map(|c| c.zoom_alpha)
        .unwrap_or(0.5);
    let strategic = water_at_strategic_zoom(frame.zoom_alpha);
    let field_seeded = gpu_field
        .as_deref()
        .is_some_and(|g| g.wired && g.width > 0 && g.height > 0);
    let mask = wet_mask.as_deref().filter(|m| field_seeded && m.is_ready());
    if let Some(catalog) = catalog.as_ref() {
        frame
            .instances
            .reserve(catalog.river_segments.len() + catalog.motion_anchors.len() + 128);
        if !field_seeded {
            for seg in &catalog.river_segments {
                let mut inst = WaterOverlayGpuInstance::from_river(seg);
                if strategic {
                    inst.flow_extent.z *= 1.35;
                }
                frame.instances.push(inst);
            }
        }
        for anchor in &catalog.motion_anchors {
            let field_dry = mask.is_some_and(|m| {
                anchor.world.x < 0.0
                    || anchor.world.y < 0.0
                    || m.level_at(anchor.world.x as u32, anchor.world.y as u32) == 0
            });
            let field_owns_flow = field_seeded && anchor.kind == WaterSurfaceKind::River;
            if field_dry || field_owns_flow {
                continue;
            }
            frame.instances.push(WaterOverlayGpuInstance::from_motion(anchor));
        }
    }
    // Flow arrows straight from the field's flux (sim snapshot `SurfaceWaterVisualFrame`).
    if let Some(sw) = sim_water.as_ref().filter(|_| field_seeded) {
        let mut arrows = 0usize;
        for &(tx, tz, _depth, speed, fdx, fdz) in &sw.samples {
            if speed <= FIELD_FLOW_ARROW_MIN_MPS {
                continue;
            }
            if arrows >= FIELD_FLOW_ARROWS_MAX {
                break;
            }
            arrows += 1;
            let dir = Vec2::new(fdx, fdz).normalize_or_zero();
            let half = (0.35 + speed * 0.15).clamp(0.25, 0.9);
            let centre = Vec2::new(tx, tz);
            frame.instances.push(WaterOverlayGpuInstance::from_river(&RiverPolylineSegment {
                path_id: 0,
                start: centre - dir * half,
                end: centre + dir * half,
                flow_dir: dir,
                half_width: 0.18,
            }));
        }
    }
}

pub struct WaterSurfaceVisualPlugin;

impl Plugin for WaterSurfaceVisualPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WaterSurfaceVisualCatalog>()
            .init_resource::<WaterOverlayDrawFrame>()
            .init_resource::<LastWaterCatalogRevision>()
            .configure_sets(
                Update,
                WaterSurfaceVisualSet.in_set(crate::render::TileWorldFallbackAfterFireExtract),
            )
            .add_plugins(bevy::render::extract_resource::ExtractResourcePlugin::<
                WaterOverlayDrawFrame,
            >::default())
            .add_systems(
                Update,
                (
                    sync_water_overlay_draw_frame,
                    mark_water_surface_motion_dirty,
                )
                    .in_set(WaterSurfaceVisualSet),
            );
    }
}

fn mark_water_surface_motion_dirty(
    catalog: Res<WaterSurfaceVisualCatalog>,
    mut last: ResMut<LastWaterCatalogRevision>,
    mut raster_ctrl: ResMut<crate::render::TileWorldFallbackRasterCtrl>,
) {
    // PLAN-FINAL-GPU-PRESENTATION-v1: CPU water paint into the terrain raster is retired.
    // Water motion is GPU-only (water_field / particles). Only re-dirty on catalog
    // topology stamp changes (land/water class bake), not on a ripple cadence.
    if last.0 != catalog.stamp {
        last.0 = catalog.stamp;
        raster_ctrl.chunk_grid.mark_all_dirty();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::generation::hydrology::{
        compute_hydrology_rect, HydrologyParams, HydrologyResult, LakeRegion,
    };

    #[test]
    fn water_w1_ocean_001_perimeter_shallow_dem_tags_ocean() {
        let w = 32u32;
        let h = 32u32;
        let n = (w * h) as usize;
        let mut filled_dem = vec![0.62f32; n];
        for x in 0..w {
            filled_dem[x as usize] = 0.18;
            filled_dem[((h - 1) * w + x) as usize] = 0.18;
        }
        for y in 0..h {
            filled_dem[(y * w) as usize] = 0.18;
            filled_dem[(y * w + (w - 1)) as usize] = 0.18;
        }
        // Water is decided by the hydrology solve, not by the catalog: the sea-level ring is standing water.
        let lake_mask: Vec<bool> = filled_dem.iter().map(|e| *e <= 0.2).collect();
        let hydro = HydrologyResult {
            rivers: Vec::new(),
            lakes: Vec::new(),
            accumulation: vec![0.0; n],
            river_mask: vec![false; n],
            lake_mask,
            eroded_dem: filled_dem.clone(),
            filled_dem,
        };
        let mut params = WorldGenParams::default();
        params.width = w;
        params.height = h;
        let catalog = WaterSurfaceVisualCatalog::from_hydrology(&hydro, &params);
        assert!(
            catalog.w1_ocean_green(),
            "edge-connected sea-level water should produce ocean_tiles"
        );
        assert!(catalog.ocean_tiles.len() > (w + h) as usize);
    }

    #[test]
    fn water_w1_ocean_001_swell_overlay_animates_with_time() {
        let base = [40u8, 80, 90, 255];
        let a = ocean_overlay_pixel(base, 12, 7, 0.0);
        let b = ocean_overlay_pixel(base, 12, 7, 31.4);
        let delta: u32 = a
            .iter()
            .zip(b.iter())
            .take(3)
            .map(|(x, y)| x.abs_diff(*y) as u32)
            .sum();
        assert!(
            delta > 0,
            "D-W04 ocean swell should animate (rgb delta {delta})"
        );
    }

    #[test]
    fn water_w1_ocean_001_dem_deep_band_fills_ocean_tiles() {
        let w = 8u32;
        let h = 8u32;
        let n = (w * h) as usize;
        // Sea on the left five columns (edge-connected standing water), land on the right.
        let mut lake_mask = vec![false; n];
        for y in 0..h as usize {
            for x in 0..5usize {
                lake_mask[y * w as usize + x] = true;
            }
        }
        let hydro = HydrologyResult {
            rivers: Vec::new(),
            lakes: Vec::new(),
            accumulation: vec![0.0; n],
            river_mask: vec![false; n],
            lake_mask,
            filled_dem: vec![0.05; n],
            eroded_dem: vec![0.05; n],
        };
        let mut params = WorldGenParams::default();
        params.width = w;
        params.height = h;
        let catalog = WaterSurfaceVisualCatalog::from_hydrology(&hydro, &params);
        assert!(
            catalog.w1_ocean_green(),
            "WATER-W1-OCEAN-001: expected ocean_tiles from an all-sea-level map"
        );
        assert!(!catalog.ocean_tiles.is_empty());
    }

    #[test]
    fn border_lake_regions_split_between_ocean_and_lake_tiles() {
        let w = 16u32;
        let h = 16u32;
        let n = (w * h) as usize;
        let mut filled_dem = vec![0.55f32; n];
        for x in 0..w {
            filled_dem[x as usize] = 0.12;
        }
        for &(x, y) in &[(8u32, 8), (9, 8), (8, 9), (9, 9)] {
            filled_dem[y as usize * w as usize + x as usize] = 0.28;
        }
        let mut lake_mask = vec![false; n];
        for x in 0..w {
            lake_mask[x as usize] = true;
        }
        for &(x, y) in &[(8u32, 8), (9, 8), (8, 9), (9, 9)] {
            lake_mask[y as usize * w as usize + x as usize] = true;
        }
        let hydro = HydrologyResult {
            rivers: Vec::new(),
            lakes: vec![
                LakeRegion {
                    cells: (0..w).map(|x| (x, 0)).collect(),
                },
                LakeRegion {
                    cells: vec![(8, 8), (9, 8), (8, 9), (9, 9)],
                },
            ],
            accumulation: vec![0.0; n],
            river_mask: vec![false; n],
            lake_mask,
            eroded_dem: filled_dem.clone(),
            filled_dem,
        };
        let mut params = WorldGenParams::default();
        params.width = w;
        params.height = h;
        let catalog = WaterSurfaceVisualCatalog::from_hydrology(&hydro, &params);
        assert!(catalog.w1_ocean_green());
        assert_eq!(catalog.ocean_tiles.len(), w as usize);
        assert_eq!(catalog.lake_tiles.len(), 4);
    }

    #[test]
    fn catalog_builds_river_segments_from_hydro_paths() {
        let w = 32usize;
        let h = 32usize;
        let mut dem = vec![0.55f32; w * h];
        for x in 4..28 {
            dem[x] = 0.35;
            dem[w + x] = 0.34;
        }
        let p = HydrologyParams::default();
        let hydro = compute_hydrology_rect(w as u32, h as u32, &dem, &p, 2, None);
        let mut params = WorldGenParams::default();
        params.width = w as u32;
        params.height = h as u32;
        let catalog = WaterSurfaceVisualCatalog::from_hydrology(&hydro, &params);
        assert!(
            catalog.w1_green(),
            "expected river segments or tiles, got segments={} tiles={}",
            catalog.river_segments.len(),
            catalog.river_tiles.len()
        );
    }

    #[test]
    fn river_overlay_animates_with_time() {
        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.grid_width = 4;
        catalog.grid_height = 4;
        catalog.river_tiles.insert((1, 1));
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(1.0, 1.0),
            end: Vec2::new(2.0, 1.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });

        let mut buf = vec![0u8; 4 * 4 * 4];
        for px in buf.chunks_mut(4) {
            px.copy_from_slice(&[0, 0, 200, 255]);
        }
        apply_water_surface_overlay_subregion(&mut buf, 4, 0, 0, 4, 4, &catalog, 0.0, 0.85);
        let a = buf[4 * 4 + 4];
        apply_water_surface_overlay_subregion(&mut buf, 4, 0, 0, 4, 4, &catalog, 2.5, 0.85);
        let b = buf[4 * 4 + 4];
        assert_ne!(a, b, "directional scroll should animate pixel tint");
    }

    #[test]
    fn water_surface_overlay_wgsl_exists() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let src =
            std::fs::read_to_string(root.join("assets/shaders/water/water_surface_overlay.wgsl"))
                .expect("water_surface_overlay.wgsl");
        assert!(src.contains("river_flow_scroll"));
        assert!(src.contains("lake_ripple"));
    }

    #[test]
    fn water_overlay_cpu_gpu_palette_parity() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let gpu = std::fs::read_to_string(root.join("assets/shaders/water/water_overlay.wgsl"))
            .expect("water_overlay.wgsl");
        assert!(gpu.contains("0.118, 0.271, 0.267"), "river deep token");
        assert!(gpu.contains("0.165, 0.353, 0.345"), "lake teal token");
        assert_eq!(WATER_RIVER_DEEP, [30, 69, 68]);
        assert_eq!(WATER_TEAL, [42, 90, 88]);
    }

    #[test]
    fn water_overlay_wgsl_w1_contract() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let src = std::fs::read_to_string(root.join("assets/shaders/water/water_overlay.wgsl"))
            .expect("water_overlay.wgsl");
        assert!(src.contains("t * 0.6"), "D-W02 lake ripple uses time_secs");
        assert!(
            src.contains("t * scroll_hz") && src.contains("scroll_hz"),
            "D-W03 river scroll uses time_secs"
        );
        assert!(
            src.contains("globals.zoom_alpha < 0.35"),
            "WATER-W1-RIVER-001 strategic river read branch"
        );
        assert!(src.contains("smoothstep(0.6, 1.0, d)"), "D-W04 ocean haze");
        assert!(
            !src.contains("zoom_alpha * t") && !src.contains("t * zoom_alpha"),
            "motion must not scale with zoom_alpha (D-W09)"
        );
    }

    #[test]
    fn river_reads_distinct_from_lake_at_strategic_zoom() {
        let mut river_catalog = WaterSurfaceVisualCatalog::default();
        river_catalog.grid_width = 8;
        river_catalog.grid_height = 8;
        river_catalog.river_tiles.insert((4, 4));
        river_catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(4.0, 4.0),
            end: Vec2::new(6.0, 4.0),
            flow_dir: Vec2::X,
            half_width: 0.72,
        });

        let mut lake_catalog = WaterSurfaceVisualCatalog::default();
        lake_catalog.grid_width = 8;
        lake_catalog.grid_height = 8;
        lake_catalog.lake_tiles.insert((4, 4));

        assert!(
            river_catalog.w1_river_read_green_at_zoom(0.25),
            "strategic zoom must separate river ribbon from lake teal"
        );
        assert!(
            river_catalog.river_lake_rgb_delta_at_zoom(4, 4, 0.0, 0.25)
                > river_catalog.river_lake_rgb_delta_at_zoom(4, 4, 0.0, 0.85),
            "strategic zoom should increase river vs lake contrast"
        );
    }

    #[test]
    fn river_ribbon_tint_differs_from_lake_teal() {
        let base_blue = [0u8, 0u8, 200u8, 255u8];
        let mut river_catalog = WaterSurfaceVisualCatalog::default();
        river_catalog.grid_width = 8;
        river_catalog.grid_height = 8;
        river_catalog.river_tiles.insert((4, 4));
        river_catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(4.0, 4.0),
            end: Vec2::new(6.0, 4.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });

        let mut lake_catalog = WaterSurfaceVisualCatalog::default();
        lake_catalog.grid_width = 8;
        lake_catalog.grid_height = 8;
        lake_catalog.lake_tiles.insert((4, 4));

        let mut river_buf = vec![0u8; 8 * 8 * 4];
        let mut lake_buf = vec![0u8; 8 * 8 * 4];
        for px in river_buf.chunks_mut(4) {
            px.copy_from_slice(&base_blue);
        }
        for px in lake_buf.chunks_mut(4) {
            px.copy_from_slice(&base_blue);
        }

        apply_water_surface_overlay_subregion(&mut river_buf, 8, 0, 0, 8, 8, &river_catalog, 0.0, 0.85);
        apply_water_surface_overlay_subregion(&mut lake_buf, 8, 0, 0, 8, 8, &lake_catalog, 0.0, 0.85);

        let river_px = &river_buf[4 * (4 * 8 + 4)..4 * (4 * 8 + 4) + 3];
        let lake_px = &lake_buf[4 * (4 * 8 + 4)..4 * (4 * 8 + 4) + 3];
        assert_ne!(
            river_px, lake_px,
            "directional river ribbon must read distinct from lake teal"
        );
    }

    #[test]
    fn overlay_gpu_instances_distinct_river_and_lake_kinds() {
        let river = WaterOverlayGpuInstance::from_river(&RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(1.0, 1.0),
            end: Vec2::new(3.0, 1.0),
            flow_dir: Vec2::X,
            half_width: 0.42,
        });
        let lake = WaterOverlayGpuInstance::from_motion(&WaterMotionAnchor {
            kind: WaterSurfaceKind::Lake,
            world: Vec2::new(8.0, 8.0),
            extent: 1.25,
        });
        assert!(
            (river.world_kind.z - WaterSurfaceKind::River.as_f32()).abs() < 1e-4,
            "river ribbon instance kind"
        );
        assert!(
            (lake.world_kind.z - WaterSurfaceKind::Lake.as_f32()).abs() < 1e-4,
            "lake motion anchor kind"
        );
        assert_ne!(river.world_kind.z, lake.world_kind.z, "rivers ≠ lakes on GPU path");
    }

    fn overlay_app(seeded: bool) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<Time>();
        app.init_resource::<WaterOverlayDrawFrame>();
        let mut catalog = WaterSurfaceVisualCatalog::default();
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 0,
            start: Vec2::new(2.0, 2.0),
            end: Vec2::new(4.0, 2.0),
            flow_dir: Vec2::X,
            half_width: 0.4,
        });
        catalog.river_segments.push(RiverPolylineSegment {
            path_id: 1,
            start: Vec2::new(90.0, 90.0),
            end: Vec2::new(92.0, 90.0),
            flow_dir: Vec2::X,
            half_width: 0.4,
        });
        app.insert_resource(catalog);
        app.insert_resource(crate::systems::water::SurfaceWaterGpuFieldMeta {
            wired: seeded,
            width: if seeded { 320 } else { 0 },
            height: if seeded { 320 } else { 0 },
            step: 1,
            ..Default::default()
        });
        app.add_systems(Update, sync_water_overlay_draw_frame);
        app
    }

    /// WM-5: before the field is seeded the catalog ribbons draw everywhere.
    #[test]
    fn wm5_catalog_ribbons_draw_until_the_field_is_seeded() {
        let mut app = overlay_app(false);
        app.update();
        assert_eq!(app.world().resource::<WaterOverlayDrawFrame>().instances.len(), 2);
    }

    /// WM-5: once the map-wide field is seeded, motion comes from the field's flux (arrows), the
    /// catalog river ribbons no longer dual-draw, and slow water gets no arrow.
    #[test]
    fn wm5_seeded_field_replaces_catalog_ribbons_with_field_flow_arrows() {
        let mut app = overlay_app(true);
        app.insert_resource(crate::systems::water::SurfaceWaterVisualFrame {
            samples: vec![
                (200.5, 150.5, 2.0, 1.5, 0.0, 1.0), // fast river cell far from any ROI
                (10.5, 10.5, 2.0, 0.05, 1.0, 0.0),  // slow
            ],
            ..Default::default()
        });
        app.update();
        let frame = app.world().resource::<WaterOverlayDrawFrame>();
        assert_eq!(frame.instances.len(), 1, "one field-flow arrow, no catalog ribbons");
        let i = &frame.instances[0];
        assert!((i.world_kind.x - (200.5 - 0.0)).abs() < 1.0 && i.world_kind.y < 150.5, "arrow centred on the sample");
        assert!(i.flow_extent.y > 0.9, "arrow points along the field flow (+z)");
    }
}
