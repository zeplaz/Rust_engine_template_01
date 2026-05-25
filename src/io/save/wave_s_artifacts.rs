//! Wave **S** parallel product artifacts beside `manifest.ron` (BQ-128 / BQ-133).
//!
//! **Layout:** `{bundle_dir}/product_shell.ron` — [`ProductShellPersistenceBundleR8`]
//! **Blueprints:** `{bundle_dir}/blueprints/presets.ron` — [`BlueprintPresetCollectionR8`]

use std::fs;
use std::io;
use std::path::Path;

use bevy::prelude::*;

use crate::construction::BlueprintPresetCollectionR8;
use crate::gui::hud::ProductShellPersistenceBundleR8;
use crate::io::save::pipeline::write_artifact_atomic;

/// Relative path under a world save bundle (BQ-133).
pub const WAVE_S_PRODUCT_SHELL_REL_PATH: &str = "product_shell.ron";
/// Relative path under a world save bundle (BQ-128).
pub const WAVE_S_BLUEPRINT_PRESETS_REL_PATH: &str = "blueprints/presets.ron";

#[derive(Resource, Default, Debug)]
pub struct WaveSShellCapturePending {
    pub requested: bool,
    pub last_error: Option<String>,
    pub last_written_path: Option<String>,
}

/// User-triggered restore from bundle (DQ-POST-01); autoload uses `WAVE_S_AUTOLOAD_SHELL=1`.
#[derive(Resource, Default, Debug)]
pub struct WaveSShellRestorePending {
    pub requested: bool,
}

#[derive(Resource, Default, Debug)]
pub struct WaveSShellHydrateState {
    pub last_bundle_dir: Option<std::path::PathBuf>,
    pub autoload_attempted: bool,
}

#[derive(Resource, Default, Debug, Clone)]
pub struct WaveSShellHydrateWitness {
    pub shell_loaded: bool,
    pub blueprint_count: u32,
    pub layout_widget_count: u32,
    pub autoload_enabled: bool,
    pub restore_triggered: bool,
    pub last_error: Option<String>,
}

/// Read-only blueprint presets imported from the active save bundle (WS-A03).
#[derive(Resource, Default, Debug, Clone)]
pub struct WaveSImportedBlueprints {
    pub collection: Option<BlueprintPresetCollectionR8>,
}

#[must_use]
pub fn wave_s_autoload_shell_enabled() -> bool {
    std::env::var("WAVE_S_AUTOLOAD_SHELL")
        .map(|v| {
            let v = v.to_ascii_lowercase();
            v != "0" && v != "false" && !v.is_empty()
        })
        .unwrap_or(false)
}

#[must_use]
pub fn product_shell_bundle_exists(bundle_dir: &Path) -> bool {
    bundle_dir.join(WAVE_S_PRODUCT_SHELL_REL_PATH).is_file()
}

/// Load shell + optional blueprints into presentation stores (no gameplay mutation).
#[must_use]
pub fn hydrate_wave_s_artifacts_from_bundle(
    bundle_dir: &Path,
    layout_store: &mut crate::gui::hud::HudLayoutStore,
    dock: &mut crate::gui::hud::HudDockRegistry,
    witness: &mut WaveSShellHydrateWitness,
    imported: &mut WaveSImportedBlueprints,
) -> bool {
    witness.shell_loaded = false;
    witness.blueprint_count = 0;
    witness.layout_widget_count = 0;
    witness.last_error = None;
    imported.collection = None;

    let bundle = match read_product_shell_bundle(bundle_dir) {
        Ok(bundle) => bundle,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return false,
        Err(err) => {
            witness.last_error = Some(err.to_string());
            return false;
        }
    };

    layout_store.apply_collection_with_dock(&bundle.layout, dock);
    witness.layout_widget_count = bundle.layout.widgets.len().min(u32::MAX as usize) as u32;
    witness.shell_loaded = true;

    let load_blueprints = bundle
        .blueprint_preset_ref
        .as_deref()
        .is_some_and(|p| p == WAVE_S_BLUEPRINT_PRESETS_REL_PATH)
        || bundle_dir.join(WAVE_S_BLUEPRINT_PRESETS_REL_PATH).is_file();

    if load_blueprints {
        match read_blueprint_presets(bundle_dir) {
            Ok(collection) => {
                witness.blueprint_count = collection.presets.len().min(u32::MAX as usize) as u32;
                imported.collection = Some(collection);
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => witness.last_error = Some(err.to_string()),
        }
    }

    true
}

pub fn write_product_shell_bundle(
    bundle_dir: &Path,
    bundle: &ProductShellPersistenceBundleR8,
) -> io::Result<()> {
    if bundle.schema_version != ProductShellPersistenceBundleR8::CURRENT_SCHEMA {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported product shell schema_version {} (expected {})",
                bundle.schema_version,
                ProductShellPersistenceBundleR8::CURRENT_SCHEMA
            ),
        ));
    }
    let ron = ron::ser::to_string(bundle)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let path = bundle_dir.join(WAVE_S_PRODUCT_SHELL_REL_PATH);
    write_artifact_atomic(&path, ron.as_bytes())
}

