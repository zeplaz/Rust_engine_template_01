//! Preview worker job JSON — [`aps_preview_004_bevy_worker_v1.md`](../dev/aps_preview_004_bevy_worker_v1.md).

use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct PreviewCameraSpec {
    #[serde(default = "default_camera_preset")]
    pub preset: String,
    #[serde(default = "default_camera_distance")]
    pub distance_m: f32,
}

fn default_camera_preset() -> String {
    "iso_ne".into()
}

fn default_camera_distance() -> f32 {
    24.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreviewOutputSpec {
    pub png: String,
    #[serde(default = "default_width")]
    pub width: u32,
    #[serde(default = "default_height")]
    pub height: u32,
}

fn default_width() -> u32 {
    512
}

fn default_height() -> u32 {
    512
}

#[derive(Debug, Clone, Deserialize)]
pub struct PreviewAssemblyJob {
    pub schema_version: u32,
    pub operation: String,
    #[serde(default)]
    pub job_id: String,
    pub assembly_snapshot: String,
    #[serde(default)]
    pub camera: PreviewCameraSpec,
    pub output: PreviewOutputSpec,
}

impl PreviewAssemblyJob {
    pub fn load(path: &Path, repo_root: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("read job: {e}"))?;
        let job: Self = serde_json::from_str(&text).map_err(|e| format!("parse job: {e}"))?;
        if job.operation != "preview_assembly" {
            return Err(format!("unsupported operation {:?}", job.operation));
        }
        if job.schema_version != 1 {
            return Err(format!("unsupported schema_version {}", job.schema_version));
        }
        let snap = repo_root.join(job.assembly_snapshot.replace('\\', "/"));
        if !snap.is_file() {
            return Err(format!("assembly_snapshot missing: {}", snap.display()));
        }
        Ok(job)
    }

    #[must_use]
    pub fn snapshot_path<'a>(&self, repo_root: &'a Path) -> PathBuf {
        repo_root.join(self.assembly_snapshot.replace('\\', "/"))
    }

    #[must_use]
    pub fn png_path<'a>(&self, repo_root: &'a Path) -> PathBuf {
        repo_root.join(self.output.png.replace('\\', "/"))
    }

    #[must_use]
    pub fn status_path<'a>(&self, job_path: &'a Path) -> PathBuf {
        job_path.with_extension("status.json")
    }
}
