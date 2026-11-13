//! Per-chunk [`ChunkCellStand`] + per-cell fire overlay — RON sibling to chunk-mean fire
//! (`plan_weather_wind_v1.md` W2 burn-scar persistence).

use std::collections::{HashMap, HashSet};
use std::path::Path;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::systems::ecology::{ChunkCellStand, StandKind};
use crate::systems::fire::ChunkFireOverlay;
use crate::terrain::generation::Chunk;

pub const STAND_OVERLAY_NAME: &str = "chunk_cell_stand";
pub const STAND_STATE_REL_PATH: &str = "overlays/chunk_cell_stand.ron";
pub const STAND_STATE_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SavedCellStandRow {
    pub chunk: [i32; 2],
    pub kind: Vec<u8>,
    pub full_cap: Vec<f32>,
    pub fuel_cap: Vec<f32>,
    pub burnt_at: Vec<f32>,
    pub burnt_count: u32,
    pub seeded: bool,
    #[serde(default)]
    pub heat: Vec<f32>,
    #[serde(default)]
    pub fuel: Vec<f32>,
    #[serde(default)]
    pub burn_hold: Vec<u16>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChunkCellStandSnapshot {
    pub schema_version: u32,
    pub rows: Vec<SavedCellStandRow>,
}

#[must_use]
pub fn default_stand_overlay_ref() -> crate::io::save::OverlaySnapshotRef {
    crate::io::save::OverlaySnapshotRef {
        overlay_name: STAND_OVERLAY_NAME.into(),
        artifact_path: STAND_STATE_REL_PATH.into(),
    }
}

fn kind_to_u8(k: StandKind) -> u8 {
    k as u8
}

fn kind_from_u8(v: u8) -> StandKind {
    match v {
        1 => StandKind::Grass,
        2 => StandKind::Brush,
        3 => StandKind::Timber,
        _ => StandKind::Bare,
    }
}

#[must_use]
pub fn capture_chunk_cell_stand(
    query: &Query<(&Chunk, &ChunkCellStand, Option<&ChunkFireOverlay>)>,
) -> ChunkCellStandSnapshot {
    let mut rows: Vec<SavedCellStandRow> = query
        .iter()
        .map(|(chunk, stand, ovl)| {
            let (heat, fuel, burn_hold) = ovl
                .map(|o| (o.heat.clone(), o.fuel.clone(), o.burn_hold.clone()))
                .unwrap_or_default();
            SavedCellStandRow {
                chunk: [chunk.coord.x, chunk.coord.y],
                kind: stand.kind.iter().copied().map(kind_to_u8).collect(),
                full_cap: stand.full_cap.clone(),
                fuel_cap: stand.fuel_cap.clone(),
                burnt_at: stand.burnt_at.clone(),
                burnt_count: stand.burnt_count,
                seeded: stand.seeded,
                heat,
                fuel,
                burn_hold,
            }
        })
        .collect();
    rows.sort_by_key(|r| (r.chunk[0], r.chunk[1]));
    ChunkCellStandSnapshot {
        schema_version: STAND_STATE_SCHEMA_VERSION,
        rows,
    }
}

pub fn apply_chunk_cell_stand(snapshot: &ChunkCellStandSnapshot, world: &mut World) {
    let wanted: HashMap<IVec2, &SavedCellStandRow> = snapshot
        .rows
        .iter()
        .map(|r| (IVec2::new(r.chunk[0], r.chunk[1]), r))
        .collect();
    let mut seen = HashSet::new();
    {
        let mut q = world.query::<(&Chunk, &mut ChunkCellStand, Option<&mut ChunkFireOverlay>)>();
        for (chunk, mut stand, ovl) in q.iter_mut(world) {
            let Some(row) = wanted.get(&chunk.coord) else {
                continue;
            };
            seen.insert(chunk.coord);
            let n = row.kind.len();
            if n == 0 {
                continue;
            }
            if stand.kind.len() != n {
                *stand = ChunkCellStand::new(n);
            }
            stand.kind = row.kind.iter().copied().map(kind_from_u8).collect();
            stand.full_cap = row.full_cap.clone();
            stand.fuel_cap = row.fuel_cap.clone();
            stand.burnt_at = row.burnt_at.clone();
            stand.burnt_count = row.burnt_count;
            stand.seeded = row.seeded;
            if let Some(mut ovl) = ovl {
                if row.heat.len() == n {
                    ovl.heat = row.heat.clone();
                }
                if row.fuel.len() == n {
                    ovl.fuel = row.fuel.clone();
                }
                if row.burn_hold.len() == n {
                    ovl.burn_hold = row.burn_hold.clone();
                }
            }
        }
    }
    for (coord, row) in wanted {
        if seen.contains(&coord) || row.kind.is_empty() {
            continue;
        }
        let n = row.kind.len();
        let mut stand = ChunkCellStand::new(n);
        stand.kind = row.kind.iter().copied().map(kind_from_u8).collect();
        stand.full_cap = row.full_cap.clone();
        stand.fuel_cap = row.fuel_cap.clone();
        stand.burnt_at = row.burnt_at.clone();
        stand.burnt_count = row.burnt_count;
        stand.seeded = row.seeded;
        world.spawn((
            Chunk { coord },
            stand,
            ChunkFireOverlay {
                heat: if row.heat.len() == n {
                    row.heat.clone()
                } else {
                    vec![0.0; n]
                },
                fuel: if row.fuel.len() == n {
                    row.fuel.clone()
                } else {
                    vec![0.0; n]
                },
                smoke: vec![0.0; n],
                toxic: vec![0.0; n],
                burn_hold: if row.burn_hold.len() == n {
                    row.burn_hold.clone()
                } else {
                    vec![0; n]
                },
            },
        ));
    }
}

pub fn write_chunk_cell_stand_ron(
    path: impl AsRef<Path>,
    snapshot: &ChunkCellStandSnapshot,
) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = ron::ser::to_string_pretty(snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(path, body)
}

pub fn read_chunk_cell_stand_ron(path: impl AsRef<Path>) -> std::io::Result<ChunkCellStandSnapshot> {
    let body = std::fs::read_to_string(path)?;
    ron::from_str(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))
}

