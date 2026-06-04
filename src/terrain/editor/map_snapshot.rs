//! Serializable map grid for editor save/load — terrain **family names** on disk (not raw [`TerrainFamilyId`](crate::terrain::family::TerrainFamilyId)).

use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

/// Schema for [`MapSnapshotV1`]; bump when breaking on-disk layout.
pub const MAP_SNAPSHOT_SCHEMA_VERSION: u32 = 1;

/// **INFRA-E0-002** — v2 drops per-cell `road` (transport graph is authoritative).
pub const MAP_SNAPSHOT_SCHEMA_VERSION_V2: u32 = 2;

static MAP_SNAPSHOT_ROAD_STRIP_LOGGED: AtomicBool = AtomicBool::new(false);

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
    /// Legacy editor road marker hint — stripped on load (**INFRA-E0-002**).
    #[serde(default)]
    #[deprecated(
        since = "2026-05-28",
        note = "INFRA-E0-002: road markers are editor entities / transport snapshot — not tile flags"
    )]
    pub road: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapSnapshotCellV2 {
    pub height: f32,
    pub terrain_family: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MapSnapshotV2 {
    pub schema_version: u32,
    pub width: u32,
    pub height: u32,
    pub cells: Vec<MapSnapshotCellV2>,
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

impl MapSnapshotV2 {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != MAP_SNAPSHOT_SCHEMA_VERSION_V2 {
            return Err(format!(
                "map snapshot schema_version={} unsupported (expected {})",
                self.schema_version, MAP_SNAPSHOT_SCHEMA_VERSION_V2
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
}

/// Load v1 or v2 RON; v1 is migrated (road flags stripped, logged once).
pub fn load_map_snapshot_from_ron(s: &str) -> Result<MapSnapshotV1, String> {
    if let Ok(v2) = ron::de::from_str::<MapSnapshotV2>(s) {
        if v2.schema_version == MAP_SNAPSHOT_SCHEMA_VERSION_V2 {
            v2.validate()?;
            return Ok(v2_to_v1_working(v2));
        }
    }
    let v1 = MapSnapshotV1::from_ron_str(s)?;
    Ok(migrate_map_snapshot_v1_to_v2(v1))
}

fn v2_to_v1_working(v2: MapSnapshotV2) -> MapSnapshotV1 {
    MapSnapshotV1 {
        schema_version: MAP_SNAPSHOT_SCHEMA_VERSION,
        width: v2.width,
        height: v2.height,
        cells: v2
            .cells
            .into_iter()
            .map(|c| MapSnapshotCellV1 {
                height: c.height,
                terrain_family: c.terrain_family,
                road: false,
            })
            .collect(),
    }
}

/// **INFRA-E0-002** — strip legacy `road` bools; transport graph owns corridors.
#[must_use]
pub fn migrate_map_snapshot_v1_to_v2(mut snap: MapSnapshotV1) -> MapSnapshotV1 {
    let road_cells = snap.cells.iter().filter(|c| c.road).count();
    if road_cells > 0 && !MAP_SNAPSHOT_ROAD_STRIP_LOGGED.swap(true, Ordering::Relaxed) {
        bevy::log::warn!(
            "INFRA-E0-002: stripped {road_cells} legacy map_snapshot road flags — use transport graph / editor markers"
        );
    }
    for cell in &mut snap.cells {
        cell.road = false;
    }
    snap
}

#[must_use]
pub fn map_snapshot_v1_to_v2(snap: &MapSnapshotV1) -> MapSnapshotV2 {
    MapSnapshotV2 {
        schema_version: MAP_SNAPSHOT_SCHEMA_VERSION_V2,
        width: snap.width,
        height: snap.height,
        cells: snap
            .cells
            .iter()
            .map(|c| MapSnapshotCellV2 {
                height: c.height,
                terrain_family: c.terrain_family.clone(),
            })
            .collect(),
    }
}

impl MapSnapshotV2 {
    pub fn to_ron_string(&self) -> Result<String, ron::Error> {
        let cfg = ron::ser::PrettyConfig::new().depth_limit(64).indentor("    ".into());
        ron::ser::to_string_pretty(self, cfg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn infra_e0_002_migrate_strips_road_flags() {
        let snap = MapSnapshotV1 {
            schema_version: MAP_SNAPSHOT_SCHEMA_VERSION,
            width: 2,
            height: 1,
            cells: vec![
                MapSnapshotCellV1 {
                    height: 0.1,
                    terrain_family: "Grassland".into(),
                    road: true,
                },
                MapSnapshotCellV1 {
                    height: 0.2,
                    terrain_family: "Grassland".into(),
                    road: false,
                },
            ],
        };
        let migrated = migrate_map_snapshot_v1_to_v2(snap);
        assert!(migrated.cells.iter().all(|c| !c.road));
    }

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
