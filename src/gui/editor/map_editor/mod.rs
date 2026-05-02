//! Map editor: [`MapEditorPlugin`], **TEMP-EGUI** palette, terrain brushes (M3), road markers (M4).
//!
//! ## Road markers (M4) — audit
//! - **Legacy ECS:** `src/entities/structure/components.rs` has **private** `Road` / `RoadSegment` / `RoadConnection` stubs (no world-gen spawn, not wired to runtime nav).
//! - **Editor v1 pattern:** [`MapEditorRoadMarkerV1`] — tile-aligned scaffold; **`placement_seq`** preserves **click order** for bake (R9). Do not lexicographically sort tiles for transport graph building.
//! - See also [`map_editor_matrix_v1.md`](../../../../prompts/matrix/map_editor/map_editor_matrix_v1.md) §5 · **R9 bake order:** [`../../../../prompts/matrix/transport/runbook/r9_authoring_bake_order_steps_v1.md`](../../../../prompts/matrix/transport/runbook/r9_authoring_bake_order_steps_v1.md).
//! - **G4 dev:** Road tool — **Save / Load transport (dev JSON)** → `assets/saves/dev_transport_network.json` (paths via `CARGO_MANIFEST_DIR`).
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

use std::path::{Path, PathBuf};
use std::sync::Arc;

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
use crate::systems::transport::{
    bake_snapshot_from_ordered_tile_markers, hydrate_transport_from_snapshot,
    transport_network_snapshot_from_world, transport_network_snapshot_save_json_path,
    LoadTransportNetworkSnapshotFromDisk, TransportEdgeDirectory, TransportFieldStore,
    TransportLastHydratedSnapshot, TransportTopology,
};
use crate::terrain::material::{MaterialId, MaterialRegistry};

/// Request: build **W1** transport topology from current [`MapEditorRoadMarkerV1`] entities.
#[derive(Message)]
pub struct MapEditorBakeTransportRequest;

/// **G4** dev: write `TransportNetworkSnapshot` JSON under `assets/saves/` (crate root at compile time).
#[derive(Message)]
pub struct MapEditorSaveDevTransportRequest;

/// **G4** dev: load same path via [`LoadTransportNetworkSnapshotFromDisk`].
#[derive(Message)]
pub struct MapEditorLoadDevTransportRequest;

fn dev_transport_network_save_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/saves/dev_transport_network.json")
}

/// Monotonic **click order** for the current editor session (reset when entering editor).
/// Drives bake polyline order — **R9**; see `r9_authoring_bake_order_steps_v1.md`.
#[derive(Resource, Default, Debug)]
pub struct MapEditorRoadPlacementSeq {
    pub next: u32,
}

