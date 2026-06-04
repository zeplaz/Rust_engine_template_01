//! Map editor: [`MapEditorPlugin`], **TEMP-EGUI** palette, terrain brushes (M3), road markers (M4).
//!
//! ## Road markers (M4) — audit
//! - **Legacy ECS:** `src/entities/structure/components.rs` has **private** `Road` / `RoadSegment` / `RoadConnection` stubs (no world-gen spawn, not wired to runtime nav).
//! - **Editor v1 pattern:** [`MapEditorRoadMarkerV1`] — tile-aligned scaffold; **`placement_seq`** preserves **click order** for bake (R9). Do not lexicographically sort tiles for transport graph building.
//! - See also [`map_editor_matrix_v1.md`](../../../../prompts/matrix/map_editor/map_editor_matrix_v1.md) §5 · **R9 bake order:** [`../../../../prompts/matrix/transport/runbook/r9_authoring_bake_order_steps_v1.md`](../../../../prompts/matrix/transport/runbook/r9_authoring_bake_order_steps_v1.md).
//! - **G4 dev:** Road tool — **Save / Load transport (dev RON)** → `assets/saves/dev_transport_network.ron` (paths via `CARGO_MANIFEST_DIR`). `.json` fixtures still load when path ends in `.json`. Hydrating transport updates [`CorridorConstructionBook`](../../../strategic/construction_book.rs).
//! - **M5:** **Save / Load map (RON)** → `assets/saves/maps/last.ron` (`crate::terrain::editor::map_snapshot`).
//! - **Scenario Wave 2–4:** `scenario_script_panel` + **Scenario tools** entry window — `*.scenario.ron`, `RegisterObjectives` / `ScenarioObjectiveMarker`.
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

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy_egui::egui::{self, Sense};
use bevy_egui::{EguiContexts, EguiPrimaryContextPass, EguiTextureHandle};

use crate::engine::{AppState, BaseState, InGameEditorState, MainMenuState, WorldGenFlowState};
use crate::gui::std_floating;
use crate::gui::ui_gates::map_editor_chrome_active;
use crate::gui::editor::editor_world_commit_bridge::{
    write_editor_world_grid_commit, EditorTileEditCommitted, EditorTileEditKind,
};
use crate::gui::editor::scenario_script_panel::{
    scenario_editor_tools_entry_window, scenario_script_panel_system,
    toggle_scenario_script_panel_hotkey, ScenarioScriptPanelState,
};
use crate::gui::style::{
    framed_group, muted_label, path_hint, primary_label, section_heading, v_space, weak_body,
    widget_scroll_both, widget_scroll_vertical_fill, CmdHeadingStyle, UiPalette, VertSpace,
};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::editor::map_snapshot::{
    load_map_snapshot_from_ron, map_snapshot_v1_to_v2, MapSnapshotCellV1, MapSnapshotV1,
    MAP_SNAPSHOT_SCHEMA_VERSION,
};
use crate::terrain::family::{TerrainFamilyId, TerrainFamilyRegistry, DEFAULT_TERRAIN_FAMILY_ID};
use crate::terrain::generation::polygon_world_semantics::MacroStrategicKind;
use crate::terrain::generation::world_generator_enhanced::{
    despawn_generated_world_entities, Height, Moisture, Temperature, TerrainType, TileMarker,
    TileRegionIndex, WorldGenParams, WorldMarker,
};
use crate::terrain::generation::brush_tile_inclusive_bounds;
use crate::io::snapshot::{read_hybrid_world_snapshot_dev_v0, write_hybrid_world_snapshot_dev_v0};
use crate::strategic::{
    apply_corridor_book_from_transport_snapshot, transport_construction_records_from_book,
    CorridorConstructionBook,
};
use crate::systems::transport::{
    bake_snapshot_from_ordered_markers_with_world_positions, hydrate_transport_from_snapshot_text,
    hydrate_transport_from_snapshot, transport_network_snapshot_from_world_with_construction, transport_network_snapshot_save_ron_path,
    transport_network_snapshot_to_ron_string,
    LoadTransportNetworkSnapshotFromDisk, TransportEdgeDirectory, TransportFieldStore,
    TransportLastHydratedSnapshot, TransportNetworkSnapshot, TransportTopology,
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

/// **M5 / wave S** stub: write hybrid-shaped dev snapshot (JSON header line + transport JSON body).
#[derive(Message)]
pub struct MapEditorSaveHybridWorldDevRequest;

/// Load transport body from [`dev_hybrid_world_save_path`] after validating header.
#[derive(Message)]
pub struct MapEditorLoadHybridWorldDevRequest;

/// **M5:** save or load terrain grid snapshot at `assets/saves/maps/last.ron`.
#[derive(Message, Clone, Copy)]
pub enum MapEditorMapSnapshotIoRequest {
    Save,
    Load,
}

/// **R9:** undo last road stroke (stack captured **before** each mouse-down on the minimap).
#[derive(Message)]
pub struct MapEditorRoadUndoRequest;

fn dev_transport_network_save_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/saves/dev_transport_network.ron")
}

fn dev_hybrid_world_save_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/saves/dev_world_hybrid_v0.sav")
}

fn dev_map_snapshot_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/saves/maps/last.ron")
}

/// Live **preview** polyline from markers (R9 ghost) — not hydrated until **Bake**.
#[derive(Resource, Clone, Debug, Default)]
pub struct RoadAuthoringGhostPreview {
    pub snapshot: Option<TransportNetworkSnapshot>,
}

/// One undo frame: full marker set **before** a placement action.
#[derive(Clone, Debug, Default)]
pub struct RoadMarkerUndoFrame {
    pub entries: Vec<(u32, u32, u32, Vec3)>,
}

