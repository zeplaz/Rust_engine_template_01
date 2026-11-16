//! `VoxelOccupationSnapshot` save overlay — hydrate stub (PLAN-VOXEL-OCCUPATION-v1 · X2-P0).
//!
//! Load order (charter): transport topology → voxel occupation → logistics GraphSync.
//! P0: parse/apply helpers only — manifest registration lands in P1+.

use std::path::Path;

use bevy::prelude::*;

use crate::io::save::manifest::{OverlaySnapshotRef, SaveWorldManifest};
use crate::strategic::{
    capture_voxel_occupation_snapshot, read_voxel_occupation_snapshot_ron,
    write_voxel_occupation_snapshot_ron, VoxelOccupationBook, VoxelOccupationConfig,
    VoxelOccupationSnapshot,
};

pub const VOXEL_OCCUPATION_OVERLAY_NAME: &str = "voxel_occupation";
pub const VOXEL_OCCUPATION_REL_PATH: &str = "overlays/voxel_occupation.ron";

#[must_use]
pub fn voxel_occupation_overlay_ref(artifact_path: impl Into<String>) -> OverlaySnapshotRef {
    OverlaySnapshotRef {
        overlay_name: VOXEL_OCCUPATION_OVERLAY_NAME.into(),
        artifact_path: artifact_path.into(),
    }
}

#[must_use]
pub fn default_voxel_occupation_overlay_ref() -> OverlaySnapshotRef {
    voxel_occupation_overlay_ref(VOXEL_OCCUPATION_REL_PATH)
}

#[must_use]
pub fn capture_voxel_occupation_overlay(book: &VoxelOccupationBook) -> VoxelOccupationSnapshot {
    capture_voxel_occupation_snapshot(book)
}

pub fn write_voxel_occupation_overlay_to_bundle(
    bundle_dir: &Path,
    snapshot: &VoxelOccupationSnapshot,
) -> std::io::Result<OverlaySnapshotRef> {
    let path = bundle_dir.join(VOXEL_OCCUPATION_REL_PATH);
    write_voxel_occupation_snapshot_ron(&path, snapshot)?;
    Ok(default_voxel_occupation_overlay_ref())
}

#[must_use]
pub fn try_hydrate_voxel_occupation_from_manifest(
    bundle_dir: &Path,
    manifest: &SaveWorldManifest,
    book: &mut VoxelOccupationBook,
    config: &VoxelOccupationConfig,
) -> bool {
    if !config.derived_mirror_enabled {
        return false;
    }
    let Some(entry) = manifest
        .overlays
        .iter()
        .find(|o| o.overlay_name == VOXEL_OCCUPATION_OVERLAY_NAME)
    else {
        return false;
    };
    let path = bundle_dir.join(&entry.artifact_path);
    let Ok(loaded) = read_voxel_occupation_snapshot_ron(&path) else {
        return false;
    };
    book.apply_snapshot(&loaded);
    true
}

/// Tracks hydrate attempts for voxel occupation overlay (P0 stub — not wired to load pipeline).
#[derive(Resource, Default, Debug)]
pub struct VoxelOccupationHydrateState {
    pub last_bundle_dir: Option<std::path::PathBuf>,
    pub hydrate_attempted: bool,
    pub hydrated_from_manifest: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::{
        VoxelOccupationCellRecord, VOXEL_OCCUPATION_SNAPSHOT_SCHEMA_V1,
    };

    #[test]
    fn voxel_occupation_overlay_manifest_hydrate_stub() {
        let dir = std::env::temp_dir().join("voxel_occ_overlay");
        std::fs::create_dir_all(&dir).ok();
        let snap = VoxelOccupationSnapshot {
            schema_version: VOXEL_OCCUPATION_SNAPSHOT_SCHEMA_V1,
            cells: vec![VoxelOccupationCellRecord {
                x: 3,
                y: 4,
                layer_index: 0,
                weight: 1.0,
                z_layers: 1,
                site_id: None,
                corridor_reserve: true,
            }],
        };
        write_voxel_occupation_overlay_to_bundle(&dir, &snap).expect("write overlay");

        let manifest = SaveWorldManifest {
            schema_version: crate::io::save::SAVE_WORLD_MANIFEST_SCHEMA_VERSION,
            world_seed: 0,
            chunk_sets: Vec::new(),
            registries: Vec::new(),
            overlays: vec![default_voxel_occupation_overlay_ref()],
        };

        let config = VoxelOccupationConfig {
            derived_mirror_enabled: true,
            commit_authority_enabled: false,
        };
        let mut book = VoxelOccupationBook::default();
        assert!(try_hydrate_voxel_occupation_from_manifest(
            &dir,
            &manifest,
            &mut book,
            &config
        ));
        assert!((book.weight_at(IVec2::new(3, 4)) - 1.0).abs() < 1e-5);
    }
}