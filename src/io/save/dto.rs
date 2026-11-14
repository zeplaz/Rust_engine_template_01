//! Save DTO graph — wire payloads use names, not runtime handles (IO-07).
//!
//! **E7 audit (`SavedChunkBody` / `SavedTerrainCell`):**
//! | field | truth / derived | rebuilt where |
//! |:---|:---|:---|
//! | `schema_version` | truth | loader gate in `load.rs` |
//! | `chunk` | truth (stable chunk coord) | — |
//! | `material_name` | truth | `material_ids_from_saved_body` (`load.rs`) |
//! | `tags` | truth (names) | `tag_sets_from_saved_body` (`load.rs`) |
//! | runtime `MaterialId` / `TagSet` | derived (not on wire) | `apply_saved_body_to_chunk_state` (`apply.rs`) |
//! | Bevy `Entity` | not stored | chunk coord remap in `apply_pending_save_pipeline_jobs` / streaming apply |

use serde::{Deserialize, Serialize};

pub const SAVED_CHUNK_BODY_SCHEMA_VERSION: u32 = 1;

/// One terrain cell row in a chunk save body.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedTerrainCell {
    pub material_name: String,
    pub tags: Vec<String>,
}

/// Incremental chunk body written by the save pipeline.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SavedChunkBody {
    pub schema_version: u32,
    pub chunk: [i32; 2],
    pub cells: Vec<SavedTerrainCell>,
}

#[must_use]
pub fn encode_chunk_body_ron(body: &SavedChunkBody) -> Result<Vec<u8>, ron::Error> {
    ron::ser::to_string(body).map(|s| s.into_bytes())
}

#[must_use]
pub fn decode_chunk_body_ron(bytes: &[u8]) -> Result<SavedChunkBody, String> {
    let text = std::str::from_utf8(bytes).map_err(|e| e.to_string())?;
    ron::de::from_str(text).map_err(|e| e.to_string())
}
