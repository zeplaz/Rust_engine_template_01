use tiles::Tile;
use idgen::EntityId;
use traits::Region;

use std::collections::BTreeMap;
use bevy::utils::HashMap;

pub const HASH_CELL_SIZE:u32 = 128;

pub struct GeoRegion {
    id: EntityId,
    center: Vec2,
    tiles: BTreeMap<(usize, usize), Tile>,
    tile_id_map: HashMap<EntityId, (usize, usize)>,

}

impl Region for GeoRegion {

}

impl GeoRegion {
    pub fn new(id: EntityId, center: Vec2) -> Self {
        GeoRegion {
            id,
            center,
            tiles: BTreeMap::new(),
        }
    }

    pub fn get_tile_by_id(&self, tile_id: EntityId) -> Option<&Tile> {

    self.tile_id_map.get(&tile_id).and_then(|pos| self.tiles.get(pos))
    }


// Get all neighboring tiles of a specific tile within the region
pub fn get_neighboring_tiles(&self, tile_id: EntityId) -> Vec<&Tile> {
    let mut neighbors = Vec::new();
    if let Some(tile) = self.get_tile_by_id(tile_id) {
        let position = tile.position;

        for candidate in &self.tiles {
            let delta = candidate.position - position;
            let distance_squared = delta.x.powi(2) + delta.y.powi(2);

            // Check if the candidate tile is a direct neighbor (distance of 1 unit)
            if (distance_squared - 1.0).abs() < f32::EPSILON {
                neighbors.push(candidate);
            }
        }
    }

    neighbors
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

pub struct SpatialHash {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<EntityId>>,
}

impl SpatialHash {
    pub fn new(cell_size: f32) -> Self {
        SpatialHash {
            cell_size,
            cells: HashMap::new(),
        }
    }

    pub fn insert_tile(&mut self, position: Vec2, entity_id: EntityId) {
        let cell_coords = self.world_to_cell_coords(position);
        self.cells.entry(cell_coords).or_insert_with(Vec::new).push(entity_id);
    }

    pub fn find_tile_id(&self, position: Vec2) -> Option<EntityId> {
        let cell_coords = self.world_to_cell_coords(position);
        self.cells.get(&cell_coords).and_then(|ids| ids.iter().find(|&&id| id == id).copied())
    }

    pub fn find_tile(&self, position: Vec2, regions: &Vec<GeoRegion>) -> Option<&Tile> {
        let tile_id = self.find_tile_id(position)?;

        for region in regions {
            if let Some(tile) = region.tiles.get(&tile_id) {
                return Some(tile);
            }
        }

        None
    }

    fn world_to_cell_coords(&self, position: Vec2) -> (i32, i32) {
        let x = (position.x / self.cell_size).floor() as i32;
        let y = (position.y / self.cell_size).floor() as i32;
        (x, y)
    }
}


pub struct World {
    regions: Vec<GeoRegion>,
    spatial_hash: SpatialHash,
    width: u32,
    height: u32,

}

impl World {
    pub fn new(regions: Vec<GeoRegion>, spatial_hash: SpatialHash, width: u32, height: u32) -> Self {
        World {
            regions,
            spatial_hash,
            width,
            height,
        }
    }

    pub fn find_tile_at_position(&self, position: Vec2) -> Option<&Tile> {
        let tile_id = self.spatial_hash.find_tile(position)?;
        for georegions in &self.regions {
            if let Some(tile) = georegions.find_tile_by_id(tile_id) {
                return Some(tile);
            }
        }
        None
    }
}
