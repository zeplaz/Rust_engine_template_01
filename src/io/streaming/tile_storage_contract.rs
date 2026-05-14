//! TileStorage diff contract inputs for Wave C — renderer semantics live in **BQ-101**.

use bevy::prelude::{IVec2, Resource};

pub const TILE_STORAGE_DIFF_CONTRACT_BQ: &str = "BQ-101";

/// One chunk’s changed tile indices pending a TileStorage apply.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TileStorageDiffChunk {
    pub chunk: IVec2,
    pub changed_tile_indices: Vec<u32>,
}

/// Batch of TileStorage diffs produced after domain reconstruct (timing TBD in BQ-101).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TileStorageDiffBatch {
    pub chunks: Vec<TileStorageDiffChunk>,
}

#[derive(Resource, Debug, Default, Clone)]
pub struct PendingTileStorageDiffQueue {
    pub batch: TileStorageDiffBatch,
}

#[must_use]
pub fn tile_storage_diff_for_chunk(chunk: IVec2, changed_tile_indices: Vec<u32>) -> TileStorageDiffChunk {
    TileStorageDiffChunk {
        chunk,
        changed_tile_indices,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_storage_diff_contract_is_bq_101() {
        assert_eq!(TILE_STORAGE_DIFF_CONTRACT_BQ, "BQ-101");
    }
}
