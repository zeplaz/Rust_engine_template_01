//! Map editor: [`MapEditorPlugin`], **TEMP-EGUI** palette, terrain brushes (M3), road markers (M4).
//!
//! ## Road markers (M4) — audit
//! - **Legacy ECS:** `src/entities/structure/components.rs` has **private** `Road` / `RoadSegment` / `RoadConnection` stubs (no world-gen spawn, not wired to runtime nav).
//! - **Editor v1 pattern:** [`MapEditorRoadMarkerV1`] — tile-aligned scaffold only, parented under the single [`WorldMarker`] root; **not** a full road graph. Future work maps markers → public `Road` / segments or snapshot DTO (M5+).
//! - See also [`map_editor_matrix_v1.md`](../../../../prompts/matrix/map_editor/map_editor_matrix_v1.md) §5.
//!
//! ## Tile / pick convention (M3-S01)
//! Matches [`crate::terrain::generation::world_generator_enhanced`] spawn layout:
//! - Grid column → `Transform.translation.x` (0 … `WorldGenParams.width - 1`).
//! - Grid row → `Transform.translation.z` (0 … `WorldGenParams.height - 1`).
//! - Normalized elevation → [`Height`] (0…1). World Y → `translation.y = Height.0 * HEIGHT_WORLD_SCALE`.
//! - Picking uses the **map minimap** texture: pixel `(px, py)` ↔ tile `(px, py)`. Off-map → no pick (`None`).
//!
//! ## Biome brush (M3-S03)
//! Sets [`TerrainType`] directly — **no** [`classify_biome`](crate::terrain::biome::classify_biome); manual paint only.

use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy_egui::egui::{self, Sense};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle};

use crate::engine::{BaseState, InGameEditorState, MainMenuState, WorldGenFlowState};
use crate::gui::editor::world_preview::preview_biome_rgba_for_tile;
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::biome::TerrainClass;
use crate::terrain::generation::world_generator_enhanced::{
    Height, TerrainType, TileMarker, WorldGenParams, WorldMarker,
};
use crate::terrain::material::{MaterialId, MaterialRegistry};

/// Tile-aligned **road placeholder** for map editor M4. Does not replace `entities::structure` `Road` stubs;
/// serialisation / nav parity is a later step (M5+).
#[derive(Component, Clone, Copy, Debug)]
pub struct MapEditorRoadMarkerV1 {
    pub tile_x: u32,
    pub tile_z: u32,
}

fn height_at_tile(tiles: &Query<(&Transform, &Height), With<TileMarker>>, tx: u32, tz: u32) -> f32 {
    for (tf, h) in tiles.iter() {
        if tf.translation.x.round() as u32 == tx && tf.translation.z.round() as u32 == tz {
            return h.0;
        }
    }
    0.0
}

fn despawn_road_markers_at(
    commands: &mut Commands,
    road_q: &Query<(Entity, &MapEditorRoadMarkerV1)>,
    tx: u32,
    tz: u32,
) {
    let victims: Vec<Entity> = road_q
        .iter()
        .filter(|(_, m)| m.tile_x == tx && m.tile_z == tz)
        .map(|(e, _)| e)
        .collect();
    for e in victims {
        commands.entity(e).despawn();
    }
}

fn place_road_marker(
    commands: &mut Commands,
    world_roots: &Query<Entity, With<WorldMarker>>,
    road_q: &Query<(Entity, &MapEditorRoadMarkerV1)>,
    tx: u32,
    tz: u32,
    height_normalized: f32,
) {
    let Ok(world_root) = world_roots.single() else {
        warn!("Map editor road: expected exactly one WorldMarker");
        return;
    };
    despawn_road_markers_at(commands, road_q, tx, tz);
    let y = height_normalized * HEIGHT_WORLD_SCALE + 0.25;
    commands.entity(world_root).with_children(|parent| {
        parent.spawn((
            MapEditorRoadMarkerV1 {
                tile_x: tx,
                tile_z: tz,
            },
            Transform::from_translation(Vec3::new(tx as f32, y, tz as f32)),
            Name::new(format!("Road marker v1 ({tx},{tz})")),
        ));
    });
}

