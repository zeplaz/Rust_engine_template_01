//! Chunk fire state — RON overlay slice (VSS-T3-003).
//!
//! Terrain stays on `SavedChunkBody`; fire heat/fuel is a sibling overlay (settlement pattern).
//! Does **not** write `SharedOverlayFieldBuffers` — VT-4 extract remains sole overlay writer.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::systems::fire::{ChunkFireOverlay, ChunkSurfaceFire};
use crate::terrain::generation::Chunk;

pub const FIRE_OVERLAY_NAME: &str = "chunk_fire_state";
pub const FIRE_STATE_REL_PATH: &str = "overlays/chunk_fire_state.ron";
pub const FIRE_STATE_SCHEMA_VERSION: u32 = 1;
pub const FIRE_SAVE_ROUNDTRIP_JSON: &str = "debug_runs/fire_save_roundtrip_live.json";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SavedFireChunkRow {
    pub chunk: [i32; 2],
    pub heat: f32,
    pub fuel: f32,
    #[serde(default)]
    pub smoke: f32,
    #[serde(default)]
    pub toxic: f32,
    #[serde(default)]
    pub has_overlay: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChunkFireStateSnapshot {
    pub schema_version: u32,
    pub rows: Vec<SavedFireChunkRow>,
}

#[must_use]
pub fn fire_overlay_ref(artifact_path: impl Into<String>) -> crate::io::save::OverlaySnapshotRef {
    crate::io::save::OverlaySnapshotRef {
        overlay_name: FIRE_OVERLAY_NAME.into(),
        artifact_path: artifact_path.into(),
    }
}

#[must_use]
pub fn default_fire_overlay_ref() -> crate::io::save::OverlaySnapshotRef {
    fire_overlay_ref(FIRE_STATE_REL_PATH)
}

/// Stable FNV-1a over sorted chunk fire rows (quantize floats ×10000, same as VT-4 heat hash).
#[must_use]
pub fn hash_saved_fire_state(snapshot: &ChunkFireStateSnapshot) -> u64 {
    let mut rows = snapshot.rows.clone();
    rows.sort_by_key(|r| (r.chunk[0], r.chunk[1]));
    let mut hash = 0xcbf29ce484222325u64;
    for row in &rows {
        for byte in row.chunk[0].to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        for byte in row.chunk[1].to_le_bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        for q in [
            quantize_fire_f32(row.heat),
            quantize_fire_f32(row.fuel),
            quantize_fire_f32(row.smoke),
            quantize_fire_f32(row.toxic),
        ] {
            for byte in q.to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
        hash ^= u64::from(u8::from(row.has_overlay));
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[inline]
fn quantize_fire_f32(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * 10_000.0).round() as u32
}

#[must_use]
pub fn capture_chunk_fire_state(
    query: &Query<(&Chunk, &ChunkSurfaceFire, Option<&ChunkFireOverlay>)>,
) -> ChunkFireStateSnapshot {
    let mut rows: Vec<SavedFireChunkRow> = query
        .iter()
        .map(|(chunk, surface, overlay)| {
            let (smoke, toxic, has_overlay) = overlay
                .map(|o| {
                    let smoke = o.smoke.iter().copied().fold(0.0_f32, f32::max);
                    let toxic = o.toxic.iter().copied().fold(0.0_f32, f32::max);
                    (smoke, toxic, true)
                })
                .unwrap_or((0.0, 0.0, false));
            SavedFireChunkRow {
                chunk: [chunk.coord.x, chunk.coord.y],
                heat: surface.heat,
                fuel: surface.fuel,
                smoke,
                toxic,
                has_overlay,
            }
        })
        .collect();
    rows.sort_by_key(|r| (r.chunk[0], r.chunk[1]));
    ChunkFireStateSnapshot {
        schema_version: FIRE_STATE_SCHEMA_VERSION,
        rows,
    }
}

/// Pure capture from owned rows (lib witness — no ECS).
#[must_use]
pub fn capture_chunk_fire_state_from_rows(rows: Vec<SavedFireChunkRow>) -> ChunkFireStateSnapshot {
    let mut rows = rows;
    rows.sort_by_key(|r| (r.chunk[0], r.chunk[1]));
    ChunkFireStateSnapshot {
        schema_version: FIRE_STATE_SCHEMA_VERSION,
        rows,
    }
}

/// Surface heat/fuel hash only (gameplay authority for hydrate).
#[must_use]
pub fn hash_surface_fire_rows(rows: &[SavedFireChunkRow]) -> u64 {
    let surface_only: Vec<SavedFireChunkRow> = rows
        .iter()
        .map(|r| SavedFireChunkRow {
            chunk: r.chunk,
            heat: r.heat,
            fuel: r.fuel,
            smoke: 0.0,
            toxic: 0.0,
            has_overlay: false,
        })
        .collect();
    hash_saved_fire_state(&ChunkFireStateSnapshot {
        schema_version: FIRE_STATE_SCHEMA_VERSION,
        rows: surface_only,
    })
}

pub fn apply_chunk_fire_state(snapshot: &ChunkFireStateSnapshot, world: &mut World) {
    let wanted: HashMap<IVec2, (f32, f32)> = snapshot
        .rows
        .iter()
        .map(|r| (IVec2::new(r.chunk[0], r.chunk[1]), (r.heat, r.fuel)))
        .collect();
    let mut seen = HashSet::new();
    {
        let mut q = world.query::<(&Chunk, &mut ChunkSurfaceFire)>();
        for (chunk, mut surface) in q.iter_mut(world) {
            if let Some(&(heat, fuel)) = wanted.get(&chunk.coord) {
                surface.heat = heat;
                surface.fuel = fuel;
                seen.insert(chunk.coord);
            }
        }
    }
    for (coord, (heat, fuel)) in wanted {
        if !seen.contains(&coord) {
            world.spawn((
                Chunk { coord },
                ChunkSurfaceFire { heat, fuel },
            ));
        }
    }
}

pub fn write_chunk_fire_state_ron(
    path: impl AsRef<Path>,
    snapshot: &ChunkFireStateSnapshot,
) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = ron::ser::to_string_pretty(snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(path, body)
}

pub fn read_chunk_fire_state_ron(path: impl AsRef<Path>) -> std::io::Result<ChunkFireStateSnapshot> {
    let body = std::fs::read_to_string(path)?;
    ron::from_str(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
}

#[must_use]
pub fn fire_save_roundtrip_lib_green() -> bool {
    fire_save_roundtrip_self_check().is_ok()
}

fn fire_save_roundtrip_self_check() -> Result<(), &'static str> {
    let snapshot = capture_chunk_fire_state_from_rows(vec![
        SavedFireChunkRow {
            chunk: [2, -1],
            heat: 0.75,
            fuel: 0.4,
            smoke: 0.2,
            toxic: 0.05,
            has_overlay: true,
        },
        SavedFireChunkRow {
            chunk: [0, 0],
            heat: 0.1,
            fuel: 0.9,
            smoke: 0.0,
            toxic: 0.0,
            has_overlay: false,
        },
    ]);
    let hash_before = hash_saved_fire_state(&snapshot);
    if hash_before == 0 {
        return Err("hash_zero");
    }

    let path = std::env::temp_dir().join("vss_t3_003_fire_save_roundtrip.ron");
    write_chunk_fire_state_ron(&path, &snapshot).map_err(|_| "write_failed")?;
    let loaded = read_chunk_fire_state_ron(&path).map_err(|_| "read_failed")?;
    if hash_saved_fire_state(&loaded) != hash_before {
        return Err("ron_hash_mismatch");
    }

    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    apply_chunk_fire_state(&loaded, app.world_mut());

    let mut q = app.world_mut().query::<(&Chunk, &ChunkSurfaceFire)>();
    let mut hydrated: Vec<SavedFireChunkRow> = q
        .iter(app.world())
        .map(|(chunk, surface)| SavedFireChunkRow {
            chunk: [chunk.coord.x, chunk.coord.y],
            heat: surface.heat,
            fuel: surface.fuel,
            smoke: 0.0,
            toxic: 0.0,
            has_overlay: false,
        })
        .collect();
    hydrated.sort_by_key(|r| (r.chunk[0], r.chunk[1]));
    if hash_surface_fire_rows(&hydrated) != hash_surface_fire_rows(&loaded.rows) {
        return Err("hydrate_hash_mismatch");
    }
    Ok(())
}

#[must_use]
pub fn build_fire_save_roundtrip_witness_body() -> serde_json::Value {
    let green = fire_save_roundtrip_lib_green();
    let dirty_green = crate::dev::spectator_parity_live_proof::save_fire_dirty_queue_lib_green();
    let snapshot = capture_chunk_fire_state_from_rows(vec![SavedFireChunkRow {
        chunk: [1, 1],
        heat: 0.5,
        fuel: 0.5,
        smoke: 0.0,
        toxic: 0.0,
        has_overlay: false,
    }]);
    serde_json::json!({
        "schema": "fire_save_roundtrip_witness_v1",
        "slice_id": "VSS-T3-003",
        "status": if green && dirty_green { "green" } else { "pending" },
        "green": green && dirty_green,
        "exit_predicates": {
            "fire_save_roundtrip_hash": green,
            "save_fire_dirty_queue": dirty_green
        },
        "hash_sample": hash_saved_fire_state(&snapshot),
        "overlay": FIRE_OVERLAY_NAME,
        "artifact": FIRE_STATE_REL_PATH,
        "authority": "ChunkSurfaceFire (+ overlay smoke/toxic max when present)",
        "forbidden_as_sole_proof": ["SharedOverlayFieldBuffers", "harness apply_test_scene_fire_seeds"]
    })
}

#[must_use]
pub fn refresh_fire_save_roundtrip_live_witness() -> bool {
    crate::dev::runtime_witness::write_enveloped_witness_unchecked(
        "VSS-T3-003-FIRE-SAVE-RT",
        "coder_fire_save_roundtrip",
        FIRE_SAVE_ROUNDTRIP_JSON,
        build_fire_save_roundtrip_witness_body(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fire_save_roundtrip_hash_stable() {
        assert!(fire_save_roundtrip_lib_green());
    }

    #[test]
    fn hash_order_independent() {
        let a = capture_chunk_fire_state_from_rows(vec![
            SavedFireChunkRow {
                chunk: [1, 0],
                heat: 0.3,
                fuel: 0.7,
                smoke: 0.0,
                toxic: 0.0,
                has_overlay: false,
            },
            SavedFireChunkRow {
                chunk: [0, 0],
                heat: 0.9,
                fuel: 0.1,
                smoke: 0.2,
                toxic: 0.0,
                has_overlay: true,
            },
        ]);
        let b = capture_chunk_fire_state_from_rows(vec![
            SavedFireChunkRow {
                chunk: [0, 0],
                heat: 0.9,
                fuel: 0.1,
                smoke: 0.2,
                toxic: 0.0,
                has_overlay: true,
            },
            SavedFireChunkRow {
                chunk: [1, 0],
                heat: 0.3,
                fuel: 0.7,
                smoke: 0.0,
                toxic: 0.0,
                has_overlay: false,
            },
        ]);
        assert_eq!(hash_saved_fire_state(&a), hash_saved_fire_state(&b));
    }
}
