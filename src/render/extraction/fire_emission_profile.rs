//! Data-driven **fire emission inference** — single bridge for lights, smoke, particles, hazards (`base_fire2_smoke.md`).
//!
//! Derived from existing sim authority ([`ChunkSurfaceFire`](crate::systems::fire::ChunkSurfaceFire),
//! [`ChunkEcology`](crate::systems::ecology::ChunkEcology), fuel/material rows). **No** per-scenario fire ECS types;
//! extend profiles as fuel / structure data grows.
//!
//! Populated only from [`super::fire_visual_extract::extract_fire_visual_frame`] (one ECS pass per frame).

use bevy::math::IVec2;
use bevy::prelude::*;

use crate::systems::ecology::ChunkEcology;
use crate::systems::fire::{ChunkFuelProfile, ChunkSmokeField, ChunkSurfaceFire, FireLightEmission};
use crate::systems::weather::ChunkWeather;
use crate::terrain::family::{TerrainFamilyId, DEFAULT_TERRAIN_FAMILY_ID};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::{MaterialId, MaterializedChunk};

use crate::render::sim_visual_extract::FireVisualGpuInstance;

/// High-level combustion category for VFX / gameplay hints (not a separate sim system).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CombustionClass {
    #[default]
    Vegetation,
    Hydrocarbon,
    Electrical,
    Chemical,
    Structural,
}

/// Master **visual** row for one burning chunk this frame (rewritten each extract tick; not sim state).
#[derive(Clone, Copy, Debug)]
pub struct FireEmissionProfile {
    pub chunk_coord: IVec2,
    pub world_pos: Vec3,
    pub heat: f32,
    pub luminosity: f32,
    pub smoke_density: f32,
    pub smoke_color: Vec3,
    pub fog_tint: Vec3,
    pub ember_rate: f32,
    pub toxic_density: f32,
    pub visibility_reduction: f32,
    pub combustion_class: CombustionClass,
    /// Light clustering / budgeting (from sim [`FireLightEmission::extract_priority`](crate::systems::fire::FireLightEmission)).
    pub extract_priority: f32,
    /// Physical influence radius for pooled lights (from sim emission).
    pub influence_radius: f32,
}

/// Canonical **render-facing** fire row (`P1-E`): packed GPU/storage layout in
/// [`crate::render::sim_visual_extract::FireVisualFrame::instances`] (one ECS extract pass per frame).
pub type FireVisualProxy = FireVisualGpuInstance;

#[inline]
pub(crate) fn combustion_class_storage_ord(c: CombustionClass) -> f32 {
    match c {
        CombustionClass::Vegetation => 0.0,
        CombustionClass::Hydrocarbon => 1.0,
        CombustionClass::Chemical => 2.0,
        CombustionClass::Electrical => 3.0,
        CombustionClass::Structural => 4.0,
    }
}

impl From<&FireEmissionProfile> for FireVisualGpuInstance {
    fn from(p: &FireEmissionProfile) -> Self {
        Self {
            chunk_xy_heat_lum: Vec4::new(
                p.chunk_coord.x as f32,
                p.chunk_coord.y as f32,
                p.heat,
                p.luminosity,
            ),
            world_xyz_radius: Vec4::new(p.world_pos.x, p.world_pos.y, p.world_pos.z, p.influence_radius),
            smoke_ember_vis_priority: Vec4::new(
                p.smoke_density,
                p.ember_rate,
                p.visibility_reduction,
                p.extract_priority,
            ),
            smoke_color_toxic: Vec4::new(
                p.smoke_color.x,
                p.smoke_color.y,
                p.smoke_color.z,
                p.toxic_density,
            ),
            fog_rgb_combust_ord: Vec4::new(
                p.fog_tint.x,
                p.fog_tint.y,
                p.fog_tint.z,
                combustion_class_storage_ord(p.combustion_class),
            ),
        }
    }
}

