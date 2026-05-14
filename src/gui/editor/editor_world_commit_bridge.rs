//! **Editor → simulation glue** — one causality injection point for tile authoring:
//! [`EditorTileEditCommitted`] → [`ChunkGenQueue`] + live [`ChunkDirty`] + [`invalidate_world`] (preview epoch + chunk queue).
//!
//! Map editor uses a dense [`TileMarker`](crate::terrain::generation::world_generator_enhanced::TileMarker) grid; U7 uses
//! [`Chunk`](crate::terrain::generation::Chunk) slabs. [`crate::terrain::generation::tile_chunk_map`] bridges coordinates.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::engine::BaseState;
use crate::terrain::generation::world_generator_enhanced::{
    Height, Moisture, Temperature, TerrainType, TileMarker, WorldGenParams,
};
use crate::terrain::generation::{
    sync_tile_markers_into_affected_chunk_matrices, tile_rect_to_chunk_coords, Chunk, ChunkCellMatrix,
    ChunkGenQueue,
};
use crate::terrain::material::{
    invalidate_world, ChunkDependency, ChunkDirty, InvalidationReason, WorldPreviewState, DIRTY_ALL,
};
use crate::io::save::DirtyChunkSaveQueue;
use crate::io::save::RequestWorldSaveFlush;

/// What changed in the map editor (for logging / future AI read models).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditorTileEditKind {
    TerrainHeight,
    TerrainBiome,
    RoadMarker,
    /// Full grid replace (map snapshot load, procedural hydrate hooks).
    MapSnapshotImport,
    /// Transport graph / corridor topology baked from road markers — invalidate logistics + world fields.
    TransportTopology,
}

/// Inclusive tile AABB in editor space: `min_tile.x` = column, `min_tile.y` = row (`z` in world).
#[derive(Clone, Copy, Debug, Message)]
pub struct EditorTileEditCommitted {
    pub min_tile: UVec2,
    pub max_tile: UVec2,
    pub kind: EditorTileEditKind,
}

/// Fallback when no [`ChunkCellMatrix`] exists yet (must match game chunk slab size).
#[derive(Resource, Clone, Copy, Debug)]
pub struct EditorWorldCommitSettings {
    pub cells_per_chunk: UVec2,
}

impl Default for EditorWorldCommitSettings {
    fn default() -> Self {
        Self {
            cells_per_chunk: UVec2::new(32, 32),
        }
    }
}

/// Full-world tile AABB commit (snapshot load, transport reroute, etc.).
pub fn write_editor_world_grid_commit(
    writer: &mut MessageWriter<EditorTileEditCommitted>,
    params: &WorldGenParams,
    kind: EditorTileEditKind,
) {
    let w = params.width;
    let h = params.height;
    if w == 0 || h == 0 {
        return;
    }
    writer.write(EditorTileEditCommitted {
        min_tile: UVec2::ZERO,
        max_tile: UVec2::new(w - 1, h - 1),
        kind,
    });
}

#[inline]
fn clamp_tile_rect_to_params(
    mut min: UVec2,
    mut max: UVec2,
    params: &WorldGenParams,
) -> Option<(UVec2, UVec2)> {
    let w = params.width;
    let h = params.height;
    if w == 0 || h == 0 {
        return None;
    }
    let mx = w.saturating_sub(1);
    let mz = h.saturating_sub(1);
    min.x = min.x.min(mx);
    min.y = min.y.min(mz);
    max.x = max.x.min(mx);
    max.y = max.y.min(mz);
    if min.x > max.x {
        core::mem::swap(&mut min.x, &mut max.x);
    }
    if min.y > max.y {
        core::mem::swap(&mut min.y, &mut max.y);
    }
    Some((min, max))
}

