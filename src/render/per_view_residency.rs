//! Per-[`ViewSurfaceId`] residency consumer windows (S6-2 / DQ-S6-06).

use std::collections::HashMap;

use bevy::math::Rect;
use bevy::prelude::*;

use crate::gui::ViewInstance;
use crate::io::streaming::ChunkResidencyTable;
use crate::render::view_runtime::ViewSurfaceId;

/// Chunk index step aligned with fire / preview cull.
pub const RESIDENCY_VIEW_CHUNK_SPACING_WORLD: f32 = 64.0;

#[derive(Resource, Default, Clone, Debug)]
pub struct PerViewResidencyConsumerWindow {
    pub by_surface: HashMap<ViewSurfaceId, Vec<IVec2>>,
}

fn chunk_bounds_for_world_rect(world: Rect, spacing: f32) -> (IVec2, IVec2) {
    let s = spacing.max(1.0);
    let min_c = IVec2::new(
        (world.min.x / s).floor() as i32,
        (world.min.y / s).floor() as i32,
    );
    let max_c = IVec2::new(
        (world.max.x / s).floor() as i32,
        (world.max.y / s).floor() as i32,
    );
    (min_c, max_c)
}

fn chunk_in_rect(coord: IVec2, min_c: IVec2, max_c: IVec2) -> bool {
    coord.x >= min_c.x && coord.x <= max_c.x && coord.y >= min_c.y && coord.y <= max_c.y
}

/// Chunks in `table` that fall inside the view's [`ViewInstance::visible_world_rect`].
#[must_use]
pub fn residency_coords_for_view_instance(
    view: &ViewInstance,
    table: &ChunkResidencyTable,
) -> Vec<IVec2> {
    if table.entries.is_empty() {
        return Vec::new();
    }
    let world = view.visible_world_rect();
    let (min_c, max_c) = chunk_bounds_for_world_rect(world, RESIDENCY_VIEW_CHUNK_SPACING_WORLD);
    let mut coords: Vec<IVec2> = table
        .entries
        .keys()
        .copied()
        .filter(|c| chunk_in_rect(*c, min_c, max_c))
        .collect();
    coords.sort_by_key(|c| (c.y, c.x));
    coords
}

#[must_use]
pub fn per_view_residency_contains(surface: ViewSurfaceId, coord: IVec2, windows: &PerViewResidencyConsumerWindow) -> bool {
    match windows.by_surface.get(&surface) {
        Some(coords) if !coords.is_empty() => coords.contains(&coord),
        _ => true,
    }
}