/// Vertical exaggeration in world units; must stay in sync with world generator tile spawn.
pub const HEIGHT_WORLD_SCALE: f32 = 20.0;

/// Terrain tool sub-mode: height sculpt vs biome repaint.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum MapEditorTerrainPaint {
    #[default]
    Height,
    Biome,
}

/// Brush / tool kind for palettes; kept in sync with [`InGameEditorState`].
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum MapEditorToolKind {
    #[default]
    Select,
    Terrain,
    Road,
}

impl MapEditorToolKind {
    fn to_in_game(self) -> InGameEditorState {
        match self {
            MapEditorToolKind::Select => InGameEditorState::Select,
            MapEditorToolKind::Terrain => InGameEditorState::Terrain,
            MapEditorToolKind::Road => InGameEditorState::Road,
        }
    }

    const ALL: [Self; 3] = [Self::Select, Self::Terrain, Self::Road];

    fn label(self) -> &'static str {
        match self {
            MapEditorToolKind::Select => "Select",
            MapEditorToolKind::Terrain => "Terrain",
            MapEditorToolKind::Road => "Road",
        }
    }
}

#[derive(Resource, Clone)]
pub struct MapEditorTool {
    pub kind: MapEditorToolKind,
    pub brush_radius: f32,
    pub terrain_paint: MapEditorTerrainPaint,
    /// Biome used when `terrain_paint == Biome` (manual override only).
    pub paint_biome: TerrainClass,
}

impl Default for MapEditorTool {
    fn default() -> Self {
        Self {
            kind: MapEditorToolKind::default(),
            brush_radius: 3.0,
            terrain_paint: MapEditorTerrainPaint::default(),
            paint_biome: TerrainClass::Grassland,
        }
    }
}

fn sync_tool_to_substate(tool: &MapEditorTool, next_sub: &mut NextState<InGameEditorState>) {
    NextState::set_if_neq(next_sub, tool.kind.to_in_game());
}

fn on_enter_editor(mut tool: ResMut<MapEditorTool>, mut next_sub: ResMut<NextState<InGameEditorState>>) {
    *tool = MapEditorTool::default();
    NextState::set_if_neq(&mut *next_sub, InGameEditorState::Select);
}

/// Last-hovered tile from the minimap (`None` = off-map or not over minimap).
#[derive(Resource, Default)]
pub struct MapEditorHover {
    pub tile: Option<(u32, u32)>,
}

#[derive(Resource)]
pub struct MapEditorGridView {
    pub zoom: f32,
}

impl MapEditorGridView {
    pub const ZOOM_MIN: f32 = 0.02;
    pub const ZOOM_MAX: f32 = 32.0;
}

impl Default for MapEditorGridView {
    fn default() -> Self {
        Self { zoom: 1.0 }
    }
}

#[derive(Resource)]
pub struct MapEditorMapTexture {
    pub texture: Handle<Image>,
    pub width: u32,
    pub height: u32,
}

impl Default for MapEditorMapTexture {
    fn default() -> Self {
        Self {
            texture: Handle::default(),
            width: 0,
            height: 0,
        }
    }
}

fn rgba_map_image(width: u32, height: u32) -> Image {
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("map_editor_minimap"),
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
    let len = 4 * width as usize * height as usize;
    image.data = Some(vec![0; len]);
    image
}

fn map_editor_sync_map_texture_size(
    mut images: ResMut<Assets<Image>>,
    params: Res<WorldGenParams>,
    mut map_tex: ResMut<MapEditorMapTexture>,
) {
    if map_tex.width == params.width && map_tex.height == params.height {
        if images.get(&map_tex.texture).is_some() {
            return;
        }
    }

    let w = params.width;
    let h = params.height;
    let image = rgba_map_image(w, h);
    let new_handle = images.add(image);
    if map_tex.texture != Handle::default() {
        let _ = images.remove(map_tex.texture.id());
    }
    map_tex.texture = new_handle;
    map_tex.width = w;
    map_tex.height = h;
}

