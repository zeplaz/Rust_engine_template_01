//! Atmosphere clipmap stack — RON overlay slice (**ES-6-4**).
//!
//! Persists [`AtmosphereClipmapStack`] L0–L3 channel densities + focus.
//! Schema v3 adds ash/ember/heat (DEBT-010c). Field→L0 tactical fold is **ES-6-2**.

use std::path::Path;

use bevy::math::{DVec2, UVec2};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::substrate::atmosphere::AtmosphereClipmapStack;

pub const ATMOS_CLIPMAP_OVERLAY_NAME: &str = "atmosphere_clipmap_stack";
pub const ATMOS_CLIPMAP_STATE_REL_PATH: &str = "overlays/atmosphere_clipmap_stack.ron";
pub const ATMOS_CLIPMAP_SCHEMA_VERSION: u32 = 3;
pub const ATMOS_CLIPMAP_SAVE_ROUNDTRIP_JSON: &str =
    "debug_runs/atmosphere_clipmap_save_roundtrip_live.json";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SavedAtmosClipLevel {
    pub resolution: [u32; 2],
    pub smoke_density: Vec<f32>,
    /// DEBT-010b — fog lane (schema v2+).
    #[serde(default)]
    pub fog_density: Vec<f32>,
    /// DEBT-010b — toxicity lane (schema v2+).
    #[serde(default)]
    pub toxicity: Vec<f32>,
    /// DEBT-010c — ash lane (schema v3).
    #[serde(default)]
    pub ash_density: Vec<f32>,
    /// DEBT-010c — ember lane (schema v3).
    #[serde(default)]
    pub ember_density: Vec<f32>,
    /// DEBT-010c — heat-distortion lane (schema v3).
    #[serde(default)]
    pub heat_distortion: Vec<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AtmosphereClipmapSnapshot {
    pub schema_version: u32,
    pub active_focus: [f64; 2],
    pub levels: Vec<SavedAtmosClipLevel>,
}

#[must_use]
pub fn atmos_clipmap_overlay_ref(
    artifact_path: impl Into<String>,
) -> crate::io::save::OverlaySnapshotRef {
    crate::io::save::OverlaySnapshotRef {
        overlay_name: ATMOS_CLIPMAP_OVERLAY_NAME.into(),
        artifact_path: artifact_path.into(),
    }
}

#[must_use]
pub fn default_atmos_clipmap_overlay_ref() -> crate::io::save::OverlaySnapshotRef {
    atmos_clipmap_overlay_ref(ATMOS_CLIPMAP_STATE_REL_PATH)
}

#[must_use]
pub fn build_atmos_clipmap_overlay_refs() -> Vec<crate::io::save::OverlaySnapshotRef> {
    vec![default_atmos_clipmap_overlay_ref()]
}

#[must_use]
pub fn capture_atmosphere_clipmap_snapshot(stack: &AtmosphereClipmapStack) -> AtmosphereClipmapSnapshot {
    AtmosphereClipmapSnapshot {
        schema_version: ATMOS_CLIPMAP_SCHEMA_VERSION,
        active_focus: [stack.active_focus.x, stack.active_focus.y],
        levels: stack
            .levels
            .iter()
            .map(|l| SavedAtmosClipLevel {
                resolution: [l.resolution.x, l.resolution.y],
                smoke_density: l.smoke_density.clone(),
                fog_density: l.fog_density.clone(),
                toxicity: l.toxicity.clone(),
                ash_density: l.ash_density.clone(),
                ember_density: l.ember_density.clone(),
                heat_distortion: l.heat_distortion.clone(),
            })
            .collect(),
    }
}

/// Apply snapshot into an existing stack. Returns `Err` on schema / shape mismatch.
pub fn hydrate_atmosphere_clipmap_snapshot(
    stack: &mut AtmosphereClipmapStack,
    snapshot: &AtmosphereClipmapSnapshot,
) -> Result<(), String> {
    if snapshot.schema_version != 1
        && snapshot.schema_version != 2
        && snapshot.schema_version != ATMOS_CLIPMAP_SCHEMA_VERSION
    {
        return Err(format!(
            "atmos clipmap schema {} not in {{1, 2, {}}}",
            snapshot.schema_version, ATMOS_CLIPMAP_SCHEMA_VERSION
        ));
    }
    if snapshot.levels.len() != stack.levels.len() {
        return Err(format!(
            "atmos clipmap level count {} != {}",
            snapshot.levels.len(),
            stack.levels.len()
        ));
    }
    for (dst, src) in stack.levels.iter_mut().zip(snapshot.levels.iter()) {
        let res = UVec2::new(src.resolution[0], src.resolution[1]);
        let expected = (res.x * res.y) as usize;
        if src.smoke_density.len() != expected {
            return Err(format!(
                "level {:?} density len {} != {}",
                src.resolution,
                src.smoke_density.len(),
                expected
            ));
        }
        dst.resolution = res;
        dst.smoke_density = src.smoke_density.clone();
        // Schema v1/v2 loads may omit newer lanes — zero-fill to resolution.
        dst.fog_density = if src.fog_density.len() == expected {
            src.fog_density.clone()
        } else {
            vec![0.0; expected]
        };
        dst.toxicity = if src.toxicity.len() == expected {
            src.toxicity.clone()
        } else {
            vec![0.0; expected]
        };
        dst.ash_density = if src.ash_density.len() == expected {
            src.ash_density.clone()
        } else {
            vec![0.0; expected]
        };
        dst.ember_density = if src.ember_density.len() == expected {
            src.ember_density.clone()
        } else {
            vec![0.0; expected]
        };
        dst.heat_distortion = if src.heat_distortion.len() == expected {
            src.heat_distortion.clone()
        } else {
            vec![0.0; expected]
        };
    }
    stack.active_focus = DVec2::new(snapshot.active_focus[0], snapshot.active_focus[1]);
    Ok(())
}

pub fn write_atmosphere_clipmap_snapshot(
    path: &Path,
    snapshot: &AtmosphereClipmapSnapshot,
) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let ron = ron::ser::to_string_pretty(snapshot, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, ron)
}

