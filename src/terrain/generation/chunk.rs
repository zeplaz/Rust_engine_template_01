//! Chunk grid index for streaming / materialization — see material unification U5.

use bevy::prelude::{Component, IVec2, UVec2, Vec2};

#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Chunk {
    pub coord: IVec2,
}

/// World XY origin (min corner) for a chunk slab in tile space.
#[inline]
#[must_use]
pub fn chunk_world_origin(coord: IVec2, size: UVec2) -> Vec2 {
    Vec2::new(
        coord.x as f32 * size.x as f32,
        coord.y as f32 * size.y as f32,
    )
}

/// World XY center for a chunk slab in tile space.
#[inline]
#[must_use]
pub fn chunk_world_center(coord: IVec2, size: UVec2) -> Vec2 {
    let o = chunk_world_origin(coord, size);
    Vec2::new(o.x + size.x as f32 * 0.5, o.y + size.y as f32 * 0.5)
}

/// World XY center for one cell inside a chunk slab.
#[inline]
#[must_use]
pub fn chunk_cell_world_center(coord: IVec2, size: UVec2, cell_idx: usize) -> Vec2 {
    let sx = size.x as usize;
    let sy = size.y as usize;
    if sx == 0 || sy == 0 {
        return chunk_world_center(coord, size);
    }
    let lx = (cell_idx % sx) as f32;
    let ly = (cell_idx / sx) as f32;
    let o = chunk_world_origin(coord, size);
    Vec2::new(o.x + lx + 0.5, o.y + ly + 0.5)
}
