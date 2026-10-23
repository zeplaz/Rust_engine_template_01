//! **Fortification index** (PLAN-FRONT-COMBAT-v1 FC-7 groundwork): tile -> owner + fortification family.
//!
//! Read model for pressure scoring and combat cover. The writer (rebuild from Operational defense
//! sites) lands with FC-7; until then callers may [`FortificationIndex::set`] directly.

use std::collections::HashMap;

use bevy::prelude::*;

use super::balance::{FortKind, FrontBalance};

#[derive(Resource, Clone, Debug, Default)]
pub struct FortificationIndex {
    tiles: HashMap<(i32, i32), (u8, FortKind)>,
}

impl FortificationIndex {
    pub fn set(&mut self, tile: IVec2, owner: u8, kind: FortKind) {
        self.tiles.insert((tile.x, tile.y), (owner, kind));
    }

    pub fn clear(&mut self) {
        self.tiles.clear();
    }

    #[must_use]
    pub fn get(&self, tile: IVec2) -> Option<(u8, FortKind)> {
        self.tiles.get(&(tile.x, tile.y)).copied()
    }

    /// Cover (0..1) `owner` enjoys on `tile`; 0 when the tile is unfortified or fortified by another slot.
    #[must_use]
    pub fn cover_at(&self, tile: IVec2, owner: u8, balance: &FrontBalance) -> f32 {
        match self.get(tile) {
            Some((o, kind)) if o == owner => balance.fortification(kind).map_or(0.0, |f| f.cover),
            _ => 0.0,
        }
    }
}
