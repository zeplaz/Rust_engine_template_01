//! Chunk fire state — RON overlay slice (VSS-T3-003).
//!
//! Terrain stays on `SavedChunkBody`; fire heat/fuel is a sibling overlay (settlement pattern).
//! Does **not** write `SharedOverlayFieldBuffers` — VT-4 extract remains sole overlay writer.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::systems::fire::{ChunkFireOverlay, ChunkSmokeField, ChunkSurfaceFire};
use crate::terrain::generation::Chunk;

pub const FIRE_OVERLAY_NAME: &str = "chunk_fire_state";
pub const FIRE_STATE_REL_PATH: &str = "overlays/chunk_fire_state.ron";
/// v1: chunk-mean surface heat/fuel only. v2 adds per-cell fire and chunk smoke/toxic.
pub const FIRE_STATE_SCHEMA_VERSION: u32 = 2;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct SavedFireChunkRow {
    pub chunk: [i32; 2],
    /// Chunk-mean surface heat (always persisted).
    pub heat: f32,
    /// Chunk-mean surface fuel (always persisted).
    pub fuel: f32,
    /// Chunk smoke density. Absent on v1 loads (defaults to 0).
    #[serde(default)]
    pub smoke: f32,
    /// Chunk toxic density. Absent on v1 loads (defaults to 0).
    #[serde(default)]
    pub toxic: f32,
    /// Per-cell heat. Empty on v1 loads.
    #[serde(default)]
    pub cell_heat: Vec<f32>,
    #[serde(default)]
    pub cell_fuel: Vec<f32>,
    #[serde(default)]
    pub cell_burn_hold: Vec<u16>,
    #[serde(default)]
    pub cell_smoke: Vec<f32>,
    #[serde(default)]
    pub cell_toxic: Vec<f32>,
    /// Derived from the cell vectors. Not written.
    #[serde(default, skip_serializing)]
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
        mix_u32(&mut hash, row.cell_heat.len() as u32);
        for v in &row.cell_heat {
            mix_f32(&mut hash, *v);
        }
        mix_u32(&mut hash, row.cell_fuel.len() as u32);
        for v in &row.cell_fuel {
            mix_f32(&mut hash, *v);
        }
        mix_u32(&mut hash, row.cell_smoke.len() as u32);
        for v in &row.cell_smoke {
            mix_f32(&mut hash, *v);
        }
        mix_u32(&mut hash, row.cell_toxic.len() as u32);
        for v in &row.cell_toxic {
            mix_f32(&mut hash, *v);
        }
        mix_u32(&mut hash, row.cell_burn_hold.len() as u32);
        for ticks in &row.cell_burn_hold {
            for byte in ticks.to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
    }
    hash
}

fn mix_u32(hash: &mut u64, v: u32) {
    for byte in v.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
}

fn mix_f32(hash: &mut u64, v: f32) {
    mix_u32(hash, quantize_fire_f32(v));
}

#[inline]
fn quantize_fire_f32(v: f32) -> u32 {
    (v.clamp(0.0, 1.0) * 10_000.0).round() as u32
}

type FireCaptureItem<'a> = (
    &'a Chunk,
    &'a ChunkSurfaceFire,
    Option<&'a ChunkFireOverlay>,
    Option<&'a ChunkSmokeField>,
);

