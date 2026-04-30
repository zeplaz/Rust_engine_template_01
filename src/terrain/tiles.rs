use bevy::prelude::*;

use crate::idgen::EntityId;

/// High-level tile category for placement / mask rules (see `traits::mask`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileType {
    Any,
    Buildable,
    Road,
    Water,
}

/// Terrain tile state shared by worldgen bookkeeping and navigation queries.
/// ECS components can wrap or extend this as the chunk/streaming model lands.
#[derive(Debug, Clone, Component)]
pub struct Tile {
    pub id: EntityId,
    pub grid_x: i32,
    pub grid_y: i32,
    /// Optional owning agent / faction for influence fields (`navigation/implementation_questions_v1.md` § dynamic obstacles).
    pub owner_id: Option<EntityId>,
    pub tile_type: TileType,
    pub safety_rating: f32,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            id: EntityId::default(),
            grid_x: 0,
            grid_y: 0,
            owner_id: None,
            tile_type: TileType::Any,
            safety_rating: 1.0,
        }
    }
}

impl Tile {
    pub fn get_id(&self) -> EntityId {
        self.id
    }

    pub fn get_position(&self) -> (usize, usize) {
        (self.grid_x as usize, self.grid_y as usize)
    }
}

