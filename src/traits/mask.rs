//! Footprint / mask placement helpers (serializable geometry only — no ECS writes here).
//! Pair with chunk material masks when tying footprints to `ChunkCellMatrix` (`terrain_biome` runbooks).

use bevy::prelude::Vec2;

use crate::terrain::{TileFootprintKind, World};

pub trait MaskOperations {
    fn size(&self) -> Vec2;
    fn fits_in_mask(&self, other_mask: &Self, position: Vec2) -> bool;
    fn is_valid_placement(
        &self,
        position: Vec2,
        world: &World,
        valid_footprints: &[TileFootprintKind],
        valid_mask: Option<&Self>,
    ) -> bool;
}

impl MaskOperations for Vec<Vec<u8>> {
    fn size(&self) -> Vec2 {
        if self.is_empty() || self[0].is_empty() {
            return Vec2::ZERO;
        }
        Vec2::new(self[0].len() as f32, self.len() as f32)
    }

    /// For every occupied cell in `self` (non-zero), require `other_mask` to be occupied at the offset from `position`.
    fn fits_in_mask(&self, other_mask: &Self, position: Vec2) -> bool {
        let base_x = position.x as usize;
        let base_y = position.y as usize;
        for (y, row) in self.iter().enumerate() {
            for (x, &cell) in row.iter().enumerate() {
                if cell == 0 {
                    continue;
                }
                let ox = base_x + x;
                let oy = base_y + y;
                if oy >= other_mask.len() || ox >= other_mask[oy].len() {
                    return false;
                }
                if other_mask[oy][ox] == 0 {
                    return false;
                }
            }
        }
        true
    }

    fn is_valid_placement(
        &self,
        _position: Vec2,
        _world: &World,
        _valid_footprints: &[TileFootprintKind],
        valid_mask: Option<&Self>,
    ) -> bool {
        if let Some(vm) = valid_mask {
            if self.size() == Vec2::ZERO {
                return true;
            }
            // Full overlap check: mask must sit entirely on allowed footprint.
            if !self.fits_in_mask(vm, Vec2::ZERO) {
                return false;
            }
        }
        // World / chunk sampling: wire `World` + `ChunkCellMatrix` / material tags when mask needs sim truth.
        let _ = (_valid_footprints, _world);
        true
    }
}
