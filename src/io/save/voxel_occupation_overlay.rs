//! `VoxelOccupationSnapshot` save overlay (PLAN-VOXEL-OCCUPATION-v1 · CQ-07).
//!
//! Load order (charter): transport topology → voxel occupation → logistics GraphSync.
//! Write/hydrate are scheduled on [`crate::io::save::WorldSaveSpinePlugin`].

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

/// Apply a snapshot when the manifest names the overlay. Independent of live cutover flags —
/// flags gate writers; hydrate restores saved state.
#[must_use]
pub fn try_hydrate_voxel_occupation_from_manifest(
    bundle_dir: &Path,
    manifest: &SaveWorldManifest,
    book: &mut VoxelOccupationBook,
    _config: &VoxelOccupationConfig,
) -> bool {
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

/// Tracks hydrate attempts for voxel occupation overlay.
#[derive(Resource, Default, Debug)]
pub struct VoxelOccupationHydrateState {
    pub last_bundle_dir: Option<std::path::PathBuf>,
    pub hydrate_attempted: bool,
    pub hydrated_from_manifest: bool,
}

/// Write the voxel occupation overlay when a save flush is armed.
pub fn write_voxel_occupation_overlay_on_save_flush(
    flush: Res<crate::io::save::SaveFlushRequested>,
    settings: Res<crate::io::save::WorldSaveBundleSettings>,
    book: Option<Res<VoxelOccupationBook>>,
) {
    if !flush.0 {
        return;
    }
    let Some(book) = book else {
        return;
    };
    if book.cell_count() == 0 {
        return;
    }
    let snap = capture_voxel_occupation_overlay(&book);
    if let Err(err) = write_voxel_occupation_overlay_to_bundle(&settings.bundle_dir, &snap) {
        bevy::log::warn!(
            "voxel occupation overlay write failed at {:?}: {err}",
            settings.bundle_dir.join(VOXEL_OCCUPATION_REL_PATH)
        );
    }
}

/// Hydrate voxel occupation once per bundle directory when the overlay file is present.
pub fn try_hydrate_voxel_occupation_on_bundle_dir(
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    mut state: ResMut<VoxelOccupationHydrateState>,
    book: Option<ResMut<VoxelOccupationBook>>,
) {
    let Some(settings) = settings else {
        return;
    };
    let Some(mut book) = book else {
        return;
    };
    let bundle = settings.bundle_dir.clone();
    if state.last_bundle_dir.as_ref() == Some(&bundle) && state.hydrate_attempted {
        return;
    }
    state.last_bundle_dir = Some(bundle.clone());
    state.hydrate_attempted = true;

    let path = bundle.join(VOXEL_OCCUPATION_REL_PATH);
    let Ok(loaded) = read_voxel_occupation_snapshot_ron(&path) else {
        return;
    };
    book.apply_snapshot(&loaded);
    state.hydrated_from_manifest = true;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategic::{
        VoxelOccupationCellRecord, VOXEL_OCCUPATION_SNAPSHOT_SCHEMA_V1,
    };

    #[test]
    fn voxel_occupation_overlay_round_trip_through_bundle() {
        let dir = std::env::temp_dir().join("voxel_occ_overlay_cq07");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("tmpdir");
        let snap = VoxelOccupationSnapshot {
            schema_version: VOXEL_OCCUPATION_SNAPSHOT_SCHEMA_V1,
            cells: vec![VoxelOccupationCellRecord {
                x: 3,
                y: 4,
                layer_index: 0,
                weight: 1.0,
                z_layers: 2,
                site_id: Some(9),
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

        // Flags off must still hydrate saved state (CQ-07).
        let config = VoxelOccupationConfig::default();
        assert!(!config.derived_mirror_enabled);
        assert!(!config.commit_authority_enabled);
        let mut book = VoxelOccupationBook::default();
        assert!(try_hydrate_voxel_occupation_from_manifest(
            &dir,
            &manifest,
            &mut book,
            &config
        ));
        assert!((book.weight_at(IVec2::new(3, 4)) - 1.0).abs() < 1e-5);
        assert_eq!(book.cells[&IVec2::new(3, 4)].layers.z_layers, 2);
    }
}
