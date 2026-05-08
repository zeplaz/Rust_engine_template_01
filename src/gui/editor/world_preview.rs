//! World raster preview — tints align with [`WorldGenParams`] width/height.
//!
//! **Authoritative when chunks exist:** height, moisture, temperature, and classified **biome family**
//! come from [`ChunkCellMatrix`] on materialized chunk entities (same grid as tags / materials).
//! ECS [`TileMarker`](crate::terrain::generation::world_generator_enhanced::TileMarker) tiles remain
//! the **fallback** until chunk data covers that world tile.

use bevy::math::{IVec2, UVec2};
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::gui::editor::world_gen_hints as hints;
use crate::gui::editor::world_gen_ui::WorldGenUiState;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::ChunkCellKey;
use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry};
use crate::terrain::generation::world_generator_enhanced::{
    Height, Moisture, Temperature, TerrainType, TileMarker, WorldGenParams,
};
use crate::terrain::generation::{Chunk, ChunkCellMatrix, ChunkDerivedMetrics};
use crate::terrain::mobility::{evaluate_tile, MobilityProfileRegistry, MovementHint};
use crate::terrain::material::{
    family_default_material_def, MaterialId, MaterialRegistry, MaterializedChunk, TagId, TagRegistry,
    TagSet,
};
use bevy_egui::{egui, EguiPrimaryContextPass, EguiTextureHandle};

/// Highlight color for tag-overlay preview mode (U5).
pub const TAG_OVERLAY_HIGHLIGHT: [u8; 4] = [255, 220, 0, 255];

/// RGBA for editor / minimap when material registry is missing or has no row for `family`.
pub fn terrain_family_preview_rgba(
    families: Option<&TerrainFamilyRegistry>,
    id: TerrainFamilyId,
) -> [u8; 4] {
    fn by_name(name: &str) -> [u8; 4] {
        match name {
            "DeepWater" => [0, 0, 128, 255],
            "ShallowWater" => [0, 0, 255, 255],
            "Beach" => [240, 240, 64, 255],
            "Desert" => [255, 255, 128, 255],
            "Grassland" => [0, 255, 0, 255],
            "Forest" => [0, 128, 0, 255],
            "DenseForest" => [0, 64, 0, 255],
            "Mountain" => [128, 128, 128, 255],
            "SnowCappedMountain" => [255, 255, 255, 255],
            "Tundra" => [192, 192, 255, 255],
            "Swamp" => [64, 64, 0, 255],
            "Cliff" => [90, 90, 90, 255],
            "Concrete" => [170, 170, 170, 255],
            "Dirt" => [139, 69, 19, 255],
            "Snow" => [250, 250, 250, 255],
            "Stone" => [120, 120, 120, 255],
            _ => [128, 0, 128, 255],
        }
    }
    if let Some(reg) = families {
        if let Some(d) = reg.def(id) {
            return by_name(&d.name);
        }
    }
    let u = (id.0 as u32)
        .wrapping_mul(1103515245)
        .wrapping_add(12345);
    [
        (u & 0xff) as u8,
        ((u >> 8) & 0xff) as u8,
        ((u >> 16) & 0xff) as u8,
        255,
    ]
}

/// Row-major cell layer at world tile `(tx, ty)` from overlapping chunk slabs `(coord, size, slice)`.
pub fn chunk_cell_layer_at_world_tile<T: Copy>(
    tx: u32,
    ty: u32,
    chunks: &[(IVec2, UVec2, &[T])],
) -> Option<T> {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size, layer) in chunks {
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
        let idx = (ly * sx + lx) as usize;
        if idx < layer.len() {
            return Some(layer[idx]);
        }
    }
    None
}

/// [`ChunkCellKey`] for world tile `(tx, ty)` when it falls on a listed chunk slab.
pub fn chunk_cell_key_for_world_tile(
    tx: u32,
    ty: u32,
    chunks: &[(IVec2, UVec2)],
) -> Option<ChunkCellKey> {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size) in chunks {
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
        let idx = (ly * sx + lx) as u32;
        return Some(ChunkCellKey::new(*coord, idx));
    }
    None
}

pub fn cell_tags_for_world_tile(
    tx: u32,
    ty: u32,
    chunks: &[(IVec2, UVec2, &[TagSet])],
) -> Option<TagSet> {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size, tags_vec) in chunks {
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
        let idx = (ly * sx + lx) as usize;
        if idx < tags_vec.len() {
            return Some(tags_vec[idx]);
        }
    }
    None
}

