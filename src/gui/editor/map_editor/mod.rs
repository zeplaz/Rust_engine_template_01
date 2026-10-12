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

mod brush;
mod road_markers;
mod transport_io;

pub use brush::{MapEditorBrushShape, MapEditorTerrainPaint, MapEditorTool, MapEditorToolKind};
pub use road_markers::{
    MapEditorRoadDragState, MapEditorRoadMarkerV1, MapEditorRoadPlacementSeq,
    MapEditorRoadUndoRequest, MapEditorRoadUndoStack, RoadAuthoringGhostPreview, RoadMarkerUndoFrame,
};
pub use transport_io::{
    MapEditorBakeTransportRequest, MapEditorLoadDevTransportRequest, MapEditorLoadHybridWorldDevRequest,
    MapEditorMapSnapshotIoRequest, MapEditorSaveDevTransportRequest, MapEditorSaveHybridWorldDevRequest,
};

use bevy::ecs::hierarchy::ChildSpawnerCommands;
use bevy::math::IVec2;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::ui::widget::ImageNode;
use bevy::ui::FocusPolicy;

use crate::engine::{AppState, BaseState, InGameEditorState};
use crate::gui::style::{CmdUiMonoFont, UiPalette};
use crate::gui::{map_editor_chrome_active, PanelPresence};
use crate::gui::editor::scenario_script_panel::{
    toggle_scenario_script_panel_hotkey, ScenarioScriptPanelState,
};
use crate::systems::terrain::TerrainRegistriesHandles;
use crate::terrain::generation::world_generator_enhanced::{
    Height, TerrainType, TileMarker, WorldGenParams,
};
use crate::terrain::material::{MaterialId, MaterialRegistry};

use brush::{sync_tool_to_substate, tile_in_brush};
use road_markers::{
    height_at_tile, map_editor_road_undo, place_road_marker, road_authoring_ghost_refresh,
};
use transport_io::{
    dev_hybrid_world_save_path, dev_map_snapshot_path, dev_transport_network_save_path,
    map_editor_bake_transport, map_editor_dev_load_hybrid_world, map_editor_dev_load_transport,
    map_editor_dev_save_hybrid_world, map_editor_dev_save_transport, map_editor_map_snapshot_io,
};

/// Vertical exaggeration in world units; must stay in sync with world generator tile spawn.
pub const HEIGHT_WORLD_SCALE: f32 = 20.0;

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
    pub egui_texture_cache: Option<(Handle<Image>, ())>,
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
    let Some(mut image) = images.get_mut(&map_tex.texture) else {
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

/// Single map-editor chrome root: palette | minimap | scenario (F10).
#[derive(Component)]
pub struct MapEditorBevyShell;

#[derive(Component)]
struct MapEditorPaletteBody;

#[derive(Component)]
struct MapEditorMinimapImage;

#[derive(Component)]
struct MapEditorMinimapFallbackText;

#[derive(Component)]
struct MapEditorScenarioBody;

#[derive(Component, Clone, Copy)]
struct MapEditorPalettePick(MapEditorToolKind);

#[derive(Default)]
struct MapEditorShellStamp {
    key: u64,
}

/// Retired egui body — Bevy sync owns chrome via [`sync_map_editor_bevy_shell`].
fn map_editor_minimap_window() -> Result {
    Ok(())
}

/// Retired egui body — Bevy sync owns chrome via [`sync_map_editor_bevy_shell`].
fn map_editor_palette_system() -> Result {
    Ok(())
}

fn map_editor_shell_stamp(
    tool: &MapEditorTool,
    map_tex: &MapEditorMapTexture,
    scenario: &ScenarioScriptPanelState,
    image_ready: bool,
) -> u64 {
    let mut key = 0u64;
    for b in format!("{:?}", tool.kind).bytes() {
        key = key.wrapping_mul(31).wrapping_add(b as u64);
    }
    key = key
        .wrapping_mul(31)
        .wrapping_add(map_tex.width as u64)
        .wrapping_mul(31)
        .wrapping_add(map_tex.height as u64)
        .wrapping_add(image_ready as u64)
        .wrapping_add(scenario.window_open as u64)
        .wrapping_add(scenario.tools_entry_visible as u64);
    for b in scenario.file_path.bytes() {
        key = key.wrapping_mul(31).wrapping_add(b as u64);
    }
    for b in scenario.status_line.bytes() {
        key = key.wrapping_mul(31).wrapping_add(b as u64);
    }
    key
}

fn spawn_palette_button(
    parent: &mut ChildSpawnerCommands,
    palette: &UiPalette,
    tf: &TextFont,
    kind: MapEditorToolKind,
    active: bool,
) {
    let label = if active {
        format!("▶ {}", kind.label())
    } else {
        kind.label().to_string()
    };
    parent
        .spawn((
            MapEditorPalettePick(kind),
            Button,
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(if active {
                palette.bevy_button_idle()
            } else {
                palette.bevy_hud_panel_fill()
            }),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Block,
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label),
                tf.clone(),
                TextColor(palette.bevy_primary_text()),
            ));
        });
}