pub fn editor_world_commit_to_simulation_system(
    mut events: MessageReader<EditorTileEditCommitted>,
    mut queue: ResMut<ChunkGenQueue>,
    mut preview: ResMut<WorldPreviewState>,
    save_dirty: Option<ResMut<DirtyChunkSaveQueue>>,
    save_flush: Option<MessageWriter<RequestWorldSaveFlush>>,
    mut chunk_access: ParamSet<(
        Query<&ChunkCellMatrix, With<ChunkDependency>>,
        Query<(&Chunk, &mut ChunkCellMatrix), With<ChunkDependency>>,
        Query<(&Chunk, &mut ChunkDirty), With<ChunkDependency>>,
    )>,
    tiles: Query<(&Transform, &Height, &Moisture, &Temperature, &TerrainType), With<TileMarker>>,
    settings: Res<EditorWorldCommitSettings>,
    params: Res<WorldGenParams>,
) {
    let cells = chunk_access
        .p0()
        .iter()
        .next()
        .map(|m| m.size)
        .unwrap_or(settings.cells_per_chunk);
    let cells = UVec2::new(cells.x.max(1), cells.y.max(1));

    let mut merged: Vec<IVec2> = Vec::new();
    for evt in events.read() {
        let Some((min, max)) = clamp_tile_rect_to_params(evt.min_tile, evt.max_tile, &params) else {
            continue;
        };
        merged.extend(tile_rect_to_chunk_coords(
            min.x, min.y, max.x, max.y, cells,
        ));
    }
    merged.sort_by(|a, b| (a.y, a.x).cmp(&(b.y, b.x)));
    merged.dedup();
    if merged.is_empty() {
        return;
    }

    let affected: HashSet<IVec2> = merged.iter().copied().collect();
    sync_tile_markers_into_affected_chunk_matrices(
        &affected,
        cells,
        &params,
        &tiles,
        &mut chunk_access.p1(),
    );

    for coord in &merged {
        queue.push_editor_edit(*coord);
    }
    if let Some(mut save_dirty) = save_dirty {
        for coord in &merged {
            save_dirty.enqueue(*coord);
        }
        if let Some(mut save_flush) = save_flush {
            save_flush.write(RequestWorldSaveFlush);
        }
    }
    for coord in &merged {
        for (chunk, mut dirty) in chunk_access.p2().iter_mut() {
            if chunk.coord == *coord {
                dirty.passes |= DIRTY_ALL;
                break;
            }
        }
    }
    invalidate_world(
        InvalidationReason::Noise,
        &mut preview,
        merged.iter().copied(),
    );
}

fn editor_or_simulation_active(state: Option<Res<State<BaseState>>>) -> bool {
    match state {
        None => true,
        Some(s) => matches!(*s.get(), BaseState::Editor | BaseState::Simulation),
    }
}

pub struct EditorWorldCommitBridgePlugin;

impl Plugin for EditorWorldCommitBridgePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<EditorTileEditCommitted>()
            .init_resource::<EditorWorldCommitSettings>()
            .add_systems(
                Update,
                editor_world_commit_to_simulation_system
                    .run_if(editor_or_simulation_active)
                    .before(crate::terrain::generation::dispatch_chunk_jobs),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::material::ChunkDependency;
    use bevy::prelude::{MinimalPlugins, Transform};

    #[test]
    fn commit_marks_dirty_and_enqueues_editor_job() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<EditorTileEditCommitted>()
            .init_resource::<ChunkGenQueue>()
            .init_resource::<WorldPreviewState>()
            .init_resource::<EditorWorldCommitSettings>()
            .init_resource::<WorldGenParams>()
            .add_systems(Update, editor_world_commit_to_simulation_system);

        {
            let world = app.world_mut();
            let matrix = ChunkCellMatrix::new(UVec2::new(8, 8));
            world.spawn((
                Chunk {
                    coord: IVec2::ZERO,
                },
                matrix,
                ChunkDirty::default(),
                ChunkDependency {
                    source_noise_id: 0,
                    registry_hash: 0,
                    families_hash: 0,
                    rules_hash: 0,
                    tags_hash: 0,
                    tuning_hash: 0,
                    preview_hash: 0,
                },
            ));
            world.spawn((
                TileMarker,
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                Height(0.42),
                Moisture(0.11),
                Temperature(0.22),
                TerrainType(crate::terrain::family::DEFAULT_TERRAIN_FAMILY_ID),
            ));
            world.resource_mut::<WorldGenParams>().width = 64;
            world.resource_mut::<WorldGenParams>().height = 64;
            world.write_message(EditorTileEditCommitted {
                min_tile: UVec2::ZERO,
                max_tile: UVec2::new(3, 3),
                kind: EditorTileEditKind::TerrainHeight,
            });
        }
        app.update();
        assert!(
            app.world().resource::<ChunkGenQueue>().pending_len() >= 1,
            "editor job should queue"
        );
        let world = app.world_mut();
        let dirty = world
            .query::<&ChunkDirty>()
            .iter(world)
            .next()
            .expect("dirty");
        assert_ne!(dirty.passes & DIRTY_ALL, 0);
        assert!(world.resource::<WorldPreviewState>().epoch.0 > 0);
        let matrix = world
            .query::<&ChunkCellMatrix>()
            .iter(world)
            .next()
            .expect("matrix");
        assert!((matrix.elevation[0] - 0.42).abs() < 1e-6);
        assert!((matrix.moisture[0] - 0.11).abs() < 1e-6);
        assert!((matrix.temperature[0] - 0.22).abs() < 1e-6);
    }
}
