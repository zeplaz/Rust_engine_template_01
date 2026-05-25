//! Load / hydrate helpers for Wave S save bundles.
//!
//! **On-disk layout (Wave S):** `{bundle_dir}/manifest.ron` + chunk artifacts referenced by manifest.
//! **Product shell (BQ-133):** [`crate::io::save::WAVE_S_PRODUCT_SHELL_REL_PATH`] — RON
//! [`ProductShellPersistenceBundleR8`]; fixture `debug_runs/wave_s_shell_roundtrip.json`.
//! **Blueprints (BQ-128):** [`crate::io::save::WAVE_S_BLUEPRINT_PRESETS_REL_PATH`] — RON
//! [`BlueprintPresetCollectionR8`]; fixture `debug_runs/wave_s_blueprint_roundtrip.json`.

use std::fs;
use std::io;
use std::path::Path;

use bevy::prelude::IVec2;

use crate::io::save::dto::{decode_chunk_body_ron, SavedChunkBody};
use crate::io::save::manifest::{SaveWorldManifest, SAVE_WORLD_MANIFEST_SCHEMA_VERSION};
use crate::io::save::wire_format::unwrap_chunk_artifact_body;

pub fn read_manifest_from_bundle(bundle_dir: &Path) -> io::Result<SaveWorldManifest> {
    let bytes = fs::read(bundle_dir.join("manifest.ron"))?;
    let text = std::str::from_utf8(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let manifest: SaveWorldManifest =
        ron::de::from_str(text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    validate_manifest(&manifest)?;
    Ok(manifest)
}

pub fn validate_manifest(manifest: &SaveWorldManifest) -> io::Result<()> {
    if manifest.schema_version != SAVE_WORLD_MANIFEST_SCHEMA_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported manifest schema_version {} (expected {SAVE_WORLD_MANIFEST_SCHEMA_VERSION})",
                manifest.schema_version
            ),
        ));
    }
    Ok(())
}

pub fn load_chunk_body_from_artifact(bundle_dir: &Path, artifact_path: &str) -> io::Result<SavedChunkBody> {
    let path = bundle_dir.join(artifact_path);
    let bytes = fs::read(path)?;
    let payload = unwrap_chunk_artifact_body(&bytes)?;
    let body = decode_chunk_body_ron(payload).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    if body.schema_version != crate::io::save::dto::SAVED_CHUNK_BODY_SCHEMA_VERSION {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported chunk schema_version {}", body.schema_version),
        ));
    }
    Ok(body)
}

pub fn hydrate_chunk_bodies_from_manifest(
    bundle_dir: &Path,
    manifest: &SaveWorldManifest,
) -> io::Result<Vec<SavedChunkBody>> {
    validate_manifest(manifest)?;
    manifest
        .chunk_sets
        .iter()
        .map(|chunk_ref| load_chunk_body_from_artifact(bundle_dir, &chunk_ref.artifact_path))
        .collect()
}

pub fn load_chunk_body_for_coord(
    bundle_dir: &Path,
    manifest: &SaveWorldManifest,
    chunk: IVec2,
) -> io::Result<Option<SavedChunkBody>> {
    let Some(chunk_ref) = manifest
        .chunk_sets
        .iter()
        .find(|entry| entry.chunk[0] == chunk.x && entry.chunk[1] == chunk.y)
    else {
        return Ok(None);
    };
    load_chunk_body_from_artifact(bundle_dir, &chunk_ref.artifact_path).map(Some)
}

/// Resolve saved material names back to runtime ids (load / hydrate path).
#[must_use]
pub fn material_ids_from_saved_body(
    body: &SavedChunkBody,
    registry: &crate::terrain::material::MaterialRegistry,
) -> Vec<crate::terrain::material::MaterialId> {
    body.cells
        .iter()
        .map(|cell| {
            registry
                .name_to_id
                .get(&cell.material_name)
                .copied()
                .unwrap_or(crate::terrain::material::MaterialId(0))
        })
        .collect()
}

/// Resolve saved tag names back to runtime bitsets (load / hydrate path).
#[must_use]
pub fn tag_sets_from_saved_body(
    body: &SavedChunkBody,
    registry: &crate::terrain::material::TagRegistry,
) -> Vec<crate::terrain::material::TagSet> {
    body.cells
        .iter()
        .map(|cell| {
            let mut set = crate::terrain::material::TagSet::default();
            for name in &cell.tags {
                if let Some(id) = registry.tag_id(name) {
                    set.insert(id);
                }
            }
            set
        })
        .collect()
}

#[cfg(test)]
mod hydrate_error_tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn read_manifest_missing_bundle_returns_not_found() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/wave_s_missing_bundle_test");
        let _ = std::fs::remove_dir_all(&dir);
        let err = read_manifest_from_bundle(&dir).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::NotFound);
    }
}
