//! Save DTO graph â€” wire payloads use names, not runtime handles.

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
