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

#[must_use]
pub fn settlement_books_save_roundtrip_witness_green() -> bool {
    use crate::strategic::settlement::{
        assign_block_for_tile, portland_fixture_district, portland_fixture_town,
    };

    let town = portland_fixture_town();
    let districts = portland_fixture_district(&town);
    let mut blocks = BlockBook::default();
    let _ = assign_block_for_tile(&districts, &mut blocks, IVec2::new(8, 8));

    let snapshot = capture_settlement_books(&town, &districts, &blocks);
    let path = std::env::temp_dir().join("settlement_books_roundtrip_test.ron");
    if write_settlement_books_ron(&path, &snapshot).is_err() {
        return false;
    }
    let loaded = match read_settlement_books_ron(&path) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let _ = std::fs::remove_file(&path);
    if loaded != snapshot {
        return false;
    }

    let mut town2 = TownBook::default();
    let mut districts2 = DistrictBook::default();
    let mut blocks2 = BlockBook::default();
    apply_settlement_books(&loaded, &mut town2, &mut districts2, &mut blocks2);
    let roundtrip = capture_settlement_books(&town2, &districts2, &blocks2);
    roundtrip == snapshot
}

/// **ECON-OG-SAVE-001** — manifest + overlay artifact bundle round-trip (Wave S hybrid save slice).
#[must_use]
pub fn settlement_books_manifest_roundtrip_witness_green() -> bool {
    use crate::io::save::manifest::build_save_world_manifest;
    use crate::io::save::pipeline::write_manifest_atomic;
    use crate::strategic::settlement::{
        assign_block_for_tile, portland_fixture_district, portland_fixture_town,
    };

    let town = portland_fixture_town();
    let districts = portland_fixture_district(&town);
    let mut blocks = BlockBook::default();
    let _ = assign_block_for_tile(&districts, &mut blocks, IVec2::new(8, 8));

    let dir = std::env::temp_dir().join(format!(
        "settlement_manifest_rt_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    if write_settlement_overlay_to_bundle(&dir, &town, &districts, &blocks).is_err() {
        return false;
    }
    let manifest = build_save_world_manifest(
        42,
        Vec::new(),
        Vec::new(),
        build_settlement_overlay_refs(),
    );
    if write_manifest_atomic(&dir, &manifest).is_err() {
        let _ = std::fs::remove_dir_all(&dir);
        return false;
    }

    let loaded_manifest = match crate::io::save::read_manifest_from_bundle(&dir) {
        Ok(m) => m,
        Err(_) => {
            let _ = std::fs::remove_dir_all(&dir);
            return false;
        }
    };
    let mut town2 = TownBook::default();
    let mut districts2 = DistrictBook::default();
    let mut blocks2 = BlockBook::default();
    let hydrated = try_hydrate_settlement_books_from_manifest(
        &dir,
        &loaded_manifest,
        &mut town2,
        &mut districts2,
        &mut blocks2,
    );
    let roundtrip = capture_settlement_books(&town2, &districts2, &blocks2);
    let expected = capture_settlement_books(&town, &districts, &blocks);
    let _ = std::fs::remove_dir_all(&dir);
    hydrated && roundtrip == expected
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

#[must_use]
pub fn econ_og_save_001_witness_green() -> bool {
    settlement_books_save_roundtrip_witness_green()
        && settlement_books_manifest_roundtrip_witness_green()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settlement_overlay_ref_uses_canonical_name() {
        let overlay = settlement_overlay_ref("overlays/settlement.ron");
        assert_eq!(overlay.overlay_name, SETTLEMENT_OVERLAY_NAME);
    }

    #[test]
    fn settlement_books_ron_roundtrip() {
        assert!(settlement_books_save_roundtrip_witness_green());
    }

    #[test]
    fn settlement_books_manifest_bundle_roundtrip() {
        assert!(settlement_books_manifest_roundtrip_witness_green());
        assert!(econ_og_save_001_witness_green());
    }
}