/// Chunk-derived stitched `slope_grade` at world tile `(tx, ty)` when present on a materialized chunk.
#[inline]
pub fn slope_grade_for_world_tile(
    tx: u32,
    ty: u32,
    chunks: &[(IVec2, UVec2, &[f32])],
) -> Option<f32> {
    chunk_cell_layer_at_world_tile(tx, ty, chunks)
}

pub fn movement_hint_rgba(hint: &MovementHint) -> [u8; 4] {
    if hint.blocked {
        [220, 50, 50, 255]
    } else {
        let stress = ((hint.cost_mul - 1.0).max(0.0) / 2.0).min(1.0);
        let g = ((1.0 - stress) * 220.0) as u8;
        let r = (stress * 200.0) as u8;
        [r, g, 70, 255]
    }
}

fn slope_grade_to_color(s: f32) -> [u8; 4] {
    let u = (s.clamp(0.0, 1.0) * 255.0) as u8;
    [u, 255u8.saturating_sub(u), 120, 255]
}

pub fn preview_biome_rgba_for_tile(
    tx: u32,
    ty: u32,
    terrain_family: TerrainFamilyId,
    chunks: &[(IVec2, UVec2, &[MaterialId])],
    registry: &MaterialRegistry,
    families: Option<&TerrainFamilyRegistry>,
) -> [u8; 4] {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size, materials) in chunks {
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
        let idx = (ly * sx + lx) as usize;
        if idx < materials.len() {
            let mid = materials[idx];
            return registry.materials[mid.0 as usize].preview_color;
        }
    }
    if let Some(def) = family_default_material_def(registry, terrain_family) {
        return def.preview_color;
    }
    terrain_family_preview_rgba(families, terrain_family)
}

/// `sim.traction_mod` from the resolved material at world tile `(tx, ty)`, else **1.0** (same lookup order as [`preview_biome_rgba_for_tile`]).
pub fn material_traction_mod_for_world_tile(
    tx: u32,
    ty: u32,
    terrain_family: TerrainFamilyId,
    chunks: &[(IVec2, UVec2, &[MaterialId])],
    registry: &MaterialRegistry,
) -> f32 {
    let tx_i = tx as i32;
    let ty_i = ty as i32;
    for (coord, size, materials) in chunks {
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
        let idx = (ly * sx + lx) as usize;
        if idx < materials.len() {
            let mid = materials[idx];
            return registry.materials[mid.0 as usize]
                .sim_f32("traction_mod")
                .unwrap_or(1.0);
        }
    }
    if let Some(def) = family_default_material_def(registry, terrain_family) {
        return def.sim_f32("traction_mod").unwrap_or(1.0);
    }
    1.0
}

pub fn tag_overlay_rgba(tag_target: TagId, cell_tags: &TagSet) -> [u8; 4] {
    if cell_tags.contains(tag_target) {
        TAG_OVERLAY_HIGHLIGHT
    } else {
        [0, 0, 0, 0]
    }
}

#[inline]
pub fn tag_overlay_rgba_pool(cell_tags: &TagSet, pool: &TagSet) -> [u8; 4] {
    if pool.intersects(cell_tags) {
        TAG_OVERLAY_HIGHLIGHT
    } else {
        [0, 0, 0, 0]
    }
}

/// Raster tint mode for [`update_world_preview_texture`] (one component per tile, no full regen).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PreviewMode {
    None,
    Height,
    Moisture,
    Temperature,
    Biome,
    Regions,
    /// Highlights cells whose tags intersect [`WorldGenParams::tag_pool`](crate::terrain::generation::world_generator_enhanced::WorldGenParams::tag_pool).
    Tag,
    /// Stitched [`ChunkDerivedMetrics::slope_grade`] from chunk entities (authoritative for terrain).
    DerivedSlope,
    /// Mobility hint for the selected profile — interpretation, not stored terrain state.
    Mobility,
}