fn map_editor_raster_minimap(
    mut images: ResMut<Assets<Image>>,
    map_tex: Res<MapEditorMapTexture>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    tile_q: Query<(&Transform, &TerrainType), With<TileMarker>>,
    road_q: Query<&MapEditorRoadMarkerV1>,
) {
    let Some(image) = images.get_mut(&map_tex.texture) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let tex_w = map_tex.width as usize;
    let tex_h = map_tex.height as usize;
    let len = 4 * tex_w * tex_h;
    if data.len() != len {
        data.resize(len, 0);
    }
    data.fill(0);

    let mat_slices: Vec<(IVec2, bevy::math::UVec2, &[MaterialId])> = vec![];
    let reg_opt = materials.get(&handles.material_registry);

    for (transform, terrain) in tile_q.iter() {
        let x = transform.translation.x.round() as isize;
        let y = transform.translation.z.round() as isize;
        if x < 0 || y < 0 {
            continue;
        }
        let x = x as usize;
        let y = y as usize;
        if x >= tex_w || y >= tex_h {
            continue;
        }
        let pixel_index = 4 * (y * tex_w + x);
        if pixel_index + 3 >= data.len() {
            continue;
        }
        let color = match reg_opt {
            Some(reg) => preview_biome_rgba_for_tile(x as u32, y as u32, &terrain.0, &mat_slices, reg),
            None => editor_fallback_biome_rgba(&terrain.0),
        };
        data[pixel_index] = color[0];
        data[pixel_index + 1] = color[1];
        data[pixel_index + 2] = color[2];
        data[pixel_index + 3] = color[3];
    }

    const ROAD_OVERLAY: [u8; 4] = [255, 120, 0, 255];
    for marker in road_q.iter() {
        let x = marker.tile_x as usize;
        let y = marker.tile_z as usize;
        if x >= tex_w || y >= tex_h {
            continue;
        }
        let pixel_index = 4 * (y * tex_w + x);
        if pixel_index + 3 >= data.len() {
            continue;
        }
        data[pixel_index] = ROAD_OVERLAY[0];
        data[pixel_index + 1] = ROAD_OVERLAY[1];
        data[pixel_index + 2] = ROAD_OVERLAY[2];
        data[pixel_index + 3] = ROAD_OVERLAY[3];
    }
}

