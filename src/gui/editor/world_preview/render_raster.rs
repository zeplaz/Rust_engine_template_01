//! Full-world CPU raster into [`super::texture_cache::WorldPreviewTexture`], with chunk-diff partial updates.

use super::ecology_preview::EcologyRasterChunkRow;
use super::layers::PreviewLayers;
use super::texture_cache::WorldPreviewTexture;
use crate::gui::map_view::MapViewInstances;
use crate::gui::preview_partial_min_interval_from_hz;
use crate::systems::ecology::{ChunkEcology, VegetationField};
use crate::render::SharedOverlayFieldBuffers;
use crate::systems::fire::{ChunkSmokeField, FireFuelField};
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
pub(crate) struct WorldPreviewRasterImageTargets<'w> {
    pub images: ResMut<'w, Assets<Image>>,
    pub swap: ResMut<'w, crate::gui::SwapImageBuffers>,
    pub preview_texture: Res<'w, WorldPreviewTexture>,
}

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
            Option<&'static ChunkSmokeField>,
        ),
    >,
    pub(crate) terrain_overlay: Res<'w, DynamicTerrainOverlay>,
    pub(crate) shared_overlay_fields: Res<'w, SharedOverlayFieldBuffers>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct PreviewRasterScratch {
    last_epoch: u64,
    last_layers: PreviewLayers,
    last_tex_w: u32,
    last_tex_h: u32,
    last_shared_overlay_revision: u64,
    initialized: bool,
}

#[derive(SystemParam)]
pub(crate) struct PreviewRasterGovernance<'w> {
    pub(crate) preview_budget: Res<'w, super::PreviewRenderBudget>,
    pub(crate) world_frame: Res<'w, crate::gui::WorldRepresentationFrame>,
    pub(crate) chunk_caches: ResMut<'w, super::WorldPreviewChunkCaches>,
    pub(crate) lifecycle_signals: ResMut<'w, super::preview_lifecycle::WorldPreviewLifecycleSignals>,
    pub(crate) preview_ready: Res<'w, super::preview_readiness::WorldPreviewReady>,
    pub(crate) resolved: Res<'w, crate::render::ResolvedViewports>,
    pub(crate) preview_render: ResMut<'w, super::preview_render_state::PreviewRenderState>,
}