pub fn read_product_shell_bundle(bundle_dir: &Path) -> io::Result<ProductShellPersistenceBundleR8> {
    let path = bundle_dir.join(WAVE_S_PRODUCT_SHELL_REL_PATH);
    let bytes = fs::read(path)?;
    let text = std::str::from_utf8(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let bundle: ProductShellPersistenceBundleR8 =
        ron::from_str(text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    if bundle.schema_version != ProductShellPersistenceBundleR8::CURRENT_SCHEMA {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported product shell schema_version {} (expected {})",
                bundle.schema_version,
                ProductShellPersistenceBundleR8::CURRENT_SCHEMA
            ),
        ));
    }
    Ok(bundle)
}

pub fn write_blueprint_presets(
    bundle_dir: &Path,
    collection: &BlueprintPresetCollectionR8,
) -> io::Result<()> {
    if collection.schema_version != BlueprintPresetCollectionR8::CURRENT_SCHEMA {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported blueprint schema_version {} (expected {})",
                collection.schema_version,
                BlueprintPresetCollectionR8::CURRENT_SCHEMA
            ),
        ));
    }
    let ron = ron::ser::to_string(collection)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    let path = bundle_dir.join(WAVE_S_BLUEPRINT_PRESETS_REL_PATH);
    write_artifact_atomic(&path, ron.as_bytes())
}

pub fn read_blueprint_presets(bundle_dir: &Path) -> io::Result<BlueprintPresetCollectionR8> {
    let path = bundle_dir.join(WAVE_S_BLUEPRINT_PRESETS_REL_PATH);
    let bytes = fs::read(path)?;
    let text = std::str::from_utf8(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let collection: BlueprintPresetCollectionR8 =
        ron::from_str(text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
    if collection.schema_version != BlueprintPresetCollectionR8::CURRENT_SCHEMA {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "unsupported blueprint schema_version {} (expected {})",
                collection.schema_version,
                BlueprintPresetCollectionR8::CURRENT_SCHEMA
            ),
        ));
    }
    Ok(collection)
}

pub fn try_autoload_wave_s_on_bundle_dir(
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    mut state: ResMut<WaveSShellHydrateState>,
    mut layout_store: ResMut<crate::gui::hud::HudLayoutStore>,
    mut dock: ResMut<crate::gui::hud::HudDockRegistry>,
    mut witness: ResMut<WaveSShellHydrateWitness>,
    mut imported: ResMut<WaveSImportedBlueprints>,
) {
    witness.autoload_enabled = wave_s_autoload_shell_enabled();
    let Some(settings) = settings else {
        return;
    };
    let same_dir = state
        .last_bundle_dir
        .as_deref()
        .is_some_and(|p| p == settings.bundle_dir.as_path());
    if same_dir && state.autoload_attempted {
        return;
    }
    state.last_bundle_dir = Some(settings.bundle_dir.clone());
    if !witness.autoload_enabled {
        return;
    }
    state.autoload_attempted = true;
    if hydrate_wave_s_artifacts_from_bundle(
        &settings.bundle_dir,
        layout_store.as_mut(),
        dock.as_mut(),
        witness.as_mut(),
        imported.as_mut(),
    ) {
        info!(
            target: "wave_s::hydrate",
            "autoloaded product shell ({} widgets, {} blueprints)",
            witness.layout_widget_count,
            witness.blueprint_count
        );
    }
}