impl PreviewMode {
    fn label(self) -> &'static str {
        match self {
            PreviewMode::None => "None",
            PreviewMode::Height => "Height",
            PreviewMode::Moisture => "Moisture",
            PreviewMode::Temperature => "Temperature",
            PreviewMode::Biome => "Biome",
            PreviewMode::Regions => "Regions",
            PreviewMode::Tag => "Tag",
            PreviewMode::DerivedSlope => "Slope (chunk)",
            PreviewMode::Mobility => "Mobility",
        }
    }

    fn next(self) -> Self {
        match self {
            PreviewMode::None => PreviewMode::Height,
            PreviewMode::Height => PreviewMode::Moisture,
            PreviewMode::Moisture => PreviewMode::Temperature,
            PreviewMode::Temperature => PreviewMode::Biome,
            PreviewMode::Biome => PreviewMode::Regions,
            PreviewMode::Regions => PreviewMode::Tag,
            PreviewMode::Tag => PreviewMode::DerivedSlope,
            PreviewMode::DerivedSlope => PreviewMode::Mobility,
            PreviewMode::Mobility => PreviewMode::None,
        }
    }

    fn prev(self) -> Self {
        match self {
            PreviewMode::None => PreviewMode::Mobility,
            PreviewMode::Height => PreviewMode::None,
            PreviewMode::Moisture => PreviewMode::Height,
            PreviewMode::Temperature => PreviewMode::Moisture,
            PreviewMode::Biome => PreviewMode::Temperature,
            PreviewMode::Regions => PreviewMode::Biome,
            PreviewMode::Tag => PreviewMode::Regions,
            PreviewMode::DerivedSlope => PreviewMode::Tag,
            PreviewMode::Mobility => PreviewMode::DerivedSlope,
        }
    }
}

/// Toggles for the World Preview egui window (independent of whether the World Generator panel is open).
#[derive(Resource)]
pub struct WorldPreviewUiState {
    pub window_open: bool,
}

impl Default for WorldPreviewUiState {
    fn default() -> Self {
        Self { window_open: true }
    }
}

#[inline]
fn tt_egui(response: egui::Response, text: &'static str) -> egui::Response {
    response.on_hover_text(text)
}

/// Zoom / pan state for the World Preview egui window.
#[derive(Resource)]
pub struct WorldPreviewView {
    /// Scale factor: 1.0 = one screen pixel per map tile (at default fit).
    pub zoom: f32,
}

impl WorldPreviewView {
    pub const ZOOM_MIN: f32 = 0.02;
    pub const ZOOM_MAX: f32 = 32.0;
}

impl Default for WorldPreviewView {
    fn default() -> Self {
        Self { zoom: 1.0 }
    }
}

fn rgba_preview_image(width: u32, height: u32) -> Image {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    image.data = Some(vec![0; 4 * width as usize * height as usize]);
    image
}

/// Resize the RGBA preview buffer when `WorldGenParams` width/height changes.
pub fn sync_world_preview_texture_size(
    params: Res<WorldGenParams>,
    mut preview: ResMut<WorldPreviewTexture>,
    mut images: ResMut<Assets<Image>>,
) {
    if preview.width == params.width && preview.height == params.height {
        return;
    }

    let width = params.width;
    let height = params.height;
    let old = preview.texture.clone();
    let image = rgba_preview_image(width, height);
    preview.texture = images.add(image);
    preview.width = width;
    preview.height = height;

    let _ = images.remove(old.id());
}

#[derive(Resource)]
pub struct WorldPreviewTexture {
    pub texture: Handle<Image>,
    pub width: u32,
    pub height: u32,
}

impl Default for WorldPreviewTexture {
    fn default() -> Self {
        Self {
            texture: Handle::default(),
            width: 512,
            height: 512,
        }
    }
}

// Initialize the preview texture
pub fn init_world_preview_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    params: Res<WorldGenParams>,
) {
    // Create a new image
    let image = rgba_preview_image(params.width, params.height);
    let texture_handle = images.add(image);
    
    commands.insert_resource(WorldPreviewTexture {
        texture: texture_handle,
        width: params.width,
        height: params.height,
    });
}