impl RoadMarkerUndoFrame {
    fn capture(
        q: &Query<(&MapEditorRoadMarkerV1, &Transform), Without<TileMarker>>,
    ) -> Self {
        let mut rows: Vec<_> = q
            .iter()
            .map(|(m, t)| (m.placement_seq, m.tile_x, m.tile_z, t.translation))
            .collect();
        rows.sort_by_key(|(seq, _, _, _)| *seq);
        Self {
            entries: rows
                .into_iter()
                .map(|(seq, tx, tz, pos)| (seq, tx, tz, pos))
                .collect(),
        }
    }
}

#[derive(Resource, Debug)]
pub struct MapEditorRoadUndoStack {
    pub frames: Vec<RoadMarkerUndoFrame>,
    pub max_frames: usize,
}

impl Default for MapEditorRoadUndoStack {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            max_frames: 50,
        }
    }
}

impl MapEditorRoadUndoStack {
    fn push_frame(&mut self, frame: RoadMarkerUndoFrame) {
        while self.frames.len() >= self.max_frames {
            self.frames.remove(0);
        }
        self.frames.push(frame);
    }
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

fn height_at_tile(
    tiles: &Query<
        (&Transform, &Height),
        (With<TileMarker>, Without<MapEditorRoadMarkerV1>),
    >,
    tx: u32,
    tz: u32,
) -> f32 {
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

/// Terrain brush footprint in the XZ tile plane (column = x, row = z).
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum MapEditorBrushShape {
    #[default]
    Disk,
    Square,
    Diamond,
}

impl MapEditorBrushShape {
    const ALL: [Self; 3] = [Self::Disk, Self::Square, Self::Diamond];

    fn label(self) -> &'static str {
        match self {
            Self::Disk => "Disk",
            Self::Square => "Square",
            Self::Diamond => "Diamond",
        }
    }
}

#[inline]
fn tile_in_brush(
    shape: MapEditorBrushShape,
    cx: f32,
    cy: f32,
    tx: f32,
    tz: f32,
    r: f32,
) -> bool {
    let dx = tx - cx;
    let dz = tz - cy;
    match shape {
        MapEditorBrushShape::Disk => dx * dx + dz * dz <= r * r,
        MapEditorBrushShape::Square => dx.abs() <= r && dz.abs() <= r,
        MapEditorBrushShape::Diamond => dx.abs() + dz.abs() <= r,
    }
}

/// Raster-ordered grid cells from `(x0,y0)` to `(x1,y1)` inclusive (tile column, tile row).
fn bresenham_tile_line(x0: u32, y0: u32, x1: u32, y1: u32) -> Vec<(u32, u32)> {
    let mut out = Vec::new();
    let xa = x0 as i32;
    let ya = y0 as i32;
    let xb = x1 as i32;
    let yb = y1 as i32;
    let dx = (xb - xa).abs();
    let dy = -(yb - ya).abs();
    let sx = if xa < xb { 1 } else { -1 };
    let sy = if ya < yb { 1 } else { -1 };
    let mut err = dx + dy;
    let mut x = xa;
    let mut y = ya;
    loop {
        if x >= 0 && y >= 0 {
            out.push((x as u32, y as u32));
        }
        if x == xb && y == yb {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    out
}

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
    /// Footprint placement — not yet implemented (see palette copy).
    Building,
    /// Curves / tiles distinct from roads — not yet implemented.
    Rail,
}

impl MapEditorToolKind {
    fn to_in_game(self) -> InGameEditorState {
        match self {
            MapEditorToolKind::Select => InGameEditorState::Select,
            MapEditorToolKind::Terrain => InGameEditorState::Terrain,
            MapEditorToolKind::Road => InGameEditorState::Road,
            MapEditorToolKind::Building => InGameEditorState::Create,
            MapEditorToolKind::Rail => InGameEditorState::Rail,
        }
    }

    const ALL: [Self; 5] = [
        Self::Select,
        Self::Terrain,
        Self::Road,
        Self::Building,
        Self::Rail,
    ];

    fn label(self) -> &'static str {
        match self {
            MapEditorToolKind::Select => "Select",
            MapEditorToolKind::Terrain => "Terrain",
            MapEditorToolKind::Road => "Road",
            MapEditorToolKind::Building => "Building (stub)",
            MapEditorToolKind::Rail => "Rail (stub)",
        }
    }
}

#[derive(Resource, Clone)]
pub struct MapEditorTool {
    pub kind: MapEditorToolKind,
    pub brush_radius: f32,
    pub brush_shape: MapEditorBrushShape,
    pub terrain_paint: MapEditorTerrainPaint,
    /// Biome family (manual override only) — dense id into [`TerrainFamilyRegistry`].
    pub paint_biome: TerrainFamilyId,
}

impl Default for MapEditorTool {
    fn default() -> Self {
        Self {
            kind: MapEditorToolKind::default(),
            brush_radius: 3.0,
            brush_shape: MapEditorBrushShape::default(),
            terrain_paint: MapEditorTerrainPaint::default(),
            paint_biome: DEFAULT_TERRAIN_FAMILY_ID,
        }
    }
}

fn sync_tool_to_substate(tool: &MapEditorTool, next_sub: &mut NextState<InGameEditorState>) {
    NextState::set_if_neq(next_sub, tool.kind.to_in_game());
}

#[inline]
fn emit_editor_tile_commit_for_brush(
    edit_commits: &mut MessageWriter<EditorTileEditCommitted>,
    params: &WorldGenParams,
    cx: u32,
    cy: u32,
    radius: f32,
    kind: EditorTileEditKind,
) {
    if params.width == 0 || params.height == 0 {
        return;
    }
    let (mut min, mut max) = brush_tile_inclusive_bounds(cx, cy, radius);
    let mx = params.width - 1;
    let mz = params.height - 1;
    min.x = min.x.min(mx);
    min.y = min.y.min(mz);
    max.x = max.x.min(mx);
    max.y = max.y.min(mz);
    edit_commits.write(EditorTileEditCommitted {
        min_tile: min,
        max_tile: max,
        kind,
    });
}

fn on_enter_editor(
    app: Res<State<AppState>>,
    mut tool: ResMut<MapEditorTool>,
    mut next_sub: ResMut<NextState<InGameEditorState>>,
    mut road_seq: ResMut<MapEditorRoadPlacementSeq>,
    mut undo: ResMut<MapEditorRoadUndoStack>,
    mut ghost: ResMut<RoadAuthoringGhostPreview>,
    mut road_drag: ResMut<MapEditorRoadDragState>,
    mut minimap_dirty: ResMut<MapEditorMinimapRasterDirty>,
) {
    if matches!(
        *app.get(),
        AppState::WorldGen | AppState::InGame | AppState::Paused
    ) {
        return;
    }
    *tool = MapEditorTool::default();
    *road_seq = MapEditorRoadPlacementSeq::default();
    *undo = MapEditorRoadUndoStack::default();
    *ghost = RoadAuthoringGhostPreview::default();
    *road_drag = MapEditorRoadDragState::default();
    minimap_dirty.bump();
    NextState::set_if_neq(&mut *next_sub, InGameEditorState::Select);
}

/// While primary is held, extends polyline road placement between hovered minimap tiles.
#[derive(Resource, Default)]
pub struct MapEditorRoadDragState {
    pub last_hover_tile: Option<(u32, u32)>,
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
    /// Stable egui binding; cleared when [`Self::texture`] is recreated (`P0` — no `add_image` every frame).
    pub egui_texture_cache: Option<(Handle<Image>, egui::TextureId)>,
}

impl Default for MapEditorMapTexture {
    fn default() -> Self {
        Self {
            texture: Handle::default(),
            width: 0,
            height: 0,
            egui_texture_cache: None,
        }
    }
}

/// **P0** dirty epoch for the map-editor CPU minimap — avoid full-grid raster every `Update` tick.
#[derive(Resource, Debug, Clone, Copy, Default)]
pub struct MapEditorMinimapRasterDirty {
    revision: u64,
}

impl MapEditorMinimapRasterDirty {
    #[inline]
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.revision
    }

