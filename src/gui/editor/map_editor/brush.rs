//! Terrain brush shape / Bresenham / tool kinds (M3).

use bevy::prelude::*;

use crate::engine::InGameEditorState;
use crate::gui::editor::editor_world_commit_bridge::{
    EditorTileEditCommitted, EditorTileEditKind,
};
use crate::terrain::family::{TerrainFamilyId, DEFAULT_TERRAIN_FAMILY_ID};
use crate::terrain::generation::brush_tile_inclusive_bounds;
use crate::terrain::generation::world_generator_enhanced::WorldGenParams;

/// Terrain brush footprint in the XZ tile plane (column = x, row = z).
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum MapEditorBrushShape {
    #[default]
    Disk,
    Square,
    Diamond,
}

impl MapEditorBrushShape {
    pub(crate) const ALL: [Self; 3] = [Self::Disk, Self::Square, Self::Diamond];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Disk => "Disk",
            Self::Square => "Square",
            Self::Diamond => "Diamond",
        }
    }
}

#[inline]
pub(crate) fn tile_in_brush(
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
pub(crate) fn bresenham_tile_line(x0: u32, y0: u32, x1: u32, y1: u32) -> Vec<(u32, u32)> {
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
    pub(crate) fn to_in_game(self) -> InGameEditorState {
        match self {
            MapEditorToolKind::Select => InGameEditorState::Select,
            MapEditorToolKind::Terrain => InGameEditorState::Terrain,
            MapEditorToolKind::Road => InGameEditorState::Road,
            MapEditorToolKind::Building => InGameEditorState::Create,
            MapEditorToolKind::Rail => InGameEditorState::Rail,
        }
    }

    pub(crate) const ALL: [Self; 5] = [
        Self::Select,
        Self::Terrain,
        Self::Road,
        Self::Building,
        Self::Rail,
    ];

    pub(crate) fn label(self) -> &'static str {
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

pub(crate) fn sync_tool_to_substate(tool: &MapEditorTool, next_sub: &mut NextState<InGameEditorState>) {
    NextState::set_if_neq(next_sub, tool.kind.to_in_game());
}

#[inline]
pub(crate) fn emit_editor_tile_commit_for_brush(
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

pub(crate) fn terrain_family_combo(ui: &mut bevy_egui::egui::Ui, current: &mut TerrainFamilyId) {
    let reg = crate::terrain::default_terrain_families();
    let sel = reg.def(*current).map(|d| d.name.as_str()).unwrap_or("?");
    bevy_egui::egui::ComboBox::from_id_salt("map_editor_biome_pick")
        .selected_text(sel)
        .show_ui(ui, |ui| {
            for (i, def) in reg.families.iter().enumerate() {
                let id = TerrainFamilyId(i as u16);
                ui.selectable_value(current, id, def.name.as_str());
            }
        });
}