// Update the preview texture based on the current world state
pub fn update_world_preview_texture(
    mut images: ResMut<Assets<Image>>,
    preview_texture: Res<WorldPreviewTexture>,
    world_preview_ui: Res<WorldPreviewUiState>,
    world_gen_ui_state: Res<WorldGenUiState>,
    world_gen_params: Res<WorldGenParams>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    family_assets: Res<Assets<TerrainFamilyRegistry>>,
    tag_assets: Res<Assets<TagRegistry>>,
    mobility_assets: Res<Assets<MobilityProfileRegistry>>,
    tile_query: Query<(&Transform, &Height, &Moisture, &Temperature, &TerrainType), With<TileMarker>>,
    chunk_mats: Query<(&Chunk, &MaterializedChunk)>,
    chunk_cells: Query<(&Chunk, &ChunkCellMatrix)>,
    chunk_derived: Query<(&Chunk, &ChunkDerivedMetrics)>,
    overlay: Res<crate::terrain::DynamicTerrainOverlay>,
) {
    if !world_preview_ui.window_open && !world_gen_ui_state.visible {
        return;
    }
    
    // Get the texture
    let image = match images.get_mut(&preview_texture.texture) {
        Some(image) => image,
        None => return,
    };
    
    // Clear the image
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
    data.fill(0);

    let mat_slices: Vec<(IVec2, UVec2, &[MaterialId])> = chunk_mats
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.materials.as_slice()))
        .collect();
    let chunk_geom: Vec<(IVec2, UVec2)> = chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size))
        .collect();
    let tag_slices: Vec<(IVec2, UVec2, &[TagSet])> = chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.tags.as_slice()))
        .collect();
    let elev_slices: Vec<(IVec2, UVec2, &[f32])> = chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.elevation.as_slice()))
        .collect();
    let moist_slices: Vec<(IVec2, UVec2, &[f32])> = chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.moisture.as_slice()))
        .collect();
    let temp_slices: Vec<(IVec2, UVec2, &[f32])> = chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.temperature.as_slice()))
        .collect();
    let family_slices: Vec<(IVec2, UVec2, &[TerrainFamilyId])> = chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.family.as_slice()))
        .collect();
    let slope_slices: Vec<(IVec2, UVec2, &[f32])> = chunk_derived
        .iter()
        .map(|(c, d)| (c.coord, d.size, d.slope_grade.as_slice()))
        .collect();
    let reg_opt = materials.get(&handles.material_registry);
    let fam_opt = family_assets.get(&handles.terrain_families);
    let tag_reg_opt = tag_assets.get(&handles.tag_registry);
    let mob_reg_opt = mobility_assets.get(&handles.mobility_profiles);

    // Draw tiles based on the preview mode
    for (transform, tile_height, moisture, temperature, terrain) in tile_query.iter() {
        let x = transform.translation.x as usize;
        let y = transform.translation.z as usize;
        
        if x >= tex_w || y >= tex_h {
            continue;
        }
        
        let pixel_index = 4 * (y * tex_w + x);
        
        // Skip if out of bounds
        if pixel_index + 3 >= data.len() {
            continue;
        }
        
        // Choose color based on the preview mode
        let color = match world_gen_ui_state.preview_mode {
            PreviewMode::Height => {
                let tx = x as u32;
                let ty = y as u32;
                let h = chunk_cell_layer_at_world_tile(tx, ty, &elev_slices)
                    .unwrap_or(tile_height.0);
                height_to_color(h)
            }
            PreviewMode::Moisture => {
                let tx = x as u32;
                let ty = y as u32;
                let m = chunk_cell_layer_at_world_tile(tx, ty, &moist_slices)
                    .unwrap_or(moisture.0);
                moisture_to_color(m)
            }
            PreviewMode::Temperature => {
                let tx = x as u32;
                let ty = y as u32;
                let t = chunk_cell_layer_at_world_tile(tx, ty, &temp_slices)
                    .unwrap_or(temperature.0);
                temperature_to_color(t)
            }
            PreviewMode::Biome => {
                let tx = x as u32;
                let ty = y as u32;
                let terrain_family = chunk_cell_layer_at_world_tile(tx, ty, &family_slices)
                    .unwrap_or(terrain.0);
                match reg_opt {
                    Some(reg) => {
                        preview_biome_rgba_for_tile(
                            tx,
                            ty,
                            terrain_family,
                            &mat_slices,
                            reg,
                            fam_opt,
                        )
                    }
                    None => terrain_family_preview_rgba(fam_opt, terrain_family),
                }
            }
            PreviewMode::Tag => {
                let tx = x as u32;
                let ty = y as u32;
                match cell_tags_for_world_tile(tx, ty, &tag_slices) {
                    Some(ts) => tag_overlay_rgba_pool(&ts, &world_gen_params.tag_pool),
                    None => [0, 0, 0, 0],
                }
            }
            PreviewMode::Regions => {
                // For region preview, we would need information about which region each tile belongs to
                // For now, just default to black
                [0, 0, 0, 255]
            }
            PreviewMode::DerivedSlope => {
                let tx = x as u32;
                let ty = y as u32;
                match slope_grade_for_world_tile(tx, ty, &slope_slices) {
                    Some(s) => slope_grade_to_color(s),
                    None => [0, 0, 0, 0],
                }
            }
            PreviewMode::Mobility => {
                let tx = x as u32;
                let ty = y as u32;
                if let (Some(tag_reg), Some(mob_reg)) = (tag_reg_opt, mob_reg_opt) {
                    if !mob_reg.profiles.is_empty() {
                        let pi = world_gen_ui_state
                            .mobility_profile_index
                            .min(mob_reg.profiles.len() - 1);
                        let profile = &mob_reg.profiles[pi];
                        let slope = slope_grade_for_world_tile(tx, ty, &slope_slices).unwrap_or(0.0);
                        let tags = cell_tags_for_world_tile(tx, ty, &tag_slices).unwrap_or_default();
                        let terrain_family = chunk_cell_layer_at_world_tile(tx, ty, &family_slices)
                            .unwrap_or(terrain.0);
                        let mud_boost = chunk_cell_key_for_world_tile(tx, ty, &chunk_geom)
                            .and_then(|k| overlay.mud.get(&k).copied())
                            .filter(|&m| m > 1e-6)
                            .map(|mud| 1.0 + mud * 0.25)
                            .unwrap_or(1.0);
                        let traction = reg_opt
                            .map(|r| {
                                material_traction_mod_for_world_tile(
                                    tx,
                                    ty,
                                    terrain_family,
                                    &mat_slices,
                                    r,
                                ) * mud_boost
                            })
                            .unwrap_or(mud_boost);
                        let hint =
                            evaluate_tile(profile, &tags, slope, 1.0, tag_reg, traction);
                        movement_hint_rgba(&hint)
                    } else {
                        [0, 0, 0, 0]
                    }
                } else {
                    [0, 0, 0, 0]
                }
            }
            PreviewMode::None => [0, 0, 0, 0], // Transparent
        };
        
        // Set the pixel color
        data[pixel_index] = color[0];
        data[pixel_index + 1] = color[1];
        data[pixel_index + 2] = color[2];
        data[pixel_index + 3] = color[3];
    }
}

