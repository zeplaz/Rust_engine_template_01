//! Weighted footprint raster — single authoritative path for preview + commit.
//!
//! See `src/dev/plan_construction_parametric_placement_v1.md` § Weighted footprint raster.

use std::collections::HashMap;

use bevy::prelude::{IVec2, Vec2};

use crate::construction::building_catalog::FootprintMatrix;
use crate::strategic::{BuildSiteTile, FootprintTiles};

pub const WEIGHT_OMIT_THRESHOLD: f32 = 0.01;
pub const OVERLAP_EPSILON: f32 = 0.001;
const SUBCELL_SAMPLES: u32 = 4;

/// Authoritative parametric placement — committed with site.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacementParams {
    pub scale_factor: f32,
    pub effective_scale: f32,
    pub rotation_quarter_turns: u8,
    pub mirror_x: bool,
}

impl PlacementParams {
    #[must_use]
    pub fn identity(scale_factor: f32) -> Self {
        Self {
            scale_factor,
            effective_scale: scale_factor,
            rotation_quarter_turns: 0,
            mirror_x: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WeightedFootprint {
    pub origin: BuildSiteTile,
    pub weights: Vec<(IVec2, f32)>,
    pub bounds: FootprintTiles,
}

impl WeightedFootprint {
    #[must_use]
    pub fn occupied_mass(&self) -> f32 {
        self.weights.iter().map(|(_, w)| *w).sum()
    }

    #[must_use]
    pub fn weight_at(&self, tile: IVec2) -> f32 {
        self.weights
            .iter()
            .find(|(t, _)| *t == tile)
            .map(|(_, w)| *w)
            .unwrap_or(0.0)
    }
}

/// Sparse tile weights for overlap validation (committed + preview scratch).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TileWeightMap {
    weights: HashMap<IVec2, f32>,
}

impl TileWeightMap {
    #[must_use]
    pub fn weight_at(&self, tile: IVec2) -> f32 {
        self.weights.get(&tile).copied().unwrap_or(0.0)
    }

    pub fn insert_footprint(&mut self, fp: &WeightedFootprint) {
        for (tile, w) in &fp.weights {
            *self.weights.entry(*tile).or_insert(0.0) += w;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeightedOverlapError {
    pub tile: IVec2,
    pub existing: f32,
    pub incoming: f32,
}

/// Reject when Σ existing + new > 1.0 (+ ε).
pub fn validate_weighted_overlap(
    existing: &TileWeightMap,
    new: &WeightedFootprint,
) -> Result<(), WeightedOverlapError> {
    for (tile, w_new) in &new.weights {
        let w_exist = existing.weight_at(*tile);
        if w_exist + w_new > 1.0 + OVERLAP_EPSILON {
            return Err(WeightedOverlapError {
                tile: *tile,
                existing: w_exist,
                incoming: *w_new,
            });
        }
    }
    Ok(())
}

#[must_use]
pub fn derive_effective_scale(base: &FootprintMatrix, weighted: &WeightedFootprint) -> f32 {
    let base_count = base.occupied_local_offsets().count() as f32;
    if base_count <= 0.0 {
        return 1.0;
    }
    weighted.occupied_mass() / base_count
}

#[must_use]
pub fn rasterize_weighted_footprint(
    base: &FootprintMatrix,
    params: &PlacementParams,
    origin: BuildSiteTile,
) -> WeightedFootprint {
    let (min_x, min_z, max_x, max_z) = compute_world_bounds(base, params);
    let ox = origin.x as i32;
    let oz = origin.z as i32;

    let mut weights = Vec::new();
    for tx in (ox + min_x)..=(ox + max_x) {
        for tz in (oz + min_z)..=(oz + max_z) {
            let w = sample_tile_weight(tx, tz, origin, base, params);
            if w >= WEIGHT_OMIT_THRESHOLD {
                weights.push((IVec2::new(tx, tz), w));
            }
        }
    }

    let rel_w = (max_x - min_x + 1).max(0) as u32;
    let rel_d = (max_z - min_z + 1).max(0) as u32;
    WeightedFootprint {
        origin,
        weights,
        bounds: FootprintTiles {
            width: rel_w.max(1),
            depth: rel_d.max(1),
        },
    }
}

#[must_use]
pub fn rasterize_with_effective_scale(
    base: &FootprintMatrix,
    mut params: PlacementParams,
    origin: BuildSiteTile,
) -> (PlacementParams, WeightedFootprint) {
    let weighted = rasterize_weighted_footprint(base, &params, origin);
    params.effective_scale = derive_effective_scale(base, &weighted);
    (params, weighted)
}

fn base_centroid(base: &FootprintMatrix) -> Vec2 {
    Vec2::new(base.width as f32 * 0.5, base.depth as f32 * 0.5)
}

fn rotate_about_centroid(p: Vec2, c: Vec2, quarter_turns: u8) -> Vec2 {
    let t = quarter_turns % 4;
    if t == 0 {
        return p;
    }
    let mut v = p - c;
    for _ in 0..t {
        v = Vec2::new(-v.y, v.x);
    }
    v + c
}

fn inverse_rotate_about_centroid(p: Vec2, c: Vec2, quarter_turns: u8) -> Vec2 {
    let t = (4 - (quarter_turns % 4)) % 4;
    rotate_about_centroid(p, c, t)
}

fn mirror_x_about_centroid(p: Vec2, c: Vec2) -> Vec2 {
    Vec2::new(2.0 * c.x - p.x, p.y)
}

fn forward_transform_local_to_world_rel(
    local: Vec2,
    params: &PlacementParams,
    base: &FootprintMatrix,
) -> Vec2 {
    let c = base_centroid(base);
    let scale = params.scale_factor;
    let mut p = c + scale * (local - c);
    if params.mirror_x {
        p = mirror_x_about_centroid(p, c);
    }
    rotate_about_centroid(p, c, params.rotation_quarter_turns)
}

fn inverse_transform_to_base_local(
    world_rel: Vec2,
    params: &PlacementParams,
    base: &FootprintMatrix,
) -> Vec2 {
    let c = base_centroid(base);
    let mut p = world_rel;
    p = inverse_rotate_about_centroid(p, c, params.rotation_quarter_turns);
    if params.mirror_x {
        p = mirror_x_about_centroid(p, c);
    }
    let scale = params.scale_factor.max(f32::EPSILON);
    c + (p - c) / scale
}

fn point_in_occupied_base(base: &FootprintMatrix, local: Vec2) -> bool {
    if local.x < 0.0 || local.y < 0.0 {
        return false;
    }
    if local.x >= base.width as f32 || local.y >= base.depth as f32 {
        return false;
    }
    let dx = local.x.floor() as u32;
    let dz = local.y.floor() as u32;
    base.is_occupied_local(dx, dz)
}

fn sample_tile_weight(
    tile_x: i32,
    tile_z: i32,
    origin: BuildSiteTile,
    base: &FootprintMatrix,
    params: &PlacementParams,
) -> f32 {
    let ox = origin.x as f32;
    let oz = origin.z as f32;
    let mut inside = 0u32;
    let n = SUBCELL_SAMPLES;
    for i in 0..n {
        for j in 0..n {
            let wx = tile_x as f32 + (i as f32 + 0.5) / n as f32;
            let wz = tile_z as f32 + (j as f32 + 0.5) / n as f32;
            let rel = Vec2::new(wx - ox, wz - oz);
            let local = inverse_transform_to_base_local(rel, params, base);
            if point_in_occupied_base(base, local) {
                inside += 1;
            }
        }
    }
    (inside as f32 / (n * n) as f32).clamp(0.0, 1.0)
}

fn compute_world_bounds(base: &FootprintMatrix, params: &PlacementParams) -> (i32, i32, i32, i32) {
    let mut min_x = i32::MAX;
    let mut min_z = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_z = i32::MIN;

    for (dx, dz) in base.occupied_local_offsets() {
        for lx in [dx as f32, dx as f32 + 1.0] {
            for lz in [dz as f32, dz as f32 + 1.0] {
                let p = forward_transform_local_to_world_rel(Vec2::new(lx, lz), params, base);
                min_x = min_x.min(p.x.floor() as i32);
                min_z = min_z.min(p.y.floor() as i32);
                max_x = max_x.max(p.x.ceil() as i32);
                max_z = max_z.max(p.y.ceil() as i32);
            }
        }
    }

    if min_x == i32::MAX {
        return (0, 0, 0, 0);
    }
    (min_x, min_z, max_x, max_z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_footprint_full_mass_at_unity_scale() {
        let base = FootprintMatrix::from_size(2, 2, true);
        let origin = BuildSiteTile { x: 1, z: 1 };
        let (_, fp) = rasterize_with_effective_scale(&base, PlacementParams::identity(1.0), origin);
        assert!(
            (fp.occupied_mass() - 4.0).abs() < 0.05,
            "mass={}",
            fp.occupied_mass()
        );
    }
}
