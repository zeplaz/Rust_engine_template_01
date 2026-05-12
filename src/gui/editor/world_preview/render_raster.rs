//! Full-world CPU raster into [`super::texture_cache::WorldPreviewTexture`], with chunk-diff partial updates.

use super::ecology_preview::EcologyRasterChunkRow;
use super::layers::PreviewLayers;
use super::texture_cache::WorldPreviewTexture;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::systems::ecology::{ChunkEcology, VegetationField};
use crate::systems::fire::{ChunkSmokeField, ChunkSurfaceFire, FireFuelField};
use crate::systems::weather::ChunkWeather;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::world_generator_enhanced::{
    Height, MacroRegionRaster, Moisture, Temperature, TerrainType, TileMarker, TileRegionIndex,
    WorldGenParams,
};
use crate::terrain::generation::{Chunk, ChunkCellMatrix, ChunkDerivedMetrics};
use crate::terrain::material::{
    MaterialId, MaterialRegistry, MaterializedChunk, TagRegistry, TagSet, WorldPreviewState,
};
use crate::terrain::mobility::MobilityProfileRegistry;
use crate::terrain::DynamicTerrainOverlay;

use bevy::ecs::system::SystemParam;
use bevy::math::{IVec2, UVec2};
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(SystemParam)]
pub(crate) struct WorldPreviewTileChunkQueries<'w, 's> {
    tile_query: Query<
        'w,
        's,
        (
            &'static Transform,
            &'static Height,
            &'static Moisture,
            &'static Temperature,
            &'static TerrainType,
            Option<&'static TileRegionIndex>,
        ),
        With<TileMarker>,
    >,
    chunk_mats: Query<'w, 's, (&'static Chunk, &'static MaterializedChunk)>,
    chunk_cells: Query<'w, 's, (&'static Chunk, &'static ChunkCellMatrix)>,
    chunk_derived: Query<'w, 's, (&'static Chunk, &'static ChunkDerivedMetrics)>,
    chunk_ecology_bundle: Query<
        'w,
        's,
        (
            &'static Chunk,
            &'static ChunkCellMatrix,
            Option<&'static ChunkEcology>,
            Option<&'static VegetationField>,
            Option<&'static ChunkWeather>,
            Option<&'static FireFuelField>,
            Option<&'static ChunkSurfaceFire>,
            Option<&'static ChunkSmokeField>,
        ),
    >,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PreviewRasterScratch {
    last_epoch: u64,
    last_layers: PreviewLayers,
    last_tex_w: u32,
    last_tex_h: u32,
    initialized: bool,
}

#[inline]
fn tile_in_chunk_world_rect(tx: u32, ty: u32, chunk_coord: IVec2, size: UVec2) -> bool {
    let sx = size.x as i32;
    let sy = size.y as i32;
    let x0 = chunk_coord.x * sx;
    let y0 = chunk_coord.y * sy;
    let x1 = x0 + sx - 1;
    let y1 = y0 + sy - 1;
    let txi = tx as i32;
    let tyi = ty as i32;
    txi >= x0 && tyi >= y0 && txi <= x1 && tyi <= y1
}

fn tile_in_any_dirty_chunk(
    tx: u32,
    ty: u32,
    dirty: &HashSet<IVec2>,
    chunk_geom: &HashMap<IVec2, UVec2>,
) -> bool {
    dirty
        .iter()
        .any(|c| chunk_geom.get(c).is_some_and(|sz| tile_in_chunk_world_rect(tx, ty, *c, *sz)))
}

pub fn update_world_preview_texture(
    mut images: ResMut<Assets<Image>>,
    preview_texture: Res<WorldPreviewTexture>,
    world_preview_ui: Res<super::WorldPreviewUiState>,
    world_gen_ui_state: Res<WorldGenUiState>,
    world_gen_params: Res<WorldGenParams>,
    mut preview_state: ResMut<WorldPreviewState>,
    macro_region_raster: Option<Res<MacroRegionRaster>>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    family_assets: Res<Assets<crate::terrain::TerrainFamilyRegistry>>,
    tag_assets: Res<Assets<TagRegistry>>,
    mobility_assets: Res<Assets<MobilityProfileRegistry>>,
    queries: WorldPreviewTileChunkQueries,
    overlay: Res<DynamicTerrainOverlay>,
    mut scratch: Local<PreviewRasterScratch>,
) {
    if !world_preview_ui.window_open && !world_gen_ui_state.visible {
        return;
    }

    let image = match images.get_mut(&preview_texture.texture) {
        Some(image) => image,
        None => return,
    };

    let width = preview_texture.width;
    let height = preview_texture.height;
    let tex_w = width as usize;
    let tex_h = height as usize;
    let len = 4 * tex_w * tex_h;
    let data = match image.data.as_mut() {
        Some(d) => d,
        None => return,
    };
    data.resize(len, 0);

    let epoch = preview_state.epoch.0;
    let drained_dirty: Vec<IVec2> = std::mem::take(&mut preview_state.dirty_queue);
    let dirty_set: HashSet<IVec2> = drained_dirty.into_iter().collect();
    let layers = world_gen_ui_state.preview_layers;

    let epoch_changed = scratch.last_epoch != epoch;
    let layers_changed = scratch.last_layers != layers;
    let tex_changed = scratch.last_tex_w != width || scratch.last_tex_h != height;
    let need_full =
        !scratch.initialized || epoch_changed || layers_changed || tex_changed;

    let partial_ok = !dirty_set.is_empty() && !need_full;

    if !need_full && dirty_set.is_empty() {
        return;
    }

    if need_full {
        data.fill(0);
    }

    let mat_slices: Vec<(IVec2, UVec2, &[MaterialId])> = queries
        .chunk_mats
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.materials.as_slice()))
        .collect();
    let chunk_geom: Vec<(IVec2, UVec2)> = queries
        .chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size))
        .collect();
    let chunk_geom_map: HashMap<IVec2, UVec2> = chunk_geom.iter().copied().collect();
    let tag_slices: Vec<(IVec2, UVec2, &[TagSet])> = queries
        .chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.tags.as_slice()))
        .collect();
    let elev_slices: Vec<(IVec2, UVec2, &[f32])> = queries
        .chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.elevation.as_slice()))
        .collect();
    let moist_slices: Vec<(IVec2, UVec2, &[f32])> = queries
        .chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.moisture.as_slice()))
        .collect();
    let temp_slices: Vec<(IVec2, UVec2, &[f32])> = queries
        .chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.temperature.as_slice()))
        .collect();
    let family_slices: Vec<(IVec2, UVec2, &[crate::terrain::family::TerrainFamilyId])> =
        queries
            .chunk_cells
            .iter()
            .map(|(c, m)| (c.coord, m.size, m.family.as_slice()))
            .collect();
    let slope_slices: Vec<(IVec2, UVec2, &[f32])> = queries
        .chunk_derived
        .iter()
        .map(|(c, d)| (c.coord, d.size, d.slope_grade.as_slice()))
        .collect();
    let ecology_slices: Vec<EcologyRasterChunkRow> = queries
        .chunk_ecology_bundle
        .iter()
        .map(|(c, m, eco, veg, wx, fuel, fire, smoke)| {
            (
                c.coord,
                m.size,
                eco.copied(),
                veg.copied(),
                wx.copied(),
                fuel.copied(),
                fire.map(|f| f.heat).unwrap_or(0.0),
                smoke.copied(),
            )
        })
        .collect();
    let reg_opt = materials.get(&handles.material_registry);
    let fam_opt = family_assets.get(&handles.terrain_families);
    let tag_reg_opt = tag_assets.get(&handles.tag_registry);
    let mob_reg_opt = mobility_assets.get(&handles.mobility_profiles);

    let mut tile_heights_lut: HashMap<(u32, u32), f32> = HashMap::new();
    for (transform, tile_height, _, _, _, _) in queries.tile_query.iter() {
        tile_heights_lut.insert(
            (transform.translation.x as u32, transform.translation.z as u32),
            tile_height.0,
        );
    }
    let tile_heights_ref = (!tile_heights_lut.is_empty()).then_some(&tile_heights_lut);
    let macro_r = macro_region_raster.as_ref().map(|r| &**r);

    for (transform, tile_height, moisture, temperature, terrain, region_ix) in queries.tile_query.iter()
    {
        let x = transform.translation.x as usize;
        let y = transform.translation.z as usize;

        if x >= tex_w || y >= tex_h {
            continue;
        }

        let tx = x as u32;
        let ty = y as u32;

        if partial_ok && !tile_in_any_dirty_chunk(tx, ty, &dirty_set, &chunk_geom_map) {
            continue;
        }

        let pixel_index = 4 * (y * tex_w + x);

        if pixel_index + 3 >= data.len() {
            continue;
        }

        let color = layers.composite_rgba_for_tile(
            tx,
            ty,
            width,
            height,
            tile_height.0,
            moisture.0,
            temperature.0,
            terrain.0,
            region_ix.copied(),
            &world_gen_params,
            macro_r,
            &elev_slices,
            &moist_slices,
            &temp_slices,
            &family_slices,
            &mat_slices,
            reg_opt,
            fam_opt,
            tag_reg_opt,
            &tag_slices,
            tile_heights_ref,
            &slope_slices,
            &chunk_geom,
            &overlay,
            mob_reg_opt,
            world_gen_ui_state.mobility_profile_index,
            &ecology_slices,
        );

        data[pixel_index] = color[0];
        data[pixel_index + 1] = color[1];
        data[pixel_index + 2] = color[2];
        data[pixel_index + 3] = color[3];
    }

    scratch.initialized = true;
    scratch.last_epoch = epoch;
    scratch.last_layers = layers;
    scratch.last_tex_w = width;
    scratch.last_tex_h = height;
}