    #[inline]
    pub fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1);
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
    mut raster_dirty: ResMut<MapEditorMinimapRasterDirty>,
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
    map_tex.egui_texture_cache = None;
    raster_dirty.bump();
}

fn mark_map_editor_minimap_dirty(
    mut dirty: ResMut<MapEditorMinimapRasterDirty>,
    added_tiles: Query<(), Added<TileMarker>>,
    changed_terrain: Query<(), (With<TileMarker>, Changed<TerrainType>)>,
    changed_height: Query<(), (With<TileMarker>, Changed<Height>)>,
    added_roads: Query<(), Added<MapEditorRoadMarkerV1>>,
    changed_roads: Query<(), Changed<MapEditorRoadMarkerV1>>,
    handles: Res<TerrainRegistriesHandles>,
) {
    if added_tiles.iter().next().is_some()
        || changed_terrain.iter().next().is_some()
        || changed_height.iter().next().is_some()
        || added_roads.iter().next().is_some()
        || changed_roads.iter().next().is_some()
        || handles.is_changed()
    {
        dirty.bump();
    }
}

fn map_editor_raster_minimap(
    mut images: ResMut<Assets<Image>>,
    map_tex: Res<MapEditorMapTexture>,
    handles: Res<TerrainRegistriesHandles>,
    materials: Res<Assets<MaterialRegistry>>,
    tile_q: Query<(&Transform, &TerrainType), With<TileMarker>>,
    road_q: Query<&MapEditorRoadMarkerV1>,
    raster_dirty: Res<MapEditorMinimapRasterDirty>,
    mut last_applied_revision: Local<Option<u64>>,
) {
    let rev = raster_dirty.revision();
    if *last_applied_revision == Some(rev) {
        return;
    }
    let Some(image) = images.get_mut(&map_tex.texture) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let tex_w = map_tex.width as usize;
    let tex_h = map_tex.height as usize;

    let mat_slices: Vec<(IVec2, bevy::math::UVec2, &[MaterialId])> = vec![];
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

    crate::gui::map_tile_raster::raster_tiles_and_roads_to_rgba(
        data,
        tex_w,
        tex_h,
        tile_iter,
        road_iter,
        &mat_slices,
        reg_opt,
        fam_opt,
    );
    *last_applied_revision = Some(rev);
}

fn terrain_family_combo(ui: &mut egui::Ui, current: &mut TerrainFamilyId) {
    let reg = crate::terrain::default_terrain_families();
    let sel = reg.def(*current).map(|d| d.name.as_str()).unwrap_or("?");
    egui::ComboBox::from_id_salt("map_editor_biome_pick")
        .selected_text(sel)
        .show_ui(ui, |ui| {
            for (i, def) in reg.families.iter().enumerate() {
                let id = TerrainFamilyId(i as u16);
                ui.selectable_value(current, id, def.name.as_str());
            }
        });
}

