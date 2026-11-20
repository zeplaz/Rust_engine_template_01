//! DF-2 — surface-water ROI save overlay (depth + flux + level-pool).

use std::path::Path;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::io::save::manifest::{OverlaySnapshotRef, SaveWorldManifest};
use crate::systems::water::{
    BarrierHydraulics, ReservoirLevelPool, SurfaceWaterField, SurfaceWaterMode,
    SURFACE_WATER_SCHEMA_V1,
};

pub const SURFACE_WATER_OVERLAY_NAME: &str = "surface_water";
pub const SURFACE_WATER_OVERLAY_REL_PATH: &str = "overlays/surface_water.ron";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SurfaceWaterOverlaySnapshot {
    pub schema_version: u32,
    pub wired: bool,
    pub mode: SurfaceWaterMode,
    pub origin_x: i32,
    pub origin_z: i32,
    pub width: u32,
    pub height: u32,
    pub dx_m: f32,
    pub depth_m: Vec<f32>,
    pub bed_m: Vec<f32>,
    pub qx_m3s: Vec<f32>,
    pub qy_m3s: Vec<f32>,
    pub wet_since_s: Vec<f32>,
    pub pool: ReservoirLevelPool,
    pub hydraulics: BarrierHydraulics,
    pub inflow_m3s: f32,
    pub rain_runoff_m3s: f32,
    pub outlet_flow_m3s: f32,
    #[serde(default)]
    pub mid_outlet_flow_m3s: f32,
    pub spillway_flow_m3s: f32,
    #[serde(default)]
    pub overtopping_flow_m3s: f32,
    #[serde(default)]
    pub turbine_flow_m3s: f32,
    pub breach_flow_m3s: f32,
    pub mass_m3: f64,
}

impl SurfaceWaterOverlaySnapshot {
    #[must_use]
    pub fn capture(field: &SurfaceWaterField) -> Self {
        Self {
            schema_version: SURFACE_WATER_SCHEMA_V1,
            wired: field.wired,
            mode: field.mode,
            origin_x: field.origin_x,
            origin_z: field.origin_z,
            width: field.width,
            height: field.height,
            dx_m: field.dx_m,
            depth_m: field.depth_m.clone(),
            bed_m: field.bed_m.clone(),
            qx_m3s: field.qx_m3s.clone(),
            qy_m3s: field.qy_m3s.clone(),
            wet_since_s: field.wet_since_s.clone(),
            pool: field.pool.clone(),
            hydraulics: field.hydraulics.clone(),
            inflow_m3s: field.inflow_m3s,
            rain_runoff_m3s: field.rain_runoff_m3s,
            outlet_flow_m3s: field.outlet_flow_m3s,
            mid_outlet_flow_m3s: field.mid_outlet_flow_m3s,
            spillway_flow_m3s: field.spillway_flow_m3s,
            overtopping_flow_m3s: field.overtopping_flow_m3s,
            turbine_flow_m3s: field.turbine_flow_m3s,
            breach_flow_m3s: field.breach_flow_m3s,
            mass_m3: field.mass_m3,
        }
    }

    pub fn restore_into(&self, field: &mut SurfaceWaterField) {
        field.schema_version = self.schema_version;
        field.wired = self.wired;
        field.mode = self.mode;
        field.origin_x = self.origin_x;
        field.origin_z = self.origin_z;
        field.width = self.width;
        field.height = self.height;
        field.dx_m = self.dx_m;
        field.depth_m = self.depth_m.clone();
        field.bed_m = self.bed_m.clone();
        field.qx_m3s = self.qx_m3s.clone();
        field.qy_m3s = self.qy_m3s.clone();
        field.wet_since_s = self.wet_since_s.clone();
        field.pool = self.pool.clone();
        field.hydraulics = self.hydraulics.clone();
        field.inflow_m3s = self.inflow_m3s;
        field.rain_runoff_m3s = self.rain_runoff_m3s;
        field.outlet_flow_m3s = self.outlet_flow_m3s;
        field.mid_outlet_flow_m3s = self.mid_outlet_flow_m3s;
        field.spillway_flow_m3s = self.spillway_flow_m3s;
        field.overtopping_flow_m3s = self.overtopping_flow_m3s;
        field.turbine_flow_m3s = self.turbine_flow_m3s;
        field.breach_flow_m3s = self.breach_flow_m3s;
        field.mass_m3 = self.mass_m3;
        field.mass_error_frac = 0.0;
        // Tile activity, dam sites and boundary cells are runtime state: rebuilt from the arrays.
        field.rebuild_runtime();
    }
}