fn mean_or_zero(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn fire_row_from_parts(
    chunk: &Chunk,
    surface: &ChunkSurfaceFire,
    overlay: Option<&ChunkFireOverlay>,
    smoke_field: Option<&ChunkSmokeField>,
) -> SavedFireChunkRow {
    let (cell_heat, cell_fuel, cell_burn_hold, cell_smoke, cell_toxic) = overlay
        .map(|ovl| {
            (
                ovl.heat.clone(),
                ovl.fuel.clone(),
                ovl.burn_hold.clone(),
                ovl.smoke.clone(),
                ovl.toxic.clone(),
            )
        })
        .unwrap_or_default();
    let (smoke, toxic) = if let Some(field) = smoke_field {
        (field.density, field.toxicity)
    } else {
        (mean_or_zero(&cell_smoke), mean_or_zero(&cell_toxic))
    };
    let mut row = SavedFireChunkRow {
        chunk: [chunk.coord.x, chunk.coord.y],
        heat: surface.heat,
        fuel: surface.fuel,
        smoke,
        toxic,
        cell_heat,
        cell_fuel,
        cell_burn_hold,
        cell_smoke,
        cell_toxic,
        has_overlay: false,
    };
    crate::io::save::strip_fire_row_derived_for_save(&mut row);
    row
}

fn collect_fire_rows<'a, I>(iter: I) -> Vec<SavedFireChunkRow>
where
    I: Iterator<Item = FireCaptureItem<'a>>,
{
    let mut rows: Vec<SavedFireChunkRow> = iter
        .map(|(chunk, surface, overlay, smoke)| {
            fire_row_from_parts(chunk, surface, overlay, smoke)
        })
        .collect();
    rows.sort_by_key(|r| (r.chunk[0], r.chunk[1]));
    rows
}

pub fn capture_chunk_fire_state(
    query: &Query<(
        &Chunk,
        &ChunkSurfaceFire,
        Option<&ChunkFireOverlay>,
        Option<&ChunkSmokeField>,
    )>,
) -> ChunkFireStateSnapshot {
    ChunkFireStateSnapshot {
        schema_version: FIRE_STATE_SCHEMA_VERSION,
        rows: collect_fire_rows(query.iter()),
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
            ..SavedFireChunkRow::default()
        })
        .collect();
    hash_saved_fire_state(&ChunkFireStateSnapshot {
        schema_version: FIRE_STATE_SCHEMA_VERSION,
        rows: surface_only,
    })
}

fn overlay_from_row(row: &SavedFireChunkRow) -> Option<ChunkFireOverlay> {
    let n = row.cell_heat.len();
    if n == 0 || row.cell_fuel.len() != n || row.cell_burn_hold.len() != n {
        return None;
    }
    let smoke = if row.cell_smoke.len() == n {
        row.cell_smoke.clone()
    } else {
        vec![0.0; n]
    };
    let toxic = if row.cell_toxic.len() == n {
        row.cell_toxic.clone()
    } else {
        vec![0.0; n]
    };
    Some(ChunkFireOverlay {
        heat: row.cell_heat.clone(),
        fuel: row.cell_fuel.clone(),
        smoke,
        toxic,
        burn_hold: row.cell_burn_hold.clone(),
    })
}

fn zero_overlay(ovl: &mut ChunkFireOverlay) {
    ovl.heat.fill(0.0);
    ovl.fuel.fill(0.0);
    ovl.smoke.fill(0.0);
    ovl.toxic.fill(0.0);
    ovl.burn_hold.fill(0);
}

fn smoke_from_row(row: &SavedFireChunkRow) -> ChunkSmokeField {
    let density = row.smoke.clamp(0.0, 1.0);
    let toxicity = row.toxic.clamp(0.0, 1.0);
    ChunkSmokeField {
        density,
        toxicity,
        visibility_penalty: (density * 0.75 + toxicity * 0.25).clamp(0.0, 0.98),
    }
}

