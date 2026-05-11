//! Serializable map grid for editor save/load — terrain **family names** on disk (not raw [`TerrainFamilyId`](crate::terrain::family::TerrainFamilyId)).

use serde::{Deserialize, Serialize};

/// Schema for [`MapSnapshotV1`]; bump when breaking on-disk layout.
pub const MAP_SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// M5 v1 snapshot: flat `width × height` cells in row-major order (z / row outer, x / column inner).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapSnapshotV1 {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub cells: Vec<MapSnapshotCellV1>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapSnapshotCellV1 {
    /// Normalized elevation 0..1 ([`crate::terrain::generation::world_generator_enhanced::Height`]).
    pub height: f32,
    pub terrain_family: String,
    #[serde(default)]
    pub road: bool,
}

impl MapSnapshotV1 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != MAP_SNAPSHOT_SCHEMA_VERSION {
            return Err(format!(
                "map snapshot schema_version={} unsupported (expected {})",
                self.schema_version, MAP_SNAPSHOT_SCHEMA_VERSION
            ));
        }
        let n = self
            .width
            .checked_mul(self.height)
            .and_then(|x| usize::try_from(x).ok())
            .ok_or_else(|| "map snapshot width*height overflow".to_string())?;
        if self.cells.len() != n {
            return Err(format!(
                "map snapshot cells len {} != width*height {}",
                self.cells.len(),
                n
            ));
        }
        Ok(())
    }

    pub fn to_ron_string(&self) -> Result<String, ron::Error> {
        let cfg = ron::ser::PrettyConfig::new().depth_limit(64).indentor("    ".into());
        ron::ser::to_string_pretty(self, cfg)
    }

    pub fn from_ron_str(s: &str) -> Result<Self, String> {
        ron::de::from_str(s).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_snapshot_v1_round_trips_ron() {
        let snap = MapSnapshotV1 {
            schema_version: MAP_SNAPSHOT_SCHEMA_VERSION,
            width: 1,
            height: 1,
            cells: vec![MapSnapshotCellV1 {
                height: 0.42,
                terrain_family: "Grassland".into(),
                road: false,
            }],
        };
        snap.validate().unwrap();
        let ron = snap.to_ron_string().unwrap();
        let back = MapSnapshotV1::from_ron_str(&ron).unwrap();
        assert_eq!(snap, back);
    }
}