#[must_use]
pub fn default_surface_water_overlay_ref() -> OverlaySnapshotRef {
    OverlaySnapshotRef {
        overlay_name: SURFACE_WATER_OVERLAY_NAME.into(),
        artifact_path: SURFACE_WATER_OVERLAY_REL_PATH.into(),
    }
}

pub fn write_surface_water_ron(
    path: impl AsRef<Path>,
    snap: &SurfaceWaterOverlaySnapshot,
) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let body = ron::ser::to_string_pretty(snap, ron::ser::PrettyConfig::default())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    std::fs::write(path, body)
}

pub fn read_surface_water_ron(path: impl AsRef<Path>) -> std::io::Result<SurfaceWaterOverlaySnapshot> {
    let body = std::fs::read_to_string(path)?;
    let snap: SurfaceWaterOverlaySnapshot = ron::from_str(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    if snap.schema_version != SURFACE_WATER_SCHEMA_V1 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("surface water schema {} unsupported", snap.schema_version),
        ));
    }
    Ok(snap)
}

pub fn write_surface_water_overlay_to_bundle(
    bundle_dir: &Path,
    field: &SurfaceWaterField,
) -> std::io::Result<OverlaySnapshotRef> {
    let snap = SurfaceWaterOverlaySnapshot::capture(field);
    write_surface_water_ron(bundle_dir.join(SURFACE_WATER_OVERLAY_REL_PATH), &snap)?;
    Ok(default_surface_water_overlay_ref())
}

pub fn write_surface_water_overlay_on_save_flush(
    flush: Res<crate::io::save::SaveFlushRequested>,
    settings: Res<crate::io::save::WorldSaveBundleSettings>,
    field: Option<Res<SurfaceWaterField>>,
) {
    if !flush.0 {
        return;
    }
    let Some(field) = field else {
        return;
    };
    if !field.wired {
        return;
    }
    if let Err(err) = write_surface_water_overlay_to_bundle(&settings.bundle_dir, &field) {
        bevy::log::warn!(
            "surface water overlay write failed at {:?}: {err}",
            settings.bundle_dir.join(SURFACE_WATER_OVERLAY_REL_PATH)
        );
    }
}

#[must_use]
pub fn read_surface_water_overlay_from_manifest(
    bundle_dir: &Path,
    manifest: &SaveWorldManifest,
) -> Option<SurfaceWaterOverlaySnapshot> {
    let entry = manifest
        .overlays
        .iter()
        .find(|o| o.overlay_name == SURFACE_WATER_OVERLAY_NAME)?;
    read_surface_water_ron(bundle_dir.join(&entry.artifact_path)).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn water_save_roundtrip() {
        let mut field = SurfaceWaterField::default();
        field.allocate(10, 20, 4, 4, vec![1.0; 16], 150.0);
        field.depth_m[3] = 2.5;
        field.qx_m3s[3] = 12.0;
        field.pool.storage_m3 = 999.0;
        field.pool.stage_m = 7.5;
        let snap = SurfaceWaterOverlaySnapshot::capture(&field);
        let dir = std::env::temp_dir().join("df2_surface_water_roundtrip");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        write_surface_water_ron(dir.join("overlays/surface_water.ron"), &snap).unwrap();
        let loaded = read_surface_water_ron(dir.join("overlays/surface_water.ron")).unwrap();
        let mut restored = SurfaceWaterField::default();
        loaded.restore_into(&mut restored);
        assert!(restored.wired);
        assert_eq!(restored.origin_x, 10);
        assert_eq!(restored.depth_m[3], 2.5);
        assert_eq!(restored.qx_m3s[3], 12.0);
        assert_eq!(restored.pool.storage_m3, 999.0);
        assert_eq!(restored.pool.stage_m, 7.5);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
