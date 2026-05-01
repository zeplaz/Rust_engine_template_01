use bevy::math::{IVec2, UVec2};
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::gui::editor::world_gen_ui::{PreviewMode, WorldGenUiState};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::biome::TerrainClass;
use crate::terrain::generation::world_generator_enhanced::{
    Height, Moisture, Temperature, TerrainType, TileMarker, WorldGenParams,
};
use crate::terrain::generation::{Chunk, ChunkCellMatrix};
use crate::terrain::material::{
    family_default_material_def, MaterialId, MaterialRegistry, MaterializedChunk, TagId, TagSet,
};
use bevy_egui::{egui, EguiPrimaryContextPass, EguiTextureHandle};

/// Highlight color for tag-overlay preview mode (U5).
pub const TAG_OVERLAY_HIGHLIGHT: [u8; 4] = [255, 220, 0, 255];

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

pub fn preview_biome_rgba_for_tile(
    tx: u32,
    ty: u32,
    terrain: &TerrainClass,
    chunks: &[(IVec2, UVec2, &[MaterialId])],
    registry: &MaterialRegistry,
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
    if let Some(def) = family_default_material_def(registry, *terrain) {
        return def.preview_color;
    }
    #[allow(deprecated)]
    {
        biome_to_color(terrain)
    }
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
    world_gen_ui_state: Res<WorldGenUiState>,
    world_gen_params: Res<WorldGenParams>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    tile_query: Query<(&Transform, &Height, &Moisture, &Temperature, &TerrainType), With<TileMarker>>,
    chunk_mats: Query<(&Chunk, &MaterializedChunk)>,
    chunk_cells: Query<(&Chunk, &ChunkCellMatrix)>,
) {
    // Only update if the preview is visible
    if !world_gen_ui_state.visible {
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
    let tag_slices: Vec<(IVec2, UVec2, &[TagSet])> = chunk_cells
        .iter()
        .map(|(c, m)| (c.coord, m.size, m.tags.as_slice()))
        .collect();
    let reg_opt = materials.get(&handles.material_registry);

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
            PreviewMode::Height => height_to_color(tile_height.0),
            PreviewMode::Moisture => moisture_to_color(moisture.0),
            PreviewMode::Temperature => temperature_to_color(temperature.0),
            PreviewMode::Biome => {
                let tx = x as u32;
                let ty = y as u32;
                match reg_opt {
                    Some(reg) => {
                        preview_biome_rgba_for_tile(tx, ty, &terrain.0, &mat_slices, reg)
                    }
                    None => {
                        #[allow(deprecated)]
                        {
                            biome_to_color(&terrain.0)
                        }
                    }
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
    world_gen_ui_state: Res<WorldGenUiState>,
    mut view: ResMut<WorldPreviewView>,
) -> Result {
    if !world_gen_ui_state.visible {
        return Ok(());
    }

    let texture_id = contexts.add_image(EguiTextureHandle::Strong(preview_texture.texture.clone()));
    let tex_w = preview_texture.width as f32;
    let tex_h = preview_texture.height as f32;

    egui::Window::new("World Preview")
        .resizable(true)
        .show(contexts.ctx_mut()?, |ui| {
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
            ui.small("Ctrl + scroll wheel (⌘ on macOS) over the map to zoom. Scroll to pan.");

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

#[deprecated(note = "use MaterialDef.preview_color via MaterialRegistry")]
fn biome_to_color(biome: &TerrainClass) -> [u8; 4] {
    match biome {
        TerrainClass::DeepWater => [0, 0, 128, 255],
        TerrainClass::ShallowWater => [0, 0, 255, 255],
        TerrainClass::Beach => [240, 240, 64, 255],
        TerrainClass::Desert => [255, 255, 128, 255],
        TerrainClass::Grassland => [0, 255, 0, 255],
        TerrainClass::Forest => [0, 128, 0, 255],
        TerrainClass::DenseForest => [0, 64, 0, 255],
        TerrainClass::Mountain => [128, 128, 128, 255],
        TerrainClass::SnowCappedMountain => [255, 255, 255, 255],
        TerrainClass::Tundra => [192, 192, 255, 255],
        TerrainClass::Swamp => [64, 64, 0, 255],
        TerrainClass::Cliff => [90, 90, 90, 255],
        TerrainClass::Concrete => [170, 170, 170, 255],
        TerrainClass::Dirt => [139, 69, 19, 255],
        TerrainClass::Snow => [250, 250, 250, 255],
        TerrainClass::Stone => [120, 120, 120, 255],
    }
}

// Plugin to register all world preview systems
pub struct WorldPreviewPlugin;

impl Plugin for WorldPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldPreviewTexture>()
            .init_resource::<WorldPreviewView>()
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
    use std::collections::HashMap;

    fn tiny_grass_registry() -> MaterialRegistry {
        MaterialRegistry {
            schema_version: 1,
            materials: vec![MaterialDef {
                name: "grass_default".into(),
                family: TerrainClass::Grassland,
                tags: vec![],
                properties: serde_json::json!({}),
                preview_color: [10, 20, 30, 255],
            }],
            name_to_id: HashMap::from([("grass_default".into(), MaterialId(0))]),
        }
    }

    #[test]
    fn preview_uses_material_def_color() {
        let reg = tiny_grass_registry();
        let chunks: Vec<(IVec2, UVec2, &[MaterialId])> = vec![];
        let c = preview_biome_rgba_for_tile(0, 0, &TerrainClass::Grassland, &chunks, &reg);
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