/// Tile-aligned **road placeholder** for map editor M4. Does not replace `entities::structure` `Road` stubs;
/// `placement_seq` is **authoring order** for [`bake_snapshot_from_ordered_tile_markers`] (not lexicographic).
#[derive(Component, Clone, Copy, Debug)]
pub struct MapEditorRoadMarkerV1 {
    pub tile_x: u32,
    pub tile_z: u32,
    pub placement_seq: u32,
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
    placement: &mut MapEditorRoadPlacementSeq,
    tx: u32,
    tz: u32,
    height_normalized: f32,
) {
    let Ok(world_root) = world_roots.single() else {
        warn!("Map editor road: expected exactly one WorldMarker");
        return;
    };
    despawn_road_markers_at(commands, road_q, tx, tz);
    let seq = placement.next;
    placement.next = placement.next.saturating_add(1);
    let y = height_normalized * HEIGHT_WORLD_SCALE + 0.25;
    commands.entity(world_root).with_children(|parent| {
        parent.spawn((
            MapEditorRoadMarkerV1 {
                tile_x: tx,
                tile_z: tz,
                placement_seq: seq,
            },
            Transform::from_translation(Vec3::new(tx as f32, y, tz as f32)),
            Name::new(format!("Road marker v1 ({tx},{tz}) seq={seq}")),
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

fn on_enter_editor(
    mut tool: ResMut<MapEditorTool>,
    mut next_sub: ResMut<NextState<InGameEditorState>>,
    mut road_seq: ResMut<MapEditorRoadPlacementSeq>,
) {
    *tool = MapEditorTool::default();
    *road_seq = MapEditorRoadPlacementSeq::default();
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
    mut road_placement: ResMut<MapEditorRoadPlacementSeq>,
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
                                &mut *road_placement,
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

fn map_editor_bake_transport(
    mut events: MessageReader<MapEditorBakeTransportRequest>,
    markers: Query<&MapEditorRoadMarkerV1>,
    tiles: Query<(&Transform, &Height), With<TileMarker>>,
    mut topology: ResMut<TransportTopology>,
    mut fields: ResMut<TransportFieldStore>,
    mut directory: ResMut<TransportEdgeDirectory>,
    mut last_hydrated: ResMut<TransportLastHydratedSnapshot>,
) {
    for _ in events.read() {
        let mut rows: Vec<(u32, u32, u32)> = markers
            .iter()
            .map(|m| (m.placement_seq, m.tile_x, m.tile_z))
            .collect();
        rows.sort_by_key(|(seq, _, _)| *seq);
        let coords: Vec<(u32, u32)> = rows.into_iter().map(|(_, x, z)| (x, z)).collect();
        let snap = bake_snapshot_from_ordered_tile_markers(
            &coords,
            |x, z| height_at_tile(&tiles, x, z),
            HEIGHT_WORLD_SCALE,
            0.25,
        );
        if snap.edges.is_empty() {
            warn!("Bake transport: need ≥2 markers after removing consecutive duplicates on same tile.");
            continue;
        }
        match hydrate_transport_from_snapshot(&mut topology, &mut fields, &mut directory, &snap) {
            Ok(()) => {
                last_hydrated.snapshot = Some(snap);
            }
            Err(e) => warn!("Bake transport hydrate failed: {e:?}"),
        }
    }
}

fn map_editor_dev_save_transport(
    mut events: MessageReader<MapEditorSaveDevTransportRequest>,
    last: Res<TransportLastHydratedSnapshot>,
    topology: Res<TransportTopology>,
    directory: Res<TransportEdgeDirectory>,
) {
    for _ in events.read() {
        let snap = last
            .snapshot
            .clone()
            .or_else(|| transport_network_snapshot_from_world(&topology, &directory));
        let Some(snap) = snap else {
            warn!("Save transport: bake or load a graph first (nothing to save).");
            continue;
        };
        let path = dev_transport_network_save_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match transport_network_snapshot_save_json_path(&snap, &path) {
            Ok(()) => info!("Saved transport R8 JSON to {}", path.display()),
            Err(e) => warn!("Save transport failed: {e:?}"),
        }
    }
}

fn map_editor_dev_load_transport(
    mut events: MessageReader<MapEditorLoadDevTransportRequest>,
    mut load_tx: MessageWriter<LoadTransportNetworkSnapshotFromDisk>,
) {
    for _ in events.read() {
        let path = dev_transport_network_save_path();
        let s: String = path.to_string_lossy().into_owned();
        load_tx.write(LoadTransportNetworkSnapshotFromDisk {
            path: Arc::from(s.into_boxed_str()),
        });
    }
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
    mut bake_events: MessageWriter<MapEditorBakeTransportRequest>,
    mut save_dev_transport: MessageWriter<MapEditorSaveDevTransportRequest>,
    mut load_dev_transport: MessageWriter<MapEditorLoadDevTransportRequest>,
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
                if ui
                    .button("Bake roads → transport graph (W1 / R8 hydrate)")
                    .on_hover_text("Markers in click order → TransportTopology; needs ≥2 markers after dedup.")
                    .clicked()
                {
                    bake_events.write(MapEditorBakeTransportRequest);
                }
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui
                        .button("Save transport (dev JSON)")
                        .on_hover_text(format!("Writes {}", dev_transport_network_save_path().display()))
                        .clicked()
                    {
                        save_dev_transport.write(MapEditorSaveDevTransportRequest);
                    }
                    if ui
                        .button("Load transport (dev JSON)")
                        .on_hover_text(format!("Reads {}", dev_transport_network_save_path().display()))
                        .clicked()
                    {
                        load_dev_transport.write(MapEditorLoadDevTransportRequest);
                    }
                });
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
            .add_message::<MapEditorBakeTransportRequest>()
            .add_message::<MapEditorSaveDevTransportRequest>()
            .add_message::<MapEditorLoadDevTransportRequest>()
            .init_resource::<MapEditorTool>()
            .init_resource::<MapEditorRoadPlacementSeq>()
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
                Update,
                (
                    map_editor_bake_transport,
                    map_editor_dev_save_transport,
                    map_editor_dev_load_transport,
                )
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
