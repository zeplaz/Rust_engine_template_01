//! Road intersection graph (Round 3-B).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::strategic::BuildSiteTile;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct IntersectionId(pub u64);

#[derive(Debug, Clone)]
pub struct IntersectionNode {
    pub id: IntersectionId,
    pub tile: BuildSiteTile,
    pub connected_segments: Vec<Entity>,
}

#[derive(Resource, Debug, Default)]
pub struct IntersectionRegistry {
    next_id: u64,
    pub by_id: HashMap<IntersectionId, IntersectionNode>,
    pub by_tile: HashMap<(u32, u32), IntersectionId>,
}

impl IntersectionRegistry {
    pub fn register_or_extend(&mut self, tile: BuildSiteTile, segment: Entity) -> IntersectionId {
        let key = (tile.x, tile.z);
        let id = if let Some(&existing) = self.by_tile.get(&key) {
            existing
        } else {
            let id = IntersectionId(self.next_id);
            self.next_id = self.next_id.wrapping_add(1);
            self.by_tile.insert(key, id);
            self.by_id.insert(
                id,
                IntersectionNode {
                    id,
                    tile,
                    connected_segments: Vec::new(),
                },
            );
            id
        };
        if let Some(node) = self.by_id.get_mut(&id) {
            if !node.connected_segments.contains(&segment) {
                node.connected_segments.push(segment);
            }
        }
        id
    }

    #[must_use]
    pub fn neighbors_at_tile(&self, tile: BuildSiteTile) -> Vec<Entity> {
        self.by_tile
            .get(&(tile.x, tile.z))
            .and_then(|id| self.by_id.get(id))
            .map(|n| n.connected_segments.clone())
            .unwrap_or_default()
    }

    #[must_use]
    pub fn degree_at_tile(&self, tile: BuildSiteTile) -> usize {
        self.neighbors_at_tile(tile).len()
    }
}
