//! Save pipeline â€” ECS DTO graph â†’ compression â†’ async IO â†’ atomic swap.



use std::fs;

use std::io;

use std::path::{Path, PathBuf};



use bevy::math::IVec2;

use bevy::prelude::*;



use crate::io::save::async_io::{poll_save_io_completions, SaveIoDispatcher, SaveIoWorkOrder};

use crate::io::save::dirty_queue::DirtyChunkSaveQueue;

use crate::io::save::dto::encode_chunk_body_ron;

use crate::io::save::wire_format;

use crate::io::save::manifest::{

    build_save_world_manifest, ChunkSetRef, OverlaySnapshotRef, SaveWorldManifest,

};

use crate::io::save::registry_snapshot::{

    build_default_registry_snapshot_refs, write_registry_snapshot_artifacts,

};

use crate::io::save::snapshot_builder::{

    build_chunk_save_snapshot_input, build_saved_chunk_body, ChunkSaveSnapshotInput,

};

use crate::systems::terrain::TerrainRegistriesHandles;

use crate::terrain::generation::cell_matrix::ChunkCellMatrix;

use crate::terrain::generation::Chunk;

use crate::terrain::material::{MaterializedChunk, MaterialRegistry, TagRegistry};



/// Completed save job ready for main-thread ECS apply / manifest merge.

#[derive(Debug, Clone, PartialEq, Eq)]

pub struct SavePipelineJob {

    pub chunk: IVec2,

    pub artifact_path: PathBuf,

    pub body_bytes: Vec<u8>,

}



/// Root directory for incremental save bundles.

#[derive(Resource, Clone, Debug)]

pub struct WorldSaveBundleSettings {

    pub bundle_dir: PathBuf,

}



impl Default for WorldSaveBundleSettings {

    fn default() -> Self {

        Self {

            bundle_dir: PathBuf::from("saves/incremental"),

        }

    }

}



/// Explicit save flush request (autosave / editor / dev tools).

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]

pub struct SaveFlushRequested(pub bool);



pub fn write_artifact_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {

    if let Some(parent) = path.parent() {

        fs::create_dir_all(parent)?;

    }

    let tmp = path.with_extension("tmp");

    fs::write(&tmp, bytes)?;

    if path.exists() {

        fs::remove_file(path)?;

    }

    fs::rename(tmp, path)

}



#[must_use]

pub fn chunk_artifact_path(bundle_dir: &Path, chunk: IVec2) -> PathBuf {

    bundle_dir

        .join("chunks")

        .join(format!("{}_{}.ron", chunk.x, chunk.y))

}



#[must_use]

pub fn build_incremental_save_manifest(

    world_seed: u64,

    jobs: &[SavePipelineJob],

) -> SaveWorldManifest {

    let chunk_sets = jobs

        .iter()

        .map(|job| ChunkSetRef {

            chunk: [job.chunk.x, job.chunk.y],

            artifact_path: job

                .artifact_path

                .to_string_lossy()

                .replace('\\', "/"),

        })

        .collect();

    build_save_world_manifest(

        world_seed,

        chunk_sets,

        build_default_registry_snapshot_refs(),

        Vec::<OverlaySnapshotRef>::new(),

    )

}



pub fn stage_dirty_chunk_save_jobs(

    world_seed: u64,

    bundle_dir: &Path,

    dirty: &[IVec2],

    snapshots: &[ChunkSaveSnapshotInput],

    material_registry: &MaterialRegistry,

    tag_registry: &TagRegistry,

) -> io::Result<(SaveWorldManifest, Vec<SavePipelineJob>)> {

    write_registry_snapshot_artifacts(bundle_dir)?;

    let mut jobs = Vec::new();

    for coord in dirty {

        let Some(input) = snapshots.iter().find(|row| row.coord == *coord) else {

            continue;

        };

        let body = build_saved_chunk_body(input, material_registry, tag_registry);

        let encoded = encode_chunk_body_ron(&body).map_err(|e| {

            io::Error::new(io::ErrorKind::InvalidData, e.to_string())

        })?;

        let compressed = wire_format::compress_payload(&encoded);

        let artifact_path = chunk_artifact_path(bundle_dir, *coord);

        write_artifact_atomic(&artifact_path, &compressed)?;

        jobs.push(SavePipelineJob {

            chunk: *coord,

            artifact_path,

            body_bytes: compressed,

        });

    }

    Ok((build_incremental_save_manifest(world_seed, &jobs), jobs))

}