pub fn read_atmosphere_clipmap_snapshot(path: &Path) -> std::io::Result<AtmosphereClipmapSnapshot> {
    let raw = std::fs::read_to_string(path)?;
    ron::de::from_str(&raw).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Write clipmap stack into a save bundle (main-thread ECS capture).
pub fn write_atmosphere_clipmap_overlay_to_bundle(
    bundle_dir: &Path,
    stack: &AtmosphereClipmapStack,
) -> std::io::Result<crate::io::save::OverlaySnapshotRef> {
    let snapshot = capture_atmosphere_clipmap_snapshot(stack);
    let path = bundle_dir.join(ATMOS_CLIPMAP_STATE_REL_PATH);
    write_atmosphere_clipmap_snapshot(&path, &snapshot)?;
    Ok(default_atmos_clipmap_overlay_ref())
}

/// Reload clipmap from manifest overlay entry when present.
#[must_use]
pub fn try_hydrate_atmosphere_clipmap_from_manifest(
    bundle_dir: &Path,
    manifest: &crate::io::save::SaveWorldManifest,
    stack: &mut AtmosphereClipmapStack,
) -> bool {
    let Some(entry) = manifest
        .overlays
        .iter()
        .find(|o| o.overlay_name == ATMOS_CLIPMAP_OVERLAY_NAME)
    else {
        return false;
    };
    let path = bundle_dir.join(&entry.artifact_path);
    let Ok(loaded) = read_atmosphere_clipmap_snapshot(&path) else {
        return false;
    };
    hydrate_atmosphere_clipmap_snapshot(stack, &loaded).is_ok()
}

/// Lib proof: capture → RON → hydrate preserves L0 cell + focus.
#[must_use]
pub fn atmosphere_clipmap_save_roundtrip_green() -> bool {
    let mut stack = AtmosphereClipmapStack::default();
    stack.active_focus = DVec2::new(12.5, -3.25);
    if let Some(cell) = stack
        .levels
        .first_mut()
        .and_then(|l| l.smoke_density.get_mut(0))
    {
        *cell = 0.42;
    }
    if let Some(level0) = stack.levels.first_mut() {
        if let Some(ash) = level0.ash_density.get_mut(0) {
            *ash = 0.18;
        }
        if let Some(ember) = level0.ember_density.get_mut(0) {
            *ember = 0.12;
        }
        if let Some(heat) = level0.heat_distortion.get_mut(0) {
            *heat = 0.27;
        }
    }
    let snap = capture_atmosphere_clipmap_snapshot(&stack);
    let dir = std::env::temp_dir().join("rust_engine_atmos_clipmap_es64");
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("atmosphere_clipmap_stack.ron");
    if write_atmosphere_clipmap_snapshot(&path, &snap).is_err() {
        return false;
    }
    let loaded = match read_atmosphere_clipmap_snapshot(&path) {
        Ok(s) => s,
        Err(_) => return false,
    };
    let mut restored = AtmosphereClipmapStack::default();
    if hydrate_atmosphere_clipmap_snapshot(&mut restored, &loaded).is_err() {
        return false;
    }
    let l0_ok = restored
        .levels
        .first()
        .and_then(|l| l.smoke_density.first().copied())
        == Some(0.42);
    let ash_ok = restored
        .levels
        .first()
        .and_then(|l| l.ash_density.first().copied())
        == Some(0.18);
    let focus_ok = (restored.active_focus.x - 12.5).abs() < 1e-9
        && (restored.active_focus.y + 3.25).abs() < 1e-9;
    l0_ok && ash_ok && focus_ok && loaded.schema_version == ATMOS_CLIPMAP_SCHEMA_VERSION
}

/// Bundle + manifest export/hydrate (settlement overlay pattern).
#[must_use]
pub fn atmosphere_clipmap_manifest_roundtrip_green() -> bool {
    use crate::io::save::manifest::build_save_world_manifest;
    use crate::io::save::pipeline::write_manifest_atomic;

    let mut stack = AtmosphereClipmapStack::default();
    stack.active_focus = DVec2::new(3.0, 7.5);
    if let Some(cell) = stack
        .levels
        .first_mut()
        .and_then(|l| l.smoke_density.get_mut(0))
    {
        *cell = 0.77;
    }

    let dir = std::env::temp_dir().join(format!(
        "atmos_clipmap_manifest_rt_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    if write_atmosphere_clipmap_overlay_to_bundle(&dir, &stack).is_err() {
        return false;
    }
    let manifest = build_save_world_manifest(
        42,
        Vec::new(),
        Vec::new(),
        build_atmos_clipmap_overlay_refs(),
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
    let mut restored = AtmosphereClipmapStack::default();
    let hydrated =
        try_hydrate_atmosphere_clipmap_from_manifest(&dir, &loaded_manifest, &mut restored);
    let expected = capture_atmosphere_clipmap_snapshot(&stack);
    let got = capture_atmosphere_clipmap_snapshot(&restored);
    let _ = std::fs::remove_dir_all(&dir);
    hydrated && expected == got
}

#[must_use]
pub fn refresh_atmosphere_clipmap_save_roundtrip_witness() -> bool {
    let ron_green = atmosphere_clipmap_save_roundtrip_green();
    let manifest_green = atmosphere_clipmap_manifest_roundtrip_green();
    let green = ron_green && manifest_green;
    let body = serde_json::json!({
        "gate_id": "ES-6-4",
        "program_id": "EFFECTS-SYSTEM-UNIFY-001",
        "phase": "ES-6",
        "slice": "ES-6-4",
        "clipmap_snapshot_contract": true,
        "overlay_name": ATMOS_CLIPMAP_OVERLAY_NAME,
        "schema_version": ATMOS_CLIPMAP_SCHEMA_VERSION,
        "overlay_export_wired": true,
        "ron_roundtrip_green": ron_green,
        "manifest_roundtrip_green": manifest_green,
        "roundtrip_green": green,
        "green": green,
    });
    let wrapped = crate::dev::debug_run_envelope::wrap_debug_run(
        "ES-6-4",
        "refresh_atmosphere_clipmap_save_roundtrip_witness",
        ATMOS_CLIPMAP_SAVE_ROUNDTRIP_JSON,
        body,
    );
    green && crate::dev::debug_run_envelope::write_debug_run_json(ATMOS_CLIPMAP_SAVE_ROUNDTRIP_JSON, wrapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atmos_clipmap_ron_roundtrip_preserves_l0() {
        assert!(atmosphere_clipmap_save_roundtrip_green());
        assert!(atmosphere_clipmap_manifest_roundtrip_green());
        assert!(refresh_atmosphere_clipmap_save_roundtrip_witness());
    }
}