/// Bevy map-editor chrome (palette + minimap ImageNode/Text + scenario F10).
pub fn sync_map_editor_bevy_shell(
    mut commands: Commands,
    tool: Res<MapEditorTool>,
    map_tex: Res<MapEditorMapTexture>,
    images: Res<Assets<Image>>,
    scenario: Res<ScenarioScriptPanelState>,
    palette: Res<UiPalette>,
    mono: Res<CmdUiMonoFont>,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<MapEditorBevyShell>>,
    mut stamp: Local<MapEditorShellStamp>,
) {
    presence.map_editor_shell_root = !roots.is_empty();
    let image_ready = map_tex.texture != Handle::default() && images.get(&map_tex.texture).is_some();
    let next = map_editor_shell_stamp(tool.as_ref(), map_tex.as_ref(), scenario.as_ref(), image_ready);
    if !roots.is_empty() && stamp.key == next {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }

    let tf = TextFont::from_font_size(12.0).with_font(mono.0.clone());
    let scenario_text = if scenario.window_open {
        format!(
            "Scenario (F10)\n  path: {}\n  status: {}\n  tools_entry={}",
            scenario.file_path,
            if scenario.status_line.is_empty() {
                "(idle)"
            } else {
                scenario.status_line.as_str()
            },
            scenario.tools_entry_visible
        )
    } else {
        "Scenario (F10) — closed · press F10 or use tools entry".into()
    };

    commands
        .spawn((
            MapEditorBevyShell,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(40.0),
                right: Val::Px(12.0),
                width: Val::Px(300.0),
                max_height: Val::Px(640.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::ZERO,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            BackgroundColor(palette.bevy_hud_panel_fill()),
            BorderColor::all(palette.bevy_wire_magenta()),
            FocusPolicy::Block,
            ZIndex(920),
        ))
        .with_children(|root| {
            root.spawn((
                MapEditorPaletteBody,
                Text::new("Map editor · Palette"),
                tf.clone(),
                TextColor(palette.bevy_primary_text()),
            ));
            for kind in MapEditorToolKind::ALL {
                spawn_palette_button(root, &palette, &tf, kind, tool.kind == kind);
            }
            root.spawn((
                Text::new(format!(
                    "Brush r={:.0}  paint={:?}",
                    tool.brush_radius, tool.terrain_paint
                )),
                tf.clone(),
                TextColor(palette.bevy_text_muted()),
            ));

            root.spawn((
                Text::new("── Minimap ──"),
                tf.clone(),
                TextColor(palette.bevy_primary_text()),
            ));
            if image_ready {
                root.spawn((
                    MapEditorMinimapImage,
                    ImageNode::new(map_tex.texture.clone()),
                    Node {
                        width: Val::Px(256.0),
                        height: Val::Px(256.0),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BorderColor::all(palette.bevy_wire_magenta()),
                    FocusPolicy::Block,
                ));
            } else {
                root.spawn((
                    MapEditorMinimapFallbackText,
                    Text::new(format!(
                        "Minimap texture not ready ({}×{})",
                        map_tex.width, map_tex.height
                    )),
                    tf.clone(),
                    TextColor(palette.bevy_text_muted()),
                ));
            }

            root.spawn((
                MapEditorScenarioBody,
                Text::new(scenario_text),
                tf,
                TextColor(palette.bevy_primary_text()),
            ));
        });
    stamp.key = next;
    presence.map_editor_shell_root = true;
}

pub fn map_editor_palette_pick_system(
    q: Query<(&Interaction, &MapEditorPalettePick), (Changed<Interaction>, With<Button>)>,
    mut tool: ResMut<MapEditorTool>,
    mut next_sub: ResMut<NextState<InGameEditorState>>,
) {
    for (interaction, pick) in &q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        tool.kind = pick.0;
        sync_tool_to_substate(tool.as_ref(), &mut next_sub);
    }
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
                (
                    toggle_scenario_script_panel_hotkey,
                    sync_map_editor_bevy_shell,
                    map_editor_palette_pick_system.after(sync_map_editor_bevy_shell),
                )
                    .run_if(map_editor_chrome_active),
            )
            .add_systems(
                Update,
                despawn_map_editor_bevy_shell_when_inactive.run_if(not(map_editor_chrome_active)),
            );
    }
}

fn despawn_map_editor_bevy_shell_when_inactive(
    mut commands: Commands,
    mut presence: ResMut<PanelPresence>,
    roots: Query<Entity, With<MapEditorBevyShell>>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
    presence.map_editor_shell_root = false;
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