fn editor_fallback_biome_rgba(biome: &TerrainClass) -> [u8; 4] {
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

fn terrain_class_combo(ui: &mut egui::Ui, current: &mut TerrainClass) {
    egui::ComboBox::from_id_salt("map_editor_biome_pick")
        .selected_text(format!("{current:?}"))
        .show_ui(ui, |ui| {
            for c in ALL_TERRAIN_CLASSES {
                ui.selectable_value(current, c, format!("{c:?}"));
            }
        });
}

const ALL_TERRAIN_CLASSES: [TerrainClass; 16] = [
    TerrainClass::DeepWater,
    TerrainClass::ShallowWater,
    TerrainClass::Beach,
    TerrainClass::Desert,
    TerrainClass::Grassland,
    TerrainClass::Forest,
    TerrainClass::DenseForest,
    TerrainClass::Mountain,
    TerrainClass::SnowCappedMountain,
    TerrainClass::Tundra,
    TerrainClass::Swamp,
    TerrainClass::Cliff,
    TerrainClass::Concrete,
    TerrainClass::Dirt,
    TerrainClass::Snow,
    TerrainClass::Stone,
];

fn apply_brush_disk(
    tool: &MapEditorTool,
    center_x: u32,
    center_y: u32,
    tiles: &mut Query<
        (&mut Transform, &mut Height, &mut TerrainType),
        With<TileMarker>,
    >,
    height_delta_opt: Option<f32>,
) {
    let r = tool.brush_radius.max(1.0);
    let r2 = r * r;
    let cx = center_x as f32;
    let cy = center_y as f32;

    for (mut tf, mut height, mut terrain) in tiles.iter_mut() {
        let tx = tf.translation.x;
        let tz = tf.translation.z;
        let dx = tx - cx;
        let dz = tz - cy;
        if dx * dx + dz * dz > r2 {
            continue;
        }
        match tool.kind {
            MapEditorToolKind::Terrain => match tool.terrain_paint {
                MapEditorTerrainPaint::Height => {
                    if let Some(d) = height_delta_opt {
                        let v = (height.0 + d).clamp(0.0, 1.0);
                        height.0 = v;
                        tf.translation.y = v * HEIGHT_WORLD_SCALE;
                    }
                }
                MapEditorTerrainPaint::Biome => {
                    terrain.0 = tool.paint_biome;
                }
            },
            _ => {}
        }
    }
}

fn map_editor_minimap_window(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut hover: ResMut<MapEditorHover>,
    mut view: ResMut<MapEditorGridView>,
    map_tex: Res<MapEditorMapTexture>,
    tool: Res<MapEditorTool>,
    world_roots: Query<Entity, With<WorldMarker>>,
    road_entities: Query<(Entity, &MapEditorRoadMarkerV1)>,
    mut tile_queries: ParamSet<(
        Query<(&mut Transform, &mut Height, &mut TerrainType), With<TileMarker>>,
        Query<(&Transform, &Height), With<TileMarker>>,
    )>,
) -> Result {
    let texture_id = contexts.add_image(EguiTextureHandle::Strong(map_tex.texture.clone()));
    let tex_w = map_tex.width as f32;
    let tex_h = map_tex.height as f32;
    if tex_w < 1.0 || tex_h < 1.0 {
        hover.tile = None;
        return Ok(());
    }

    egui::Window::new("Map editor — minimap (pick / paint)")
        .resizable(true)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label(egui::RichText::new("TEMP-EGUI: one pixel ≈ one tile; Ctrl/⌘ + scroll to zoom.").weak());
            ui.small(format!(
                "Coordinates: x = column, z = row; Y = Height × {HEIGHT_WORLD_SCALE} (see module docs)."
            ));

            let z = view.zoom.clamp(MapEditorGridView::ZOOM_MIN, MapEditorGridView::ZOOM_MAX);
            view.zoom = z;
            let display_w = tex_w * z;
            let display_h = tex_h * z;

            egui::ScrollArea::both()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let sized = egui::load::SizedTexture::new(texture_id, [display_w, display_h]);
                    let resp = ui.add(
                        egui::Image::new(sized)
                            .corner_radius(0.0)
                            .sense(Sense::click_and_drag()),
                    );

                    hover.tile = None;
                    if let Some(pos) = resp.hover_pos() {
                        let rect = resp.rect;
                        let local = pos - rect.min;
                        if local.x >= 0.0 && local.y >= 0.0 && local.x < rect.width() && local.y < rect.height() {
                            let px = (local.x / z).floor() as i32;
                            let py = (local.y / z).floor() as i32;
                            if px >= 0 && py >= 0 && (px as u32) < map_tex.width && (py as u32) < map_tex.height {
                                hover.tile = Some((px as u32, py as u32));
                            }
                        }
                    }

                    if resp.hovered() {
                        let zoom_mod = ui.ctx().input(|i| i.modifiers.ctrl || i.modifiers.command);
                        let scroll = ui.ctx().input(|i| i.smooth_scroll_delta.y);
                        if zoom_mod && scroll != 0.0 {
                            view.zoom *= 1.0 + scroll * 0.002;
                            view.zoom = view
                                .zoom
                                .clamp(MapEditorGridView::ZOOM_MIN, MapEditorGridView::ZOOM_MAX);
                        }
                    }

                    let primary = ui.ctx().input(|i| i.pointer.primary_down());
                    let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta.y);
                    if let Some((cx, cy)) = hover.tile {
                        if tool.kind == MapEditorToolKind::Terrain {
                            let mut tiles = tile_queries.p0();
                            if primary {
                                match tool.terrain_paint {
                                    MapEditorTerrainPaint::Height => {
                                        apply_brush_disk(&tool, cx, cy, &mut tiles, Some(0.02));
                                    }
                                    MapEditorTerrainPaint::Biome => {
                                        apply_brush_disk(&tool, cx, cy, &mut tiles, None);
                                    }
                                }
                            } else if tool.terrain_paint == MapEditorTerrainPaint::Height
                                && resp.hovered()
                                && scroll_delta != 0.0
                            {
                                let step = (scroll_delta * 0.001).clamp(-0.08, 0.08);
                                apply_brush_disk(&tool, cx, cy, &mut tiles, Some(step));
                            }
                        } else if tool.kind == MapEditorToolKind::Road
                            && resp.clicked_by(egui::PointerButton::Primary)
                        {
                            let hn = {
                                let read = tile_queries.p1();
                                height_at_tile(&read, cx, cy)
                            };
                            place_road_marker(
                                &mut commands,
                                &world_roots,
                                &road_entities,
                                cx,
                                cy,
                                hn,
                            );
                        }
                    }
                });
        });

    Ok(())
}