pub fn apply_chunk_fire_state(snapshot: &ChunkFireStateSnapshot, world: &mut World) {
    let schema = snapshot.schema_version;
    let mut rows = snapshot.rows.clone();
    for row in &mut rows {
        crate::io::save::rebuild_saved_fire_chunk_row_derived(row);
    }
    let wanted: HashMap<IVec2, SavedFireChunkRow> = rows
        .into_iter()
        .map(|r| (IVec2::new(r.chunk[0], r.chunk[1]), r))
        .collect();
    let mut seen = HashSet::new();
    let mut overlay_inserts: Vec<(Entity, ChunkFireOverlay)> = Vec::new();
    let mut smoke_inserts: Vec<(Entity, ChunkSmokeField)> = Vec::new();
    {
        let mut q = world.query::<(
            Entity,
            &Chunk,
            &mut ChunkSurfaceFire,
            Option<&mut ChunkFireOverlay>,
            Option<&mut ChunkSmokeField>,
        )>();
        for (entity, chunk, mut surface, mut ovl, mut smoke) in q.iter_mut(world) {
            let Some(row) = wanted.get(&chunk.coord) else {
                continue;
            };
            seen.insert(chunk.coord);
            surface.heat = row.heat;
            surface.fuel = row.fuel;
            if schema < 2 {
                continue;
            }
            if let Some(next) = overlay_from_row(row) {
                if let Some(ovl) = ovl.as_deref_mut() {
                    *ovl = next;
                } else {
                    overlay_inserts.push((entity, next));
                }
            } else if let Some(ovl) = ovl.as_deref_mut() {
                zero_overlay(ovl);
            }
            let field = smoke_from_row(row);
            if let Some(smoke) = smoke.as_deref_mut() {
                *smoke = field;
            } else {
                smoke_inserts.push((entity, field));
            }
        }
    }
    for (entity, ovl) in overlay_inserts {
        world.entity_mut(entity).insert(ovl);
    }
    for (entity, smoke) in smoke_inserts {
        world.entity_mut(entity).insert(smoke);
    }
    for (coord, row) in wanted {
        if seen.contains(&coord) {
            continue;
        }
        let mut spawned = world.spawn((
            Chunk { coord },
            ChunkSurfaceFire {
                heat: row.heat,
                fuel: row.fuel,
            },
        ));
        if schema >= 2 {
            if let Some(ovl) = overlay_from_row(&row) {
                spawned.insert(ovl);
            }
            spawned.insert(smoke_from_row(&row));
        }
    }
}

/// Write the fire overlay when a save flush is armed.
pub fn write_fire_overlay_on_save_flush(
    flush: Res<crate::io::save::SaveFlushRequested>,
    settings: Res<crate::io::save::WorldSaveBundleSettings>,
    q: Query<(
        &Chunk,
        &ChunkSurfaceFire,
        Option<&ChunkFireOverlay>,
        Option<&ChunkSmokeField>,
    )>,
) {
    if !flush.0 {
        return;
    }
    let snap = capture_chunk_fire_state(&q);
    if snap.rows.is_empty() {
        return;
    }
    let path = settings.bundle_dir.join(FIRE_STATE_REL_PATH);
    if let Err(err) = write_chunk_fire_state_ron(&path, &snap) {
        bevy::log::warn!("fire overlay write failed at {path:?}: {err}");
    }
}

#[derive(Resource, Default, Debug)]
pub struct FireOverlayHydrateState {
    pub last_bundle_dir: Option<std::path::PathBuf>,
    pub hydrate_attempted: bool,
    pub hydrated_from_manifest: bool,
}