#[derive(Default)]
pub(crate) struct PreviewRasterRuntime {
    pub scratch: PreviewRasterScratch,
    pub last_partial_raster_secs: f32,
    pub visible_chunks: HashSet<IVec2>,
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

/// Cap partial preview CPU raster rate from [`super::PreviewRenderBudget::max_hz`] (synced from global visual budgets when present).
fn world_preview_partial_min_interval_secs(preview_budget: &super::PreviewRenderBudget) -> f32 {
    preview_partial_min_interval_from_hz(preview_budget.max_hz)
}

pub fn update_world_preview_texture(
    mut targets: WorldPreviewRasterImageTargets,
    world_preview_ui: Res<super::WorldPreviewUiState>,
    map_views: Res<MapViewInstances>,
    world_gen_ui_state: Res<crate::gui::editor::world_gen_ui::WorldGenUiState>,
    world_gen_params: Res<WorldGenParams>,
    mut preview_state: ResMut<WorldPreviewState>,
    macro_region_raster: Option<Res<MacroRegionRaster>>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    family_assets: Res<Assets<crate::terrain::TerrainFamilyRegistry>>,
    tag_assets: Res<Assets<TagRegistry>>,
    mobility_assets: Res<Assets<MobilityProfileRegistry>>,
    queries: WorldPreviewTileChunkQueries,
    time: Res<Time>,
    mut runtime: Local<PreviewRasterRuntime>,
    mut governance: PreviewRasterGovernance,
) {
    if !world_preview_ui.window_open && !world_gen_ui_state.visible {
        return;
    }
    if !governance.preview_ready.0 {
        return;
    }

    let epoch = preview_state.epoch.0;
    let layers = map_views.world_preview.layers;
    let width = targets.preview_texture.width;
    let height = targets.preview_texture.height;

    let epoch_changed = runtime.scratch.last_epoch != epoch;
    let layers_changed = runtime.scratch.last_layers != layers;
    let tex_changed = runtime.scratch.last_tex_w != width || runtime.scratch.last_tex_h != height;
    let need_full = !runtime.scratch.initialized
        || epoch_changed
        || layers_changed
        || tex_changed;

    let has_dirty = !preview_state.dirty_queue.is_empty();
    let overlay_rev = queries.shared_overlay_fields.revision;
    let overlay_revision_changed =
        runtime.scratch.last_shared_overlay_revision != overlay_rev;

    if !need_full && !has_dirty && !overlay_revision_changed {
        return;
    }

    let overlay_only = overlay_revision_changed && !need_full && !has_dirty;

    // Throttle partial dirty-chunk passes and fire-overlay-only full passes.
    if (has_dirty && !need_full) || overlay_only {
        let now = time.elapsed_secs();
        let min_dt = world_preview_partial_min_interval_secs(&governance.preview_budget);
        if now - runtime.last_partial_raster_secs < min_dt {
            return;
        }
        runtime.last_partial_raster_secs = now;
    } else {
        runtime.last_partial_raster_secs = time.elapsed_secs();
    }

    let write_handle = if targets.swap.back != Handle::default() {
        targets.swap.back.clone()
    } else {
        targets.preview_texture.texture.clone()
    };
    let wrote_to_swap_back = targets.swap.back != Handle::default();

    let image = match targets.images.get_mut(&write_handle) {
        Some(image) => image,
        None => return,
    };

    let tex_w = width as usize;
    let tex_h = height as usize;
    let len = 4 * tex_w * tex_h;
    let data = match image.data.as_mut() {
        Some(d) => d,
        None => return,
    };
    if need_full && !governance.resolved.world_preview.valid {
        governance.preview_render.held_last_raster_due_to_invalid_viewport = true;
        return;
    }
    governance.preview_render.held_last_raster_due_to_invalid_viewport = false;

    data.resize(len, 0);

    let drained_dirty: Vec<IVec2> = std::mem::take(&mut preview_state.dirty_queue);
    let dirty_set: HashSet<IVec2> = drained_dirty.into_iter().collect();
    runtime.visible_chunks = dirty_set.clone();
    let partial_ok = !dirty_set.is_empty() && !need_full;

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
        .map(|(c, m, eco, veg, wx, fuel, smoke)| {
            (
                c.coord,
                m.size,
                eco.copied(),
                veg.copied(),
                wx.copied(),
                fuel.copied(),
                queries.shared_overlay_fields.fire_surface_heat_at(c.coord),
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
            &queries.terrain_overlay,
            mob_reg_opt,
            world_gen_ui_state.mobility_profile_index,
            &ecology_slices,
        );

        data[pixel_index] = color[0];
        data[pixel_index + 1] = color[1];
        data[pixel_index + 2] = color[2];
        data[pixel_index + 3] = color[3];
    }

    runtime.scratch.initialized = true;
    runtime.scratch.last_epoch = epoch;
    runtime.scratch.last_layers = layers;
    runtime.scratch.last_tex_w = width;
    runtime.scratch.last_tex_h = height;
    if overlay_revision_changed {
        runtime.scratch.last_shared_overlay_revision = overlay_rev;
    }

    if partial_ok {
        let mut cache_coords: HashSet<IVec2> = dirty_set.clone();
        for coord in crate::io::streaming::ghost_band_neighbor_coords_for_preview(
            governance.world_frame.focus_chunk,
            governance.world_frame.interest_radius_chunks.max(1),
        ) {
            cache_coords.insert(coord);
        }
        for coord in cache_coords {
            if let Some(&size) = chunk_geom_map.get(&coord) {
                super::cache::sync_chunk_preview_cache(
                    coord,
                    size,
                    data,
                    tex_w,
                    tex_h,
                    &mut governance.chunk_caches,
                    overlay_rev,
                );
            }
        }
    }

    if wrote_to_swap_back {
        targets.swap.dirty = true;
    }
    super::preview_lifecycle::note_world_preview_raster_wrote(&mut governance.lifecycle_signals);
}
