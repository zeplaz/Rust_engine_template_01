//! Raster [`TileMarker`] worlds to a single sprite when chunk tilemaps are absent (default build has no `bevy_tilemap_adapter`).
//!
//! Without this, generated tiles have no mesh/material and the main camera shows nothing.

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages};

use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::engine::BaseState;
use crate::gui::MainWorldCamera;
use crate::gui::map_tile_raster::raster_tiles_and_roads_to_rgba;
use crate::gui::std_floating;
use crate::gui::editor::map_editor::MapEditorRoadMarkerV1;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::world_generator_enhanced::{TerrainType, TileMarker, WorldGenParams};
use crate::terrain::material::MaterialRegistry;

/// Marks the full-map fallback sprite entity.
#[derive(Component)]
pub struct TileWorldFallbackSprite;

#[derive(Resource, Default)]
pub struct TileWorldFallbackState {
    pub sprite_entity: Option<Entity>,
    pub image: Handle<Image>,
    pub last_w: u32,
    pub last_h: u32,
    #[cfg(feature = "bevy_tilemap_adapter")]
    pub suppressed: bool,
}

#[derive(Resource)]
pub struct SimMinimapUiState {
    pub open: bool,
}

impl Default for SimMinimapUiState {
    fn default() -> Self {
        Self { open: true }
    }
}

pub struct TileWorldFallbackPlugin;

impl Plugin for TileWorldFallbackPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TileWorldFallbackState>()
            .init_resource::<SimMinimapUiState>()
            .add_systems(
                OnEnter(BaseState::Simulation),
                focus_main_camera_on_world_params,
            )
            .add_systems(
                OnEnter(BaseState::Editor),
                focus_main_camera_on_world_params,
            )
            .add_systems(
                Update,
                (
                    tile_world_fallback_sync_spawner,
                    tile_world_fallback_rasterize,
                )
                    .chain(),
            )
            .add_systems(EguiPrimaryContextPass, simulation_minimap_egui_window);
    }
}

fn focus_main_camera_on_world_params(
    mut cam: Query<&mut Transform, With<MainWorldCamera>>,
    params: Res<WorldGenParams>,
) {
    if params.width == 0 || params.height == 0 {
        return;
    }
    let cx = params.width as f32 * 0.5;
    let cy = params.height as f32 * 0.5;
    for mut t in cam.iter_mut() {
        t.translation.x = cx;
        t.translation.y = cy;
    }
}

fn make_rgba_image(w: u32, h: u32) -> Image {
    let size = Extent3d {
        width: w,
        height: h,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("tile_world_fallback"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        ..default()
    };
    let len = 4 * w as usize * h as usize;
    image.data = Some(vec![0u8; len]);
    image
}

fn tile_world_fallback_sync_spawner(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    params: Res<WorldGenParams>,
    tiles: Query<(), With<TileMarker>>,
    #[cfg(feature = "bevy_tilemap_adapter")] chunk_maps: Query<(), With<crate::render::ChunkTilemaps>>,
    mut state: ResMut<TileWorldFallbackState>,
) {
    #[cfg(feature = "bevy_tilemap_adapter")]
    let active = !tiles.is_empty() && chunk_maps.is_empty();
    #[cfg(not(feature = "bevy_tilemap_adapter"))]
    let active = !tiles.is_empty();

    if !active {
        if let Some(e) = state.sprite_entity.take() {
            commands.entity(e).despawn();
        }
        state.last_w = 0;
        state.last_h = 0;
        state.image = Handle::default();
        return;
    }

    let w = params.width;
    let h = params.height;

    let need_new = state.sprite_entity.is_none() || state.last_w != w || state.last_h != h;
    if need_new {
        if let Some(e) = state.sprite_entity.take() {
            commands.entity(e).despawn();
        }
        let image = images.add(make_rgba_image(w, h));
        let e = commands
            .spawn((
                TileWorldFallbackSprite,
                Sprite {
                    image: image.clone(),
                    custom_size: Some(Vec2::new(w as f32, h as f32)),
                    ..default()
                },
                Transform::from_xyz(w as f32 * 0.5, h as f32 * 0.5, 0.0),
            ))
            .id();
        state.sprite_entity = Some(e);
        state.image = image;
        state.last_w = w;
        state.last_h = h;
    }
}

fn tile_world_fallback_rasterize(
    mut images: ResMut<Assets<Image>>,
    state: Res<TileWorldFallbackState>,
    base: Res<State<BaseState>>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    tile_q: Query<(&Transform, &TerrainType), With<TileMarker>>,
    road_q: Query<&MapEditorRoadMarkerV1>,
) {
    if !matches!(base.get(), BaseState::Simulation | BaseState::Editor) {
        return;
    }
    if state.sprite_entity.is_none() || state.image == Handle::default() {
        return;
    }
    let Some(image) = images.get_mut(&state.image) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let tex_w = state.last_w as usize;
    let tex_h = state.last_h as usize;
    if tex_w == 0 || tex_h == 0 {
        return;
    }

    let mat_slices: Vec<(bevy::math::IVec2, bevy::math::UVec2, &[crate::terrain::material::MaterialId])> =
        vec![];
    let reg_opt = materials.get(&handles.material_registry);
    let fam_opt = Some(crate::terrain::default_terrain_families());

    let tile_iter = tile_q.iter().filter_map(|(tf, terrain)| {
        let x = tf.translation.x.round() as isize;
        let y = tf.translation.z.round() as isize;
        if x < 0 || y < 0 {
            return None;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= tex_w || y >= tex_h {
            return None;
        }
        Some((x, y, terrain.0))
    });
    let road_iter = road_q.iter().map(|m| (m.tile_x as usize, m.tile_z as usize));

    raster_tiles_and_roads_to_rgba(
        data,
        tex_w,
        tex_h,
        tile_iter,
        road_iter,
        &mat_slices,
        reg_opt,
        fam_opt,
    );
}

fn simulation_minimap_egui_window(
    mut contexts: EguiContexts,
    base: Res<State<BaseState>>,
    mut minimap_ui: ResMut<SimMinimapUiState>,
    fallback: Res<TileWorldFallbackState>,
) -> Result {
    if !matches!(base.get(), BaseState::Simulation) {
        return Ok(());
    }
    if !minimap_ui.open || fallback.sprite_entity.is_none() || fallback.image == Handle::default() {
        return Ok(());
    }

    let tex_id = contexts.add_image(bevy_egui::EguiTextureHandle::Strong(fallback.image.clone()));
    let w = fallback.last_w as f32;
    let h = fallback.last_h as f32;

    std_floating(egui::Window::new("Minimap"))
        .id(egui::Id::new("sim_minimap_fallback"))
        .default_size([320.0, 280.0])
        .min_size([180.0, 160.0])
        .open(&mut minimap_ui.open)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label(egui::RichText::new("Overworld preview · close to hide").small().weak());
            let max_side = 280.0;
            let scale = (max_side / w.max(h).max(1.0)).min(1.0);
            let dw = (w * scale).max(64.0);
            let dh = (h * scale).max(64.0);
            ui.add(egui::Image::new(egui::load::SizedTexture::new(tex_id, [dw, dh])));
        });
    Ok(())
}
