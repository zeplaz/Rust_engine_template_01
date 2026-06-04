//! APS / MCP assembly preview worker (Bevy subprocess).

pub mod assembly_worker;
pub mod job;

pub use assembly_worker::{load_assembly_snapshot_json, repo_root_from_manifest, run_preview_job};
pub use job::PreviewAssemblyJob;
