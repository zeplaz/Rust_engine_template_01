//! Weighted tile occupation book — single writer on commit/demolish.

use std::collections::HashMap;

use bevy::prelude::{IVec2, Resource};

use super::resources::SiteId;

pub const TILE_OCCUPATION_OVERLAP_EPSILON: f32 = 0.001;

/// Aggregated Σw per world tile for committed parametric sites.
#[derive(Resource, Clone, Debug, Default, PartialEq)]
pub struct TileOccupationBook {
    pub by_tile: HashMap<IVec2, f32>,
    by_site: HashMap<SiteId, Vec<(IVec2, f32)>>,
}

impl TileOccupationBook {
    #[must_use]
    pub fn weight_at(&self, tile: IVec2) -> f32 {
        self.by_tile.get(&tile).copied().unwrap_or(0.0)
    }

    #[must_use]
    pub fn can_apply(&self, weights: &[(IVec2, f32)]) -> bool {
        weights.iter().all(|(tile, w_new)| {
            self.weight_at(*tile) + w_new <= 1.0 + TILE_OCCUPATION_OVERLAP_EPSILON
        })
    }

    /// True when Σ existing + candidate > 1 (+ ε) on any tile.
    #[must_use]
    pub fn would_overlap(&self, weights: &[(IVec2, f32)]) -> bool {
        !self.can_apply(weights)
    }

    pub fn apply_site(&mut self, site_id: SiteId, weights: &[(IVec2, f32)]) {
        for (tile, w) in weights {
            *self.by_tile.entry(*tile).or_insert(0.0) += w;
        }
        self.by_site.insert(site_id, weights.to_vec());
    }

    pub fn remove_site(&mut self, site_id: SiteId) -> Option<Vec<(IVec2, f32)>> {
        let weights = self.by_site.remove(&site_id)?;
        for (tile, w) in &weights {
            if let Some(entry) = self.by_tile.get_mut(tile) {
                *entry = (*entry - w).max(0.0);
                if *entry < TILE_OCCUPATION_OVERLAP_EPSILON {
                    self.by_tile.remove(tile);
                }
            }
        }
        Some(weights)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_occupation_book_apply_and_remove() {
        let mut book = TileOccupationBook::default();
        let id = SiteId(7);
        let weights = vec![(IVec2::new(2, 3), 0.6)];
        assert!(book.can_apply(&weights));
        book.apply_site(id, &weights);
        assert!((book.weight_at(IVec2::new(2, 3)) - 0.6).abs() < 1e-5);
        book.remove_site(id);
        assert!(book.weight_at(IVec2::new(2, 3)) < TILE_OCCUPATION_OVERLAP_EPSILON);
    }

    #[test]
    fn tile_occupation_book_rejects_sum_over_one() {
        let mut book = TileOccupationBook::default();
        book.apply_site(SiteId(1), &[(IVec2::new(0, 0), 0.7)]);
        assert!(!book.can_apply(&[(IVec2::new(0, 0), 0.4)]));
        assert!(book.would_overlap(&[(IVec2::new(0, 0), 0.4)]));
    }

}