/// Hydrate chunk fire once per bundle directory.
pub fn try_hydrate_fire_overlay_on_bundle_dir(world: &mut World) {
    let Some(bundle) = world
        .get_resource::<crate::io::save::WorldSaveBundleSettings>()
        .map(|s| s.bundle_dir.clone())
    else {
        return;
    };
    {
        let state = world.resource::<FireOverlayHydrateState>();
        if state.last_bundle_dir.as_ref() == Some(&bundle) && state.hydrate_attempted {
            return;
        }
    }
    {
        let mut state = world.resource_mut::<FireOverlayHydrateState>();
        state.last_bundle_dir = Some(bundle.clone());
        state.hydrate_attempted = true;
    }
    let path = bundle.join(FIRE_STATE_REL_PATH);
    let Ok(snap) = read_chunk_fire_state_ron(&path) else {
        return;
    };
    apply_chunk_fire_state(&snap, world);
    world
        .resource_mut::<FireOverlayHydrateState>()
        .hydrated_from_manifest = true;
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





#[cfg(test)]
mod tests {
    use super::*;

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
                ..Default::default()
            },
            SavedFireChunkRow {
                chunk: [0, 0],
                heat: 0.9,
                fuel: 0.1,
                smoke: 0.2,
                toxic: 0.0,
                has_overlay: true,
                ..Default::default()
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
                ..Default::default()
            },
            SavedFireChunkRow {
                chunk: [1, 0],
                heat: 0.3,
                fuel: 0.7,
                smoke: 0.0,
                toxic: 0.0,
                has_overlay: false,
                ..Default::default()
            },
        ]);
        assert_eq!(hash_saved_fire_state(&a), hash_saved_fire_state(&b));
    }

    #[test]
    fn burning_chunk_round_trips_cell_fire_and_keeps_v1() {
        let mut world = World::new();
        world.spawn((
            Chunk {
                coord: IVec2::new(3, -2),
            },
            ChunkSurfaceFire {
                heat: 0.42,
                fuel: 0.61,
            },
            ChunkFireOverlay {
                heat: vec![0.8, 0.15],
                fuel: vec![0.3, 0.9],
                smoke: vec![0.5, 0.05],
                toxic: vec![0.2, 0.0],
                burn_hold: vec![4, 1],
            },
            ChunkSmokeField {
                density: 0.55,
                toxicity: 0.22,
                visibility_penalty: 0.4,
            },
        ));
        let mut q = world.query::<(
            &Chunk,
            &ChunkSurfaceFire,
            Option<&ChunkFireOverlay>,
            Option<&ChunkSmokeField>,
        )>();
        let snap = ChunkFireStateSnapshot {
            schema_version: FIRE_STATE_SCHEMA_VERSION,
            rows: collect_fire_rows(q.iter(&world)),
        };
        assert_eq!(snap.schema_version, 2);
        let row = &snap.rows[0];
        assert!((row.cell_heat[0] - 0.8).abs() < 1e-5);
        assert_eq!(row.cell_burn_hold[0], 4);
        assert!((row.smoke - 0.55).abs() < 1e-5);
        let body = ron::ser::to_string(&snap).unwrap();
        assert!(body.contains("cell_heat"));
        assert!(body.contains("smoke"));
        let loaded: ChunkFireStateSnapshot = ron::from_str(&body).unwrap();

        let v1 = "(schema_version:1,rows:[(chunk:(1,0),heat:0.2,fuel:0.3)])";
        let old: ChunkFireStateSnapshot = ron::from_str(v1).expect("v1 fire overlay");
        assert!(old.rows[0].cell_heat.is_empty());
        assert_eq!(old.rows[0].smoke, 0.0);

        world.clear_entities();
        apply_chunk_fire_state(&loaded, &mut world);
        let mut hydrated = world.query::<(&Chunk, &ChunkSurfaceFire, &ChunkFireOverlay, &ChunkSmokeField)>();
        let (chunk, surface, ovl, smoke) = hydrated.iter(&world).next().expect("hydrated");
        assert_eq!(chunk.coord, IVec2::new(3, -2));
        assert!((surface.heat - 0.42).abs() < 1e-5);
        assert!((ovl.heat[0] - 0.8).abs() < 1e-5);
        assert_eq!(ovl.burn_hold, vec![4, 1]);
        assert!((ovl.fuel[1] - 0.9).abs() < 1e-5);
        assert!((smoke.density - 0.55).abs() < 1e-5);
        assert!((smoke.toxicity - 0.22).abs() < 1e-5);

        let mut v1_world = World::new();
        v1_world.spawn((
            Chunk {
                coord: IVec2::new(1, 0),
            },
            ChunkSurfaceFire {
                heat: 0.0,
                fuel: 0.0,
            },
            ChunkFireOverlay {
                heat: vec![0.7],
                fuel: vec![0.4],
                smoke: vec![0.0],
                toxic: vec![0.0],
                burn_hold: vec![3],
            },
        ));
        apply_chunk_fire_state(&old, &mut v1_world);
        let mut kept = v1_world.query::<(&ChunkSurfaceFire, &ChunkFireOverlay)>();
        let (surface, ovl) = kept.iter(&v1_world).next().expect("v1 entity");
        assert!((surface.heat - 0.2).abs() < 1e-5);
        assert!((ovl.heat[0] - 0.7).abs() < 1e-5);
        assert_eq!(ovl.burn_hold[0], 3);
    }
}