fn map_editor_palette_system(
    mut contexts: EguiContexts,
    mut tool: ResMut<MapEditorTool>,
    mut next_base: ResMut<NextState<BaseState>>,
    mut next_flow: ResMut<NextState<WorldGenFlowState>>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
    mut next_sub: ResMut<NextState<InGameEditorState>>,
    sub_state: Res<State<InGameEditorState>>,
    hover: Res<MapEditorHover>,
) -> Result {
    egui::Window::new("Map editor — tools (TEMP-EGUI)")
        .anchor(egui::Align2::LEFT_TOP, [8.0, 8.0])
        .collapsible(true)
        .show(contexts.ctx_mut()?, |ui| {
            ui.label(egui::RichText::new("TEMP-EGUI tool palette; replace with Bevy UI per gui_runbook.").weak());
            ui.add_space(6.0);
            ui.label(format!("Sub-state: {:?}", sub_state.get()));
            if let Some((x, y)) = hover.tile {
                ui.label(format!("Hover tile: ({x}, {y})"));
            } else {
                ui.label("Hover tile: off-map");
            }

            let prev = tool.kind;
            ui.horizontal(|ui| {
                for k in MapEditorToolKind::ALL {
                    ui.radio_value(&mut tool.kind, k, k.label());
                }
            });
            if prev != tool.kind {
                sync_tool_to_substate(&tool, &mut next_sub);
            }

            if tool.kind == MapEditorToolKind::Terrain {
                ui.add_space(6.0);
                ui.label("Terrain paint:");
                ui.horizontal(|ui| {
                    ui.radio_value(
                        &mut tool.terrain_paint,
                        MapEditorTerrainPaint::Height,
                        "Height (drag / scroll)",
                    );
                    ui.radio_value(
                        &mut tool.terrain_paint,
                        MapEditorTerrainPaint::Biome,
                        "Biome (manual, no classify_biome)",
                    );
                });
                if tool.terrain_paint == MapEditorTerrainPaint::Biome {
                    terrain_class_combo(ui, &mut tool.paint_biome);
                }
            } else if tool.kind == MapEditorToolKind::Road {
                ui.add_space(6.0);
                ui.label("Road: click minimap tile to place/replace orange marker (TEMP-EGUI v1).");
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.label("Brush radius (tiles):");
                ui.add(egui::Slider::new(&mut tool.brush_radius, 1.0..=32.0));
            });
            ui.add_space(12.0);
            ui.horizontal(|ui| {
                if ui.button("Play (enter simulation)").clicked() {
                    NextState::set_if_neq(&mut *next_base, BaseState::Simulation);
                    NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
                }
                if ui.button("Exit to main menu").clicked() {
                    NextState::set_if_neq(&mut *next_base, BaseState::MainMenu);
                    NextState::set_if_neq(&mut *next_flow, WorldGenFlowState::Idle);
                    NextState::set_if_neq(&mut *next_menu, MainMenuState::MainMenu);
                }
            });
        });
    Ok(())
}

pub struct MapEditorPlugin;

impl Plugin for MapEditorPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<InGameEditorState>()
            .init_resource::<MapEditorTool>()
            .init_resource::<MapEditorHover>()
            .init_resource::<MapEditorGridView>()
            .init_resource::<MapEditorMapTexture>()
            .add_systems(OnEnter(BaseState::Editor), on_enter_editor)
            .add_systems(
                Update,
                (
                    map_editor_sync_map_texture_size,
                    map_editor_raster_minimap,
                )
                    .chain()
                    .run_if(in_state(BaseState::Editor)),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (
                    map_editor_minimap_window,
                    map_editor_palette_system,
                )
                    .chain()
                    .run_if(in_state(BaseState::Editor)),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn height_y_matches_generator_scale() {
        let h = 0.5_f32;
        assert!((h * HEIGHT_WORLD_SCALE - 10.0).abs() < f32::EPSILON);
    }
}