fn apply_terrain_brush(
    tool: &MapEditorTool,
    center_x: u32,
    center_y: u32,
    tiles: &mut Query<
        (&mut Transform, &mut Height, &mut TerrainType),
        (With<TileMarker>, Without<MapEditorRoadMarkerV1>),
    >,
    height_delta_opt: Option<f32>,
) {
    let r = tool.brush_radius.max(1.0);
    let cx = center_x as f32;
    let cy = center_y as f32;

    for (mut tf, mut height, mut terrain) in tiles.iter_mut() {
        let tx = tf.translation.x;
        let tz = tf.translation.z;
        if !tile_in_brush(tool.brush_shape, cx, cy, tx, tz, r) {
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
    mut map_tex: ResMut<MapEditorMapTexture>,
    tool: Res<MapEditorTool>,
    params: Res<WorldGenParams>,
    world_roots: Query<Entity, With<WorldMarker>>,
    road_entities: Query<(Entity, &MapEditorRoadMarkerV1)>,
    road_tf: Query<(&MapEditorRoadMarkerV1, &Transform), Without<TileMarker>>,
    mut road_undo: ResMut<MapEditorRoadUndoStack>,
    mut road_placement: ResMut<MapEditorRoadPlacementSeq>,
    mut road_drag: ResMut<MapEditorRoadDragState>,
    mut tile_queries: ParamSet<(
        Query<
            (&mut Transform, &mut Height, &mut TerrainType),
            (With<TileMarker>, Without<MapEditorRoadMarkerV1>),
        >,
        Query<
            (&Transform, &Height),
            (With<TileMarker>, Without<MapEditorRoadMarkerV1>),
        >,
    )>,
    mut edit_commits: MessageWriter<EditorTileEditCommitted>,
    palette: Res<UiPalette>,
) -> Result {
    let handle = map_tex.texture.clone();
    let cache_hit = map_tex
        .egui_texture_cache
        .as_ref()
        .and_then(|(h, id)| (*h == handle).then_some(*id));
    let texture_id = if let Some(id) = cache_hit {
        id
    } else {
        let id = contexts.add_image(EguiTextureHandle::Strong(handle.clone()));
        map_tex.egui_texture_cache = Some((handle, id));
        id
    };
    let tex_w = map_tex.width as f32;
    let tex_h = map_tex.height as f32;
    if tex_w < 1.0 || tex_h < 1.0 {
        hover.tile = None;
        return Ok(());
    }

    std_floating(egui::Window::new("Map editor — minimap (pick / paint)"))
        .default_size(egui::vec2(640.0, 520.0))
        .show(contexts.ctx_mut()?, |ui| {
            let pal: &UiPalette = &*palette;
            weak_body(
                ui,
                pal,
                "TEMP-EGUI: one pixel ≈ one tile; Ctrl/⌘ + scroll to zoom. Road: click–drag on minimap to stroke a polyline (Ctrl/⌘+Z undoes last stroke).",
            );
            muted_label(
                ui,
                pal,
                format!(
                    "Coordinates: x = column, z = row; Y = Height × {HEIGHT_WORLD_SCALE} (see module docs)."
                ),
            );

            let z = view.zoom.clamp(MapEditorGridView::ZOOM_MIN, MapEditorGridView::ZOOM_MAX);
            view.zoom = z;
            let display_w = tex_w * z;
            let display_h = tex_h * z;

            widget_scroll_both("map_editor_minimap_scroll").show(ui, |ui| {
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
                                        apply_terrain_brush(&tool, cx, cy, &mut tiles, Some(0.02));
                                        emit_editor_tile_commit_for_brush(
                                            &mut edit_commits,
                                            &params,
                                            cx,
                                            cy,
                                            tool.brush_radius,
                                            EditorTileEditKind::TerrainHeight,
                                        );
                                    }
                                    MapEditorTerrainPaint::Biome => {
                                        apply_terrain_brush(&tool, cx, cy, &mut tiles, None);
                                        emit_editor_tile_commit_for_brush(
                                            &mut edit_commits,
                                            &params,
                                            cx,
                                            cy,
                                            tool.brush_radius,
                                            EditorTileEditKind::TerrainBiome,
                                        );
                                    }
                                }
                            } else if tool.terrain_paint == MapEditorTerrainPaint::Height
                                && resp.hovered()
                                && scroll_delta != 0.0
                            {
                                let step = (scroll_delta * 0.001).clamp(-0.08, 0.08);
                                apply_terrain_brush(&tool, cx, cy, &mut tiles, Some(step));
                                emit_editor_tile_commit_for_brush(
                                    &mut edit_commits,
                                    &params,
                                    cx,
                                    cy,
                                    tool.brush_radius,
                                    EditorTileEditKind::TerrainHeight,
                                );
                            }
                        } else if tool.kind == MapEditorToolKind::Road {
                            let just_pressed = ui.ctx().input(|i| i.pointer.primary_pressed());
                            let primary_down = ui.ctx().input(|i| i.pointer.primary_down());

                            if just_pressed {
                                let hn = {
                                    let read = tile_queries.p1();
                                    height_at_tile(&read, cx, cy)
                                };
                                let before = RoadMarkerUndoFrame::capture(&road_tf);
                                road_undo.push_frame(before);
                                place_road_marker(
                                    &mut commands,
                                    &world_roots,
                                    &road_entities,
                                    &mut *road_placement,
                                    cx,
                                    cy,
                                    hn,
                                );
                                if params.width > 0 && params.height > 0 {
                                    edit_commits.write(EditorTileEditCommitted {
                                        min_tile: UVec2::new(cx, cy),
                                        max_tile: UVec2::new(cx, cy),
                                        kind: EditorTileEditKind::RoadMarker,
                                    });
                                }
                                road_drag.last_hover_tile = Some((cx, cy));
                            } else if primary_down {
                                if let Some((lx, ly)) = road_drag.last_hover_tile {
                                    if (lx, ly) != (cx, cy) {
                                        let line = bresenham_tile_line(lx, ly, cx, cy);
                                        let read = tile_queries.p1();
                                        for (tx, tz) in line.into_iter().skip(1) {
                                            let hn = height_at_tile(&read, tx, tz);
                                            place_road_marker(
                                                &mut commands,
                                                &world_roots,
                                                &road_entities,
                                                &mut *road_placement,
                                                tx,
                                                tz,
                                                hn,
                                            );
                                            if params.width > 0 && params.height > 0 {
                                                edit_commits.write(EditorTileEditCommitted {
                                                    min_tile: UVec2::new(tx, tz),
                                                    max_tile: UVec2::new(tx, tz),
                                                    kind: EditorTileEditKind::RoadMarker,
                                                });
                                            }
                                        }
                                        road_drag.last_hover_tile = Some((cx, cy));
                                    }
                                }
                            }
                        }
                    }
                    if tool.kind == MapEditorToolKind::Road && !primary {
                        road_drag.last_hover_tile = None;
                    }
                });
        });

    Ok(())
}

