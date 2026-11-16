//! Pass 5 — agent / scenario overlay (**TAC-05** terrain analysis).
//!
//! Writes chokepoint / defensibility / high_ground into [`TerrainAnalysisBook`].
//! Does not mutate combat grids or fortification.

use bevy::prelude::IVec2;

use crate::terrain::analysis::{compute_terrain_analysis_rect, TerrainAnalysisBook};
use crate::terrain::generation::cell_matrix::ChunkCellMatrix;

/// Legacy entry (no book) — computes analysis and drops it (keeps pass pipeline callable).
pub fn apply_agent_overlay(matrix: &mut ChunkCellMatrix) {
    let _ = compute_terrain_analysis_rect(matrix.size.x, matrix.size.y, &matrix.elevation);
}

/// TAC-05-P1: merge chunk analysis into the world book (world tile origin = chunk min corner).
pub fn apply_agent_overlay_into_book(
    matrix: &ChunkCellMatrix,
    chunk_coord: IVec2,
    book: &mut TerrainAnalysisBook,
) {
    let cells = compute_terrain_analysis_rect(matrix.size.x, matrix.size.y, &matrix.elevation);
    let origin = IVec2::new(
        chunk_coord.x * matrix.size.x as i32,
        chunk_coord.y * matrix.size.y as i32,
    );
    book.merge_rect(origin, matrix.size.x, matrix.size.y, &cells);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::prelude::UVec2;

    #[test]
    fn p5_merges_into_book() {
        let mut matrix = ChunkCellMatrix::new(UVec2::new(4, 4));
        for i in 0..matrix.elevation.len() {
            matrix.elevation[i] = 0.3;
        }
        let peak = matrix.idx(2, 2);
        matrix.elevation[peak] = 0.95;
        let mut book = TerrainAnalysisBook::default();
        apply_agent_overlay_into_book(&matrix, IVec2::new(1, 0), &mut book);
        assert!(book.defensibility_at(IVec2::new(1 * 4 + 2, 2)) > 0.0);
    }
}