pub fn apply_wave_s_shell_restore_requests(
    mut pending: ResMut<WaveSShellRestorePending>,
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    mut layout_store: ResMut<crate::gui::hud::HudLayoutStore>,
    mut dock: ResMut<crate::gui::hud::HudDockRegistry>,
    mut witness: ResMut<WaveSShellHydrateWitness>,
    mut imported: ResMut<WaveSImportedBlueprints>,
) {
    if !pending.requested {
        return;
    }
    pending.requested = false;
    witness.restore_triggered = true;
    let Some(settings) = settings else {
        witness.last_error = Some("WorldSaveBundleSettings missing".into());
        return;
    };
    if hydrate_wave_s_artifacts_from_bundle(
        &settings.bundle_dir,
        layout_store.as_mut(),
        dock.as_mut(),
        witness.as_mut(),
        imported.as_mut(),
    ) {
        info!(target: "wave_s::hydrate", "restored product shell from save bundle");
    } else if witness.last_error.is_none() {
        witness.last_error = Some("product_shell.ron not found in bundle".into());
    }
}

pub fn apply_wave_s_shell_capture_requests(
    mut pending: ResMut<WaveSShellCapturePending>,
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    layout_store: Res<crate::gui::hud::HudLayoutStore>,
    dock: Res<crate::gui::hud::HudDockRegistry>,
    queue: Option<Res<crate::construction::PendingConstructionQueue>>,
) {
    if !pending.requested {
        return;
    }
    pending.requested = false;
    pending.last_error = None;
    pending.last_written_path = None;

    let Some(settings) = settings else {
        pending.last_error = Some("WorldSaveBundleSettings missing".into());
        warn!(
            target: "wave_s::product_shell",
            "shell capture skipped: no bundle_dir resource"
        );
        return;
    };

    let mut bundle = ProductShellPersistenceBundleR8 {
        schema_version: ProductShellPersistenceBundleR8::CURRENT_SCHEMA,
        layout: layout_store.to_collection(dock.as_ref()),
        ..ProductShellPersistenceBundleR8::new()
    };

    if let Some(queue) = queue.as_deref() {
        if !queue.entries.is_empty() {
            let collection =
                crate::construction::blueprint_collection_from_pending(queue);
            if let Err(err) =
                write_blueprint_presets(&settings.bundle_dir, &collection)
            {
                pending.last_error = Some(format!("blueprint write: {err}"));
                warn!(target: "wave_s::blueprint", "{err}");
                return;
            }
            bundle.blueprint_preset_ref = Some(WAVE_S_BLUEPRINT_PRESETS_REL_PATH.to_string());
        }
    }

    match write_product_shell_bundle(&settings.bundle_dir, &bundle) {
        Ok(()) => {
            let path = settings
                .bundle_dir
                .join(WAVE_S_PRODUCT_SHELL_REL_PATH)
                .display()
                .to_string();
            pending.last_written_path = Some(path.clone());
            info!(target: "wave_s::product_shell", "wrote {path}");
        }
        Err(err) => {
            pending.last_error = Some(err.to_string());
            warn!(target: "wave_s::product_shell", "shell write failed: {err}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::{
        blueprint_preset_entry_from_pending, BlueprintPresetCollectionR8,
    };
    use crate::strategic::{BuildSiteTile, FootprintTiles, SiteArchetype};
    use crate::gui::hud::HudLayoutCollectionR8;

    fn write_fixture_json(name: &str, payload: serde_json::Value) {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = root.join("debug_runs").join(name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, serde_json::to_vec_pretty(&payload).expect("json")).expect("write");
    }

    fn temp_bundle_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "proc_A_dine01_wave_s_{label}_{}",
            std::process::id()
        ));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn wave_s_product_shell_bundle_disk_roundtrip() {
        let dir = temp_bundle_dir("shell");
        let bundle = ProductShellPersistenceBundleR8::new();
        write_product_shell_bundle(&dir, &bundle).expect("write");
        let back = read_product_shell_bundle(&dir).expect("read");
        assert_eq!(bundle, back);
        let _ = std::fs::remove_dir_all(&dir);

        let ron = ron::ser::to_string(&bundle).expect("ron");
        write_fixture_json(
            "wave_s_shell_roundtrip.json",
            serde_json::json!({
                "profile": "WAVE_S_SHELL_ROUNDTRIP",
                "artifact_path": WAVE_S_PRODUCT_SHELL_REL_PATH,
                "schema_version": bundle.schema_version,
                "ron_body": ron,
                "roundtrip_ok": bundle == back,
            }),
        );
    }

    #[test]
    fn wave_s_blueprint_presets_disk_roundtrip() {
        let dir = temp_bundle_dir("blueprint");
        let mut collection = BlueprintPresetCollectionR8::new();
        collection.push(blueprint_preset_entry_from_pending(
            "depot_a",
            SiteArchetype::RailDepot,
            BuildSiteTile { x: 4, z: 8 },
            FootprintTiles {
                width: 2,
                depth: 2,
            },
            "Surface",
            1,
            false,
        ));
        write_blueprint_presets(&dir, &collection).expect("write");
        let back = read_blueprint_presets(&dir).expect("read");
        assert_eq!(collection, back);
        let _ = std::fs::remove_dir_all(&dir);

        let ron = ron::ser::to_string(&collection).expect("ron");
        write_fixture_json(
            "wave_s_blueprint_roundtrip.json",
            serde_json::json!({
                "profile": "WAVE_S_BLUEPRINT_ROUNDTRIP",
                "artifact_path": WAVE_S_BLUEPRINT_PRESETS_REL_PATH,
                "schema_version": collection.schema_version,
                "preset_count": collection.presets.len(),
                "ron_body": ron,
                "roundtrip_ok": collection == back,
            }),
        );
    }

    #[test]
    fn wave_s_artifact_paths_documented() {
        assert_eq!(WAVE_S_PRODUCT_SHELL_REL_PATH, "product_shell.ron");
        assert_eq!(WAVE_S_BLUEPRINT_PRESETS_REL_PATH, "blueprints/presets.ron");
        let _layout: HudLayoutCollectionR8 = HudLayoutCollectionR8::new();
    }

    #[test]
    fn hydrate_wave_s_applies_layout_and_blueprints() {
        let dir = temp_bundle_dir("hydrate");
        let mut bundle = ProductShellPersistenceBundleR8::new();
        bundle.layout.upsert(crate::gui::hud::HudWidgetLayoutEntryR8 {
            widget: "minimap".into(),
            rect: crate::gui::hud::HudWidgetRectR8 {
                x: 1.0,
                y: 2.0,
                width: 100.0,
                height: 80.0,
            },
            minimized: false,
            detached: true,
        });
        write_product_shell_bundle(&dir, &bundle).expect("shell");
        let mut collection = BlueprintPresetCollectionR8::new();
        collection.push(blueprint_preset_entry_from_pending(
            "x",
            SiteArchetype::RailDepot,
            BuildSiteTile { x: 0, z: 0 },
            FootprintTiles {
                width: 1,
                depth: 1,
            },
            "Surface",
            0,
            false,
        ));
        write_blueprint_presets(&dir, &collection).expect("bp");
        bundle.blueprint_preset_ref = Some(WAVE_S_BLUEPRINT_PRESETS_REL_PATH.to_string());
        write_product_shell_bundle(&dir, &bundle).expect("shell2");

        let mut layout_store = crate::gui::hud::HudLayoutStore::default();
        let mut dock = crate::gui::hud::HudDockRegistry::default();
        let mut witness = WaveSShellHydrateWitness::default();
        let mut imported = WaveSImportedBlueprints::default();
        assert!(hydrate_wave_s_artifacts_from_bundle(
            &dir,
            &mut layout_store,
            &mut dock,
            &mut witness,
            &mut imported
        ));
        assert!(witness.shell_loaded);
        assert_eq!(witness.blueprint_count, 1);
        assert!(imported.collection.is_some());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
