//! Phase 0 containment guard — no new `*live_proof*.rs` outside this tree (except manifest shims).

use std::path::{Path, PathBuf};

use super::MIGRATION_SHIM_PATHS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveProofContainmentViolation {
    pub relative_path: String,
}

/// Scan `src/` for `*live_proof*.rs` files outside [`super::MIGRATION_SHIM_PATHS`].
#[must_use]
pub fn scan_live_proof_containment_violations(repo_root: &Path) -> Vec<LiveProofContainmentViolation> {
    let src_root = repo_root.join("src");
    let witness_root = repo_root.join("src/dev/runtime_witness");
    let allowed: std::collections::HashSet<&str> = MIGRATION_SHIM_PATHS.iter().copied().collect();
    let mut violations = Vec::new();
    if !src_root.is_dir() {
        return violations;
    }
    for entry in walkdir_light(&src_root) {
        let Some(path) = entry else { continue };
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if !file_name.contains("live_proof") || !file_name.ends_with(".rs") {
            continue;
        }
        if path.starts_with(&witness_root) {
            continue;
        }
        let rel = path
            .strip_prefix(repo_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        if allowed.contains(rel.as_str()) {
            continue;
        }
        violations.push(LiveProofContainmentViolation {
            relative_path: rel,
        });
    }
    violations.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    violations
}

#[must_use]
pub fn phase0_containment_green(repo_root: &Path) -> bool {
    scan_live_proof_containment_violations(repo_root).is_empty()
}

fn walkdir_light(root: &Path) -> Vec<Option<PathBuf>> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let read = match std::fs::read_dir(&dir) {
            Ok(r) => r,
            Err(_) => {
                out.push(None);
                continue;
            }
        };
        for entry in read.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                out.push(Some(path));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_contain_hardfail_ci_001_manifest_shims_only() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        assert!(
            phase0_containment_green(&repo),
            "CI -HardFail: unlisted *live_proof*.rs — migrate to runtime_witness/ or exceptions_manifest.json"
        );
    }

    /// **CONTAIN-D-001** — retired Slice-D shims removed; manifest + MIGRATION_SHIM_PATHS aligned.
    #[test]
    fn contain_d_001_retired_shim_paths_absent() {
        use super::super::MIGRATION_SHIM_PATHS;

        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("exceptions_manifest.json")).expect("manifest");
        for path in manifest["contain_d_001_retired"]
            .as_array()
            .expect("contain_d_001_retired")
        {
            let rel = path.as_str().expect("path str");
            assert!(
                !repo.join(rel).exists(),
                "CONTAIN-D-001: retired shim still on disk: {rel}"
            );
        }
        assert_eq!(
            MIGRATION_SHIM_PATHS.len(),
            manifest["allowed_shim_paths"]
                .as_array()
                .map(|a| a.len())
                .unwrap_or(0),
            "MIGRATION_SHIM_PATHS must match exceptions_manifest allowed_shim_paths"
        );
    }

    /// **CONTAIN-MINIMAP-001** — minimap shim retired; writer-only in `runtime_witness/minimap.rs`.
    #[test]
    fn contain_minimap_001_retired_shim_absent() {
        let repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let shim = "src/render/minimap_compositor/live_proof.rs";
        assert!(
            !repo.join(shim).exists(),
            "CONTAIN-MINIMAP-001: shim still on disk: {shim}"
        );
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("exceptions_manifest.json")).expect("manifest");
        let allowed = manifest["allowed_shim_paths"]
            .as_array()
            .expect("allowed_shim_paths");
        assert!(
            !allowed.iter().any(|p| p.as_str() == Some(shim)),
            "CONTAIN-MINIMAP-001: shim still listed in allowed_shim_paths"
        );
    }
}