#[inline]
fn center_cell_index(matrix: &ChunkCellMatrix) -> usize {
    let cx = matrix.size.x.saturating_sub(1) / 2;
    let cy = matrix.size.y.saturating_sub(1) / 2;
    matrix.idx(cx, cy)
}

/// Terrain family id at chunk grid center (opaque registry id; heuristics prioritize fuel overlay).
#[inline]
pub fn terrain_family_at_chunk_center(matrix: &ChunkCellMatrix) -> TerrainFamilyId {
    let i = center_cell_index(matrix);
    *matrix.family.get(i).unwrap_or(&DEFAULT_TERRAIN_FAMILY_ID)
}

/// Resolved [`MaterialId`] at chunk center when pass-6 [`MaterializedChunk`] exists.
#[inline]
pub fn material_id_at_chunk_center(mat: &MaterializedChunk) -> MaterialId {
    let cx = mat.size.x.saturating_sub(1) / 2;
    let cy = mat.size.y.saturating_sub(1) / 2;
    let idx = (cy * mat.size.x + cx) as usize;
    *mat.materials.get(idx).unwrap_or(&MaterialId(0))
}

/// Classify combustion from terrain/material/ecology/fuel **without** new fire ECS types.
///
/// `_terrain` / `_material` / `_eco` are reserved for registry-driven rules once family/material defs expose hazard tags.
pub fn infer_combustion_class(
    _terrain: TerrainFamilyId,
    _material: MaterialId,
    _eco: Option<&ChunkEcology>,
    prof: Option<&ChunkFuelProfile>,
) -> CombustionClass {
    let Some(p) = prof else {
        return CombustionClass::Vegetation;
    };
    if p.structure_overlay.is_none() {
        return CombustionClass::Vegetation;
    }
    let f = p.to_fuel_layer();
    if f.toxic_smoke > 0.88 && f.volatility > 0.82 {
        return CombustionClass::Chemical;
    }
    if f.volatility > 0.88 && f.toxic_smoke > 0.65 && f.ember_generation < 0.4 {
        return CombustionClass::Electrical;
    }
    if f.volatility > 0.82 && f.surface_fuel > 0.45 {
        return CombustionClass::Hydrocarbon;
    }
    CombustionClass::Structural
}

/// Build one per-chunk profile for [`crate::render::sim_visual_extract::FireVisualFrame`].
pub fn infer_fire_emission_profile(
    chunk: &Chunk,
    fire: &ChunkSurfaceFire,
    em: &FireLightEmission,
    smoke: Option<&ChunkSmokeField>,
    eco: Option<&ChunkEcology>,
    prof: Option<&ChunkFuelProfile>,
    wx: Option<&ChunkWeather>,
    matrix: &ChunkCellMatrix,
    materialized: Option<&MaterializedChunk>,
) -> FireEmissionProfile {
    let sx = matrix.size.x as f32;
    let sy = matrix.size.y as f32;
    let ox = chunk.coord.x as f32 * sx;
    let oy = chunk.coord.y as f32 * sy;
    let world_pos = Vec3::new(ox + sx * 0.5, oy + sy * 0.5, 8.0);

    let terrain = terrain_family_at_chunk_center(matrix);
    let material = materialized
        .map(material_id_at_chunk_center)
        .unwrap_or(MaterialId(0));
    let class = infer_combustion_class(terrain, material, eco, prof);
    let layer = prof.map(|p| p.to_fuel_layer()).unwrap_or_default();
    let heat = fire.heat.clamp(0.0, 1.0);
    let bio = eco.map(|e| e.biomass).unwrap_or(0.35);

    let smoke_field = smoke.map(|s| s.density).unwrap_or(0.0);
    let toxic = smoke
        .map(|s| s.toxicity)
        .unwrap_or(0.0)
        .max(layer.toxic_smoke * heat);

    let ember_rate = (layer.ember_generation * heat * (0.5 + bio * 0.6)).clamp(0.0, 1.5);
    let explosion_risk =
        (layer.volatility * (1.0 - layer.moisture.clamp(0.0, 0.95)) * heat).clamp(0.0, 1.0);
    let electrical_arcing = match class {
        CombustionClass::Electrical => wx
            .map(|w| w.lightning_risk * 0.35 + heat * 0.65)
            .unwrap_or(heat * 0.7),
        _ => 0.0,
    };

    let smoke_density = (smoke_field + heat * layer.toxic_smoke * 0.55).clamp(0.0, 1.0);
    let smoke_color = smoke_color_for_class(class, toxic, heat);
    let luminosity_mul = luminosity_curve(class, bio);
    let luminosity = (em.current_intensity * luminosity_mul).max(0.0);

    let fog_mix = (smoke_density * 0.65).min(1.0);
    let neutral_fog = Vec3::new(0.82, 0.84, 0.88);
    let fog_tint = neutral_fog.lerp(smoke_color, fog_mix);

    let visibility_penalty = smoke.map(|s| s.visibility_penalty).unwrap_or(0.0);
    let visibility_reduction = (visibility_penalty * 0.5
        + smoke_density * 0.35
        + explosion_risk * 0.15
        + electrical_arcing * 0.05)
        .clamp(0.0, 1.0);

    FireEmissionProfile {
        chunk_coord: chunk.coord,
        world_pos,
        heat,
        luminosity,
        smoke_density,
        smoke_color,
        fog_tint,
        ember_rate,
        toxic_density: toxic.clamp(0.0, 1.0),
        visibility_reduction,
        combustion_class: class,
        extract_priority: em.extract_priority,
        influence_radius: em.radius,
    }
}