// UI system to display the preview texture — EguiPrimaryContextPass, returns Result.
pub fn display_world_preview(
    mut contexts: bevy_egui::EguiContexts,
    preview_texture: Res<WorldPreviewTexture>,
    mut world_preview_ui: ResMut<WorldPreviewUiState>,
    mut world_gen_ui_state: ResMut<WorldGenUiState>,
    mut world_gen_params: ResMut<WorldGenParams>,
    mut view: ResMut<WorldPreviewView>,
    handles: Res<TerrainRegistriesHandles>,
    tag_assets: Res<Assets<TagRegistry>>,
    mobility_assets: Res<Assets<MobilityProfileRegistry>>,
) -> Result {
    if !world_preview_ui.window_open {
        return Ok(());
    }

    let texture_id = contexts.add_image(EguiTextureHandle::Strong(preview_texture.texture.clone()));
    let tex_w = preview_texture.width as f32;
    let tex_h = preview_texture.height as f32;

    let mut window_open = world_preview_ui.window_open;
    egui::Window::new("World Preview")
        .resizable(true)
        .open(&mut window_open)
        .show(contexts.ctx_mut()?, |ui| {
            ui.horizontal(|ui| {
                ui.label("Preview:");
                if tt_egui(ui.button("◀"), "Previous view mode").clicked() {
                    world_gen_ui_state.preview_mode = world_gen_ui_state.preview_mode.prev();
                }
                if tt_egui(ui.button("▶"), "Next view mode").clicked() {
                    world_gen_ui_state.preview_mode = world_gen_ui_state.preview_mode.next();
                }
                ui.label(
                    egui::RichText::new(world_gen_ui_state.preview_mode.label())
                        .strong(),
                );
            });
            ui.horizontal(|ui| {
                tt_egui(
                    ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::None, "None"),
                    hints::PREVIEW_NONE,
                );
                tt_egui(
                    ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Height, "Height"),
                    hints::PREVIEW_HEIGHT,
                );
                tt_egui(
                    ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Moisture, "Moisture"),
                    hints::PREVIEW_MOIST,
                );
                tt_egui(
                    ui.radio_value(
                        &mut world_gen_ui_state.preview_mode,
                        PreviewMode::Temperature,
                        "Temperature",
                    ),
                    hints::PREVIEW_TEMP,
                );
            });
            ui.horizontal(|ui| {
                tt_egui(
                    ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Biome, "Biome"),
                    hints::PREVIEW_BIOME,
                );
                tt_egui(
                    ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Regions, "Regions"),
                    hints::PREVIEW_REGIONS,
                );
                tt_egui(
                    ui.radio_value(&mut world_gen_ui_state.preview_mode, PreviewMode::Tag, "Tag"),
                    hints::PREVIEW_TAG,
                );
            });
            ui.horizontal(|ui| {
                tt_egui(
                    ui.radio_value(
                        &mut world_gen_ui_state.preview_mode,
                        PreviewMode::DerivedSlope,
                        "Slope",
                    ),
                    hints::PREVIEW_SLOPE,
                );
                tt_egui(
                    ui.radio_value(
                        &mut world_gen_ui_state.preview_mode,
                        PreviewMode::Mobility,
                        "Mobility",
                    ),
                    hints::PREVIEW_MOBILITY,
                );
            });
            if let Some(mob) = mobility_assets.get(&handles.mobility_profiles) {
                if !mob.profiles.is_empty() {
                    ui.horizontal(|ui| {
                        ui.label("Mobility profile:");
                        let n = mob.profiles.len();
                        let idx = world_gen_ui_state
                            .mobility_profile_index
                            .min(n - 1);
                        world_gen_ui_state.mobility_profile_index = idx;
                        let mut sel = idx;
                        egui::ComboBox::from_id_salt("world_preview_mobility_profile")
                            .selected_text(mob.profiles[idx].id.as_str())
                            .show_ui(ui, |ui| {
                                for (i, p) in mob.profiles.iter().enumerate() {
                                    ui.selectable_value(&mut sel, i, p.id.as_str());
                                }
                            });
                        if sel != idx {
                            world_gen_ui_state.mobility_profile_index = sel;
                        }
                    });
                }
            }
            if let Some(tag_reg) = tag_assets.get(&handles.tag_registry) {
                ui.label("Terrain tag pool (passes 2 & 4):");
                ui.small(
                    "Unchecked names are not written onto chunks; Tag view only highlights cells carrying checked tags.",
                );
                egui::ScrollArea::vertical()
                    .max_height(160.0)
                    .id_salt("world_preview_tag_pool_scroll")
                    .show(ui, |ui| {
                        for (i, t) in tag_reg.tags.iter().enumerate() {
                            let id = TagId(i as u16);
                            let mut on = world_gen_params.tag_pool.contains(id);
                            let r = ui.checkbox(&mut on, &t.name);
                            let r = tt_egui(r, hints::TAG_POOL_ENTRY);
                            if r.changed() {
                                if on {
                                    world_gen_params.tag_pool.insert(id);
                                } else {
                                    world_gen_params.tag_pool.remove(id);
                                }
                            }
                        }
                    });
            }

            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(egui::Slider::new(
                    &mut view.zoom,
                    WorldPreviewView::ZOOM_MIN..=WorldPreviewView::ZOOM_MAX,
                ));
                if ui.button("1∶1").clicked() {
                    view.zoom = 1.0;
                }
            });
            ui.small(format!(
                "Map {}×{} tiles ({} cells). Ctrl / ⌘ + scroll on the image to zoom; scroll area pans.",
                preview_texture.width,
                preview_texture.height,
                preview_texture.width as u64 * preview_texture.height as u64,
            ));
            let z = view
                .zoom
                .clamp(WorldPreviewView::ZOOM_MIN, WorldPreviewView::ZOOM_MAX);
            let display_w = tex_w * z;
            let display_h = tex_h * z;

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let sized = egui::load::SizedTexture::new(texture_id, [display_w, display_h]);
                    let resp = ui.image(sized);
                    if resp.hovered() {
                        let zoom_mod = ui.ctx().input(|i| i.modifiers.ctrl || i.modifiers.command);
                        let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
                        if zoom_mod && scroll != 0.0 {
                            view.zoom *= 1.0 + scroll * 0.002;
                            view.zoom = view
                                .zoom
                                .clamp(WorldPreviewView::ZOOM_MIN, WorldPreviewView::ZOOM_MAX);
                        }
                    }
                });
        });
    world_preview_ui.window_open = window_open;
    Ok(())
}

