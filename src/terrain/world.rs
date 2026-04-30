use std::collections::BTreeMap;
use std::collections::HashMap;

use bevy::prelude::*;

use crate::idgen::EntityId;
use crate::terrain::locational::Cell_map_2D;
use crate::terrain::tiles::Tile;
use crate::traits::Region;

pub const HASH_CELL_SIZE: u32 = 128;

/// One logical map region tracked during procedural generation.
#[derive(Debug)]
pub struct GeoRegion {
    pub id: EntityId,
    pub center: Vec2,
    /// Grid position → spawned tile entity.
    pub tiles: BTreeMap<(usize, usize), Entity>,
    pub tile_id_map: HashMap<EntityId, (usize, usize)>,
    pub cell_map: Cell_map_2D<f32>,
}

impl GeoRegion {
    pub fn new() -> Self {
        Self {
            id: EntityId::default(),
            center: Vec2::ZERO,
            tiles: BTreeMap::new(),
            tile_id_map: HashMap::default(),
            cell_map: Cell_map_2D::new(1.0),
        }
    }

    pub fn add_tile(&mut self, position: Vec2, tile_entity: Entity) {
        let x = position.x as usize;
        let y = position.y as usize;
        self.tiles.insert((x, y), tile_entity);
    }
}

/// Top-level terrain container (minimal; extend for full spatial queries).
#[derive(Debug)]
pub struct World {
    pub regions: Vec<GeoRegion>,
    pub regions_cellmap: Cell_map_2D<f64>,
    pub width: u32,
    pub height: u32,
}

impl Region for GeoRegion {}

impl World {
    pub fn new(
        regions: Vec<GeoRegion>,
        regions_cellmap: Cell_map_2D<f64>,
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            regions,
            regions_cellmap,
            width,
            height,
        }
    }

    pub fn find_tile_at_position(&self, _position: Vec2) -> Option<&Tile> {
        None
    }
}