pub fn write_manifest_atomic(bundle_dir: &Path, manifest: &SaveWorldManifest) -> io::Result<()> {

    let encoded = ron::ser::to_string(manifest).map_err(|e| {

        io::Error::new(io::ErrorKind::InvalidData, e.to_string())

    })?;

    write_artifact_atomic(&bundle_dir.join("manifest.ron"), encoded.as_bytes())

}



#[must_use]

pub fn collect_chunk_save_snapshots(

    dirty: &[IVec2],

    chunks: &Query<(&Chunk, &MaterializedChunk, Option<&ChunkCellMatrix>)>,

) -> Vec<ChunkSaveSnapshotInput> {

    dirty

        .iter()

        .filter_map(|coord| {

            chunks.iter().find(|(chunk, _, _)| chunk.coord == *coord).map(

                |(chunk, mat_chunk, cell_matrix)| {

                    build_chunk_save_snapshot_input(chunk.coord, mat_chunk, cell_matrix)

                },

            )

        })

        .collect()

}



pub fn flush_dirty_chunk_save_queue(

    settings: Res<WorldSaveBundleSettings>,

    world_seed: Res<WorldSaveSeed>,

    mut queue: ResMut<DirtyChunkSaveQueue>,

    handles: Res<TerrainRegistriesHandles>,

    materials: Res<Assets<MaterialRegistry>>,

    tags: Res<Assets<TagRegistry>>,

    chunks: Query<(&Chunk, &MaterializedChunk, Option<&ChunkCellMatrix>)>,

    mut dispatcher: ResMut<SaveIoDispatcher>,

    mut flush: ResMut<SaveFlushRequested>,

) {

    if !flush.0 || queue.is_empty() {

        return;

    }

    flush.0 = false;

    let Some(material_registry) = materials.get(&handles.material_registry) else {

        return;

    };

    let Some(tag_registry) = tags.get(&handles.tag_registry) else {

        return;

    };

    let dirty = queue.drain();

    let snapshots = collect_chunk_save_snapshots(&dirty, &chunks);

    let chunk_bodies = snapshots

        .iter()

        .map(|input| {

            (

                input.coord,

                build_saved_chunk_body(input, material_registry, tag_registry),

            )

        })

        .collect();

    dispatcher.ensure_started();

    let submitted = dispatcher.submit(SaveIoWorkOrder {

        bundle_dir: settings.bundle_dir.clone(),

        world_seed: world_seed.0,

        chunk_bodies,

    });

    if !submitted {

        bevy::log::warn!("save IO dispatcher rejected flush work order");

    }

}



/// Synchronous flush for tests and tooling (no async worker).

pub fn flush_dirty_chunk_save_queue_sync(

    bundle_dir: &Path,

    world_seed: u64,

    dirty: &[IVec2],

    snapshots: &[ChunkSaveSnapshotInput],

    material_registry: &MaterialRegistry,

    tag_registry: &TagRegistry,

) -> io::Result<(SaveWorldManifest, Vec<SavePipelineJob>)> {

    let (manifest, jobs) = stage_dirty_chunk_save_jobs(

        world_seed,

        bundle_dir,

        dirty,

        snapshots,

        material_registry,

        tag_registry,

    )?;

    write_manifest_atomic(bundle_dir, &manifest)?;

    Ok((manifest, jobs))

}



/// World seed stamped into save manifests.

#[derive(Resource, Clone, Copy, Debug, Default)]

pub struct WorldSaveSeed(pub u64);



/// Main-thread apply queue after async serialization completes.

#[derive(Resource, Debug, Default, Clone)]

pub struct PendingSaveApplyQueue {

    pub jobs: Vec<SavePipelineJob>,

}



pub struct WorldSaveSpinePlugin;



impl Plugin for WorldSaveSpinePlugin {

    fn build(&self, app: &mut App) {

        app.init_resource::<DirtyChunkSaveQueue>()

            .init_resource::<WorldSaveSeed>()

            .init_resource::<PendingSaveApplyQueue>()

            .init_resource::<WorldSaveBundleSettings>()

            .init_resource::<SaveFlushRequested>()

            .init_resource::<SaveIoDispatcher>()
            .init_resource::<crate::io::save::autosave::WorldSaveAutosaveSettings>()
            .init_resource::<crate::io::save::WaveSShellCapturePending>()
            .init_resource::<crate::io::save::WaveSShellRestorePending>()
            .init_resource::<crate::io::save::WaveSShellHydrateState>()
            .init_resource::<crate::io::save::WaveSShellHydrateWitness>()
            .init_resource::<crate::io::save::WaveSImportedBlueprints>()
            .init_resource::<crate::io::save::WaveSLiveProofState>()

            .add_message::<crate::io::save::dirty_queue::RequestWorldSaveFlush>()

            .add_systems(

                Update,

                (

                    crate::io::save::dirty_queue::enqueue_dirty_chunks_from_preview,

                    crate::io::save::dirty_queue::enqueue_dirty_chunks_from_environment_hooks,

                    crate::io::save::dirty_queue::arm_save_flush_from_requests,

                    crate::io::save::autosave::tick_world_save_autosave,

                    flush_dirty_chunk_save_queue,

                    poll_save_io_completions,

                    crate::io::save::apply::apply_pending_save_pipeline_jobs,
                    crate::io::save::apply_wave_s_shell_capture_requests,
                    crate::io::save::try_autoload_wave_s_on_bundle_dir,
                    crate::io::save::apply_wave_s_shell_restore_requests,
                    crate::io::save::write_wave_s_hydrate_live_proof_system,

                )

            );

    }

}



