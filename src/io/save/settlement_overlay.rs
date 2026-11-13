//! Settlement hierarchy books — RON overlay slice (ECON-OG-SAVE-001).

use std::collections::HashMap;
use std::path::Path;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::strategic::settlement::{
    BlockBook, BlockId, BlockRecord, DistrictBook, DistrictId, DistrictRecord, TownBook, TownId,
    TownRecord,
};

pub const SETTLEMENT_OVERLAY_NAME: &str = "settlement_books";
pub const SETTLEMENT_BOOKS_REL_PATH: &str = "overlays/settlement_books.ron";
pub const SETTLEMENT_BOOKS_SCHEMA_VERSION: u32 = 1;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SettlementBooksSnapshot {
    pub schema_version: u32,
    pub towns: HashMap<TownId, TownRecord>,
    pub default_town: Option<TownId>,
    pub districts: HashMap<DistrictId, DistrictRecord>,
    pub default_district: Option<DistrictId>,
    pub blocks: HashMap<BlockId, BlockRecord>,
    pub tile_to_block: HashMap<IVec2, BlockId>,
}

#[must_use]
pub fn settlement_overlay_ref(artifact_path: impl Into<String>) -> crate::io::save::OverlaySnapshotRef {
    crate::io::save::OverlaySnapshotRef {
        overlay_name: SETTLEMENT_OVERLAY_NAME.into(),
        artifact_path: artifact_path.into(),
    }
}

#[must_use]
pub fn default_settlement_overlay_ref() -> crate::io::save::OverlaySnapshotRef {
    settlement_overlay_ref(SETTLEMENT_BOOKS_REL_PATH)
}

#[must_use]
pub fn build_settlement_overlay_refs() -> Vec<crate::io::save::OverlaySnapshotRef> {
    vec![default_settlement_overlay_ref()]
}

/// Write settlement hierarchy books into a save bundle (main-thread ECS capture).
pub fn write_settlement_overlay_to_bundle(
    bundle_dir: &Path,
    towns: &TownBook,
    districts: &DistrictBook,
    blocks: &BlockBook,
) -> std::io::Result<crate::io::save::OverlaySnapshotRef> {
    let snapshot = capture_settlement_books(towns, districts, blocks);
    let path = bundle_dir.join(SETTLEMENT_BOOKS_REL_PATH);
    write_settlement_books_ron(&path, &snapshot)?;
    Ok(default_settlement_overlay_ref())
}

/// Reload settlement books from manifest overlay entry when present.
#[must_use]
pub fn try_hydrate_settlement_books_from_manifest(
    bundle_dir: &Path,
    manifest: &crate::io::save::SaveWorldManifest,
    towns: &mut TownBook,
    districts: &mut DistrictBook,
    blocks: &mut BlockBook,
) -> bool {
    let Some(entry) = manifest
        .overlays
        .iter()
        .find(|o| o.overlay_name == SETTLEMENT_OVERLAY_NAME)
    else {
        return false;
    };
    let path = bundle_dir.join(&entry.artifact_path);
    let Ok(loaded) = read_settlement_books_ron(&path) else {
        return false;
    };
    apply_settlement_books(&loaded, towns, districts, blocks);
    true
}

#[must_use]
pub fn capture_settlement_books(
    towns: &TownBook,
    districts: &DistrictBook,
    blocks: &BlockBook,
) -> SettlementBooksSnapshot {
    SettlementBooksSnapshot {
        schema_version: SETTLEMENT_BOOKS_SCHEMA_VERSION,
        towns: towns.towns.clone(),
        default_town: towns.default_town.clone(),
        districts: districts.districts.clone(),
        default_district: districts.default_district.clone(),
        blocks: blocks.blocks.clone(),
        tile_to_block: blocks.tile_to_block.clone(),
    }
}

pub fn apply_settlement_books(
    snapshot: &SettlementBooksSnapshot,
    towns: &mut TownBook,
    districts: &mut DistrictBook,
    blocks: &mut BlockBook,
) {
    towns.towns = snapshot.towns.clone();
    towns.default_town = snapshot.default_town.clone();
    districts.districts = snapshot.districts.clone();
    districts.default_district = snapshot.default_district.clone();
    blocks.blocks = snapshot.blocks.clone();
    blocks.tile_to_block = snapshot.tile_to_block.clone();
}

pub fn write_settlement_books_ron(
    path: impl AsRef<Path>,
    snapshot: &SettlementBooksSnapshot,
) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = ron::ser::to_string_pretty(snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(path, body)
}

pub fn read_settlement_books_ron(path: impl AsRef<Path>) -> std::io::Result<SettlementBooksSnapshot> {
    let body = std::fs::read_to_string(path)?;
    let snap: SettlementBooksSnapshot = ron::from_str(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(snap)
}

/// Write settlement books into the active save bundle when a flush is armed.
/// Runs **before** [`crate::io::save::flush_dirty_chunk_save_queue`] so the
/// manifest-referenced `overlays/settlement_books.ron` artifact exists on disk.
pub fn write_settlement_overlay_on_save_flush(
    flush: Res<crate::io::save::SaveFlushRequested>,
    settings: Res<crate::io::save::WorldSaveBundleSettings>,
    towns: Res<TownBook>,
    districts: Res<DistrictBook>,
    blocks: Res<BlockBook>,
) {
    if !flush.0 {
        return;
    }
    if let Err(err) =
        write_settlement_overlay_to_bundle(&settings.bundle_dir, &towns, &districts, &blocks)
    {
        bevy::log::warn!(
            "settlement overlay write failed at {:?}: {err}",
            settings.bundle_dir.join(SETTLEMENT_BOOKS_REL_PATH)
        );
    }
}

/// Tracks settlement overlay hydrate attempts per bundle dir (ECON-OG-SAVE-001).
#[derive(Resource, Default, Debug)]
pub struct SettlementBooksHydrateState {
    pub last_bundle_dir: Option<std::path::PathBuf>,
    pub hydrate_attempted: bool,
    pub hydrated_from_manifest: bool,
}

/// Reload Town/District/Block books when manifest references `settlement_books` overlay.
pub fn try_hydrate_settlement_books_on_bundle_dir(
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    mut state: ResMut<SettlementBooksHydrateState>,
    mut towns: ResMut<TownBook>,
    mut districts: ResMut<DistrictBook>,
    mut blocks: ResMut<BlockBook>,
) {
    let Some(settings) = settings else {
        return;
    };
    let same_dir = state
        .last_bundle_dir
        .as_deref()
        .is_some_and(|p| p == settings.bundle_dir.as_path());
    if !same_dir {
        state.last_bundle_dir = Some(settings.bundle_dir.clone());
        state.hydrate_attempted = false;
        state.hydrated_from_manifest = false;
    }
    if state.hydrate_attempted {
        return;
    }
    state.hydrate_attempted = true;
    let Ok(manifest) = crate::io::save::read_manifest_from_bundle(&settings.bundle_dir) else {
        return;
    };
    state.hydrated_from_manifest = try_hydrate_settlement_books_from_manifest(
        &settings.bundle_dir,
        &manifest,
        &mut towns,
        &mut districts,
        &mut blocks,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_overlay_ref_uses_canonical_name() {
        let overlay = settlement_overlay_ref("overlays/settlement.ron");
        assert_eq!(overlay.overlay_name, SETTLEMENT_OVERLAY_NAME);
    }

}
