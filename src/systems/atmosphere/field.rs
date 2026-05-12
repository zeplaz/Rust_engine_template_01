//! [`AtmosphereCell`] / [`AtmosphereField`] CPU authority + [`GlobalWind`] (`base_fire2_smoke.md` §1, §3).

use bevy::prelude::*;

/// One logical atmosphere column (chunk- or tile-mapped depending on [`AtmosphereField::origin`] / `size`).
#[derive(Clone, Copy, Debug, Default)]
pub struct AtmosphereCell {
    pub smoke_density: f32,
    pub fog_density: f32,
    pub toxicity: f32,
    pub heat_distortion: f32,
    pub ash_density: f32,
    pub ember_density: f32,
    /// Combined visibility in `[0, 1]` — 1 = clear.
    pub visibility: f32,
}

/// Grid keyed by world chunk indices: cell `(x,y)` covers chunk `(origin.x + x, origin.y + y)`.
#[derive(Resource, Debug, Clone)]
pub struct AtmosphereField {
    pub origin: IVec2,
    pub size: UVec2,
    pub cells: Vec<AtmosphereCell>,
}

impl Default for AtmosphereField {
    fn default() -> Self {
        let size = UVec2::splat(128);
        let n = (size.x * size.y) as usize;
        Self {
            origin: IVec2::ZERO,
            size,
            cells: vec![AtmosphereCell::default(); n],
        }
    }
}

impl AtmosphereField {
    #[inline]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    #[inline]
    pub fn idx(&self, x: u32, y: u32) -> usize {
        (y * self.size.x + x) as usize
    }

    /// `None` if out of grid.
    pub fn cell_index(&self, x: u32, y: u32) -> Option<usize> {
        if x >= self.size.x || y >= self.size.y {
            return None;
        }
        Some(self.idx(x, y))
    }

    pub fn chunk_to_tile(&self, chunk: IVec2) -> Option<(u32, u32)> {
        let lx = chunk.x - self.origin.x;
        let ly = chunk.y - self.origin.y;
        if lx < 0 || ly < 0 {
            return None;
        }
        let ux = lx as u32;
        let uy = ly as u32;
        if ux >= self.size.x || uy >= self.size.y {
            return None;
        }
        Some((ux, uy))
    }

    pub fn cell_at_chunk(&self, chunk: IVec2) -> Option<AtmosphereCell> {
        let (x, y) = self.chunk_to_tile(chunk)?;
        self.cells.get(self.idx(x, y)).copied()
    }

    pub fn cell_mut_at_chunk(&mut self, chunk: IVec2) -> Option<&mut AtmosphereCell> {
        let (x, y) = self.chunk_to_tile(chunk)?;
        let i = self.idx(x, y);
        self.cells.get_mut(i)
    }
}

/// World wind for semi-Lagrangian advection on [`AtmosphereField`].
#[derive(Resource, Debug, Clone, Copy)]
pub struct GlobalWind {
    pub direction: Vec2,
    pub speed: f32,
}

impl Default for GlobalWind {
    fn default() -> Self {
        Self {
            direction: Vec2::X,
            speed: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_field_len_matches_size() {
        let f = AtmosphereField::default();
        assert_eq!(f.len(), (128 * 128) as usize);
    }

    #[test]
    fn chunk_to_tile_roundtrip_origin_zero() {
        let f = AtmosphereField::default();
        assert_eq!(f.chunk_to_tile(IVec2::new(0, 0)), Some((0, 0)));
        assert_eq!(f.chunk_to_tile(IVec2::new(127, 0)), Some((127, 0)));
        assert_eq!(f.chunk_to_tile(IVec2::new(-1, 0)), None);
        assert_eq!(f.chunk_to_tile(IVec2::new(128, 0)), None);
    }

    #[test]
    fn idx_monotonic_in_x() {
        let f = AtmosphereField::default();
        assert!(f.idx(0, 0) < f.idx(1, 0));
    }
}