#[cfg(test)]

mod tests {

    use super::*;

    use crate::io::save::load::hydrate_chunk_bodies_from_manifest;
    use crate::io::save::{unwrap_chunk_artifact_body, SAVE_ARTIFACT_MAGIC};

    use crate::io::save::snapshot_builder::tag_names_from_set;

    use crate::terrain::family::TerrainFamilyId;

    use crate::terrain::material::{MaterialDef, TagDef, TagId, TagSet};



    #[test]

    fn incremental_save_writes_manifest_for_dirty_chunks_only() {

        let dir = std::env::temp_dir().join(format!(

            "wave_s_incremental_{}",

            std::time::SystemTime::now()

                .duration_since(std::time::UNIX_EPOCH)

                .unwrap()

                .as_nanos()

        ));

        let mut registry = MaterialRegistry {

            schema_version: 1,

            materials: vec![MaterialDef {

                name: "grass".into(),

                family: TerrainFamilyId(0),

                tags: Vec::new(),

                properties: serde_json::json!({}),

                preview_color: [0, 128, 0, 255],

            }],

            name_to_id: Default::default(),

        };

        registry.name_to_id.insert("grass".into(), crate::terrain::material::MaterialId(0));

        let tag_registry = TagRegistry {

            schema_version: 1,

            tags: vec![TagDef {

                name: "wet".into(),

                category: "moisture".into(),

            }],

            name_to_id: [("wet".into(), TagId(0))].into_iter().collect(),

        };

        let mut tags = TagSet::default();

        tags.insert(TagId(0));

        let snapshot = ChunkSaveSnapshotInput {

            coord: IVec2::new(3, 4),

            materials: vec![crate::terrain::material::MaterialId(0)],

            tags: vec![tags],

        };

        let dirty = vec![IVec2::new(3, 4)];

        let (manifest, jobs) = flush_dirty_chunk_save_queue_sync(

            &dir,

            42,

            &dirty,

            &[snapshot],

            &registry,

            &tag_registry,

        )

        .unwrap();

        assert_eq!(manifest.world_seed, 42);

        assert_eq!(manifest.chunk_sets.len(), 1);

        assert_eq!(jobs.len(), 1);

        assert!(dir.join("manifest.ron").exists());

        assert!(dir.join("registries/material_registry.ron").exists());

        let bodies = hydrate_chunk_bodies_from_manifest(&dir, &manifest).unwrap();

        assert_eq!(bodies[0].cells[0].material_name, "grass");

        assert_eq!(bodies[0].cells[0].tags, vec!["wet".to_string()]);

        let ids = crate::io::save::material_ids_from_saved_body(&bodies[0], &registry);

        assert_eq!(ids, vec![crate::terrain::material::MaterialId(0)]);

        assert_eq!(&jobs[0].body_bytes[..4], SAVE_ARTIFACT_MAGIC);
        let payload = unwrap_chunk_artifact_body(&jobs[0].body_bytes).unwrap();
        let encoded = String::from_utf8(payload.to_vec()).unwrap();

        assert!(!encoded.contains("MaterialId"));

        let _ = fs::remove_dir_all(&dir);

    }



    #[test]

    fn tag_names_from_set_resolves_registry_names() {

        let registry = TagRegistry {

            schema_version: 1,

            tags: vec![

                TagDef {

                    name: "a".into(),

                    category: "c".into(),

                },

                TagDef {

                    name: "b".into(),

                    category: "c".into(),

                },

            ],

            name_to_id: Default::default(),

        };

        let mut set = TagSet::default();

        set.insert(TagId(1));

        assert_eq!(tag_names_from_set(&set, &registry), vec!["b".to_string()]);

    }

}