fn map_editor_bake_transport(
    mut events: MessageReader<MapEditorBakeTransportRequest>,
    markers: Query<(&MapEditorRoadMarkerV1, &Transform)>,
    mut topology: ResMut<TransportTopology>,
    mut fields: ResMut<TransportFieldStore>,
    mut directory: ResMut<TransportEdgeDirectory>,
    mut last_hydrated: ResMut<TransportLastHydratedSnapshot>,
    params: Res<WorldGenParams>,
    mut edit_commits: MessageWriter<EditorTileEditCommitted>,
) {
    for _ in events.read() {
        let mut rows: Vec<(u32, u32, u32, Vec3)> = markers
            .iter()
            .map(|(m, t)| (m.placement_seq, m.tile_x, m.tile_z, t.translation))
            .collect();
        rows.sort_by_key(|(seq, _, _, _)| *seq);
        let with_pos: Vec<(u32, u32, Vec3)> =
            rows.into_iter().map(|(_, x, z, p)| (x, z, p)).collect();
        let snap = bake_snapshot_from_ordered_markers_with_world_positions(&with_pos);
        if snap.edges.is_empty() {
            warn!("Bake transport: need ≥2 markers after removing consecutive duplicates on same tile.");
            continue;
        }
        match hydrate_transport_from_snapshot(&mut topology, &mut fields, &mut directory, &snap) {
            Ok(()) => {
                last_hydrated.snapshot = Some(snap);
                write_editor_world_grid_commit(
                    &mut edit_commits,
                    &params,
                    EditorTileEditKind::TransportTopology,
                );
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
    book: Res<CorridorConstructionBook>,
) {
    for _ in events.read() {
        let construction = transport_construction_records_from_book(&book, &topology);
        let snap = transport_network_snapshot_from_world_with_construction(
            &topology,
            &directory,
            construction,
        )
        .or_else(|| last.snapshot.clone());
        let Some(snap) = snap else {
            warn!("Save transport: bake or load a graph first (nothing to save).");
            continue;
        };
        let path = dev_transport_network_save_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match transport_network_snapshot_save_ron_path(&snap, &path) {
            Ok(()) => info!("Saved transport R8 RON to {}", path.display()),
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

fn road_authoring_ghost_refresh(
    base: Res<State<BaseState>>,
    tool: Res<MapEditorTool>,
    markers: Query<(&MapEditorRoadMarkerV1, &Transform)>,
    mut ghost: ResMut<RoadAuthoringGhostPreview>,
) {
    if base.get() != &BaseState::Editor || tool.kind != MapEditorToolKind::Road {
        ghost.snapshot = None;
        return;
    }
    let mut rows: Vec<(u32, u32, u32, Vec3)> = markers
        .iter()
        .map(|(m, t)| (m.placement_seq, m.tile_x, m.tile_z, t.translation))
        .collect();
    rows.sort_by_key(|(seq, _, _, _)| *seq);
    let with_pos: Vec<(u32, u32, Vec3)> = rows.into_iter().map(|(_, x, z, p)| (x, z, p)).collect();
    let snap = bake_snapshot_from_ordered_markers_with_world_positions(&with_pos);
    ghost.snapshot = if snap.edges.is_empty() {
        None
    } else {
        Some(snap)
    };
}

fn map_editor_road_undo(
    mut events: MessageReader<MapEditorRoadUndoRequest>,
    mut commands: Commands,
    world_roots: Query<Entity, With<WorldMarker>>,
    road_entities: Query<(Entity, &MapEditorRoadMarkerV1)>,
    mut stack: ResMut<MapEditorRoadUndoStack>,
    mut placement: ResMut<MapEditorRoadPlacementSeq>,
) {
    for _ in events.read() {
        let Some(frame) = stack.frames.pop() else {
            continue;
        };
        let Ok(world_root) = world_roots.single() else {
            warn!("Map editor undo: expected exactly one WorldMarker");
            continue;
        };
        let to_remove: Vec<Entity> = road_entities.iter().map(|(e, _)| e).collect();
        for e in to_remove {
            commands.entity(e).despawn();
        }
        for (seq, tx, tz, pos) in &frame.entries {
            commands.entity(world_root).with_children(|parent| {
                parent.spawn((
                    MapEditorRoadMarkerV1 {
                        tile_x: *tx,
                        tile_z: *tz,
                        placement_seq: *seq,
                    },
                    Transform::from_translation(*pos),
                    Name::new(format!("Road marker v1 ({tx},{tz}) seq={seq}")),
                ));
            });
        }
        placement.next = frame
            .entries
            .iter()
            .map(|(s, _, _, _)| *s)
            .max()
            .map(|m| m.saturating_add(1))
            .unwrap_or(0);
    }
}

fn map_editor_dev_save_hybrid_world(
    mut events: MessageReader<MapEditorSaveHybridWorldDevRequest>,
    last: Res<TransportLastHydratedSnapshot>,
    topology: Res<TransportTopology>,
    directory: Res<TransportEdgeDirectory>,
    book: Res<CorridorConstructionBook>,
) {
    for _ in events.read() {
        let construction = transport_construction_records_from_book(&book, &topology);
        let snap = transport_network_snapshot_from_world_with_construction(
            &topology,
            &directory,
            construction,
        )
        .or_else(|| last.snapshot.clone());
        let Some(snap) = snap else {
            warn!("Save hybrid world: bake or load a graph first (nothing to save).");
            continue;
        };
        let path = dev_hybrid_world_save_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let ron = match transport_network_snapshot_to_ron_string(&snap) {
            Ok(s) => s,
            Err(e) => {
                warn!("Save hybrid: RON error {e:?}");
                continue;
            }
        };
        match write_hybrid_world_snapshot_dev_v0(&path, ron.as_bytes()) {
            Ok(()) => info!("Saved hybrid dev snapshot to {}", path.display()),
            Err(e) => warn!("Save hybrid failed: {e:?}"),
        }
    }
}

fn map_editor_dev_load_hybrid_world(
    mut events: MessageReader<MapEditorLoadHybridWorldDevRequest>,
    mut topology: ResMut<TransportTopology>,
    mut fields: ResMut<TransportFieldStore>,
    mut directory: ResMut<TransportEdgeDirectory>,
    mut last: ResMut<TransportLastHydratedSnapshot>,
    mut book: ResMut<CorridorConstructionBook>,
) {
    for _ in events.read() {
        let path = dev_hybrid_world_save_path();
        let (header, body) = match read_hybrid_world_snapshot_dev_v0(&path) {
            Ok(x) => x,
            Err(e) => {
                warn!("Load hybrid failed for {}: {e:?}", path.display());
                continue;
            }
        };
        let text = match std::str::from_utf8(&body) {
            Ok(s) => s,
            Err(e) => {
                warn!("Load hybrid: body not UTF-8: {e:?}");
                continue;
            }
        };
        match hydrate_transport_from_snapshot_text(
            topology.as_mut(),
            fields.as_mut(),
            directory.as_mut(),
            text,
        ) {
            Ok(snap) => {
                apply_corridor_book_from_transport_snapshot(
                    book.as_mut(),
                    directory.as_ref(),
                    &snap,
                );
                last.snapshot = Some(snap);
                info!(
                    "Loaded hybrid dev transport ({} bytes, header v{})",
                    header.transport_byte_len, header.format_version
                );
            }
            Err(e) => warn!("Load hybrid hydrate failed: {e:?}"),
        }
    }
}

fn map_editor_map_snapshot_io(
    mut events: MessageReader<MapEditorMapSnapshotIoRequest>,
    mut commands: Commands,
    mut params: ResMut<WorldGenParams>,
    tiles: Query<(&Transform, &Height, &TerrainType), With<TileMarker>>,
    roads: Query<&MapEditorRoadMarkerV1>,
    fam_assets: Res<Assets<TerrainFamilyRegistry>>,
    handles: Res<TerrainRegistriesHandles>,
    mut road_placement: ResMut<MapEditorRoadPlacementSeq>,
    mut road_undo: ResMut<MapEditorRoadUndoStack>,
    mut ghost: ResMut<RoadAuthoringGhostPreview>,
    world_q: Query<Entity, With<WorldMarker>>,
    road_entities: Query<(Entity, &MapEditorRoadMarkerV1)>,
    mut edit_commits: MessageWriter<EditorTileEditCommitted>,
) {
    for req in events.read() {
        match *req {
            MapEditorMapSnapshotIoRequest::Save => {
                let Some(reg) = fam_assets.get(&handles.terrain_families) else {
                    warn!("Save map snapshot: terrain family registry not loaded.");
                    continue;
                };
                let w = params.width;
                let h = params.height;
                if w == 0 || h == 0 {
                    warn!("Save map snapshot: world dimensions are zero.");
                    continue;
                }
                let mut grid: Vec<Option<(f32, TerrainFamilyId)>> = vec![None; (w * h) as usize];
                for (tf, he, terr) in &tiles {
                    let x = tf.translation.x.round() as i32;
                    let z = tf.translation.z.round() as i32;
                    if x < 0 || z < 0 {
                        continue;
                    }
                    let x = x as u32;
                    let z = z as u32;
                    if x >= w || z >= h {
                        continue;
                    }
                    let i = (z * w + x) as usize;
                    grid[i] = Some((he.0, terr.0));
                }
                let mut road_tiles = HashSet::new();
                for m in &roads {
                    road_tiles.insert((m.tile_x, m.tile_z));
                }
                let mut cells = Vec::with_capacity((w * h) as usize);
                for z in 0..h {
                    for x in 0..w {
                        let i = (z * w + x) as usize;
                        let (height, tid) = grid[i].unwrap_or((0.0, DEFAULT_TERRAIN_FAMILY_ID));
                        let terrain_family = reg
                            .def(tid)
                            .map(|d| d.name.clone())
                            .unwrap_or_else(|| "Grassland".to_string());
                        cells.push(MapSnapshotCellV1 {
                            height,
                            terrain_family,
                            road: road_tiles.contains(&(x, z)),
                        });
                    }
                }
                let snap = MapSnapshotV1 {
                    schema_version: MAP_SNAPSHOT_SCHEMA_VERSION,
                    width: w,
                    height: h,
                    cells,
                };
                let snap_v2 = map_snapshot_v1_to_v2(&snap);
                let path = dev_map_snapshot_path();
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                match snap_v2.to_ron_string() {
                    Ok(s) => match std::fs::write(&path, format!("{}\n", s.trim_end())) {
                        Ok(()) => info!("Saved map snapshot to {}", path.display()),
                        Err(e) => warn!("Save map snapshot failed: {e:?}"),
                    },
                    Err(e) => warn!("Save map snapshot RON: {e:?}"),
                }
            }
            MapEditorMapSnapshotIoRequest::Load => {
                let path = dev_map_snapshot_path();
                let bytes = match std::fs::read(&path) {
                    Ok(b) => b,
                    Err(e) => {
                        warn!("Load map snapshot: read {}: {e:?}", path.display());
                        continue;
                    }
                };
                let text = match std::str::from_utf8(&bytes) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Load map snapshot: UTF-8: {e:?}");
                        continue;
                    }
                };
                let snap = match load_map_snapshot_from_ron(text) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Load map snapshot: RON: {e}");
                        continue;
                    }
                };
                if let Err(e) = snap.validate() {
                    warn!("Load map snapshot: {e}");
                    continue;
                }
                let Some(reg) = fam_assets.get(&handles.terrain_families) else {
                    warn!("Load map snapshot: terrain family registry not loaded.");
                    continue;
                };
                for (e, _) in road_entities.iter() {
                    commands.entity(e).despawn();
                }
                despawn_generated_world_entities(&mut commands, &world_q);
                params.width = snap.width;
                params.height = snap.height;
                *road_placement = MapEditorRoadPlacementSeq::default();
                road_undo.frames.clear();
                *ghost = RoadAuthoringGhostPreview::default();

                let world_root = commands
                    .spawn((WorldMarker, Name::new("Map snapshot world")))
                    .id();

                let w = snap.width;
                let h = snap.height;
                let mut idx = 0usize;
                for z in 0..h {
                    for x in 0..w {
                        let cell = &snap.cells[idx];
                        idx += 1;
                        let tid = match reg.require_id(&cell.terrain_family) {
                            Ok(id) => id,
                            Err(_) => {
                                warn!(
                                    "Load map snapshot: unknown terrain family {:?}, using Grassland",
                                    cell.terrain_family
                                );
                                DEFAULT_TERRAIN_FAMILY_ID
                            }
                        };
                        let tile_e = commands
                            .spawn((
                                TileMarker,
                                TileRegionIndex(0),
                                Transform::from_translation(Vec3::new(
                                    x as f32,
                                    cell.height * HEIGHT_WORLD_SCALE,
                                    z as f32,
                                )),
                                Height(cell.height),
                                Moisture(0.5),
                                Temperature(0.5),
                                TerrainType(tid),
                                MacroStrategicKind::default(),
                                Name::new(format!("Tile ({x}, {z})")),
                            ))
                            .id();
                        commands.entity(world_root).add_child(tile_e);
                    }
                }

                idx = 0;
                for z in 0..h {
                    for x in 0..w {
                        let cell = &snap.cells[idx];
                        idx += 1;
                        if !cell.road {
                            continue;
                        }
                        let seq = road_placement.next;
                        road_placement.next = road_placement.next.saturating_add(1);
                        let y = cell.height * HEIGHT_WORLD_SCALE + 0.25;
                        commands.entity(world_root).with_children(|parent| {
                            parent.spawn((
                                MapEditorRoadMarkerV1 {
                                    tile_x: x,
                                    tile_z: z,
                                    placement_seq: seq,
                                },
                                Transform::from_translation(Vec3::new(x as f32, y, z as f32)),
                                Name::new(format!("Road marker v1 ({x},{z}) seq={seq}")),
                            ));
                        });
                    }
                }

                info!("Loaded map snapshot {}×{} from {}", w, h, path.display());
                write_editor_world_grid_commit(
                    &mut edit_commits,
                    &params,
                    EditorTileEditKind::MapSnapshotImport,
                );
            }
        }
    }
}

fn map_editor_palette_system(
    mut contexts: EguiContexts,
    mut tool: ResMut<MapEditorTool>,
    mut next_base: ResMut<NextState<BaseState>>,
    mut next_flow: ResMut<NextState<WorldGenFlowState>>,
    mut next_menu: ResMut<NextState<MainMenuState>>,
    mut next_sub: ResMut<NextState<InGameEditorState>>,
    hover: Res<MapEditorHover>,
    ghost: Res<RoadAuthoringGhostPreview>,
    palette: Res<UiPalette>,
    mut bake_events: MessageWriter<MapEditorBakeTransportRequest>,
    mut save_dev_transport: MessageWriter<MapEditorSaveDevTransportRequest>,
    mut load_dev_transport: MessageWriter<MapEditorLoadDevTransportRequest>,
    mut save_hybrid: MessageWriter<MapEditorSaveHybridWorldDevRequest>,
    mut load_hybrid: MessageWriter<MapEditorLoadHybridWorldDevRequest>,
    mut map_snapshot_io: MessageWriter<MapEditorMapSnapshotIoRequest>,
    mut road_undo: MessageWriter<MapEditorRoadUndoRequest>,
) -> Result {
    std_floating(egui::Window::new("Map editor — tools (TEMP-EGUI)"))
        .anchor(egui::Align2::LEFT_TOP, [8.0, 8.0])
        .default_size(egui::vec2(360.0, 720.0))
        .collapsible(true)
        .show(contexts.ctx_mut()?, |ui| {
            let pal: &UiPalette = &*palette;
            // `UiSpacing` resource matches `Default`; local avoids extra `SystemParam` (system at cap).
            let spacing = crate::gui::UiSpacing::default();
            let sp = &spacing;
            weak_body(
                ui,
                pal,
                "TEMP-EGUI tool palette; replace with Bevy UI per gui_runbook.",
            );
            v_space(ui, sp, VertSpace::Inter);
            widget_scroll_vertical_fill("map_editor_tools_scroll", ui.available_height()).show(ui, |ui| {
            framed_group(ui, pal, |ui| {
                section_heading(ui, pal, CmdHeadingStyle::Gt, "Chunk Settings");
                path_hint(ui, pal, "/assets/scenarios/test.ron");
                v_space(ui, sp, VertSpace::Xs);
                primary_label(ui, pal, format!("Active tool: {:?}", tool.kind));
                if let Some((x, y)) = hover.tile {
                    primary_label(ui, pal, format!("Hover tile: ({x}, {y})"));
                } else {
                    muted_label(ui, pal, "Hover tile: off-map");
                }
            });
            v_space(ui, sp, VertSpace::Inter);

            let prev = tool.kind;
            ui.horizontal_wrapped(|ui| {
                for k in MapEditorToolKind::ALL {
                    ui.radio_value(&mut tool.kind, k, k.label());
                }
            });
            if prev != tool.kind {
                sync_tool_to_substate(&tool, &mut next_sub);
            }

            if tool.kind == MapEditorToolKind::Terrain {
                v_space(ui, sp, VertSpace::Inter);
                primary_label(ui, pal, "Terrain paint:");
                ui.horizontal_wrapped(|ui| {
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
                    terrain_family_combo(ui, &mut tool.paint_biome);
                }
                v_space(ui, sp, VertSpace::Xs);
                primary_label(ui, pal, "Brush footprint (XZ tile plane):");
                ui.horizontal_wrapped(|ui| {
                    for s in MapEditorBrushShape::ALL {
                        ui.radio_value(&mut tool.brush_shape, s, s.label());
                    }
                });
            } else if tool.kind == MapEditorToolKind::Road {
                v_space(ui, sp, VertSpace::Inter);
                weak_body(
                    ui,
                    pal,
                    "Road: click–drag on the minimap to stroke a polyline (orange markers). Single click still works.",
                );
                match ghost.snapshot.as_ref() {
                    Some(s) => weak_body(
                        ui,
                        pal,
                        format!(
                            "Ghost preview (not baked): {} edges — bake to hydrate runtime.",
                            s.edges.len()
                        ),
                    ),
                    None => weak_body(ui, pal, "Ghost preview: need ≥2 markers after dedup."),
                };
                let key_undo = ui.ctx().input(|i| {
                    i.key_pressed(egui::Key::Z) && (i.modifiers.ctrl || i.modifiers.command)
                });
                if key_undo {
                    road_undo.write(MapEditorRoadUndoRequest);
                }
                ui.horizontal(|ui| {
                    if ui
                        .button("Undo last road stroke")
                        .on_hover_text(
                            "Restores all road markers to before this click–drag (stack ≤50). Ctrl/⌘+Z",
                        )
                        .clicked()
                    {
                        road_undo.write(MapEditorRoadUndoRequest);
                    }
                });
                if ui
                    .button("Bake roads → transport graph (W1 / R8 hydrate)")
                    .on_hover_text("Markers in click order → TransportTopology; needs ≥2 markers after dedup.")
                    .clicked()
                {
                    bake_events.write(MapEditorBakeTransportRequest);
                }
                v_space(ui, sp, VertSpace::Xs);
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
                ui.horizontal(|ui| {
                    if ui
                        .button("Save hybrid world (dev)")
                        .on_hover_text(format!(
                            "M5/S stub: JSON header + transport JSON body → {}",
                            dev_hybrid_world_save_path().display()
                        ))
                        .clicked()
                    {
                        save_hybrid.write(MapEditorSaveHybridWorldDevRequest);
                    }
                    if ui
                        .button("Load hybrid world (dev)")
                        .on_hover_text(format!("Reads {}", dev_hybrid_world_save_path().display()))
                        .clicked()
                    {
                        load_hybrid.write(MapEditorLoadHybridWorldDevRequest);
                    }
                });
            } else if matches!(
                tool.kind,
                MapEditorToolKind::Building | MapEditorToolKind::Rail
            ) {
                v_space(ui, sp, VertSpace::Inter);
                weak_body(
                    ui,
                    pal,
                    "Stub tool — no map paint yet. Buildings: spawn via production/manufacturing flows when wired. Rails: use Road markers + Bake transport for now; dedicated rail curves are planned.",
                );
            }

            v_space(ui, sp, VertSpace::Sm);
            ui.horizontal(|ui| {
                primary_label(ui, pal, "Brush radius (tiles):");
                ui.add(egui::Slider::new(&mut tool.brush_radius, 1.0..=32.0));
            });
            muted_label(
                ui,
                pal,
                "Brush radius and footprint apply to the Terrain tool only.",
            );
            v_space(ui, sp, VertSpace::Sm);
            ui.horizontal(|ui| {
                if ui
                    .button("Save map grid (M5 RON)")
                    .on_hover_text(format!("Writes {}", dev_map_snapshot_path().display()))
                    .clicked()
                {
                    map_snapshot_io.write(MapEditorMapSnapshotIoRequest::Save);
                }
                if ui
                    .button("Load map grid (M5 RON)")
                    .on_hover_text(format!("Reads {}", dev_map_snapshot_path().display()))
                    .clicked()
                {
                    map_snapshot_io.write(MapEditorMapSnapshotIoRequest::Load);
                }
            });

            v_space(ui, sp, VertSpace::Md);
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
            .add_message::<MapEditorSaveHybridWorldDevRequest>()
            .add_message::<MapEditorLoadHybridWorldDevRequest>()
            .add_message::<MapEditorMapSnapshotIoRequest>()
            .add_message::<MapEditorRoadUndoRequest>()
            .init_resource::<MapEditorTool>()
            .init_resource::<MapEditorRoadPlacementSeq>()
            .init_resource::<MapEditorRoadDragState>()
            .init_resource::<MapEditorRoadUndoStack>()
            .init_resource::<RoadAuthoringGhostPreview>()
            .init_resource::<MapEditorHover>()
            .init_resource::<MapEditorGridView>()
            .init_resource::<MapEditorMapTexture>()
            .init_resource::<MapEditorMinimapRasterDirty>()
            .init_resource::<ScenarioScriptPanelState>()
            .add_systems(OnEnter(BaseState::Editor), on_enter_editor)
            .add_systems(
                Update,
                (
                    map_editor_sync_map_texture_size,
                    mark_map_editor_minimap_dirty,
                    map_editor_raster_minimap,
                )
                    .chain()
                    .run_if(map_editor_chrome_active),
            )
            .add_systems(
                Update,
                (
                    road_authoring_ghost_refresh,
                    map_editor_road_undo,
                    map_editor_bake_transport,
                    map_editor_dev_save_transport,
                    map_editor_dev_load_transport,
                    map_editor_dev_save_hybrid_world,
                    map_editor_dev_load_hybrid_world,
                    map_editor_map_snapshot_io,
                )
                    .run_if(map_editor_chrome_active),
            )
            .add_systems(
                Update,
                toggle_scenario_script_panel_hotkey.run_if(map_editor_chrome_active),
            )
            .add_systems(
                EguiPrimaryContextPass,
                map_editor_minimap_window.run_if(map_editor_chrome_active),
            )
            .add_systems(
                EguiPrimaryContextPass,
                map_editor_palette_system.run_if(map_editor_chrome_active),
            )
            .add_systems(
                EguiPrimaryContextPass,
                scenario_editor_tools_entry_window.run_if(map_editor_chrome_active),
            )
            .add_systems(
                EguiPrimaryContextPass,
                scenario_script_panel_system.run_if(map_editor_chrome_active),
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
