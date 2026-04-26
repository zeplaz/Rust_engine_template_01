use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};

use crate::terrain::generation::world_generator_enhanced::{
    WorldGenParams, BiomeType, Height, Moisture, Temperature, TerrainType, TileMarker
};
use crate::gui::editor::world_gen_ui::{WorldGenUiState, PreviewMode};

// Resources for the world preview
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
    let width = params.width;
    let height = params.height;
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
    
    // Fill with black background
    image.data = vec![0; (4 * width * height) as usize];
    
    // Add to assets
    let texture_handle = images.add(image);
    
    // Create resource
    commands.insert_resource(WorldPreviewTexture {
        texture: texture_handle,
        width,
        height,
    });
}

// Update the preview texture based on the current world state
pub fn update_world_preview_texture(
    mut images: ResMut<Assets<Image>>,
    preview_texture: Res<WorldPreviewTexture>,
    world_gen_ui_state: Res<WorldGenUiState>,
    tile_query: Query<(&Transform, &Height, &Moisture, &Temperature, &TerrainType), With<TileMarker>>,
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
    image.data = vec![0; (4 * width * height) as usize];
    
    // Draw tiles based on the preview mode
    for (transform, height, moisture, temperature, terrain) in tile_query.iter() {
        let x = transform.translation.x as usize;
        let y = transform.translation.z as usize;
        
        if x >= width as usize || y >= height as usize {
            continue;
        }
        
        let pixel_index = 4 * (y * width as usize + x);
        
        // Skip if out of bounds
        if pixel_index + 3 >= image.data.len() {
            continue;
        }
        
        // Choose color based on preview mode
        let color = match world_gen_ui_state.preview_mode {
            PreviewMode::Height => height_to_color(height.0),
            PreviewMode::Moisture => moisture_to_color(moisture.0),
            PreviewMode::Temperature => temperature_to_color(temperature.0),
            PreviewMode::Biome => biome_to_color(&terrain.0),
            PreviewMode::Regions => {
                // For region preview, we would need information about which region each tile belongs to
                // For now, just default to black
                [0, 0, 0, 255]
            },
            PreviewMode::None => [0, 0, 0, 0], // Transparent
        };
        
        // Set the pixel color
        image.data[pixel_index] = color[0];
        image.data[pixel_index + 1] = color[1];
        image.data[pixel_index + 2] = color[2];
        image.data[pixel_index + 3] = color[3];
    }
}

// UI system to display the preview texture
pub fn display_world_preview(
    mut egui_context: ResMut<bevy_egui::EguiContext>,
    preview_texture: Res<WorldPreviewTexture>,
    world_gen_ui_state: Res<WorldGenUiState>,
    images: Res<Assets<Image>>,
) {
    // Only display if the UI is visible
    if !world_gen_ui_state.visible {
        return;
    }
    
    egui::Window::new("World Preview")
        .resizable(true)
        .show(egui_context.ctx_mut(), |ui| {
            if let Some(image) = images.get(&preview_texture.texture) {
                // Convert the Bevy image to an egui image
                let size = [preview_texture.width as f32, preview_texture.height as f32];
                let egui_texture_id = egui_context.add_image(&preview_texture.texture);
                
                // Display the image
                ui.image(egui_texture_id, size);
            } else {
                ui.label("Preview not available");
            }
        });
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

fn biome_to_color(biome: &BiomeType) -> [u8; 4] {
    match biome {
        BiomeType::DeepWater => [0, 0, 128, 255],
        BiomeType::ShallowWater => [0, 0, 255, 255],
        BiomeType::Beach => [240, 240, 64, 255],
        BiomeType::Desert => [255, 255, 128, 255],
        BiomeType::Grassland => [0, 255, 0, 255],
        BiomeType::Forest => [0, 128, 0, 255],
        BiomeType::DenseForest => [0, 64, 0, 255],
        BiomeType::Mountain => [128, 128, 128, 255],
        BiomeType::SnowCappedMountain => [255, 255, 255, 255],
        BiomeType::Tundra => [192, 192, 255, 255],
        BiomeType::Swamp => [64, 64, 0, 255],
    }
}

// Plugin to register all world preview systems
pub struct WorldPreviewPlugin;

impl Plugin for WorldPreviewPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldPreviewTexture>()
           .add_systems(Startup, init_world_preview_texture)
           .add_systems(Update, (update_world_preview_texture, display_world_preview));
    }
}