// Helper functions to convert data to colors

fn height_to_color(height: f32) -> [u8; 4] {
    let h = (height * 255.0) as u8;
    [h, h, h, 255]
}

fn moisture_to_color(moisture: f32) -> [u8; 4] {
    let m = (moisture * 255.0) as u8;
    [0, 0, m, 255]
}

fn temperature_to_color(temperature: f32) -> [u8; 4] {
    let t = (temperature * 255.0) as u8;
    [t, 0, 0, 255]
}

// Plugin to register all world preview systems
pub struct WorldPreviewPlugin;

impl Plugin for WorldPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldPreviewTexture>()
            .init_resource::<WorldPreviewView>()
            .init_resource::<WorldPreviewUiState>()
            .add_systems(Startup, init_world_preview_texture)
            .add_systems(
                Update,
                (sync_world_preview_texture_size, update_world_preview_texture).chain(),
            )
            .add_systems(EguiPrimaryContextPass, display_world_preview);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::material::{MaterialDef, MaterialRegistry};
    use crate::terrain::TerrainFamilyRegistry;
    use std::collections::HashMap;

    fn tiny_grass_registry() -> (TerrainFamilyRegistry, MaterialRegistry) {
        let fam_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("assets/config/terrain/terrain_family_registry.example.json");
        let families =
            TerrainFamilyRegistry::load_from_json(fam_path.to_str().unwrap()).unwrap();
        let grass = families.id("Grassland").unwrap();
        let reg = MaterialRegistry {
            schema_version: 2,
            materials: vec![MaterialDef {
                name: "grass_default".into(),
                family: grass,
                tags: vec![],
                properties: serde_json::json!({}),
                preview_color: [10, 20, 30, 255],
            }],
            name_to_id: HashMap::from([("grass_default".into(), MaterialId(0))]),
        };
        (families, reg)
    }

    #[test]
    fn chunk_cell_layer_prefers_matching_chunk() {
        let size = UVec2::new(2, 2);
        let elev = vec![0.1, 0.2, 0.3, 0.4];
        let slices: Vec<(IVec2, UVec2, &[f32])> = vec![(IVec2::ZERO, size, elev.as_slice())];
        assert_eq!(chunk_cell_layer_at_world_tile(0, 0, &slices), Some(0.1));
        assert_eq!(chunk_cell_layer_at_world_tile(1, 0, &slices), Some(0.2));
        assert_eq!(chunk_cell_layer_at_world_tile(0, 1, &slices), Some(0.3));
    }

    #[test]
    fn chunk_cell_key_matches_flat_index() {
        let geom = vec![(IVec2::ZERO, UVec2::new(2, 2))];
        assert_eq!(
            chunk_cell_key_for_world_tile(1, 0, &geom),
            Some(ChunkCellKey::new(IVec2::ZERO, 1))
        );
    }

    #[test]
    fn preview_uses_material_def_color() {
        let (families, reg) = tiny_grass_registry();
        let grass = families.id("Grassland").unwrap();
        let chunks: Vec<(IVec2, UVec2, &[MaterialId])> = vec![];
        let c = preview_biome_rgba_for_tile(0, 0, grass, &chunks, &reg, Some(&families));
        assert_eq!(c, [10, 20, 30, 255]);
    }

    #[test]
    fn preview_tag_overlay_highlights_match() {
        let mut ts = TagSet::default();
        ts.insert(TagId(5));
        assert_eq!(tag_overlay_rgba(TagId(5), &ts), TAG_OVERLAY_HIGHLIGHT);
        assert_eq!(tag_overlay_rgba(TagId(4), &ts), [0, 0, 0, 0]);
    }

    #[test]
    fn preview_tag_pool_highlights_overlap() {
        let mut pool = TagSet::default();
        pool.insert(TagId(4));
        let mut ts = TagSet::default();
        ts.insert(TagId(5));
        assert_eq!(tag_overlay_rgba_pool(&ts, &pool), [0, 0, 0, 0]);
        ts.insert(TagId(4));
        assert_eq!(tag_overlay_rgba_pool(&ts, &pool), TAG_OVERLAY_HIGHLIGHT);
    }
}