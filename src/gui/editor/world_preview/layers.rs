//! Layer / overlay selection for world raster preview (bitflags — one implicit base + optional overlays).

use super::color_presets::{height_to_color, moisture_to_color, temperature_to_color};
use super::overlays::{blend_overlay, movement_hint_rgba, slope_grade_to_color, voronoi_region_preview_rgba};
use super::tile_sampling::{
    cell_tags_for_world_tile, chunk_cell_layer_at_world_tile, chunk_cell_key_for_world_tile,
    slope_grade_for_world_tile, slope_grade_from_world_elevation_neighbors,
};
use crate::terrain::family::TerrainFamilyId;
use crate::terrain::generation::passes::threshold_tags_for_scalars;
use crate::terrain::generation::world_generator_enhanced::{
    MacroRegionRaster, TileRegionIndex, WorldGenParams,
};
use crate::terrain::material::{MaterialId, MaterialRegistry, TagRegistry, TagSet};
use crate::terrain::mobility::{evaluate_tile, MobilityProfileRegistry};
use crate::terrain::{DynamicTerrainOverlay, TerrainFamilyRegistry};

use bevy::math::{IVec2, UVec2};
use std::collections::HashMap;

bitflags::bitflags! {
    /// Composable preview: at most one **base** tint (priority: Regions > Biome > Height > Moisture > Temperature)
    /// plus optional overlays (tag / slope / mobility).
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
    pub struct PreviewLayers: u64 {
        const HEIGHT = 1 << 0;
        const MOISTURE = 1 << 1;
        const TEMPERATURE = 1 << 2;
        const BIOME = 1 << 3;
        const REGIONS = 1 << 4;
        const TAG_OVERLAY = 1 << 5;
        const DERIVED_SLOPE_OVERLAY = 1 << 6;
        const MOBILITY_OVERLAY = 1 << 7;
    }
}

impl PreviewLayers {
    pub const ZOOM_MIN: f32 = 0.02;
    pub const ZOOM_MAX: f32 = 32.0;

    const BASE_MASK: PreviewLayers = PreviewLayers::HEIGHT
        .union(PreviewLayers::MOISTURE)
        .union(PreviewLayers::TEMPERATURE)
        .union(PreviewLayers::BIOME)
        .union(PreviewLayers::REGIONS);

    /// Bits selected for the mutual-exclusion **base** raster.
    #[inline]
    pub fn base_bits(self) -> PreviewLayers {
        self & Self::BASE_MASK
    }

    /// Replace mutual-exclusion base bits (one logical base layer in the UI).
    pub fn replace_base(&mut self, base: PreviewLayers) {
        *self = (*self & !Self::BASE_MASK) | (base & Self::BASE_MASK);
    }

    fn base_rgba_for_tile(
        self,
        tx: u32,
        ty: u32,
        width: u32,
        height: u32,
        tile_height: f32,
        moisture: f32,
        temperature: f32,
        terrain_family: TerrainFamilyId,
        region_ix: Option<TileRegionIndex>,
        macro_raster: Option<&MacroRegionRaster>,
        elev_slices: &[(IVec2, UVec2, &[f32])],
        moist_slices: &[(IVec2, UVec2, &[f32])],
        temp_slices: &[(IVec2, UVec2, &[f32])],
        family_slices: &[(IVec2, UVec2, &[TerrainFamilyId])],
        mat_slices: &[(IVec2, UVec2, &[MaterialId])],
        reg_opt: Option<&MaterialRegistry>,
        fam_opt: Option<&TerrainFamilyRegistry>,
    ) -> [u8; 4] {
        use super::color_presets::{preview_biome_rgba_for_tile, terrain_family_preview_rgba};

        let base_bits = self & Self::BASE_MASK;
        if base_bits.is_empty() {
            return [0, 0, 0, 0];
        }

        if base_bits.contains(PreviewLayers::REGIONS) {
            let from_raster = macro_raster
                .filter(|r| r.width == width && r.height == height)
                .and_then(|r| r.region_at(tx, ty));
            let ri = from_raster
                .or_else(|| region_ix.map(|r| r.0))
                .unwrap_or(0);
            return voronoi_region_preview_rgba(ri);
        }
        if base_bits.contains(PreviewLayers::BIOME) {
            let terrain_family = chunk_cell_layer_at_world_tile(tx, ty, family_slices)
                .unwrap_or(terrain_family);
            return match reg_opt {
                Some(reg) => preview_biome_rgba_for_tile(
                    tx,
                    ty,
                    terrain_family,
                    mat_slices,
                    reg,
                    fam_opt,
                ),
                None => terrain_family_preview_rgba(fam_opt, terrain_family),
            };
        }
        if base_bits.contains(PreviewLayers::HEIGHT) {
            let h = chunk_cell_layer_at_world_tile(tx, ty, elev_slices).unwrap_or(tile_height);
            return height_to_color(h);
        }
        if base_bits.contains(PreviewLayers::MOISTURE) {
            let m = chunk_cell_layer_at_world_tile(tx, ty, moist_slices).unwrap_or(moisture);
            return moisture_to_color(m);
        }
        if base_bits.contains(PreviewLayers::TEMPERATURE) {
            let t = chunk_cell_layer_at_world_tile(tx, ty, temp_slices).unwrap_or(temperature);
            return temperature_to_color(t);
        }
        [0, 0, 0, 0]
    }

