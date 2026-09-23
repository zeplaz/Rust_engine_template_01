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
    EditorTileEditCommitted, EditorTileEditKind,
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
use crate::terrain::generation::world_generator_enhanced::{
    Height, TerrainType, TileMarker, WorldGenParams, WorldMarker,
};
use crate::terrain::material::{MaterialId, MaterialRegistry};

use brush::{
    bresenham_tile_line, emit_editor_tile_commit_for_brush, sync_tool_to_substate,
    terrain_family_combo, tile_in_brush,
};
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
