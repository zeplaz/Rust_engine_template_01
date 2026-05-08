//! **Grid / mask scaffolding** — coarse labels for legacy [`traits::mask`] and nav hooks.
//!
//! **Not terrain ontology:** authoritative surface class, hydrology, and fire barriers come from
//! [`crate::terrain::generation::ChunkCellMatrix`] (`TerrainFamilyId`, material **tags**, pass
//! p4 hydrology). When adding new gameplay rules, prefer cell tags + material resolver over
//! widening this enum.

use bevy::prelude::*;

use crate::idgen::EntityId;

/// Coarse **placement / mask** category for grid entities. Does not mirror material tags or families.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TileFootprintKind {
    /// Mask-only “allow all”; simulation still reads `ChunkCellMatrix`.
    Any,
    /// Schematic “can place structure” — verify with material rules + family, not this alone.
    Buildable,
    /// Schematic route lane; real routing uses transport graphs and chunk materialization.
    Road,
    /// Schematic open water / major wet barrier — rivers & lakes are produced in
    /// `terrain::generation::passes::p4_hydrology` + `world_generator_enhanced`, not from this flag.
    Water,
}

/// ECS tile used for **grid bookkeeping** and simple nav queries (see `systems::navigation::nav`).
/// Prefer chunk fields for simulation and rendering.
#[derive(Debug, Clone, Component)]
pub struct Tile {
    pub id: EntityId,
    pub grid_x: i32,
    pub grid_y: i32,
    /// Optional owning agent / faction for influence fields (`navigation/implementation_questions_v1.md` § dynamic obstacles).
    pub owner_id: Option<EntityId>,
    /// Mask / nav hint only; not `ChunkCellMatrix` classification.
    pub footprint: TileFootprintKind,
    pub safety_rating: f32,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            id: EntityId::default(),
            grid_x: 0,
            grid_y: 0,
            owner_id: None,
            footprint: TileFootprintKind::Any,
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
