//! Weighted tile occupation book — interim aggregate (flag off) or derived cache (P2).
//!
//! When [`super::voxel_occupation::VoxelOccupationConfig::commit_authority_enabled`] is on,
//! commit/demolish must **not** call [`Self::apply_site`] / [`Self::remove_site`] directly;
//! refresh via [`Self::replace_from_voxel`] after the voxel book mutates.

use std::collections::HashMap;

use bevy::prelude::{IVec2, Resource};

use super::resources::SiteId;
use super::voxel_occupation::VoxelOccupationBook;

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

    /// Iterates committed Σw entries (for derived voxel mirror — read-only).
    pub fn iter_tile_weights(&self) -> impl Iterator<Item = (&IVec2, &f32)> {
        self.by_tile.iter()
    }

    /// Per-site contributions (P0 mirror → voxel `by_site`).
    pub fn iter_by_site(&self) -> impl Iterator<Item = (&SiteId, &Vec<(IVec2, f32)>)> {
        self.by_site.iter()
    }

    /// First site that owns nonzero weight on `tile` (mirror heuristic at P0).
    #[must_use]
    pub fn primary_site_at(&self, tile: IVec2) -> Option<SiteId> {
        self.by_site.iter().find_map(|(site_id, weights)| {
            weights.iter().any(|(t, w)| {
                *t == tile && *w > TILE_OCCUPATION_OVERLAP_EPSILON
            })
            .then_some(*site_id)
        })
    }

    /// P2 derived-cache refresh — sole tile mutation under `commit_authority_enabled`.
    pub fn replace_from_voxel(&mut self, voxel: &VoxelOccupationBook) {
        self.by_tile.clear();
        self.by_site.clear();
        for (tile, cell) in &voxel.cells {
            if cell.layers.weight < TILE_OCCUPATION_OVERLAP_EPSILON {
                continue;
            }
            self.by_tile.insert(*tile, cell.layers.weight);
        }
        for (site_id, weights) in voxel.iter_by_site() {
            self.by_site.insert(*site_id, weights.clone());
        }
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

    #[test]
    fn tile_occupation_replace_from_voxel_matches_weights() {
        let mut voxel = VoxelOccupationBook::default();
        voxel.apply_site_commit(SiteId(5), &[(IVec2::new(3, 4), 0.55)]);
        let mut tile = TileOccupationBook::default();
        tile.replace_from_voxel(&voxel);
        assert!((tile.weight_at(IVec2::new(3, 4)) - 0.55).abs() < 1e-5);
        assert_eq!(tile.primary_site_at(IVec2::new(3, 4)), Some(SiteId(5)));
    }
}