    /// Full compositing for one world tile (CPU raster preview).
    #[allow(clippy::too_many_arguments)]
    pub fn composite_rgba_for_tile(
        self,
        tx: u32,
        ty: u32,
        width: u32,
        height: u32,
        tile_height: f32,
        moisture: f32,
        temperature: f32,
        terrain_family: TerrainFamilyId,
        region_ix: Option<TileRegionIndex>,
        world_gen_params: &WorldGenParams,
        macro_raster: Option<&MacroRegionRaster>,
        elev_slices: &[(IVec2, UVec2, &[f32])],
        moist_slices: &[(IVec2, UVec2, &[f32])],
        temp_slices: &[(IVec2, UVec2, &[f32])],
        family_slices: &[(IVec2, UVec2, &[TerrainFamilyId])],
        mat_slices: &[(IVec2, UVec2, &[MaterialId])],
        reg_opt: Option<&MaterialRegistry>,
        fam_opt: Option<&TerrainFamilyRegistry>,
        tag_reg_opt: Option<&TagRegistry>,
        tag_slices: &[(IVec2, UVec2, &[TagSet])],
        tile_heights_ref: Option<&HashMap<(u32, u32), f32>>,
        slope_slices: &[(IVec2, UVec2, &[f32])],
        chunk_geom: &[(IVec2, UVec2)],
        overlay: &DynamicTerrainOverlay,
        mob_reg_opt: Option<&MobilityProfileRegistry>,
        mobility_profile_index: usize,
    ) -> [u8; 4] {
        use super::color_presets::material_traction_mod_for_world_tile;
        use super::overlays::tag_overlay_rgba_pool;

        let mut rgba = self.base_rgba_for_tile(
            tx,
            ty,
            width,
            height,
            tile_height,
            moisture,
            temperature,
            terrain_family,
            region_ix,
            macro_raster,
            elev_slices,
            moist_slices,
            temp_slices,
            family_slices,
            mat_slices,
            reg_opt,
            fam_opt,
        );

        if self.contains(PreviewLayers::TAG_OVERLAY) {
            let h = chunk_cell_layer_at_world_tile(tx, ty, elev_slices).unwrap_or(tile_height);
            let m = chunk_cell_layer_at_world_tile(tx, ty, moist_slices).unwrap_or(moisture);
            let t = chunk_cell_layer_at_world_tile(tx, ty, temp_slices).unwrap_or(temperature);
            let mut ts = cell_tags_for_world_tile(tx, ty, tag_slices).unwrap_or_default();
            if ts == TagSet::default() {
                if let Some(reg) = tag_reg_opt {
                    ts = threshold_tags_for_scalars(
                        h,
                        m,
                        t,
                        &world_gen_params.biome_tuning,
                        reg,
                        &world_gen_params.tag_pool,
                    );
                }
            }
            let tag_c = tag_overlay_rgba_pool(&ts, &world_gen_params.tag_pool);
            rgba = blend_overlay(rgba, tag_c, 0.95);
        }

        if self.contains(PreviewLayers::DERIVED_SLOPE_OVERLAY) {
            let h0 = chunk_cell_layer_at_world_tile(tx, ty, elev_slices).unwrap_or(tile_height);
            let s = slope_grade_for_world_tile(tx, ty, slope_slices).unwrap_or_else(|| {
                slope_grade_from_world_elevation_neighbors(
                    tx,
                    ty,
                    width,
                    height,
                    h0,
                    elev_slices,
                    tile_heights_ref,
                )
            });
            rgba = blend_overlay(rgba, slope_grade_to_color(s), 0.72);
        }

        if self.contains(PreviewLayers::MOBILITY_OVERLAY) {
            if let (Some(tag_reg), Some(mob_reg)) = (tag_reg_opt, mob_reg_opt) {
                if !mob_reg.profiles.is_empty() {
                    let pi = mobility_profile_index.min(mob_reg.profiles.len() - 1);
                    let profile = &mob_reg.profiles[pi];
                    let h0 = chunk_cell_layer_at_world_tile(tx, ty, elev_slices)
                        .unwrap_or(tile_height);
                    let slope = slope_grade_for_world_tile(tx, ty, slope_slices).unwrap_or_else(|| {
                        slope_grade_from_world_elevation_neighbors(
                            tx,
                            ty,
                            width,
                            height,
                            h0,
                            elev_slices,
                            tile_heights_ref,
                        )
                    });
                    let tags = cell_tags_for_world_tile(tx, ty, tag_slices).unwrap_or_default();
                    let fam = chunk_cell_layer_at_world_tile(tx, ty, family_slices)
                        .unwrap_or(terrain_family);
                    let mud_boost = chunk_cell_key_for_world_tile(tx, ty, chunk_geom)
                        .and_then(|k| overlay.mud.get(&k).copied())
                        .filter(|&m| m > 1e-6)
                        .map(|mud| 1.0 + mud * 0.25)
                        .unwrap_or(1.0);
                    let traction = reg_opt
                        .map(|r| {
                            material_traction_mod_for_world_tile(
                                tx,
                                ty,
                                fam,
                                mat_slices,
                                r,
                            ) * mud_boost
                        })
                        .unwrap_or(mud_boost);
                    let hint = evaluate_tile(profile, &tags, slope, 1.0, tag_reg, traction);
                    let mob_c = movement_hint_rgba(&hint);
                    rgba = if rgba[3] == 0 {
                        mob_c
                    } else {
                        blend_overlay(rgba, mob_c, 0.88)
                    };
                }
            }
        }

        rgba
    }
}