#[inline]
fn luminosity_curve(class: CombustionClass, bio: f32) -> f32 {
    match class {
        CombustionClass::Hydrocarbon => 1.15 + bio * 0.1,
        CombustionClass::Electrical => 1.05,
        CombustionClass::Chemical => 0.95,
        CombustionClass::Structural => 1.02 + bio * 0.05,
        CombustionClass::Vegetation => 0.92 + bio * 0.25,
    }
}

fn smoke_color_for_class(class: CombustionClass, toxic: f32, heat: f32) -> Vec3 {
    let base = match class {
        CombustionClass::Vegetation => Vec3::new(0.25, 0.22, 0.18),
        CombustionClass::Hydrocarbon => Vec3::new(0.08, 0.07, 0.06),
        CombustionClass::Electrical => Vec3::new(0.85, 0.88, 0.95),
        CombustionClass::Chemical => Vec3::new(0.15, 0.55, 0.18),
        CombustionClass::Structural => Vec3::new(0.35, 0.28, 0.22),
    };
    let tox = toxic.clamp(0.0, 1.0);
    base.lerp(Vec3::new(0.45, 0.05, 0.08), tox * 0.35) * (0.65 + heat * 0.45)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::fire::ChunkSurfaceFire;
    use crate::terrain::generation::Chunk;
    use bevy::prelude::{IVec2, UVec2};

    fn minimal_matrix() -> ChunkCellMatrix {
        ChunkCellMatrix::new(UVec2::new(2, 2))
    }

    #[test]
    fn wildland_without_overlay_is_vegetation() {
        let chunk = Chunk {
            coord: IVec2::ZERO,
        };
        let fire = ChunkSurfaceFire {
            heat: 0.5,
            fuel: 1.0,
        };
        let em = FireLightEmission {
            radius: 100.0,
            base_intensity: 1.0,
            current_intensity: 1.0,
            flicker_strength: 0.1,
            flicker_phase: 0.0,
            extract_priority: 1.0,
        };
        let m = minimal_matrix();
        let p = infer_fire_emission_profile(&chunk, &fire, &em, None, None, None, None, &m, None);
        assert_eq!(p.combustion_class, CombustionClass::Vegetation);
        assert_eq!(p.chunk_coord, IVec2::ZERO);
        assert!(p.luminosity > 0.0);
        assert!(p.world_pos.z > 0.0);
    }
}