/// Write stand + per-cell fire overlay when a save flush is armed.
pub fn write_stand_overlay_on_save_flush(
    flush: Res<crate::io::save::SaveFlushRequested>,
    settings: Res<crate::io::save::WorldSaveBundleSettings>,
    q: Query<(&Chunk, &ChunkCellStand, Option<&ChunkFireOverlay>)>,
) {
    if !flush.0 {
        return;
    }
    let snap = capture_chunk_cell_stand(&q);
    if snap.rows.is_empty() {
        return;
    }
    let path = settings.bundle_dir.join(STAND_STATE_REL_PATH);
    if let Err(err) = write_chunk_cell_stand_ron(&path, &snap) {
        bevy::log::warn!("stand overlay write failed at {path:?}: {err}");
    }
}

#[derive(Resource, Default, Debug)]
pub struct StandOverlayHydrateState {
    pub last_bundle_dir: Option<std::path::PathBuf>,
    pub hydrate_attempted: bool,
    pub hydrated_from_manifest: bool,
}

/// Hydrate stand scars from the bundle when the overlay file is present.
pub fn try_hydrate_stand_overlay_on_bundle_dir(
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    mut state: ResMut<StandOverlayHydrateState>,
    mut commands: Commands,
    mut q: Query<(
        Entity,
        &Chunk,
        &mut ChunkCellStand,
        Option<&mut ChunkFireOverlay>,
    )>,
) {
    let Some(settings) = settings else {
        return;
    };
    let bundle = settings.bundle_dir.clone();
    if state.last_bundle_dir.as_ref() == Some(&bundle) && state.hydrate_attempted {
        return;
    }
    state.last_bundle_dir = Some(bundle.clone());
    state.hydrate_attempted = true;
    let path = bundle.join(STAND_STATE_REL_PATH);
    let Ok(snap) = read_chunk_cell_stand_ron(&path) else {
        return;
    };
    let mut seen = HashSet::new();
    let wanted: HashMap<IVec2, &SavedCellStandRow> = snap
        .rows
        .iter()
        .map(|r| (IVec2::new(r.chunk[0], r.chunk[1]), r))
        .collect();
    for (_e, chunk, mut stand, ovl) in &mut q {
        let Some(row) = wanted.get(&chunk.coord) else {
            continue;
        };
        seen.insert(chunk.coord);
        let n = row.kind.len();
        if n == 0 {
            continue;
        }
        if stand.kind.len() != n {
            *stand = ChunkCellStand::new(n);
        }
        stand.kind = row.kind.iter().copied().map(kind_from_u8).collect();
        stand.full_cap = row.full_cap.clone();
        stand.fuel_cap = row.fuel_cap.clone();
        stand.burnt_at = row.burnt_at.clone();
        stand.burnt_count = row.burnt_count;
        stand.seeded = row.seeded;
        if let Some(mut ovl) = ovl {
            if row.heat.len() == n {
                ovl.heat = row.heat.clone();
            }
            if row.fuel.len() == n {
                ovl.fuel = row.fuel.clone();
            }
            if row.burn_hold.len() == n {
                ovl.burn_hold = row.burn_hold.clone();
            }
        }
    }
    for (coord, row) in wanted {
        if seen.contains(&coord) || row.kind.is_empty() {
            continue;
        }
        let n = row.kind.len();
        let mut stand = ChunkCellStand::new(n);
        stand.kind = row.kind.iter().copied().map(kind_from_u8).collect();
        stand.full_cap = row.full_cap.clone();
        stand.fuel_cap = row.fuel_cap.clone();
        stand.burnt_at = row.burnt_at.clone();
        stand.burnt_count = row.burnt_count;
        stand.seeded = row.seeded;
        commands.spawn((
            Chunk { coord },
            stand,
            ChunkFireOverlay {
                heat: if row.heat.len() == n {
                    row.heat.clone()
                } else {
                    vec![0.0; n]
                },
                fuel: if row.fuel.len() == n {
                    row.fuel.clone()
                } else {
                    vec![0.0; n]
                },
                smoke: vec![0.0; n],
                toxic: vec![0.0; n],
                burn_hold: if row.burn_hold.len() == n {
                    row.burn_hold.clone()
                } else {
                    vec![0; n]
                },
            },
        ));
    }
    state.hydrated_from_manifest = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::ecology::StandKind;

    #[test]
    fn roundtrip_preserves_burnt_clock() {
        let mut stand = ChunkCellStand::new(4);
        stand.kind = vec![StandKind::Timber; 4];
        stand.full_cap = vec![0.5; 4];
        stand.fuel_cap = vec![0.0; 4];
        stand.burnt_at = vec![12.0, -1.0, 40.0, -1.0];
        stand.burnt_count = 2;
        stand.seeded = true;
        let mut world = World::new();
        let e = world
            .spawn((
                Chunk {
                    coord: IVec2::new(2, -1),
                },
                stand,
                ChunkFireOverlay {
                    heat: vec![0.1; 4],
                    fuel: vec![0.0; 4],
                    smoke: vec![0.0; 4],
                    toxic: vec![0.0; 4],
                    burn_hold: vec![2, 0, 0, 0],
                },
            ))
            .id();
        let snap = {
            let stand = world.get::<ChunkCellStand>(e).unwrap();
            let ovl = world.get::<ChunkFireOverlay>(e).unwrap();
            let chunk = world.get::<Chunk>(e).unwrap();
            ChunkCellStandSnapshot {
                schema_version: STAND_STATE_SCHEMA_VERSION,
                rows: vec![SavedCellStandRow {
                    chunk: [chunk.coord.x, chunk.coord.y],
                    kind: stand.kind.iter().copied().map(|k| k as u8).collect(),
                    full_cap: stand.full_cap.clone(),
                    fuel_cap: stand.fuel_cap.clone(),
                    burnt_at: stand.burnt_at.clone(),
                    burnt_count: stand.burnt_count,
                    seeded: stand.seeded,
                    heat: ovl.heat.clone(),
                    fuel: ovl.fuel.clone(),
                    burn_hold: ovl.burn_hold.clone(),
                }],
            }
        };
        world.clear_entities();
        apply_chunk_cell_stand(&snap, &mut world);
        let mut q = world.query::<(&Chunk, &ChunkCellStand, &ChunkFireOverlay)>();
        let (chunk, stand, ovl) = q.iter(&world).next().expect("hydrated");
        assert_eq!(chunk.coord, IVec2::new(2, -1));
        assert_eq!(stand.burnt_count, 2);
        assert!((stand.burnt_at[0] - 12.0).abs() < 1e-5);
        assert_eq!(ovl.burn_hold[0], 2);
        assert!(stand.seeded);
    }
}
