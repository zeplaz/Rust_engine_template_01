//! Weather field overlay slot for the save manifest (RON).
//!
//! Persists climate, pressure systems, both snapshots and the RNG of
//! [`WeatherField`](crate::systems::weather::WeatherField). Hydrate hands the loaded state to the weather
//! step through [`PendingWeatherRestore`] — the field keeps one writer.

use std::path::Path;

use bevy::prelude::*;

use crate::io::save::manifest::{OverlaySnapshotRef, SaveWorldManifest};
use crate::systems::weather::{
    migrate_weather_field_save, PendingWeatherRestore, WeatherField, WeatherFieldSave,
};

pub const WEATHER_OVERLAY_NAME: &str = "weather_field";
pub const WEATHER_OVERLAY_REL_PATH: &str = "overlays/weather_field.ron";

#[must_use]
pub fn default_weather_overlay_ref() -> OverlaySnapshotRef {
    OverlaySnapshotRef {
        overlay_name: WEATHER_OVERLAY_NAME.into(),
        artifact_path: WEATHER_OVERLAY_REL_PATH.into(),
    }
}

#[must_use]
pub fn build_weather_overlay_refs() -> Vec<OverlaySnapshotRef> {
    vec![default_weather_overlay_ref()]
}

pub fn write_weather_field_ron(path: impl AsRef<Path>, save: &WeatherFieldSave) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = ron::ser::to_string_pretty(save, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(path, body)
}

pub fn read_weather_field_ron(path: impl AsRef<Path>) -> std::io::Result<WeatherFieldSave> {
    let body = std::fs::read_to_string(path)?;
    let save: WeatherFieldSave = ron::from_str(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    migrate_weather_field_save(save)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Read the weather overlay named by `manifest`; `None` when absent (older saves) or unreadable.
#[must_use]
pub fn read_weather_overlay_from_manifest(
    bundle_dir: &Path,
    manifest: &SaveWorldManifest,
) -> Option<WeatherFieldSave> {
    let entry = manifest
        .overlays
        .iter()
        .find(|o| o.overlay_name == WEATHER_OVERLAY_NAME)?;
    read_weather_field_ron(bundle_dir.join(&entry.artifact_path)).ok()
}

/// Write the weather overlay when a save flush is armed (before the manifest flush).
pub fn write_weather_overlay_on_save_flush(
    flush: Res<crate::io::save::SaveFlushRequested>,
    settings: Res<crate::io::save::WorldSaveBundleSettings>,
    field: Option<Res<WeatherField>>,
) {
    if !flush.0 {
        return;
    }
    let Some(field) = field.filter(|f| f.is_ready()) else {
        return;
    };
    let path = settings.bundle_dir.join(WEATHER_OVERLAY_REL_PATH);
    if let Err(err) = write_weather_field_ron(&path, &field.to_save()) {
        bevy::log::warn!("weather overlay write failed at {path:?}: {err}");
    }
}

/// Tracks weather overlay hydrate attempts per bundle dir.
#[derive(Resource, Default, Debug)]
pub struct WeatherOverlayHydrateState {
    pub last_bundle_dir: Option<std::path::PathBuf>,
    pub hydrate_attempted: bool,
    pub hydrated_from_manifest: bool,
}

/// Queue the saved weather state for the weather step when the manifest references it.
pub fn try_hydrate_weather_overlay_on_bundle_dir(
    settings: Option<Res<crate::io::save::WorldSaveBundleSettings>>,
    mut state: ResMut<WeatherOverlayHydrateState>,
    restore: Option<ResMut<PendingWeatherRestore>>,
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
    let Some(mut restore) = restore else {
        return;
    };
    state.hydrate_attempted = true;
    let Ok(manifest) = crate::io::save::read_manifest_from_bundle(&settings.bundle_dir) else {
        return;
    };
    let Some(save) = read_weather_overlay_from_manifest(&settings.bundle_dir, &manifest) else {
        return;
    };
    restore.0 = Some(save);
    state.hydrated_from_manifest = true;
}
