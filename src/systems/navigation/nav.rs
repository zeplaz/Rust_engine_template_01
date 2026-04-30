//! Grid-aligned tile queries backing road / field navigation.
//! Replace linear scan with spatial index when chunk streaming lands (`terrain_world` designer docs).

use bevy::prelude::*;

use crate::terrain::Tile;

/// Discrete grid lookup: matches `Tile.grid_x` / `grid_y` to rounded world coordinates.
pub fn find_tile_at_position(position: Vec2, query: &Query<&Tile>) -> Option<Tile> {
    let gx = position.x.round() as i32;
    let gy = position.y.round() as i32;
    for tile in query.iter() {
        if tile.grid_x == gx && tile.grid_y == gy {
            return Some(tile.clone());
        }
    }
    None
}
