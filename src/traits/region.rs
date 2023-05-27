use crate::idgen::EntityId;
use bevy::prelude::*;
use terrain::world::SpatialHash;
use terrain::Tile;

pub trait Region {
    pub fn add_tile(
        &mut self,
        position: (usize, usize),
        tile: Tile,
        spatial_hash: &mut SpatialHash,
    ) {
        let vec_position = Vec2::new(position.0 as f32, position.1 as f32);
        self.tiles.insert(position, tile.clone());
        spatial_hash.insert_tile(vec_position, tile.id);
    }

    pub fn remove_tile(&mut self, tile_id: EntityId) -> Option<Tile>;

    pub fn find_tile_by_position(&self, position: (usize, usize)) -> Option<&Tile> {
        self.tiles.get(&position)
    }

    pub fn get_tile_by_id(&self, tile_id: EntityId) -> Option<&Tile> {
        self.tiles.values().find(|tile| tile.get_id() == tile_id)
    }

    // Get the distance between the center of the region and a given position
    pub fn distance_to_position_from_center(&self, position: Vec2) -> f32 {
        self.center.distance(position)
    }

    pub fn get_neighbors_mut(&mut self, tile_id: EntityId) -> Vec<&mut Tile> {
        let mut neighbors = Vec::new();
        if let Some(tile) = self.get_tile_by_id(tile_id) {
            let (x, y) = tile.get_position();
            let neighbor_positions = [
                (x - 1, y - 1), // top-left
                (x, y - 1),     // top
                (x + 1, y - 1), // top-right
                (x - 1, y),     // left
                (x + 1, y),     // right
                (x - 1, y + 1), // bottom-left
                (x, y + 1),     // bottom
                (x + 1, y + 1), // bottom-right
            ];

            for &(x, y) in neighbor_positions.iter() {
                if let Some(tile) = self.tiles.get_mut(&(x, y)) {
                    neighbors.push(tile);
                }
            }
        }
        neighbors
    }
}
